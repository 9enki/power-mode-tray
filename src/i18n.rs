//! 表示文字列。既定では Windows の表示言語に合わせ、対応が無い言語では英語を使う。
//! 起動引数 --lang で明示的に選ぶこともできる。
//!
//! 電源モードの名前は Windows 11 の 設定 > システム > 電源とバッテリー > 電源モード の
//! 表記に合わせてある。日本語と英語以外は機械翻訳で、母語話者による確認は受けていない。

use std::sync::OnceLock;
use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;

/// 画面に出る文字列。`{}` を含むものは [`fill`] で値を埋める。
pub struct Strings {
    pub mode_best_efficiency: &'static str,
    pub mode_balanced: &'static str,
    pub mode_best_performance: &'static str,
    pub mode_unknown: &'static str,

    pub tooltip: &'static str,      // "Power mode: {}"
    pub saver_notice: &'static str,

    pub menu_startup: &'static str,
    pub menu_exit: &'static str,

    pub hotkey_none: &'static str,
    pub hotkey_label: &'static str,         // "Hotkey: {}"
    pub hotkey_label_failed: &'static str,  // "Hotkey: {} (unavailable)"
    pub hotkey_in_use: &'static str,        // "The hotkey {} is already in use..."

    pub notify_title: &'static str,
    pub switch_failed: &'static str,        // "...(error {})"
    pub startup_failed: &'static str,
    pub startup_on: &'static str,
    pub startup_off: &'static str,
    pub window_failed: &'static str,

    pub usage: &'static str,
    pub arg_hotkey_needs_value: &'static str,
    pub arg_lang_needs_value: &'static str,
    pub arg_unknown: &'static str,          // "Unknown argument: {}"
    pub arg_bad_lang: &'static str,         // "Not a language: '{}' ..."
    pub hotkey_empty: &'static str,
    pub hotkey_bad_modifier: &'static str,  // "...: '{}' ..."
    pub hotkey_bad_key: &'static str,       // "...: '{}' ..."
    pub hotkey_needs_modifier: &'static str,
}

/// --lang と --help の説明。どの言語でも同じ形にしてある。
macro_rules! usage {
    ($head:expr, $hotkey:expr, $lang:expr, $help:expr) => {
        concat!(
            $head, "\n\n",
            "  --hotkey <key>  ", $hotkey, "\n",
            "  --lang <lang>   ", $lang, "\n",
            "  --help          ", $help
        )
    };
}

const EN: Strings = Strings {
    mode_best_efficiency: "Best power efficiency",
    mode_balanced: "Balanced",
    mode_best_performance: "Best performance",
    mode_unknown: "Unknown",
    tooltip: "Power mode: {}",
    saver_notice: "Energy Saver is on, so the mode cannot be changed",
    menu_startup: "Start with Windows",
    menu_exit: "Exit",
    hotkey_none: "Hotkey: none",
    hotkey_label: "Hotkey: {}",
    hotkey_label_failed: "Hotkey: {} (unavailable)",
    hotkey_in_use: "The hotkey {} is already in use by another app.",
    notify_title: "Power mode",
    switch_failed: "Could not switch the power mode (error {}).",
    startup_failed: "Could not change the startup setting.",
    startup_on: "PowerModeTray will start with Windows.",
    startup_off: "PowerModeTray will no longer start with Windows.",
    window_failed: "Could not create the window.",
    usage: usage!(
        "Usage: PowerModeTray.exe [--hotkey <key>] [--lang <lang>]",
        "Hotkey that cycles the power mode (default: Ctrl+Alt+P), or none",
        "Interface language (default: follow Windows)",
        "Show this help"
    ),
    arg_hotkey_needs_value: "--hotkey needs a key combination.",
    arg_lang_needs_value: "--lang needs a language.",
    arg_unknown: "Unknown argument: {}",
    arg_bad_lang: "Not a language: '{}'",
    hotkey_empty: "The hotkey is empty.",
    hotkey_bad_modifier: "Not a modifier: '{}' (use Ctrl / Alt / Shift / Win)",
    hotkey_bad_key: "Not a key: '{}' (use A-Z / 0-9 / F1-F24 and more)",
    hotkey_needs_modifier: "Combine the key with Ctrl / Alt / Shift / Win (F1-F24 also work on their own).",
};

