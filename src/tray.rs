//! トレイアイコンの描画（Windows 標準アイコンフォントのグリフ）と Shell_NotifyIcon の操作。

use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{HWND, RECT};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::win::{copy_wide, wide};

/// Segoe Fluent Icons / Segoe MDL2 Assets のグリフ（同じコードポイント）。
pub mod glyph {
    pub const SPEED_LOW: char = '\u{EC48}'; // ゲージ 針が左
    pub const SPEED_MEDIUM: char = '\u{EC49}'; // ゲージ 針が中央
    pub const SPEED_HIGH: char = '\u{EC4A}'; // ゲージ 針が右

    /// 葉付きバッテリー（BatterySaver0-10）。残量に応じて目盛りを変える。不明なら満充電の形。
    pub fn battery_saver(percent: Option<u32>) -> char {
        let level = match percent {
            None => 10,
            Some(p) => (p.min(100) + 5) / 10,
        };
        char::from_u32(0xEBB6 + level).unwrap()
    }
}

/// トレイからのコールバックメッセージ
pub const WM_TRAYICON: u32 = WM_APP + 1;
const ICON_ID: u32 = 1;

/// 使えるアイコンフォントを選ぶ。Windows 11 は Segoe Fluent Icons、Windows 10 は Segoe MDL2 Assets。
pub fn pick_icon_font() -> Vec<u16> {
    for name in ["Segoe Fluent Icons", "Segoe MDL2 Assets"] {
        let face = wide(name);
        unsafe {
            let font = CreateFontW(-16, 0, 0, 0, 400, 0, 0, 0, DEFAULT_CHARSET as u32, OUT_DEFAULT_PRECIS as u32,
                CLIP_DEFAULT_PRECIS as u32, ANTIALIASED_QUALITY as u32, (DEFAULT_PITCH | FF_DONTCARE) as u32, face.as_ptr());
            if font.is_null() {
                continue;
            }
            let hdc = GetDC(null_mut());
            let old = SelectObject(hdc, font);
            let mut actual = [0u16; 64];
            let n = GetTextFaceW(hdc, actual.len() as i32, actual.as_mut_ptr());
            SelectObject(hdc, old);
            ReleaseDC(null_mut(), hdc);
            DeleteObject(font);
            // 無いフォント名を渡すと GDI が別のフォントに置き換えるので、実際の名前で確認する
            if n > 0 && actual[..(n as usize - 1)] == face[..face.len() - 1] {
                return face;
            }
        }
    }
    wide("Segoe MDL2 Assets")
}

/// グリフを 1 文字描いた HICON を作る。alpha は 0-255 で全体の濃さ。呼び出し側が DestroyIcon する。
pub fn render_icon(glyph: char, size: i32, face: &[u16], light_theme: bool, alpha: u8) -> HICON {
    unsafe {
        let screen = GetDC(null_mut());
        let hdc = CreateCompatibleDC(screen);
        ReleaseDC(null_mut(), screen);

        let mut bmi: BITMAPINFO = zeroed();
        bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = size;
        bmi.bmiHeader.biHeight = -size; // 上から下
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB as u32;
        let mut bits: *mut core::ffi::c_void = null_mut();
        let color_bmp = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        let old_bmp = SelectObject(hdc, color_bmp);
        let pixel_count = (size * size) as usize;
        std::ptr::write_bytes(bits as *mut u8, 0, pixel_count * 4); // 黒・透明で初期化

        // グリフの余白分だけ大きめの em サイズで、白でアンチエイリアス描画する
        let font = CreateFontW(-(size as f32 * 1.2) as i32, 0, 0, 0, 400, 0, 0, 0, DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, ANTIALIASED_QUALITY as u32,
            (DEFAULT_PITCH | FF_DONTCARE) as u32, face.as_ptr());
        let old_font = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, 0x00FF_FFFF);
        let mut rc = RECT { left: 0, top: 0, right: size, bottom: size };
        let mut text = [0u16; 2];
        let text_len = glyph.encode_utf16(&mut text).len() as i32;
        DrawTextW(hdc, text.as_mut_ptr(), text_len, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP);
        GdiFlush();

        // 白の濃さ = カバレッジ。それをアルファにして、テーマに合わせた色を塗る（BGRA、非乗算アルファ）
        let (b, g, r) = if light_theme { (0u8, 0u8, 0u8) } else { (255u8, 255u8, 255u8) };
        let px = bits as *mut u8;
        for i in 0..pixel_count {
            let p = px.add(i * 4);
            let coverage = *p.add(1) as u32;
            *p = b;
            *p.add(1) = g;
            *p.add(2) = r;
            *p.add(3) = (coverage * alpha as u32 / 255) as u8;
        }

        SelectObject(hdc, old_font);
        DeleteObject(font);
        SelectObject(hdc, old_bmp);
        DeleteDC(hdc);

        let mask = CreateBitmap(size, size, 1, 1, null()); // アルファ付きなのでマスクは形だけ
        let info = ICONINFO { fIcon: 1, xHotspot: 0, yHotspot: 0, hbmMask: mask, hbmColor: color_bmp };
        let icon = CreateIconIndirect(&info);
        DeleteObject(mask);
        DeleteObject(color_bmp);
        icon
    }
}

