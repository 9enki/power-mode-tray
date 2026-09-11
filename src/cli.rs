//! 起動引数の解釈。

use crate::hotkey::HotkeySpec;
use crate::i18n::{self, Lang};

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// hotkey が None ならホットキー無効
    Run { hotkey: Option<HotkeySpec> },
    Help,
}

/// 表示言語だけを先に取り出す。エラーは本解釈の [`parse`] に任せるため、ここでは無視する。
/// 引数の解釈エラーも表示言語に合わせて出したいので、[`parse`] より前に呼ぶ。
pub fn prescan_lang(args: &[String]) -> Lang {
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        let value = if let Some(v) = arg.get(..7).filter(|p| p.eq_ignore_ascii_case("--lang=")).map(|_| &arg[7..]) {
            v
        } else if arg.eq_ignore_ascii_case("--lang") {
            i += 1;
            match args.get(i) {
                Some(v) => v.as_str(),
                None => return Lang::Auto,
            }
        } else {
            i += 1;
            continue;
        };
        if let Some(lang) = Lang::parse(value) {
            return lang;
        }
        i += 1;
    }
    Lang::Auto
}

/// 引数（プログラム名を除く）を解釈する。失敗なら Err(理由)。
pub fn parse(args: &[String]) -> Result<Command, String> {
    let mut hotkey = Some(HotkeySpec::default_spec());
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg.eq_ignore_ascii_case("--help") || arg == "-h" || arg == "/?" {
            return Ok(Command::Help);
        } else if arg.get(..9).map_or(false, |p| p.eq_ignore_ascii_case("--hotkey=")) {
            hotkey = HotkeySpec::parse(&arg[9..])?;
        } else if arg.eq_ignore_ascii_case("--hotkey") {
            i += 1;
            let value = args.get(i).map(|s| s.as_str()).ok_or_else(|| i18n::s().arg_hotkey_needs_value.to_string())?;
            hotkey = HotkeySpec::parse(value)?;
        } else if arg.get(..7).map_or(false, |p| p.eq_ignore_ascii_case("--lang=")) {
            check_lang(&arg[7..])?;
        } else if arg.eq_ignore_ascii_case("--lang") {
            i += 1;
            let value = args.get(i).map(|s| s.as_str()).ok_or_else(|| i18n::s().arg_lang_needs_value.to_string())?;
            check_lang(value)?;
        } else {
            return Err(i18n::fill(i18n::s().arg_unknown, arg));
        }
        i += 1;
    }
    Ok(Command::Run { hotkey })
}

/// 表示言語の値を検証する。実際の適用は [`prescan_lang`] 側で済んでいる。
fn check_lang(value: &str) -> Result<(), String> {
    match Lang::parse(value) {
        Some(_) => Ok(()),
        None => Err(format!("{} ({})", i18n::fill(i18n::s().arg_bad_lang, value), i18n::lang_list())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn display(cmd: Result<Command, String>) -> Option<String> {
        match cmd {
            Ok(Command::Run { hotkey }) => hotkey.map(|h| h.display),
            other => panic!("Run を期待したが {:?}", other),
        }
    }

    #[test]
    fn 既定はctrl_alt_p() {
        assert_eq!(display(parse(&args(&[]))), Some("Ctrl+Alt+P".to_string()));
    }

    #[test]
    fn ホットキー指定() {
        assert_eq!(display(parse(&args(&["--hotkey", "Ctrl+Shift+F9"]))), Some("Ctrl+Shift+F9".to_string()));
        assert_eq!(display(parse(&args(&["--hotkey=Win+Alt+M"]))), Some("Alt+Win+M".to_string()));
        assert_eq!(display(parse(&args(&["--HOTKEY", "none"]))), None);
        assert_eq!(
            display(parse(&args(&["--hotkey", "Ctrl+Alt+A", "--hotkey", "Ctrl+Alt+B"]))),
            Some("Ctrl+Alt+B".to_string()) // 後勝ち
        );
    }

    #[test]
    fn ヘルプ() {
        assert_eq!(parse(&args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse(&args(&["-h"])), Ok(Command::Help));
        assert_eq!(parse(&args(&["/?"])), Ok(Command::Help));
        assert_eq!(parse(&args(&["--hotkey", "Ctrl+Alt+A", "--help"])), Ok(Command::Help));
    }

    #[test]
    fn 表示言語() {
        use crate::i18n::Lang;
        assert_eq!(prescan_lang(&args(&[])), Lang::Auto);
        assert_eq!(prescan_lang(&args(&["--lang", "en"])), Lang::En);
        assert_eq!(prescan_lang(&args(&["--lang=ja"])), Lang::Ja);
        assert_eq!(prescan_lang(&args(&["--hotkey", "Ctrl+Alt+P", "--lang", "EN"])), Lang::En);
        assert_eq!(prescan_lang(&args(&["--lang", "zh-Hant"])), Lang::ZhHant);
        assert_eq!(prescan_lang(&args(&["--lang", "xx"])), Lang::Auto); // 不正値は parse 側でエラーにする
        assert_eq!(prescan_lang(&args(&["--lang"])), Lang::Auto);
        // --lang があってもホットキーの解釈は変わらない
        assert_eq!(display(parse(&args(&["--lang", "en", "--hotkey", "Ctrl+Shift+F9"]))), Some("Ctrl+Shift+F9".to_string()));
    }

    #[test]
    fn 異常系() {
        assert!(parse(&args(&["--lang"])).is_err()); // 値なし
        assert!(parse(&args(&["--lang", "xx"])).is_err()); // 未対応の言語
        assert!(parse(&args(&["--hotkey"])).is_err()); // 値なし
        assert!(parse(&args(&["--hotkey", "P"])).is_err()); // 値が不正
        assert!(parse(&args(&["--bogus"])).is_err()); // 不明なオプション
        assert!(parse(&args(&["Ctrl+Alt+P"])).is_err()); // オプション名なし
        assert!(parse(&args(&["--hotkey="])).is_err()); // 空
    }
}
