//! PowerModeTray - Windows 11 の「電源モード」をシステムトレイのクリックやホットキーで順番に切り替える最小アプリ。
//!
//! 設定 > システム > 電源とバッテリー > 電源モード が内部で使っている powrprof.dll の
//! オーバーレイ電源スキーム API をそのまま呼ぶだけで、レジストリやファイルへの書き込み、
//! 管理者権限、追加ランタイムはいずれも不要。GUI フレームワークを使わず Win32 API だけで動く。
//!
//!   左クリック : 最適な電力効率 -> バランス -> 最適なパフォーマンス -> (先頭へ戻る)
//!   右クリック : モードを直接選択 / 現在のホットキーの確認 / 終了
//!   ホットキー : 左クリックと同じ。既定は Ctrl+Alt+P。起動引数 --hotkey で変更・無効化できる
//!   アイコン   : Windows 標準アイコンフォントのゲージ。針が 左=電力効率 / 中央=バランス / 右=パフォーマンス
//!   節約機能   : Windows のバッテリー節約機能が有効な間は設定アプリと同じく切り替え不可。
//!                アイコンを葉付きバッテリーに変え、ツールチップとメニューで理由を示す
//!
//! 切り替え対象は設定アプリと同じく「現在の電源（AC 接続時 / バッテリー駆動時）」側のみ。

#![windows_subsystem = "windows"]

mod cli;
mod hotkey;
mod power;
mod tray;
mod win;

use std::cell::RefCell;
use std::mem::zeroed;
use std::ptr::{null, null_mut};

use windows_sys::core::GUID;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, ERROR_SUCCESS, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Power::{
    RegisterPowerSettingNotification, UnregisterPowerSettingNotification, HPOWERNOTIFY, POWERBROADCAST_SETTING,
};
use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey};
use windows_sys::Win32::UI::Shell::NIN_SELECT;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use cli::Command;
use hotkey::HotkeySpec;
use tray::glyph;
use win::{hiword, loword, wide};

const SAVER_NOTICE: &str = "バッテリー節約機能が有効のため切り替えできません";
const MOD_NOREPEAT: u32 = 0x4000; // 押しっぱなしで連続発火させない
const HOTKEY_ID: i32 = 1;
const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 2000; // 設定アプリなど外部での変更に追従する間隔
const PBT_APMPOWERSTATUSCHANGE: usize = 0x000A;
const PBT_POWERSETTINGCHANGE: usize = 0x8013;
const CMD_MODE_BASE: u32 = 1; // メニューのモード項目は 1, 2, 3
const CMD_EXIT: u32 = 10;
const NIN_KEYSELECT: u32 = NIN_SELECT | 0x1; // キーボードでの選択（NIN_SELECT | NINF_KEY）

#[link(name = "kernel32")]
extern "system" {
    fn CreateMutexW(attributes: *const core::ffi::c_void, initial_owner: i32, name: *const u16) -> *mut core::ffi::c_void;
}

struct Mode {
    id: GUID,
    name: &'static str,
    glyph: char,
}

// 設定アプリの 3 択と、Windows 標準アイコンフォントのゲージ（針が 左 / 中央 / 右）
const MODES: [Mode; 3] = [
    Mode { id: power::BEST_EFFICIENCY, name: "最適な電力効率", glyph: glyph::SPEED_LOW },
    Mode { id: power::BALANCED, name: "バランス", glyph: glyph::SPEED_MEDIUM },
    Mode { id: power::BEST_PERFORMANCE, name: "最適なパフォーマンス", glyph: glyph::SPEED_HIGH },
];
const BALANCED_INDEX: usize = 1;

/// 最後にトレイへ反映した状態。変化が無ければ描き直さない。
#[derive(PartialEq, Clone, Copy)]
struct Shown {
    index: Option<usize>,
    light: bool,
    saver: bool,
    glyph: char,
}

struct App {
    hwnd: HWND,
    hotkey: Option<HotkeySpec>,
    hotkey_label: String,
    font: Vec<u16>,
    icon: HICON,
    icon_added: bool,
    shown: Option<Shown>,
    /// 通知で受け取った最新のバッテリー節約機能の状態。0 より大きければ有効
    energy_saver_status: u32,
    power_notify: HPOWERNOTIFY,
    taskbar_created: u32,
}

impl App {
    /// バッテリー節約機能が有効か。通知の値と GetSystemPowerStatus のどちらかが有効ならそう扱う
    fn is_saver_active(&self) -> bool {
        self.energy_saver_status > 0 || power::battery_saver_flag()
    }
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn with_app<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|a| a.borrow_mut().as_mut().map(f))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let hotkey = match cli::parse(&args) {
        Err(msg) => {
            message_box(&format!("{}\n\n{}", msg, cli::USAGE), MB_ICONERROR);
            std::process::exit(2);
        }
        Ok(Command::Help) => {
            message_box(cli::USAGE, MB_ICONINFORMATION);
            return;
        }
        Ok(Command::Run { hotkey }) => hotkey,
    };

    unsafe {
        // 二重起動はここで静かに終了する。ハンドルはプロセス終了まで保持する
        let _mutex = CreateMutexW(null(), 1, wide(r"Local\PowerModeTray.SingleInstance").as_ptr());
        if GetLastError() == ERROR_ALREADY_EXISTS {
            return;
        }
        run(hotkey);
    }
}