const JA: Strings = Strings {
    mode_best_efficiency: "最適な電力効率",
    mode_balanced: "バランス",
    mode_best_performance: "最適なパフォーマンス",
    mode_unknown: "不明",
    tooltip: "電源モード: {}",
    saver_notice: "バッテリー節約機能が有効のため切り替えできません",
    menu_startup: "Windows 起動時に実行",
    menu_exit: "終了",
    hotkey_none: "ホットキー: なし",
    hotkey_label: "ホットキー: {}",
    hotkey_label_failed: "ホットキー: {}（登録失敗）",
    hotkey_in_use: "ホットキー {} は他のアプリが使用中のため登録できませんでした。",
    notify_title: "電源モード",
    switch_failed: "切り替えに失敗しました (エラー {})。",
    startup_failed: "自動起動の設定を変更できませんでした。",
    startup_on: "Windows 起動時に実行します。",
    startup_off: "Windows 起動時に実行しません。",
    window_failed: "ウィンドウを作成できませんでした。",
    usage: usage!(
        "使い方: PowerModeTray.exe [--hotkey <キー>] [--lang <言語>]",
        "電源モードを送るホットキー（既定: Ctrl+Alt+P）。none で無効",
        "表示言語（既定: Windows に従う）",
        "この説明を表示"
    ),
    arg_hotkey_needs_value: "--hotkey にはキーの組み合わせを指定してください。",
    arg_lang_needs_value: "--lang には言語を指定してください。",
    arg_unknown: "不明な引数です: {}",
    arg_bad_lang: "言語として解釈できません: '{}'",
    hotkey_empty: "ホットキーが空です。",
    hotkey_bad_modifier: "修飾キーとして解釈できません: '{}'（Ctrl / Alt / Shift / Win のいずれか）",
    hotkey_bad_key: "キーとして解釈できません: '{}'（A-Z / 0-9 / F1-F24 など）",
    hotkey_needs_modifier: "Ctrl / Alt / Shift / Win のいずれかと組み合わせてください（F1-F24 は単独でも可）。",
};

const ZH_HANS: Strings = Strings {
    mode_best_efficiency: "最佳能效",
    mode_balanced: "平衡",
    mode_best_performance: "最佳性能",
    mode_unknown: "未知",
    tooltip: "电源模式: {}",
    saver_notice: "节电模式已开启，无法切换电源模式",
    menu_startup: "开机时启动",
    menu_exit: "退出",
    hotkey_none: "快捷键: 无",
    hotkey_label: "快捷键: {}",
    hotkey_label_failed: "快捷键: {}（不可用）",
    hotkey_in_use: "快捷键 {} 已被其他应用占用。",
    notify_title: "电源模式",
    switch_failed: "无法切换电源模式（错误 {}）。",
    startup_failed: "无法更改开机启动设置。",
    startup_on: "PowerModeTray 将在开机时启动。",
    startup_off: "PowerModeTray 将不再在开机时启动。",
    window_failed: "无法创建窗口。",
    usage: usage!(
        "用法: PowerModeTray.exe [--hotkey <按键>] [--lang <语言>]",
        "切换电源模式的快捷键（默认: Ctrl+Alt+P）。使用 none 可禁用",
        "界面语言（默认: 跟随 Windows）",
        "显示此帮助"
    ),
    arg_hotkey_needs_value: "--hotkey 需要指定按键组合。",
    arg_lang_needs_value: "--lang 需要指定语言。",
    arg_unknown: "未知参数: {}",
    arg_bad_lang: "无法识别的语言: '{}'",
    hotkey_empty: "快捷键为空。",
    hotkey_bad_modifier: "无法识别的修饰键: '{}'（请使用 Ctrl / Alt / Shift / Win）",
    hotkey_bad_key: "无法识别的按键: '{}'（请使用 A-Z / 0-9 / F1-F24 等）",
    hotkey_needs_modifier: "请与 Ctrl / Alt / Shift / Win 组合使用（F1-F24 可单独使用）。",
};