fn base_data(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut d: NOTIFYICONDATAW = unsafe { zeroed() };
    d.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    d.hWnd = hwnd;
    d.uID = ICON_ID;
    d
}

/// トレイにアイコンを登録する。Explorer 再起動時（TaskbarCreated）にも呼ぶ。
pub fn add(hwnd: HWND, icon: HICON, tip: &str) -> bool {
    unsafe {
        let mut d = base_data(hwnd);
        d.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP;
        d.uCallbackMessage = WM_TRAYICON;
        d.hIcon = icon;
        copy_wide(&mut d.szTip, tip);
        Shell_NotifyIconW(NIM_DELETE, &d); // 前回の異常終了で残っていた分を消す
        if Shell_NotifyIconW(NIM_ADD, &d) == 0 {
            return false;
        }
        d.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        Shell_NotifyIconW(NIM_SETVERSION, &d) != 0
    }
}

/// アイコンとツールチップを更新する。
pub fn update(hwnd: HWND, icon: HICON, tip: &str) {
    unsafe {
        let mut d = base_data(hwnd);
        d.uFlags = NIF_ICON | NIF_TIP | NIF_SHOWTIP;
        d.hIcon = icon;
        copy_wide(&mut d.szTip, tip);
        Shell_NotifyIconW(NIM_MODIFY, &d);
    }
}

/// 通知（バルーン）を出す。
pub fn notify(hwnd: HWND, title: &str, text: &str, warning: bool) {
    unsafe {
        let mut d = base_data(hwnd);
        d.uFlags = NIF_INFO;
        d.dwInfoFlags = if warning { NIIF_WARNING } else { NIIF_INFO };
        copy_wide(&mut d.szInfoTitle, title);
        copy_wide(&mut d.szInfo, text);
        Shell_NotifyIconW(NIM_MODIFY, &d);
    }
}

pub fn remove(hwnd: HWND) {
    unsafe {
        let d = base_data(hwnd);
        Shell_NotifyIconW(NIM_DELETE, &d);
    }
}

#[cfg(test)]
mod tests {
    use super::glyph;

    #[test]
    fn 残量からグリフを選ぶ() {
        assert_eq!(glyph::battery_saver(Some(0)), '\u{EBB6}');
        assert_eq!(glyph::battery_saver(Some(4)), '\u{EBB6}'); // 四捨五入で 0
        assert_eq!(glyph::battery_saver(Some(5)), '\u{EBB7}'); // 四捨五入で 1
        assert_eq!(glyph::battery_saver(Some(55)), '\u{EBBC}'); // 6
        assert_eq!(glyph::battery_saver(Some(100)), '\u{EBC0}'); // 10
        assert_eq!(glyph::battery_saver(Some(150)), '\u{EBC0}'); // 上限で丸める
        assert_eq!(glyph::battery_saver(None), '\u{EBC0}'); // 不明は満充電の形
    }
}
