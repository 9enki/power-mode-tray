//! 「Windows 起動時に実行」の設定。
//!
//! 配布形態によって Windows 側の仕組みが違うので、実行時に見分けて使い分ける。
//!
//! - 通常の exe: 利用者が明示的に有効にしたときだけ、ユーザー単位の Run キーに値を 1 つ書く。
//!   管理者権限は不要で、タスク マネージャーの「スタートアップ アプリ」にも現れる。
//! - MSIX（Store 版）: パッケージ内からの HKCU 書き込みはパッケージ専用の hive に隔離されて
//!   Explorer には見えず、WindowsApps 配下の exe は Run キーからは起動できない。
//!   代わりに、マニフェストで宣言した StartupTask を WinRT の StartupTask API で切り替える。
//!   状態は Windows が持ち、設定 > アプリ > スタートアップ にも同じものが出る。

use std::ptr::null_mut;
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::APPMODEL_ERROR_NO_PACKAGE;
use windows_sys::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;

/// 有効化できなかった理由。
#[derive(Debug, PartialEq, Eq)]
pub enum EnableError {
    /// 書き込みや API の呼び出しに失敗した
    Failed,
    /// 利用者（またはポリシー）が Windows の設定で無効にしていて、アプリからは戻せない
    DisabledInSettings,
}

/// 自動起動が有効か。
pub fn is_enabled() -> bool {
    if is_packaged() {
        startup_task::state().is_some_and(state_is_on)
    } else {
        run_key::is_enabled()
    }
}

/// 自動起動を有効にする。args は Run キー方式で起動引数として引き継ぐ（StartupTask には渡せない）。
pub fn enable(args: &str) -> Result<(), EnableError> {
    if is_packaged() {
        startup_task::request_enable().map_or(Err(EnableError::Failed), enable_outcome)
    } else if run_key::enable(args) {
        Ok(())
    } else {
        Err(EnableError::Failed)
    }
}

/// 自動起動を無効にする。成功なら true。
pub fn disable() -> bool {
    if is_packaged() {
        startup_task::disable()
    } else {
        run_key::disable()
    }
}

/// MSIX パッケージとして動いているか（パッケージ ID を持っているか）。
fn is_packaged() -> bool {
    static PACKAGED: OnceLock<bool> = OnceLock::new();
    *PACKAGED.get_or_init(|| {
        let mut len = 0u32;
        // パッケージ内なら ERROR_INSUFFICIENT_BUFFER で必要な長さが返る。パッケージ外だけがこのエラーになる
        unsafe { GetCurrentPackageFullName(&mut len, null_mut()) != APPMODEL_ERROR_NO_PACKAGE }
    })
}

/// Windows.ApplicationModel.StartupTaskState の値。
pub mod state {
    pub const DISABLED: i32 = 0;
    pub const DISABLED_BY_USER: i32 = 1;
    pub const ENABLED: i32 = 2;
    pub const DISABLED_BY_POLICY: i32 = 3;
    pub const ENABLED_BY_POLICY: i32 = 4;
}

/// StartupTaskState から「自動起動する」かを決める。
pub fn state_is_on(s: i32) -> bool {
    matches!(s, state::ENABLED | state::ENABLED_BY_POLICY)
}

/// RequestEnableAsync が返した状態から結果を決める。
pub fn enable_outcome(s: i32) -> Result<(), EnableError> {
    match s {
        state::ENABLED | state::ENABLED_BY_POLICY => Ok(()),
        state::DISABLED_BY_USER | state::DISABLED_BY_POLICY => Err(EnableError::DisabledInSettings),
        _ => Err(EnableError::Failed),
    }
}

/// Run キーに書くコマンドライン。パスに空白があっても壊れないよう引用符で囲む。
pub fn command_line(exe: &str, args: &str) -> String {
    if args.is_empty() {
        format!("\"{}\"", exe)
    } else {
        format!("\"{}\" {}", exe, args)
    }
}

/// 通常の exe 向け。ユーザー単位の Run キーに値を 1 つ書く。
mod run_key {
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{ERROR_SUCCESS, MAX_PATH};
    use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
    use windows_sys::Win32::System::Registry::{
        RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_BINARY, RRF_RT_REG_SZ,
    };

    use crate::win::wide;

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    /// タスク マネージャーで無効にされた項目がここに記録される
    const APPROVED_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
    const VALUE_NAME: &str = "PowerModeTray";