const ZH_HANT: Strings = Strings {
    mode_best_efficiency: "最佳電源效率",
    mode_balanced: "平衡",
    mode_best_performance: "最佳效能",
    mode_unknown: "不明",
    tooltip: "電源模式: {}",
    saver_notice: "節省電力已開啟，無法切換電源模式",
    menu_startup: "開機時啟動",
    menu_exit: "結束",
    hotkey_none: "快速鍵: 無",
    hotkey_label: "快速鍵: {}",
    hotkey_label_failed: "快速鍵: {}（無法使用）",
    hotkey_in_use: "快速鍵 {} 已被其他應用程式占用。",
    notify_title: "電源模式",
    switch_failed: "無法切換電源模式（錯誤 {}）。",
    startup_failed: "無法變更開機啟動設定。",
    startup_on: "PowerModeTray 將在開機時啟動。",
    startup_off: "PowerModeTray 將不再於開機時啟動。",
    window_failed: "無法建立視窗。",
    usage: usage!(
        "用法: PowerModeTray.exe [--hotkey <按鍵>] [--lang <語言>]",
        "切換電源模式的快速鍵（預設: Ctrl+Alt+P）。使用 none 可停用",
        "介面語言（預設: 跟隨 Windows）",
        "顯示此說明"
    ),
    arg_hotkey_needs_value: "--hotkey 需要指定按鍵組合。",
    arg_lang_needs_value: "--lang 需要指定語言。",
    arg_unknown: "不明的引數: {}",
    arg_bad_lang: "無法識別的語言: '{}'",
    hotkey_empty: "快速鍵是空的。",
    hotkey_bad_modifier: "無法識別的輔助鍵: '{}'（請使用 Ctrl / Alt / Shift / Win）",
    hotkey_bad_key: "無法識別的按鍵: '{}'（請使用 A-Z / 0-9 / F1-F24 等）",
    hotkey_needs_modifier: "請與 Ctrl / Alt / Shift / Win 組合使用（F1-F24 可單獨使用）。",
};

const KO: Strings = Strings {
    mode_best_efficiency: "최적의 전력 효율성",
    mode_balanced: "균형 조정",
    mode_best_performance: "최고 성능",
    mode_unknown: "알 수 없음",
    tooltip: "전원 모드: {}",
    saver_notice: "절전 모드가 켜져 있어 전원 모드를 변경할 수 없습니다",
    menu_startup: "Windows 시작 시 실행",
    menu_exit: "끝내기",
    hotkey_none: "바로 가기 키: 없음",
    hotkey_label: "바로 가기 키: {}",
    hotkey_label_failed: "바로 가기 키: {}(사용 불가)",
    hotkey_in_use: "바로 가기 키 {}은(는) 다른 앱에서 이미 사용 중입니다.",
    notify_title: "전원 모드",
    switch_failed: "전원 모드를 변경할 수 없습니다(오류 {}).",
    startup_failed: "시작 프로그램 설정을 변경할 수 없습니다.",
    startup_on: "PowerModeTray가 Windows 시작 시 실행됩니다.",
    startup_off: "PowerModeTray가 더 이상 Windows 시작 시 실행되지 않습니다.",
    window_failed: "창을 만들 수 없습니다.",
    usage: usage!(
        "사용법: PowerModeTray.exe [--hotkey <키>] [--lang <언어>]",
        "전원 모드를 전환하는 바로 가기 키(기본값: Ctrl+Alt+P). none이면 사용 안 함",
        "인터페이스 언어(기본값: Windows 설정 따름)",
        "이 도움말 표시"
    ),
    arg_hotkey_needs_value: "--hotkey에는 키 조합이 필요합니다.",
    arg_lang_needs_value: "--lang에는 언어가 필요합니다.",
    arg_unknown: "알 수 없는 인수: {}",
    arg_bad_lang: "인식할 수 없는 언어: '{}'",
    hotkey_empty: "바로 가기 키가 비어 있습니다.",
    hotkey_bad_modifier: "인식할 수 없는 보조 키: '{}'(Ctrl / Alt / Shift / Win 사용)",
    hotkey_bad_key: "인식할 수 없는 키: '{}'(A-Z / 0-9 / F1-F24 등 사용)",
    hotkey_needs_modifier: "Ctrl / Alt / Shift / Win과 조합해 주세요(F1-F24는 단독 사용 가능).",
};

