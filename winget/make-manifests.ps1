# winget-pkgs へ提出するマニフェストを生成する。
#   .\winget\make-manifests.ps1                       # Cargo.toml の version と bin\PowerModeTray.exe の SHA256 から生成
#   .\winget\make-manifests.ps1 -Version 1.0.0 -Sha256 <hash>
# 生成先: winget\out\manifests\9\9enki\PowerModeTray\<version>\
# 提出: wingetcreate submit --token <PAT> winget\out\manifests\9\9enki\PowerModeTray\<version>
param(
    [string]$Version,
    [string]$Sha256,
    [string]$ReleaseDate = (Get-Date -Format 'yyyy-MM-dd')
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot '..\scripts\common.ps1')

$ver = if ($Version) { $Version } else { Get-ProjectVersion }
if (-not $Sha256) {
    $exe = Join-Path $PSScriptRoot '..\bin\PowerModeTray.exe'
    if (-not (Test-Path $exe)) { throw 'bin\PowerModeTray.exe がありません。-Sha256 を指定するか先に build.ps1 を実行してください' }
    $Sha256 = (Get-FileHash $exe -Algorithm SHA256).Hash
}
$Sha256 = $Sha256.ToUpperInvariant()

$id = '9enki.PowerModeTray'
$repo = 'https://github.com/9enki/power-mode-tray'
$url = "$repo/releases/download/v$ver/PowerModeTray.exe"
$outDir = Join-Path $PSScriptRoot "out\manifests\9\9enki\PowerModeTray\$ver"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.version.1.9.0.schema.json
PackageIdentifier: $id
PackageVersion: $ver
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.9.0
"@ | Set-Content (Join-Path $outDir "$id.yaml") -Encoding UTF8

@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.installer.1.9.0.schema.json
PackageIdentifier: $id
PackageVersion: $ver
InstallerType: portable
Commands:
- PowerModeTray
ReleaseDate: $ReleaseDate
Installers:
- Architecture: x64
  InstallerUrl: $url
  InstallerSha256: $Sha256
ManifestType: installer
ManifestVersion: 1.9.0
"@ | Set-Content (Join-Path $outDir "$id.installer.yaml") -Encoding UTF8

@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.defaultLocale.1.9.0.schema.json
PackageIdentifier: $id
PackageVersion: $ver
PackageLocale: en-US
Publisher: 9enki
PublisherUrl: https://github.com/9enki
PublisherSupportUrl: $repo/issues
PackageName: PowerModeTray
PackageUrl: $repo
License: MIT
LicenseUrl: $repo/blob/main/LICENSE
Copyright: Copyright (c) 2026 9enki
ShortDescription: Cycle Windows 11 power modes (Best power efficiency / Balanced / Best performance) from a tray icon or a hotkey.
Description: |-
  A tiny tray app for Windows 11. Left-click the tray icon (or press Ctrl+Alt+P, configurable with --hotkey)
  to cycle the power mode: Best power efficiency -> Balanced -> Best performance. It calls the same power
  overlay API as the Settings app, writes nothing to the registry or disk, needs no admin rights and no extra runtime.
Moniker: powermodetray
Tags:
- battery
- hotkey
- power
- power-mode
- tray
- windows-11
ReleaseNotesUrl: $repo/releases/tag/v$ver
ManifestType: defaultLocale
ManifestVersion: 1.9.0
"@ | Set-Content (Join-Path $outDir "$id.locale.en-US.yaml") -Encoding UTF8

@"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.locale.1.9.0.schema.json
PackageIdentifier: $id
PackageVersion: $ver
PackageLocale: ja-JP
Publisher: 9enki
PackageName: PowerModeTray
License: MIT
ShortDescription: Windows 11 の電源モード（最適な電力効率 / バランス / 最適なパフォーマンス）をトレイアイコンのクリックやホットキーで切り替える最小アプリ。
Description: |-
  トレイアイコンの左クリック、または Ctrl+Alt+P（--hotkey で変更可）で電源モードを
  最適な電力効率 -> バランス -> 最適なパフォーマンス の順に切り替えます。設定アプリと同じ API を呼ぶだけで、
  レジストリやファイルへの書き込みはなく、管理者権限も追加ランタイムも不要です。
ManifestType: locale
ManifestVersion: 1.9.0
"@ | Set-Content (Join-Path $outDir "$id.locale.ja-JP.yaml") -Encoding UTF8

"生成: $outDir"
Get-ChildItem $outDir | Select-Object -ExpandProperty Name