    /// 自分の実行ファイルのフルパス。
    fn exe_path() -> String {
        let mut buf = [0u16; MAX_PATH as usize];
        let n = unsafe { GetModuleFileNameW(null_mut(), buf.as_mut_ptr(), buf.len() as u32) };
        String::from_utf16_lossy(&buf[..n as usize])
    }

    /// 自動起動が有効か。Run キーに値があり、かつタスク マネージャーで無効にされていないこと。
    pub fn is_enabled() -> bool {
        read_run_value().is_some() && !is_disabled_by_user()
    }

    fn read_run_value() -> Option<String> {
        unsafe {
            let mut size: u32 = 0;
            let key = wide(RUN_KEY);
            let name = wide(VALUE_NAME);
            if RegGetValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr(), RRF_RT_REG_SZ, null_mut(), null_mut(), &mut size)
                != ERROR_SUCCESS
            {
                return None;
            }
            let mut buf = vec![0u16; (size as usize / 2) + 1];
            if RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_SZ,
                null_mut(),
                buf.as_mut_ptr().cast(),
                &mut size,
            ) != ERROR_SUCCESS
            {
                return None;
            }
            let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            Some(String::from_utf16_lossy(&buf[..end]))
        }
    }

    /// タスク マネージャーの「スタートアップ アプリ」で無効にされているか（先頭バイトの bit 0 が立つ）。
    fn is_disabled_by_user() -> bool {
        unsafe {
            let mut buf = [0u8; 12];
            let mut size = buf.len() as u32;
            let rc = RegGetValueW(
                HKEY_CURRENT_USER,
                wide(APPROVED_KEY).as_ptr(),
                wide(VALUE_NAME).as_ptr(),
                RRF_RT_REG_BINARY,
                null_mut(),
                buf.as_mut_ptr().cast(),
                &mut size,
            );
            rc == ERROR_SUCCESS && size >= 1 && buf[0] & 1 != 0
        }
    }

    /// 自動起動を有効にする。成功なら true。
    pub fn enable(args: &str) -> bool {
        let command = super::command_line(&exe_path(), args);
        let data = wide(&command);
        let ok = unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                wide(RUN_KEY).as_ptr(),
                wide(VALUE_NAME).as_ptr(),
                REG_SZ,
                data.as_ptr().cast(),
                (data.len() * 2) as u32,
            ) == ERROR_SUCCESS
        };
        // タスク マネージャーで無効にされていた場合、その記録を消して有効に戻す
        if ok && is_disabled_by_user() {
            unsafe {
                RegDeleteKeyValueW(HKEY_CURRENT_USER, wide(APPROVED_KEY).as_ptr(), wide(VALUE_NAME).as_ptr());
            }
        }
        ok
    }

    /// 自動起動を無効にする。成功なら true（元から無ければ何もせず true）。
    pub fn disable() -> bool {
        if read_run_value().is_none() {
            return true;
        }
        unsafe {
            RegDeleteKeyValueW(HKEY_CURRENT_USER, wide(RUN_KEY).as_ptr(), wide(VALUE_NAME).as_ptr()) == ERROR_SUCCESS
        }
    }
}

/// MSIX 向け。マニフェストで宣言した StartupTask を WinRT API で操作する。
///
/// windows-sys には WinRT の関数も StartupTask の定義も無いので、必要な分だけここで宣言する。
/// IID と vtable の並びは Windows SDK の windows.applicationmodel.h / asyncinfo.h に合わせてある。
mod startup_task {
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::time::{Duration, Instant};
    use windows_sys::core::{GUID, HRESULT};

    /// msix/AppxManifest.xml の StartupTask の TaskId と一致させる
    const TASK_ID: &str = "PowerModeTrayStartup";
    const CLASS_NAME: &str = "Windows.ApplicationModel.StartupTask";
    /// Windows.ApplicationModel.IStartupTaskStatics
    const IID_STARTUP_TASK_STATICS: GUID = GUID::from_u128(0xee5b60bd_a148_41a7_b26e_e8b88a1e62f8);
    /// Windows.Foundation.IAsyncInfo
    const IID_ASYNC_INFO: GUID = GUID::from_u128(0x00000036_0000_0000_c000_000000000046);
    /// 完了待ちの上限。通常は数十 ms で終わる
    const TIMEOUT: Duration = Duration::from_secs(10);

