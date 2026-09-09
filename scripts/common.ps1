# 各スクリプトが dot-source して使う共通ヘルパー

function Get-Cargo {
    # PATH になければ rustup の既定の場所を見る
    $cmd = Get-Command cargo -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    $default = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
    if (Test-Path $default) { return $default }
    throw 'cargo が見つかりません。https://rustup.rs から Rust（MSVC ターゲット）を入れてください'
}

function Get-ProjectVersion {
    # Cargo.toml の [package] version（形式は 1.2.3）
    $toml = Get-Content (Join-Path $PSScriptRoot '..\Cargo.toml') -Raw
    if ($toml -notmatch '(?m)^version\s*=\s*"(\d+\.\d+\.\d+)"') { throw 'Cargo.toml から version を読めません' }
    return $Matches[1]
}

function Get-CargoTargetDir {
    # UNC パス（\\wsl.localhost など）上ではビルドが遅いので Windows 側の一時フォルダーに出す
    if ($PSScriptRoot -like '\\*') { return (Join-Path $env:TEMP 'power-mode-tray-target') }
    return (Join-Path $PSScriptRoot '..\target')
}

function Get-WindowsKitTool {
    # Windows SDK (Windows Kits) の x64 ツールを最新バージョンから探す
    param([string]$Name)
    $root = 'C:\Program Files (x86)\Windows Kits\10\bin'
    $tool = Get-ChildItem (Join-Path $root "*\x64\$Name") -ErrorAction SilentlyContinue |
        Sort-Object { [version]($_.Directory.Parent.Name) } -Descending | Select-Object -First 1
    if (-not $tool) { throw "Windows SDK の $Name が見つかりません（Windows SDK をインストールしてください）" }
    return $tool.FullName
}
