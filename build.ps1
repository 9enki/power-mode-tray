# PowerModeTray をリリースビルドして bin\PowerModeTray.exe に置く。
#   WSL から : powershell.exe -NoProfile -ExecutionPolicy Bypass -File build.ps1
#   Windows  : .\build.ps1
# 必要なもの: Rust（rustup、stable-x86_64-pc-windows-msvc）と Visual Studio Build Tools の C++ ワークロード。
# バージョンは Cargo.toml の version が唯一のソース。
# 起動中の PowerModeTray があると exe を上書きできないので、先に終了するか install.ps1 を使う。
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
. .\scripts\common.ps1

$cargo = Get-Cargo
$ver = Get-ProjectVersion
$env:CARGO_TARGET_DIR = Get-CargoTargetDir

# cargo は進捗を stderr に出す。Windows PowerShell 5.1 ではそれがエラー扱いになるので文字列として流す
$ErrorActionPreference = 'Continue'
& $cargo build --release 2>&1 | ForEach-Object { "$_" }
$ErrorActionPreference = 'Stop'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

New-Item -ItemType Directory -Force -Path bin | Out-Null
Copy-Item (Join-Path $env:CARGO_TARGET_DIR 'release\PowerModeTray.exe') bin\PowerModeTray.exe -Force
"PowerModeTray $ver -> " + (Get-Item bin\PowerModeTray.exe).FullName + " (" + (Get-Item bin\PowerModeTray.exe).Length + " bytes)"