fn message_box(text: &str, icon: u32) {
    unsafe {
        MessageBoxW(null_mut(), wide(text).as_ptr(), wide("PowerModeTray").as_ptr(), MB_OK | icon);
    }
}

unsafe fn run(hotkey: Option<HotkeySpec>) {
    let hinstance = GetModuleHandleW(null());
    let class_name = wide("PowerModeTrayWindow");
    let mut wc: WNDCLASSW = zeroed();
    wc.lpfnWndProc = Some(wndproc);
    wc.hInstance = hinstance;
    wc.lpszClassName = class_name.as_ptr();
    RegisterClassW(&wc);

    // 不可視のトップレベルウィンドウ。TaskbarCreated などのブロードキャストを受けるため message-only にはしない
    let hwnd = CreateWindowExW(
        0, class_name.as_ptr(), wide("PowerModeTray").as_ptr(), WS_OVERLAPPED,
        0, 0, 0, 0, null_mut(), null_mut(), hinstance, null(),
    );
    if hwnd.is_null() {
        message_box("ウィンドウを作成できませんでした。", MB_ICONERROR);
        std::process::exit(1);
    }

    let mut hotkey_label = "ホットキー: なし".to_string();
    let mut hotkey_warning = None;
    if let Some(spec) = &hotkey {
        if RegisterHotKey(hwnd, HOTKEY_ID, spec.modifiers | MOD_NOREPEAT, spec.vk) != 0 {
            hotkey_label = format!("ホットキー: {}", spec.display);
        } else {
            hotkey_label = format!("ホットキー: {}（登録失敗）", spec.display);
            hotkey_warning = Some(format!("ホットキー {} は他のアプリが使用中のため登録できませんでした。", spec.display));
        }
    }

    // バッテリー節約機能の状態通知。登録直後に現在値が届く
    let power_notify = RegisterPowerSettingNotification(hwnd, &power::ENERGY_SAVER_STATUS, 0);
    let taskbar_created = RegisterWindowMessageW(wide("TaskbarCreated").as_ptr());

    APP.with(|a| {
        *a.borrow_mut() = Some(App {
            hwnd,
            hotkey,
            hotkey_label,
            font: tray::pick_icon_font(),
            icon: null_mut(),
            icon_added: false,
            shown: None,
            energy_saver_status: 0,
            power_notify,
            taskbar_created,
        })
    });

    refresh(true);
    if let Some(warning) = hotkey_warning {
        tray::notify(hwnd, "電源モード", &warning, true);
    }
    SetTimer(hwnd, TIMER_ID, TIMER_INTERVAL_MS, None);

    let mut msg: MSG = zeroed();
    while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_TIMER => {
            refresh(false);
            0
        }
        WM_HOTKEY => {
            next();
            0
        }
        tray::WM_TRAYICON => {
            match loword(lparam as usize) {
                NIN_SELECT | NIN_KEYSELECT => next(),
                WM_CONTEXTMENU => show_menu(hwnd, loword(wparam) as i16 as i32, hiword(wparam) as i16 as i32),
                _ => {}
            }
            0
        }
        WM_POWERBROADCAST => {
            if wparam == PBT_POWERSETTINGCHANGE && lparam != 0 {
                let setting = &*(lparam as *const POWERBROADCAST_SETTING);
                if power::guid_eq(&setting.PowerSetting, &power::ENERGY_SAVER_STATUS) && setting.DataLength >= 4 {
                    let status = std::ptr::read_unaligned(setting.Data.as_ptr() as *const u32);
                    with_app(|a| a.energy_saver_status = status);
                    refresh(false);
                }
            } else if wparam == PBT_APMPOWERSTATUSCHANGE {
                refresh(false); // AC / バッテリーの切り替わり
            }
            1
        }
        WM_SETTINGCHANGE => {
            refresh(false); // ライト / ダークテーマの変更など
            0
        }
        WM_DESTROY => {
            cleanup();
            PostQuitMessage(0);
            0
        }
        _ => {
            if msg != 0 && Some(msg) == with_app(|a| a.taskbar_created) {
                // Explorer が再起動したのでアイコンを登録し直す
                with_app(|a| a.icon_added = false);
                refresh(true);
                return 0;
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

fn current_index() -> Option<usize> {
    power::current_overlay().and_then(|g| MODES.iter().position(|m| power::guid_eq(&m.id, &g)))
}

/// 次のモードへ進める。3 択以外の値（他ツールで設定された不明なモード）ならバランスへ。
fn next() {
    if with_app(|a| a.is_saver_active()).unwrap_or(true) {
        refresh(false); // 設定アプリと同じく節約機能中は切り替えない。表示だけ最新にする
        return;
    }
    let index = current_index();
    apply(index.map_or(BALANCED_INDEX, |i| (i + 1) % MODES.len()));
}

fn apply(index: usize) {
    if with_app(|a| a.is_saver_active()).unwrap_or(true) {
        refresh(false);
        return;
    }
    let rc = power::set_overlay(MODES[index].id);
    if rc != 0 {
        if let Some(hwnd) = with_app(|a| a.hwnd) {
            tray::notify(hwnd, "電源モード", &format!("切り替えに失敗しました (エラー {})", rc), true);
        }
    }
    refresh(false);
}

/// 現在の状態をトレイアイコン・ツールチップに反映する。
fn refresh(force: bool) {
    with_app(|a| {
        let index = current_index();
        let light = is_light_taskbar();
        let saver = a.is_saver_active();
        let glyph = if saver {
            glyph::battery_saver(power::battery_percent())
        } else {
            index.map_or(glyph::SPEED_MEDIUM, |i| MODES[i].glyph)
        };
        let shown = Shown { index, light, saver, glyph };
        if !force && a.icon_added && a.shown == Some(shown) {
            return;
        }

        let name = index.map_or("不明", |i| MODES[i].name);
        let tip = if saver { format!("電源モード: {}\n{}", name, SAVER_NOTICE) } else { format!("電源モード: {}", name) };
        let alpha = if saver || index.is_some() { 255 } else { 110 }; // 不明なモードは薄く
        let size = unsafe { GetSystemMetrics(SM_CXSMICON) }; // 100%:16px, 150%:24px
        let icon = tray::render_icon(glyph, size, &a.font, light, alpha);

        if a.icon_added {
            tray::update(a.hwnd, icon, &tip);
        } else {
            a.icon_added = tray::add(a.hwnd, icon, &tip);
        }
        // Shell 側にコピーされるので、渡した直後に古いアイコンを破棄してよい
        if !a.icon.is_null() {
            unsafe { DestroyIcon(a.icon) };
        }
        a.icon = icon;
        a.shown = Some(shown);
    });
}

/// タスクバーがライトテーマか（アイコンの色を黒 / 白で切り替えるため）。
fn is_light_taskbar() -> bool {
    unsafe {
        let mut value: u32 = 0;
        let mut size: u32 = 4;
        let rc = RegGetValueW(
            HKEY_CURRENT_USER,
            wide(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize").as_ptr(),
            wide("SystemUsesLightTheme").as_ptr(),
            RRF_RT_REG_DWORD,
            null_mut(),
            &mut value as *mut u32 as *mut _,
            &mut size,
        );
        rc == ERROR_SUCCESS && value != 0
    }
}

fn show_menu(hwnd: HWND, x: i32, y: i32) {
    // TrackPopupMenuEx はモーダルループで WM_TIMER などを配送するので、借用を持ったまま入らない
    let Some((saver, hotkey_label)) = with_app(|a| (a.is_saver_active(), a.hotkey_label.clone())) else { return };
    let index = current_index();

    unsafe {
        let menu = CreatePopupMenu();
        if saver {
            AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, wide(SAVER_NOTICE).as_ptr());
            AppendMenuW(menu, MF_SEPARATOR, 0, null());
        }
        for (i, mode) in MODES.iter().enumerate() {
            let flags = if saver { MF_STRING | MF_GRAYED } else { MF_STRING };
            AppendMenuW(menu, flags, (CMD_MODE_BASE + i as u32) as usize, wide(mode.name).as_ptr());
        }
        if let Some(i) = index {
            CheckMenuRadioItem(menu, CMD_MODE_BASE, CMD_MODE_BASE + 2, CMD_MODE_BASE + i as u32, MF_BYCOMMAND);
        }
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, wide(&hotkey_label).as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(menu, MF_STRING, CMD_EXIT as usize, wide("終了").as_ptr());

        SetForegroundWindow(hwnd); // メニュー外をクリックしたときに閉じるために必要
        let cmd = TrackPopupMenuEx(menu, TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY, x, y, hwnd, null()) as u32;
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);

        match cmd {
            c if (CMD_MODE_BASE..CMD_MODE_BASE + MODES.len() as u32).contains(&c) => apply((c - CMD_MODE_BASE) as usize),
            CMD_EXIT => {
                DestroyWindow(hwnd);
            }
            _ => {}
        }
    }
}

fn cleanup() {
    with_app(|a| unsafe {
        KillTimer(a.hwnd, TIMER_ID);
        tray::remove(a.hwnd);
        if a.hotkey.is_some() {
            UnregisterHotKey(a.hwnd, HOTKEY_ID);
        }
        if a.power_notify != 0 {
            UnregisterPowerSettingNotification(a.power_notify);
            a.power_notify = 0;
        }
        if !a.icon.is_null() {
            DestroyIcon(a.icon);
            a.icon = null_mut();
        }
    });
}
