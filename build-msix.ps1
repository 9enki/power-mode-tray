# Microsoft Store 提出用の MSIX パッケージを作る（Windows SDK の makeappx.exe を使用）。
#   .\build-msix.ps1                                   # 仮の Identity で dist\PowerModeTray_<ver>.msix を作る（構成確認用）
#   .\build-msix.ps1 -IdentityName 12345Genki.PowerModeTray -Publisher 'CN=xxxxxxxx-...' -PublisherDisplayName Genki
# Identity の各値は Partner Center でアプリ名を予約すると発行される。署名は Store が行うので、ここでは署名しない。
param(
    [string]$IdentityName = 'PLACEHOLDER.PowerModeTray',
    [string]$Publisher = 'CN=PLACEHOLDER',
    [string]$PublisherDisplayName = '9enki',
    [string]$OutDir = 'dist',
    [switch]$NoBuild
)
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
. .\scripts\common.ps1

$ver = Get-ProjectVersion
if (-not $NoBuild) {
    & .\build.ps1
    if ($LASTEXITCODE -ne 0) { throw 'ビルドに失敗しました' }
}
if (-not (Test-Path .\bin\PowerModeTray.exe)) { throw 'bin\PowerModeTray.exe がありません' }

$makeappx = Get-WindowsKitTool 'makeappx.exe'
$stage = Join-Path $PSScriptRoot 'obj\msix'
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Force -Path $stage, $OutDir | Out-Null

Copy-Item .\bin\PowerModeTray.exe $stage
& .\msix\make-assets.ps1 -OutDir (Join-Path $stage 'Assets')

(Get-Content .\msix\AppxManifest.xml -Raw -Encoding UTF8) `
    -replace '__IDENTITY_NAME__', $IdentityName `
    -replace '__PUBLISHER__', $Publisher `
    -replace '__PUBLISHER_DISPLAY_NAME__', $PublisherDisplayName `
    -replace '__VERSION__', $ver |
    Set-Content (Join-Path $stage 'AppxManifest.xml') -Encoding UTF8

$out = Join-Path $OutDir "PowerModeTray_$ver.msix"
& $makeappx pack /d $stage /p $out /o
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

"MSIX: $out (" + (Get-Item $out).Length + " bytes)"
if ($IdentityName -like 'PLACEHOLDER*' -or $Publisher -like '*PLACEHOLDER*') {
    Write-Warning 'Identity が仮の値です。Store へ提出するには Partner Center の値を -IdentityName / -Publisher で指定してください。'
}