const DE: Strings = Strings {
    mode_best_efficiency: "Beste Energieeffizienz",
    mode_balanced: "Ausbalanciert",
    mode_best_performance: "Beste Leistung",
    mode_unknown: "Unbekannt",
    tooltip: "Energiemodus: {}",
    saver_notice: "Der Energiesparmodus ist aktiv, daher lässt sich der Modus nicht ändern",
    menu_startup: "Mit Windows starten",
    menu_exit: "Beenden",
    hotkey_none: "Tastenkombination: keine",
    hotkey_label: "Tastenkombination: {}",
    hotkey_label_failed: "Tastenkombination: {} (nicht verfügbar)",
    hotkey_in_use: "Die Tastenkombination {} wird bereits von einer anderen App verwendet.",
    notify_title: "Energiemodus",
    switch_failed: "Der Energiemodus konnte nicht geändert werden (Fehler {}).",
    startup_failed: "Die Autostart-Einstellung konnte nicht geändert werden.",
    startup_on: "PowerModeTray wird mit Windows gestartet.",
    startup_off: "PowerModeTray wird nicht mehr mit Windows gestartet.",
    window_failed: "Das Fenster konnte nicht erstellt werden.",
    usage: usage!(
        "Verwendung: PowerModeTray.exe [--hotkey <Taste>] [--lang <Sprache>]",
        "Tastenkombination zum Wechseln des Energiemodus (Standard: Ctrl+Alt+P) oder none",
        "Sprache der Oberfläche (Standard: wie Windows)",
        "Diese Hilfe anzeigen"
    ),
    arg_hotkey_needs_value: "--hotkey benötigt eine Tastenkombination.",
    arg_lang_needs_value: "--lang benötigt eine Sprache.",
    arg_unknown: "Unbekanntes Argument: {}",
    arg_bad_lang: "Keine gültige Sprache: '{}'",
    hotkey_empty: "Die Tastenkombination ist leer.",
    hotkey_bad_modifier: "Keine Modifikatortaste: '{}' (Ctrl / Alt / Shift / Win verwenden)",
    hotkey_bad_key: "Keine gültige Taste: '{}' (A-Z / 0-9 / F1-F24 und weitere)",
    hotkey_needs_modifier: "Mit Ctrl / Alt / Shift / Win kombinieren (F1-F24 funktionieren auch allein).",
};

const FR: Strings = Strings {
    mode_best_efficiency: "Meilleure efficacité énergétique",
    mode_balanced: "Utilisation normale",
    mode_best_performance: "Performances optimales",
    mode_unknown: "Inconnu",
    tooltip: "Mode d'alimentation : {}",
    saver_notice: "L'économiseur de batterie est activé, le mode ne peut pas être changé",
    menu_startup: "Démarrer avec Windows",
    menu_exit: "Quitter",
    hotkey_none: "Raccourci : aucun",
    hotkey_label: "Raccourci : {}",
    hotkey_label_failed: "Raccourci : {} (indisponible)",
    hotkey_in_use: "Le raccourci {} est déjà utilisé par une autre application.",
    notify_title: "Mode d'alimentation",
    switch_failed: "Impossible de changer le mode d'alimentation (erreur {}).",
    startup_failed: "Impossible de modifier le démarrage automatique.",
    startup_on: "PowerModeTray démarrera avec Windows.",
    startup_off: "PowerModeTray ne démarrera plus avec Windows.",
    window_failed: "Impossible de créer la fenêtre.",
    usage: usage!(
        "Utilisation : PowerModeTray.exe [--hotkey <touche>] [--lang <langue>]",
        "Raccourci qui change le mode d'alimentation (par défaut : Ctrl+Alt+P) ou none",
        "Langue de l'interface (par défaut : celle de Windows)",
        "Afficher cette aide"
    ),
    arg_hotkey_needs_value: "--hotkey requiert une combinaison de touches.",
    arg_lang_needs_value: "--lang requiert une langue.",
    arg_unknown: "Argument inconnu : {}",
    arg_bad_lang: "Langue non reconnue : '{}'",
    hotkey_empty: "Le raccourci est vide.",
    hotkey_bad_modifier: "Modificateur non reconnu : '{}' (utilisez Ctrl / Alt / Shift / Win)",
    hotkey_bad_key: "Touche non reconnue : '{}' (utilisez A-Z / 0-9 / F1-F24, etc.)",
    hotkey_needs_modifier: "Combinez la touche avec Ctrl / Alt / Shift / Win (F1-F24 fonctionnent seules).",
};

