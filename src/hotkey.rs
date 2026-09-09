//! "Ctrl+Alt+P" のような文字列をグローバルホットキーの指定に変換する。

// RegisterHotKey の fsModifiers
pub const MOD_ALT: u32 = 0x0001;
pub const MOD_CONTROL: u32 = 0x0002;
pub const MOD_SHIFT: u32 = 0x0004;
pub const MOD_WIN: u32 = 0x0008;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeySpec {
    pub modifiers: u32,
    pub vk: u32,
    /// 正規化した表記（例: Ctrl+Alt+P）
    pub display: String,
}

impl HotkeySpec {
    /// 既定の Ctrl+Alt+P
    pub fn default_spec() -> HotkeySpec {
        HotkeySpec { modifiers: MOD_CONTROL | MOD_ALT, vk: 0x50, display: "Ctrl+Alt+P".to_string() }
    }

    /// "Ctrl+Alt+P" 形式を解釈する。"none" なら Ok(None)（ホットキー無効）。失敗なら Err(理由)。
    pub fn parse(text: &str) -> Result<Option<HotkeySpec>, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err("ホットキーが空です。".to_string());
        }
        if trimmed.eq_ignore_ascii_case("none") {
            return Ok(None);
        }

        let parts: Vec<&str> = trimmed.split('+').map(|p| p.trim()).collect();
        let mut modifiers = 0u32;
        for token in &parts[..parts.len() - 1] {
            match parse_modifier(token) {
                Some(m) => modifiers |= m,
                None => {
                    return Err(format!(
                        "修飾キーとして解釈できません: '{}'（Ctrl / Alt / Shift / Win のいずれか）",
                        token
                    ))
                }
            }
        }

        let key_text = parts[parts.len() - 1];
        let (vk, name) = parse_key(key_text)
            .ok_or_else(|| format!("キーとして解釈できません: '{}'（A-Z / 0-9 / F1-F24 など）", key_text))?;

        let function_key = (VK_F1..=VK_F24).contains(&vk);
        if modifiers == 0 && !function_key {
            return Err("Ctrl / Alt / Shift / Win のいずれかと組み合わせてください（F1-F24 は単独でも可）。".to_string());
        }

        Ok(Some(HotkeySpec { modifiers, vk, display: build_display(modifiers, &name) }))
    }
}

const VK_F1: u32 = 0x70;
const VK_F24: u32 = 0x87;

fn parse_modifier(token: &str) -> Option<u32> {
    match token.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => Some(MOD_CONTROL),
        "alt" => Some(MOD_ALT),
        "shift" => Some(MOD_SHIFT),
        "win" | "windows" => Some(MOD_WIN),
        _ => None,
    }
}

/// キー名 → (仮想キーコード, 正規化した名前)
fn parse_key(token: &str) -> Option<(u32, String)> {
    if token.is_empty() {
        return None;
    }
    let mut chars = token.chars();
    let first = chars.next().unwrap();
    if chars.next().is_none() {
        let c = first.to_ascii_uppercase();
        if c.is_ascii_uppercase() || c.is_ascii_digit() {
            return Some((c as u32, c.to_string()));
        }
        return None;
    }
    if (first == 'F' || first == 'f') && token[1..].bytes().all(|b| b.is_ascii_digit()) {
        let n: u32 = token[1..].parse().ok()?;
        if !(1..=24).contains(&n) {
            return None;
        }
        return Some((VK_F1 + n - 1, format!("F{}", n)));
    }
    let lower = token.to_ascii_lowercase();
    NAMED_KEYS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(&lower))
        .map(|(name, vk)| (*vk, name.to_string()))
}

/// 名前で指定できるキー（修飾キー自体は含めない）。名前は正規化した表記そのもの。
const NAMED_KEYS: &[(&str, u32)] = &[
    ("Space", 0x20),
    ("Tab", 0x09),
    ("Enter", 0x0D),
    ("Escape", 0x1B),
    ("Backspace", 0x08),
    ("Pause", 0x13),
    ("CapsLock", 0x14),
    ("PageUp", 0x21),
    ("PageDown", 0x22),
    ("End", 0x23),
    ("Home", 0x24),
    ("Left", 0x25),
    ("Up", 0x26),
    ("Right", 0x27),
    ("Down", 0x28),
    ("PrintScreen", 0x2C),
    ("Insert", 0x2D),
    ("Delete", 0x2E),
    ("Apps", 0x5D),
    ("Sleep", 0x5F),
    ("NumPad0", 0x60),
    ("NumPad1", 0x61),
    ("NumPad2", 0x62),
    ("NumPad3", 0x63),
    ("NumPad4", 0x64),
    ("NumPad5", 0x65),
    ("NumPad6", 0x66),
    ("NumPad7", 0x67),
    ("NumPad8", 0x68),
    ("NumPad9", 0x69),
    ("Multiply", 0x6A),
    ("Add", 0x6B),
    ("Subtract", 0x6D),
    ("Decimal", 0x6E),
    ("Divide", 0x6F),
    ("NumLock", 0x90),
    ("ScrollLock", 0x91),
    ("BrowserBack", 0xA6),
    ("BrowserForward", 0xA7),
    ("BrowserRefresh", 0xA8),
    ("BrowserStop", 0xA9),
    ("BrowserSearch", 0xAA),
    ("BrowserFavorites", 0xAB),
    ("BrowserHome", 0xAC),
    ("VolumeMute", 0xAD),
    ("VolumeDown", 0xAE),
    ("VolumeUp", 0xAF),
    ("MediaNextTrack", 0xB0),
    ("MediaPreviousTrack", 0xB1),
    ("MediaStop", 0xB2),
    ("MediaPlayPause", 0xB3),
    ("LaunchMail", 0xB4),
    ("LaunchApplication1", 0xB6),
    ("LaunchApplication2", 0xB7),
    ("OemSemicolon", 0xBA),
    ("OemPlus", 0xBB),
    ("OemComma", 0xBC),
    ("OemMinus", 0xBD),
    ("OemPeriod", 0xBE),
    ("OemQuestion", 0xBF),
    ("OemTilde", 0xC0),
    ("OemOpenBrackets", 0xDB),
    ("OemPipe", 0xDC),
    ("OemCloseBrackets", 0xDD),
    ("OemQuotes", 0xDE),
    ("Oem102", 0xE2),
];