    type HSTRING = *mut c_void;
    const RO_INIT_MULTITHREADED: i32 = 1;
    /// AsyncStatus の値
    const STARTED: i32 = 0;
    const COMPLETED: i32 = 1;

    #[link(name = "runtimeobject")]
    extern "system" {
        fn RoInitialize(init_type: i32) -> HRESULT;
        fn RoUninitialize();
        fn RoGetActivationFactory(class_id: HSTRING, iid: *const GUID, factory: *mut *mut c_void) -> HRESULT;
        fn WindowsCreateString(source: *const u16, length: u32, string: *mut HSTRING) -> HRESULT;
        fn WindowsDeleteString(string: HSTRING) -> HRESULT;
    }

    /// IInspectable（IUnknown 込み）。使わないメソッドはポインタ幅の空きで置く
    #[repr(C)]
    struct InspectableVtbl {
        query_interface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
        add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
        release: unsafe extern "system" fn(*mut c_void) -> u32,
        get_iids: usize,
        get_runtime_class_name: usize,
        get_trust_level: usize,
    }

    /// IStartupTaskStatics
    #[repr(C)]
    struct StaticsVtbl {
        base: InspectableVtbl,
        get_for_current_package_async: usize,
        get_async: unsafe extern "system" fn(*mut c_void, HSTRING, *mut *mut c_void) -> HRESULT,
    }

    /// IStartupTask
    #[repr(C)]
    struct TaskVtbl {
        base: InspectableVtbl,
        request_enable_async: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
        disable: unsafe extern "system" fn(*mut c_void) -> HRESULT,
        get_state: unsafe extern "system" fn(*mut c_void, *mut i32) -> HRESULT,
        get_task_id: usize,
    }

    /// IAsyncOperation<T>。GetResults の出力は T により IStartupTask のポインタか StartupTaskState の値
    #[repr(C)]
    struct AsyncOperationVtbl {
        base: InspectableVtbl,
        put_completed: usize,
        get_completed: usize,
        get_results: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
    }

    /// IAsyncInfo
    #[repr(C)]
    struct AsyncInfoVtbl {
        base: InspectableVtbl,
        get_id: usize,
        get_status: unsafe extern "system" fn(*mut c_void, *mut i32) -> HRESULT,
        get_error_code: usize,
        cancel: usize,
        close: usize,
    }

    /// COM インターフェイスのポインタ。落ちるときに Release する
    struct Com(*mut c_void);

    impl Com {
        unsafe fn vtbl<V>(&self) -> &V {
            &**(self.0 as *const *const V)
        }
    }