const ES: Strings = Strings {
    mode_best_efficiency: "Mejor eficiencia energética",
    mode_balanced: "Equilibrado",
    mode_best_performance: "Máximo rendimiento",
    mode_unknown: "Desconocido",
    tooltip: "Modo de energía: {}",
    saver_notice: "El ahorro de energía está activado, no se puede cambiar el modo",
    menu_startup: "Iniciar con Windows",
    menu_exit: "Salir",
    hotkey_none: "Método abreviado: ninguno",
    hotkey_label: "Método abreviado: {}",
    hotkey_label_failed: "Método abreviado: {} (no disponible)",
    hotkey_in_use: "Otra aplicación ya usa el método abreviado {}.",
    notify_title: "Modo de energía",
    switch_failed: "No se pudo cambiar el modo de energía (error {}).",
    startup_failed: "No se pudo cambiar la configuración de inicio.",
    startup_on: "PowerModeTray se iniciará con Windows.",
    startup_off: "PowerModeTray ya no se iniciará con Windows.",
    window_failed: "No se pudo crear la ventana.",
    usage: usage!(
        "Uso: PowerModeTray.exe [--hotkey <tecla>] [--lang <idioma>]",
        "Método abreviado que cambia el modo de energía (predeterminado: Ctrl+Alt+P) o none",
        "Idioma de la interfaz (predeterminado: el de Windows)",
        "Mostrar esta ayuda"
    ),
    arg_hotkey_needs_value: "--hotkey necesita una combinación de teclas.",
    arg_lang_needs_value: "--lang necesita un idioma.",
    arg_unknown: "Argumento desconocido: {}",
    arg_bad_lang: "Idioma no reconocido: '{}'",
    hotkey_empty: "El método abreviado está vacío.",
    hotkey_bad_modifier: "Modificador no reconocido: '{}' (use Ctrl / Alt / Shift / Win)",
    hotkey_bad_key: "Tecla no reconocida: '{}' (use A-Z / 0-9 / F1-F24, etc.)",
    hotkey_needs_modifier: "Combine la tecla con Ctrl / Alt / Shift / Win (F1-F24 también funcionan solas).",
};

const PT: Strings = Strings {
    mode_best_efficiency: "Melhor eficiência de energia",
    mode_balanced: "Equilibrado",
    mode_best_performance: "Melhor desempenho",
    mode_unknown: "Desconhecido",
    tooltip: "Modo de energia: {}",
    saver_notice: "A economia de energia está ativada, não é possível mudar o modo",
    menu_startup: "Iniciar com o Windows",
    menu_exit: "Sair",
    hotkey_none: "Atalho: nenhum",
    hotkey_label: "Atalho: {}",
    hotkey_label_failed: "Atalho: {} (indisponível)",
    hotkey_in_use: "O atalho {} já está em uso por outro aplicativo.",
    notify_title: "Modo de energia",
    switch_failed: "Não foi possível mudar o modo de energia (erro {}).",
    startup_failed: "Não foi possível alterar a configuração de inicialização.",
    startup_on: "O PowerModeTray será iniciado com o Windows.",
    startup_off: "O PowerModeTray não será mais iniciado com o Windows.",
    window_failed: "Não foi possível criar a janela.",
    usage: usage!(
        "Uso: PowerModeTray.exe [--hotkey <tecla>] [--lang <idioma>]",
        "Atalho que alterna o modo de energia (padrão: Ctrl+Alt+P) ou none",
        "Idioma da interface (padrão: o do Windows)",
        "Mostrar esta ajuda"
    ),
    arg_hotkey_needs_value: "--hotkey precisa de uma combinação de teclas.",
    arg_lang_needs_value: "--lang precisa de um idioma.",
    arg_unknown: "Argumento desconhecido: {}",
    arg_bad_lang: "Idioma não reconhecido: '{}'",
    hotkey_empty: "O atalho está vazio.",
    hotkey_bad_modifier: "Modificador não reconhecido: '{}' (use Ctrl / Alt / Shift / Win)",
    hotkey_bad_key: "Tecla não reconhecida: '{}' (use A-Z / 0-9 / F1-F24 etc.)",
    hotkey_needs_modifier: "Combine a tecla com Ctrl / Alt / Shift / Win (F1-F24 também funcionam sozinhas).",
};

const IT: Strings = Strings {
    mode_best_efficiency: "Efficienza energetica ottimale",
    mode_balanced: "Bilanciata",
    mode_best_performance: "Prestazioni migliori",
    mode_unknown: "Sconosciuto",
    tooltip: "Modalità di alimentazione: {}",
    saver_notice: "Il risparmio energia è attivo, non è possibile cambiare modalità",
    menu_startup: "Avvia con Windows",
    menu_exit: "Esci",
    hotkey_none: "Tasto di scelta rapida: nessuno",
    hotkey_label: "Tasto di scelta rapida: {}",
    hotkey_label_failed: "Tasto di scelta rapida: {} (non disponibile)",
    hotkey_in_use: "Il tasto di scelta rapida {} è già usato da un'altra app.",
    notify_title: "Modalità di alimentazione",
    switch_failed: "Impossibile cambiare la modalità di alimentazione (errore {}).",
    startup_failed: "Impossibile modificare l'impostazione di avvio.",
    startup_on: "PowerModeTray verrà avviato con Windows.",
    startup_off: "PowerModeTray non verrà più avviato con Windows.",
    window_failed: "Impossibile creare la finestra.",
    usage: usage!(
        "Uso: PowerModeTray.exe [--hotkey <tasto>] [--lang <lingua>]",
        "Tasto che cambia la modalità di alimentazione (predefinito: Ctrl+Alt+P) o none",
        "Lingua dell'interfaccia (predefinita: quella di Windows)",
        "Mostra questa guida"
    ),
    arg_hotkey_needs_value: "--hotkey richiede una combinazione di tasti.",
    arg_lang_needs_value: "--lang richiede una lingua.",
    arg_unknown: "Argomento sconosciuto: {}",
    arg_bad_lang: "Lingua non riconosciuta: '{}'",
    hotkey_empty: "Il tasto di scelta rapida è vuoto.",
    hotkey_bad_modifier: "Modificatore non riconosciuto: '{}' (usa Ctrl / Alt / Shift / Win)",
    hotkey_bad_key: "Tasto non riconosciuto: '{}' (usa A-Z / 0-9 / F1-F24 e altri)",
    hotkey_needs_modifier: "Combina il tasto con Ctrl / Alt / Shift / Win (F1-F24 funzionano anche da soli).",
};

