// バージョン情報リソースとアプリマニフェスト（管理者権限不要・高 DPI 対応）を exe に埋め込む。
// Windows SDK の rc.exe を embed-resource が探して使う。
use std::{env, fs, path::PathBuf};

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let mut parts = version.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    let (major, minor, patch) = (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    );

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("app.manifest");
    let manifest_dst = out_dir.join("app.manifest");
    fs::copy(&manifest_src, &manifest_dst).expect("app.manifest をコピーできません");
    let manifest_path = manifest_dst.to_string_lossy().replace('\\', "\\\\");

    let rc = format!(
        r#"#pragma code_page(65001)
1 24 "{manifest_path}"
1 VERSIONINFO
FILEVERSION {major},{minor},{patch},0
PRODUCTVERSION {major},{minor},{patch},0
FILEOS 0x40004L
FILETYPE 0x1L
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "041104B0"
    BEGIN
      VALUE "CompanyName", "9enki"
      VALUE "FileDescription", "PowerModeTray"
      VALUE "FileVersion", "{version}.0"
      VALUE "InternalName", "PowerModeTray"
      VALUE "LegalCopyright", "Copyright (c) 2026 9enki"
      VALUE "OriginalFilename", "PowerModeTray.exe"
      VALUE "ProductName", "PowerModeTray"
      VALUE "ProductVersion", "{version}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x411, 1200
  END
END
"#
    );
    let rc_path = out_dir.join("app.rc");
    fs::write(&rc_path, rc).expect("app.rc を書けません");
    embed_resource::compile(&rc_path, embed_resource::NONE)
        .manifest_optional()
        .expect("リソースのコンパイルに失敗");

    // リンカーが既定で作る側のマニフェストを止め、埋め込んだものだけにする
    println!("cargo:rustc-link-arg-bins=/MANIFEST:NO");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=app.manifest");
}