    impl Drop for Com {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { (self.vtbl::<InspectableVtbl>().release)(self.0) };
            }
        }
    }

    /// HSTRING。落ちるときに解放する
    struct HStr(HSTRING);

    impl HStr {
        fn new(s: &str) -> Option<HStr> {
            let utf16: Vec<u16> = s.encode_utf16().collect();
            let mut h: HSTRING = null_mut();
            // 文字列はコピーされるので、utf16 はこの後すぐ捨ててよい
            unsafe { (WindowsCreateString(utf16.as_ptr(), utf16.len() as u32, &mut h) >= 0).then_some(HStr(h)) }
        }
    }

    impl Drop for HStr {
        fn drop(&mut self) {
            unsafe { WindowsDeleteString(self.0) };
        }
    }

    /// WinRT の呼び出しは専用スレッドの MTA で行う。UI スレッド（メッセージループ）に COM を初期化せず、
    /// 完了待ちでブロックしても UI スレッドへのマーシャリングが要らないので、詰まる心配がない。
    fn with_winrt<R: Send>(f: impl FnOnce() -> Option<R> + Send) -> Option<R> {
        std::thread::scope(|s| {
            s.spawn(move || unsafe {
                if RoInitialize(RO_INIT_MULTITHREADED) < 0 {
                    return None;
                }
                let result = f();
                RoUninitialize();
                result
            })
            .join()
            .ok()
            .flatten()
        })
    }

    /// 非同期操作の完了を待つ。完了したら true。取り消し・エラー・時間切れなら false
    unsafe fn wait(op: &Com) -> bool {
        let mut info = null_mut();
        if (op.vtbl::<InspectableVtbl>().query_interface)(op.0, &IID_ASYNC_INFO, &mut info) < 0 {
            return false;
        }
        let info = Com(info);
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let mut status = STARTED;
            if (info.vtbl::<AsyncInfoVtbl>().get_status)(info.0, &mut status) < 0 {
                return false;
            }
            if status != STARTED {
                return status == COMPLETED;
            }
            if Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// StartupTask.GetAsync(TASK_ID) で IStartupTask を得る
    unsafe fn task() -> Option<Com> {
        let class = HStr::new(CLASS_NAME)?;
        let mut factory = null_mut();
        if RoGetActivationFactory(class.0, &IID_STARTUP_TASK_STATICS, &mut factory) < 0 {
            return None;
        }
        let factory = Com(factory);
        let id = HStr::new(TASK_ID)?;
        let mut op = null_mut();
        if (factory.vtbl::<StaticsVtbl>().get_async)(factory.0, id.0, &mut op) < 0 {
            return None;
        }
        let op = Com(op);
        if !wait(&op) {
            return None;
        }
        let mut task: *mut c_void = null_mut();
        let hr = (op.vtbl::<AsyncOperationVtbl>().get_results)(op.0, (&mut task as *mut *mut c_void).cast());
        (hr >= 0 && !task.is_null()).then(|| Com(task))
    }

    /// 今の StartupTaskState。取得できなければ None
    pub fn state() -> Option<i32> {
        with_winrt(|| unsafe {
            let task = task()?;
            let mut s = super::state::DISABLED;
            ((task.vtbl::<TaskVtbl>().get_state)(task.0, &mut s) >= 0).then_some(s)
        })
    }

    /// 有効化を要求し、結果の StartupTaskState を返す。設定で無効にされていると DisabledByUser のまま返る
    pub fn request_enable() -> Option<i32> {
        with_winrt(|| unsafe {
            let task = task()?;
            let mut op = null_mut();
            if (task.vtbl::<TaskVtbl>().request_enable_async)(task.0, &mut op) < 0 {
                return None;
            }
            let op = Com(op);
            if !wait(&op) {
                return None;
            }
            let mut s = super::state::DISABLED;
            ((op.vtbl::<AsyncOperationVtbl>().get_results)(op.0, (&mut s as *mut i32).cast()) >= 0).then_some(s)
        })
    }

    /// 無効にする。成功なら true
    pub fn disable() -> bool {
        with_winrt(|| unsafe {
            let task = task()?;
            ((task.vtbl::<TaskVtbl>().disable)(task.0) >= 0).then_some(())
        })
        .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn コマンドラインを引用符で囲む() {
        assert_eq!(
            command_line(r"C:\Program Files\PowerModeTray.exe", "--hotkey Ctrl+Alt+P"),
            r#""C:\Program Files\PowerModeTray.exe" --hotkey Ctrl+Alt+P"#
        );
        assert_eq!(command_line(r"C:\a\b.exe", ""), r#""C:\a\b.exe""#);
        assert_eq!(command_line(r"C:\a\b.exe", "--hotkey none"), r#""C:\a\b.exe" --hotkey none"#);
    }

    #[test]
    fn 状態から有効かを決める() {
        assert!(state_is_on(state::ENABLED));
        assert!(state_is_on(state::ENABLED_BY_POLICY));
        assert!(!state_is_on(state::DISABLED));
        assert!(!state_is_on(state::DISABLED_BY_USER));
        assert!(!state_is_on(state::DISABLED_BY_POLICY));
    }

    #[test]
    fn 有効化の結果を状態から決める() {
        assert_eq!(enable_outcome(state::ENABLED), Ok(()));
        assert_eq!(enable_outcome(state::ENABLED_BY_POLICY), Ok(()));
        // 設定アプリで切られている分はアプリからは戻せないので、設定を開くよう案内する
        assert_eq!(enable_outcome(state::DISABLED_BY_USER), Err(EnableError::DisabledInSettings));
        assert_eq!(enable_outcome(state::DISABLED_BY_POLICY), Err(EnableError::DisabledInSettings));
        // 要求したのに無効のままなら失敗
        assert_eq!(enable_outcome(state::DISABLED), Err(EnableError::Failed));
        assert_eq!(enable_outcome(99), Err(EnableError::Failed));
    }
}
