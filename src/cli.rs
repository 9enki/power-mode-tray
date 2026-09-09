//! 起動引数の解釈。

use crate::hotkey::HotkeySpec;

pub const USAGE: &str = "使い方: PowerModeTray.exe [--hotkey <キー>]\n\n\
  --hotkey <キー>   モードを 1 つ進めるホットキー（既定: Ctrl+Alt+P）\n\
                    修飾キー: Ctrl / Alt / Shift / Win、キー: A-Z / 0-9 / F1-F24 など\n\
                    例: Ctrl+Alt+P, Ctrl+Shift+F12, Win+Alt+M, none（無効）\n\
  --help            この説明を表示";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// hotkey が None ならホットキー無効
    Run { hotkey: Option<HotkeySpec> },
    Help,
}

/// 引数（プログラム名を除く）を解釈する。失敗なら Err(理由)。
pub fn parse(args: &[String]) -> Result<Command, String> {
    let mut hotkey = Some(HotkeySpec::default_spec());
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        let value: &str;
        if arg.eq_ignore_ascii_case("--help") || arg == "-h" || arg == "/?" {
            return Ok(Command::Help);
        } else if arg.get(..9).map_or(false, |p| p.eq_ignore_ascii_case("--hotkey=")) {
            value = &arg[9..];
        } else if arg.eq_ignore_ascii_case("--hotkey") {
            i += 1;
            value = args.get(i).map(|s| s.as_str()).ok_or_else(|| "--hotkey にはキーの組み合わせを指定してください。".to_string())?;
        } else {
            return Err(format!("不明な引数です: {}", arg));
        }
        hotkey = HotkeySpec::parse(value)?;
        i += 1;
    }
    Ok(Command::Run { hotkey })
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
    fn 異常系() {
        assert!(parse(&args(&["--hotkey"])).is_err()); // 値なし
        assert!(parse(&args(&["--hotkey", "P"])).is_err()); // 値が不正
        assert!(parse(&args(&["--bogus"])).is_err()); // 不明なオプション
        assert!(parse(&args(&["Ctrl+Alt+P"])).is_err()); // オプション名なし
        assert!(parse(&args(&["--hotkey="])).is_err()); // 空
    }
}
