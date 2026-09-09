//! 電源モード（オーバーレイ電源スキーム）とバッテリー節約機能の状態。
//!
//! 設定アプリの「電源モード」は powrprof.dll の非公開 API で読み書きできる。
//! インポートライブラリに無い関数なので実行時に取得する。

use std::ffi::c_void;
use std::sync::OnceLock;
use windows_sys::core::GUID;
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

use crate::win::wide;

pub const fn guid(a: u32, b: u16, c: u16, d: [u8; 8]) -> GUID {
    GUID { data1: a, data2: b, data3: c, data4: d }
}

pub fn guid_eq(x: &GUID, y: &GUID) -> bool {
    x.data1 == y.data1 && x.data2 == y.data2 && x.data3 == y.data3 && x.data4 == y.data4
}

/// 設定アプリの 3 択に対応する GUID（HKLM\...\Control\Power\User\PowerSchemes 配下のサブキー名）
pub const BEST_EFFICIENCY: GUID = guid(0x961cc777, 0x2547, 0x4f9d, [0x81, 0x74, 0x7d, 0x86, 0x18, 0x1b, 0x8a, 0x7a]);
pub const BALANCED: GUID = guid(0, 0, 0, [0; 8]);
pub const BEST_PERFORMANCE: GUID = guid(0xded574b5, 0x45a0, 0x4f42, [0x87, 0x37, 0x46, 0x34, 0x5c, 0x09, 0xc2, 0x38]);

/// バッテリー節約機能（エネルギー節約機能）の状態通知 GUID_ENERGY_SAVER_STATUS。値は 0=オフ, 1=標準, 2=高
pub const ENERGY_SAVER_STATUS: GUID = guid(0x550e8400, 0xe29b, 0x41d4, [0xa7, 0x16, 0x44, 0x66, 0x55, 0x44, 0x00, 0x00]);

type GetOverlayFn = unsafe extern "system" fn(*mut GUID) -> u32;
type SetOverlayFn = unsafe extern "system" fn(GUID) -> u32;

struct Api {
    get_effective: Option<GetOverlayFn>,
    set_active: Option<SetOverlayFn>,
}

fn api() -> &'static Api {
    static API: OnceLock<Api> = OnceLock::new();
    API.get_or_init(|| unsafe {
        let lib = LoadLibraryW(wide("powrprof.dll").as_ptr());
        if lib.is_null() {
            return Api { get_effective: None, set_active: None };
        }
        let get = GetProcAddress(lib, c"PowerGetEffectiveOverlayScheme".as_ptr().cast());
        let set = GetProcAddress(lib, c"PowerSetActiveOverlayScheme".as_ptr().cast());
        Api {
            get_effective: get.map(|f| std::mem::transmute::<unsafe extern "system" fn() -> isize, GetOverlayFn>(f)),
            set_active: set.map(|f| std::mem::transmute::<unsafe extern "system" fn() -> isize, SetOverlayFn>(f)),
        }
    })
}

/// 現在有効な電源モードの GUID。取得できなければ None。
pub fn current_overlay() -> Option<GUID> {
    let f = api().get_effective?;
    let mut g = BALANCED;
    unsafe { (f(&mut g) == 0).then_some(g) }
}

/// 電源モードを切り替える。0 なら成功、それ以外は Win32 エラーコード。
pub fn set_overlay(scheme: GUID) -> u32 {
    match api().set_active {
        Some(f) => unsafe { f(scheme) },
        None => 127, // ERROR_PROC_NOT_FOUND
    }
}

fn power_status() -> Option<SYSTEM_POWER_STATUS> {
    let mut s: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
    unsafe { (GetSystemPowerStatus(&mut s) != 0).then_some(s) }
}

/// バッテリー節約機能が有効か（SYSTEM_POWER_STATUS.SystemStatusFlag）。
pub fn battery_saver_flag() -> bool {
    power_status().map_or(false, |s| s.SystemStatusFlag & 1 != 0)
}

/// バッテリー残量 (0-100)。不明なら None。
pub fn battery_percent() -> Option<u32> {
    power_status().and_then(|s| (s.BatteryLifePercent <= 100).then_some(s.BatteryLifePercent as u32))
}

/// 型消去のための補助。GetProcAddress の戻り値の型を明示する用途でのみ使う。
#[allow(dead_code)]
fn _assert_ptr(_: *const c_void) {}