const RU: Strings = Strings {
    mode_best_efficiency: "Оптимальная энергоэффективность",
    mode_balanced: "Сбалансированное",
    mode_best_performance: "Максимальная производительность",
    mode_unknown: "Неизвестно",
    tooltip: "Режим питания: {}",
    saver_notice: "Включена экономия заряда, режим питания изменить нельзя",
    menu_startup: "Запускать вместе с Windows",
    menu_exit: "Выход",
    hotkey_none: "Сочетание клавиш: нет",
    hotkey_label: "Сочетание клавиш: {}",
    hotkey_label_failed: "Сочетание клавиш: {} (недоступно)",
    hotkey_in_use: "Сочетание клавиш {} уже занято другим приложением.",
    notify_title: "Режим питания",
    switch_failed: "Не удалось изменить режим питания (ошибка {}).",
    startup_failed: "Не удалось изменить параметр автозапуска.",
    startup_on: "PowerModeTray будет запускаться вместе с Windows.",
    startup_off: "PowerModeTray больше не будет запускаться вместе с Windows.",
    window_failed: "Не удалось создать окно.",
    usage: usage!(
        "Использование: PowerModeTray.exe [--hotkey <клавиша>] [--lang <язык>]",
        "Сочетание клавиш для смены режима питания (по умолчанию: Ctrl+Alt+P) или none",
        "Язык интерфейса (по умолчанию: как в Windows)",
        "Показать эту справку"
    ),
    arg_hotkey_needs_value: "Для --hotkey нужно указать сочетание клавиш.",
    arg_lang_needs_value: "Для --lang нужно указать язык.",
    arg_unknown: "Неизвестный аргумент: {}",
    arg_bad_lang: "Нераспознанный язык: '{}'",
    hotkey_empty: "Сочетание клавиш пустое.",
    hotkey_bad_modifier: "Нераспознанный модификатор: '{}' (используйте Ctrl / Alt / Shift / Win)",
    hotkey_bad_key: "Нераспознанная клавиша: '{}' (используйте A-Z / 0-9 / F1-F24 и другие)",
    hotkey_needs_modifier: "Сочетайте клавишу с Ctrl / Alt / Shift / Win (F1-F24 работают и отдельно).",
};

/// 表示言語の指定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    /// Windows の表示言語に従う
    Auto,
    En,
    Ja,
    ZhHans,
    ZhHant,
    Ko,
    De,
    Fr,
    Es,
    Pt,
    It,
    Ru,
}

/// --lang で受け付ける名前。先頭が正規の名前で、--help の一覧にも使う。
const LANG_NAMES: &[(Lang, &[&str])] = &[
    (Lang::En, &["en", "en-us", "en-gb", "english"]),
    (Lang::Ja, &["ja", "ja-jp", "japanese"]),
    (Lang::ZhHans, &["zh-hans", "zh", "zh-cn", "zh-sg"]),
    (Lang::ZhHant, &["zh-hant", "zh-tw", "zh-hk", "zh-mo"]),
    (Lang::Ko, &["ko", "ko-kr", "korean"]),
    (Lang::De, &["de", "de-de", "german"]),
    (Lang::Fr, &["fr", "fr-fr", "french"]),
    (Lang::Es, &["es", "es-es", "es-mx", "spanish"]),
    (Lang::Pt, &["pt", "pt-br", "pt-pt", "portuguese"]),
    (Lang::It, &["it", "it-it", "italian"]),
    (Lang::Ru, &["ru", "ru-ru", "russian"]),
];

