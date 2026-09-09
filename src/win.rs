//! Win32 API 呼び出しのための小さなヘルパー。

/// Rust の文字列を NUL 終端の UTF-16 に変換する。
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// UTF-16 を固定長バッファへ NUL 終端付きでコピーする（あふれた分は切り捨て）。
pub fn copy_wide(dst: &mut [u16], s: &str) {
    let mut n = 0;
    for unit in s.encode_utf16() {
        if n + 1 >= dst.len() {
            break;
        }
        dst[n] = unit;
        n += 1;
    }
    dst[n] = 0;
}

pub fn loword(v: usize) -> u32 {
    (v & 0xFFFF) as u32
}

pub fn hiword(v: usize) -> u32 {
    ((v >> 16) & 0xFFFF) as u32
}
