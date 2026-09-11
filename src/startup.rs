//! 「Windows 起動時に実行」の設定。
//!
//! 利用者が明示的に有効にしたときだけ、ユーザー単位の Run キーに値を 1 つ書く。
//! 管理者権限は不要で、タスク マネージャーの「スタートアップ アプリ」にも現れる。

use std::ptr::{null, null_mut};
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
pub fn exe_path() -> String {
    let mut buf = [0u16; MAX_PATH as usize];
    let n = unsafe { GetModuleFileNameW(null_mut(), buf.as_mut_ptr(), buf.len() as u32) };
    String::from_utf16_lossy(&buf[..n as usize])
}

/// Run キーに書くコマンドライン。パスに空白があっても壊れないよう引用符で囲む。
pub fn command_line(exe: &str, args: &str) -> String {
    if args.is_empty() {
        format!("\"{}\"", exe)
    } else {
        format!("\"{}\" {}", exe, args)
    }
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
    let command = command_line(&exe_path(), args);
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

/// 未使用の警告を避けるためだけの参照。
#[allow(dead_code)]
const _: *const u16 = null();

#[cfg(test)]
mod tests {
    use super::command_line;

    #[test]
    fn コマンドラインを引用符で囲む() {
        assert_eq!(
            command_line(r"C:\Program Files\PowerModeTray.exe", "--hotkey Ctrl+Alt+P"),
            r#""C:\Program Files\PowerModeTray.exe" --hotkey Ctrl+Alt+P"#
        );
        assert_eq!(command_line(r"C:\a\b.exe", ""), r#""C:\a\b.exe""#);
        assert_eq!(command_line(r"C:\a\b.exe", "--hotkey none"), r#""C:\a\b.exe" --hotkey none"#);
    }
}
