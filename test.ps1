# ユニットテスト（ホットキー・引数の解釈、節約機能アイコンの選択）を実行する。
#   WSL から : powershell.exe -NoProfile -ExecutionPolicy Bypass -File test.ps1
#   Windows  : .\test.ps1
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
. .\scripts\common.ps1

$cargo = Get-Cargo
$env:CARGO_TARGET_DIR = Get-CargoTargetDir
$ErrorActionPreference = 'Continue'   # cargo の stderr 出力をエラー扱いにしない
& $cargo test 2>&1 | ForEach-Object { "$_" }
exit $LASTEXITCODE