fn build_display(modifiers: u32, key_name: &str) -> String {
    let mut s = String::new();
    if modifiers & MOD_CONTROL != 0 {
        s.push_str("Ctrl+");
    }
    if modifiers & MOD_ALT != 0 {
        s.push_str("Alt+");
    }
    if modifiers & MOD_SHIFT != 0 {
        s.push_str("Shift+");
    }
    if modifiers & MOD_WIN != 0 {
        s.push_str("Win+");
    }
    s.push_str(key_name);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parses(text: &str, modifiers: u32, vk: u32, display: &str) {
        let spec = HotkeySpec::parse(text).unwrap_or_else(|e| panic!("'{}' が解釈できない: {}", text, e));
        let spec = spec.unwrap_or_else(|| panic!("'{}' が none 扱いになった", text));
        assert_eq!(spec.modifiers, modifiers, "'{}' の修飾キー", text);
        assert_eq!(spec.vk, vk, "'{}' のキー", text);
        assert_eq!(spec.display, display, "'{}' の表記", text);
    }

    fn fails(text: &str) {
        match HotkeySpec::parse(text) {
            Err(msg) => assert!(!msg.is_empty(), "'{}' のエラー文が空", text),
            Ok(spec) => panic!("'{}' が拒否されず {:?} になった", text, spec),
        }
    }

    #[test]
    fn 正常系() {
        parses("Ctrl+Alt+P", MOD_CONTROL | MOD_ALT, 0x50, "Ctrl+Alt+P");
        parses("ctrl+shift+f12", MOD_CONTROL | MOD_SHIFT, 0x7B, "Ctrl+Shift+F12");
        parses("Win+Alt+M", MOD_WIN | MOD_ALT, 0x4D, "Alt+Win+M"); // 表記は Ctrl, Alt, Shift, Win の順
        parses("Control+Windows+z", MOD_CONTROL | MOD_WIN, 0x5A, "Ctrl+Win+Z");
        parses("Ctrl+Alt+1", MOD_CONTROL | MOD_ALT, 0x31, "Ctrl+Alt+1");
        parses("Ctrl+Alt+Pause", MOD_CONTROL | MOD_ALT, 0x13, "Ctrl+Alt+Pause");
        parses("ctrl+alt+pause", MOD_CONTROL | MOD_ALT, 0x13, "Ctrl+Alt+Pause"); // 名前は正規化
        parses(" ctrl + alt + p ", MOD_CONTROL | MOD_ALT, 0x50, "Ctrl+Alt+P"); // 空白は無視
        parses("F13", 0, 0x7C, "F13"); // F キーは単独でも可
        parses("Ctrl+Alt+VolumeMute", MOD_CONTROL | MOD_ALT, 0xAD, "Ctrl+Alt+VolumeMute");
    }

    #[test]
    fn 無効化() {
        assert_eq!(HotkeySpec::parse("none"), Ok(None));
        assert_eq!(HotkeySpec::parse("NONE"), Ok(None));
        assert_eq!(HotkeySpec::parse(" none "), Ok(None));
    }

    #[test]
    fn 異常系() {
        fails("");
        fails("   ");
        fails("P"); // 修飾キーなし
        fails("Ctrl+Alt"); // キーなし
        fails("Ctrl+Alt+Foo"); // 不明なキー
        fails("Ctrl+Alt+P+Q"); // キーが 2 つ
        fails("Ctrl+Alt+12"); // 数値
        fails("Ctrl+Alt+F25");
        fails("Ctrl+Alt+F0");
        fails("Ctrl+Alt+ShiftKey"); // 修飾キー単体
        fails("Ctrl+Alt+Control");
        fails("Ctrl+Alt+LWin");
        fails("Meta+P"); // 未対応の修飾キー名
        fails("Ctrl+Alt+"); // 末尾が空
    }

    #[test]
    fn 既定値() {
        assert_eq!(HotkeySpec::default_spec(), HotkeySpec::parse("Ctrl+Alt+P").unwrap().unwrap());
    }
}