// LANGID の下位 10 bit（主言語 ID）
const LANG_CHINESE: u16 = 0x04;
const LANG_GERMAN: u16 = 0x07;
const LANG_SPANISH: u16 = 0x0A;
const LANG_FRENCH: u16 = 0x0C;
const LANG_ITALIAN: u16 = 0x10;
const LANG_JAPANESE: u16 = 0x11;
const LANG_KOREAN: u16 = 0x12;
const LANG_PORTUGUESE: u16 = 0x16;
const LANG_RUSSIAN: u16 = 0x19;

impl Lang {
    /// "en" や "zh-Hant" などを解釈する。"auto" は Auto。未知の値なら None。
    pub fn parse(text: &str) -> Option<Lang> {
        let name = text.trim().to_ascii_lowercase();
        if name == "auto" {
            return Some(Lang::Auto);
        }
        LANG_NAMES
            .iter()
            .find(|(_, names)| names.contains(&name.as_str()))
            .map(|(lang, _)| *lang)
    }

    /// Windows の LANGID から表示言語を決める。対応が無ければ英語。
    fn from_langid(langid: u16) -> Lang {
        let primary = langid & 0x3FF;
        let sub = langid >> 10;
        match primary {
            LANG_JAPANESE => Lang::Ja,
            // 繁体字は台湾(1) / 香港(3) / マカオ(5)、簡体字は中国(2) / シンガポール(4)
            LANG_CHINESE => {
                if sub == 1 || sub == 3 || sub == 5 {
                    Lang::ZhHant
                } else {
                    Lang::ZhHans
                }
            }
            LANG_KOREAN => Lang::Ko,
            LANG_GERMAN => Lang::De,
            LANG_FRENCH => Lang::Fr,
            LANG_SPANISH => Lang::Es,
            LANG_PORTUGUESE => Lang::Pt,
            LANG_ITALIAN => Lang::It,
            LANG_RUSSIAN => Lang::Ru,
            _ => Lang::En,
        }
    }

    fn strings(self) -> &'static Strings {
        match self {
            Lang::Auto => Lang::from_langid(unsafe { GetUserDefaultUILanguage() }).strings(),
            Lang::En => &EN,
            Lang::Ja => &JA,
            Lang::ZhHans => &ZH_HANS,
            Lang::ZhHant => &ZH_HANT,
            Lang::Ko => &KO,
            Lang::De => &DE,
            Lang::Fr => &FR,
            Lang::Es => &ES,
            Lang::Pt => &PT,
            Lang::It => &IT,
            Lang::Ru => &RU,
        }
    }
}

/// --lang に指定できる言語名を並べた一文。エラーメッセージに添える。
pub fn lang_list() -> String {
    let names: Vec<&str> = LANG_NAMES.iter().map(|(_, n)| n[0]).collect();
    names.join(", ")
}

static CHOSEN: OnceLock<&'static Strings> = OnceLock::new();

/// 表示言語を決める。[`s`] を最初に呼ぶ前に 1 回だけ呼ぶこと。
pub fn init(lang: Lang) {
    let _ = CHOSEN.set(lang.strings());
}

/// 表示言語に合った文字列。[`init`] が呼ばれていなければ Windows の設定に従う。
pub fn s() -> &'static Strings {
    CHOSEN.get_or_init(|| Lang::Auto.strings())
}

/// テンプレート内の最初の `{}` を値に置き換える。
pub fn fill(template: &str, value: &str) -> String {
    template.replacen("{}", value, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 収録している全言語。テストで網羅するために使う。
    const ALL: &[(&str, &Strings)] = &[
        ("en", &EN), ("ja", &JA), ("zh-Hans", &ZH_HANS), ("zh-Hant", &ZH_HANT), ("ko", &KO),
        ("de", &DE), ("fr", &FR), ("es", &ES), ("pt", &PT), ("it", &IT), ("ru", &RU),
    ];

    #[test]
    fn 値を埋める() {
        assert_eq!(fill("Power mode: {}", "Balanced"), "Power mode: Balanced");
        assert_eq!(fill("no placeholder", "x"), "no placeholder");
        assert_eq!(fill("{} and {}", "a"), "a and {}"); // 置き換えるのは最初の 1 つだけ
    }

    #[test]
    fn 言語名を解釈する() {
        assert_eq!(Lang::parse("en"), Some(Lang::En));
        assert_eq!(Lang::parse("EN-US"), Some(Lang::En));
        assert_eq!(Lang::parse(" ja "), Some(Lang::Ja));
        assert_eq!(Lang::parse("zh-Hant"), Some(Lang::ZhHant));
        assert_eq!(Lang::parse("zh-TW"), Some(Lang::ZhHant));
        assert_eq!(Lang::parse("zh-CN"), Some(Lang::ZhHans));
        assert_eq!(Lang::parse("pt-BR"), Some(Lang::Pt));
        assert_eq!(Lang::parse("auto"), Some(Lang::Auto));
        assert_eq!(Lang::parse("xx"), None);
        assert_eq!(Lang::parse(""), None);
    }

    #[test]
    fn 言語名が重複していない() {
        let mut seen = Vec::new();
        for (_, names) in LANG_NAMES {
            for n in *names {
                assert!(!seen.contains(n), "重複した言語名: {}", n);
                assert_eq!(*n, n.to_ascii_lowercase(), "小文字で書くこと: {}", n);
                seen.push(n);
            }
        }
    }

    #[test]
    fn langid_から言語を選ぶ() {
        assert_eq!(Lang::from_langid(0x0411), Lang::Ja); // ja-JP
        assert_eq!(Lang::from_langid(0x0409), Lang::En); // en-US
        assert_eq!(Lang::from_langid(0x0804), Lang::ZhHans); // zh-CN
        assert_eq!(Lang::from_langid(0x0404), Lang::ZhHant); // zh-TW
        assert_eq!(Lang::from_langid(0x0C04), Lang::ZhHant); // zh-HK
        assert_eq!(Lang::from_langid(0x0412), Lang::Ko); // ko-KR
        assert_eq!(Lang::from_langid(0x0416), Lang::Pt); // pt-BR
        assert_eq!(Lang::from_langid(0x0419), Lang::Ru); // ru-RU
        assert_eq!(Lang::from_langid(0x040B), Lang::En); // fi-FI（未対応なので英語）
    }

    #[test]
    fn 書式指定子の数が全言語で一致する() {
        for (name, t) in ALL {
            for (field, template) in [
                ("tooltip", t.tooltip),
                ("hotkey_label", t.hotkey_label),
                ("hotkey_label_failed", t.hotkey_label_failed),
                ("hotkey_in_use", t.hotkey_in_use),
                ("switch_failed", t.switch_failed),
                ("arg_unknown", t.arg_unknown),
                ("arg_bad_lang", t.arg_bad_lang),
                ("hotkey_bad_modifier", t.hotkey_bad_modifier),
                ("hotkey_bad_key", t.hotkey_bad_key),
            ] {
                assert_eq!(template.matches("{}").count(), 1, "{} の {} に {{}} が 1 つない", name, field);
            }
        }
    }

    #[test]
    fn 値を持たない文字列に書式指定子がない() {
        for (name, t) in ALL {
            for (field, text) in [
                ("mode_best_efficiency", t.mode_best_efficiency),
                ("mode_balanced", t.mode_balanced),
                ("mode_best_performance", t.mode_best_performance),
                ("mode_unknown", t.mode_unknown),
                ("saver_notice", t.saver_notice),
                ("menu_startup", t.menu_startup),
                ("menu_exit", t.menu_exit),
                ("hotkey_none", t.hotkey_none),
                ("notify_title", t.notify_title),
                ("startup_failed", t.startup_failed),
                ("startup_on", t.startup_on),
                ("startup_off", t.startup_off),
                ("window_failed", t.window_failed),
                ("usage", t.usage),
                ("arg_hotkey_needs_value", t.arg_hotkey_needs_value),
                ("arg_lang_needs_value", t.arg_lang_needs_value),
                ("hotkey_empty", t.hotkey_empty),
                ("hotkey_needs_modifier", t.hotkey_needs_modifier),
            ] {
                assert!(!text.contains("{}"), "{} の {} に不要な {{}} がある", name, field);
                assert!(!text.is_empty(), "{} の {} が空", name, field);
            }
        }
    }

    #[test]
    fn 全言語に文字列がそろっている() {
        assert_eq!(ALL.len(), LANG_NAMES.len(), "ALL と LANG_NAMES の件数が違う");
        for (lang, names) in LANG_NAMES {
            // 正規名で引いた Strings が、その言語の strings() と同じ実体であること
            let by_name = Lang::parse(names[0]).expect("正規名が解釈できない");
            assert_eq!(by_name, *lang);
            assert!(std::ptr::eq(by_name.strings(), lang.strings()));
        }
    }

    #[test]
    fn 言語一覧を作れる() {
        let list = lang_list();
        assert!(list.starts_with("en, ja, zh-hans"), "実際の値: {}", list);
        assert_eq!(list.split(", ").count(), LANG_NAMES.len());
    }
}
