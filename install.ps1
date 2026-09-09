<#
.SYNOPSIS
  PowerModeTray を自分用にインストール・更新・アンインストールする。
.DESCRIPTION
  ビルド → 起動中のインスタンスを停止 → %LOCALAPPDATA%\Programs\PowerModeTray\ に上書きコピー → 起動。
  「^」の外に常時表示する設定（システムトレイの昇格）は exe のパス単位で保存されるため、
  毎回同じ場所へ上書きすることで更新後も維持される。
  既定ではスタートアップ登録もレジストリ書き込みも行わない。
.PARAMETER Hotkey
  起動時に渡す --hotkey の値（例: Ctrl+Shift+F9、none）。省略時は前回のスタートアップ設定、それも無ければ既定の Ctrl+Alt+P。
.PARAMETER Startup
  ログイン時に自動起動するショートカットをスタートアップフォルダーに作る（-Hotkey も引数として埋め込む）。
.PARAMETER NoBuild
  ビルドを省略し、既存の bin\PowerModeTray.exe をそのまま配置する。
.PARAMETER NoLaunch
  配置後に起動しない。
.PARAMETER Uninstall
  アプリを停止し、インストール先フォルダーとスタートアップのショートカットを削除する。
.EXAMPLE
  .\install.ps1                              # ビルドして入れ替え、起動（更新もこれ 1 回）
  .\install.ps1 -Hotkey Ctrl+Shift+F9 -Startup  # ホットキーを変えてログイン時に自動起動
  .\install.ps1 -Uninstall
#>
param(
    [string]$Hotkey,
    [switch]$Startup,
    [switch]$NoBuild,
    [switch]$NoLaunch,
    [switch]$Uninstall
)
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

$installDir = Join-Path $env:LOCALAPPDATA 'Programs\PowerModeTray'
$target = Join-Path $installDir 'PowerModeTray.exe'
$shortcut = Join-Path ([Environment]::GetFolderPath('Startup')) 'PowerModeTray.lnk'

function Stop-App {
    $procs = @(Get-Process PowerModeTray -ErrorAction SilentlyContinue)
    if ($procs.Count -gt 0) {
        $procs | Stop-Process -Force
        Start-Sleep -Milliseconds 800
        "停止: PowerModeTray ($($procs.Count) プロセス)"
    }
}

if ($Uninstall) {
    Stop-App
    if (Test-Path $shortcut) { Remove-Item $shortcut; "削除: $shortcut" }
    if (Test-Path $installDir) { Remove-Item $installDir -Recurse; "削除: $installDir" }
    "アンインストール完了。レジストリには何も残っていません。"
    return
}

if (-not $NoBuild) {
    Stop-App | Out-Null   # 起動中だと bin\PowerModeTray.exe を上書きできない
    & .\build.ps1
    if ($LASTEXITCODE -ne 0) { throw 'ビルドに失敗しました' }
}
if (-not (Test-Path .\bin\PowerModeTray.exe)) { throw 'bin\PowerModeTray.exe がありません。先に build.ps1 を実行してください' }

$firstInstall = -not (Test-Path $target)
Stop-App
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item .\bin\PowerModeTray.exe $target -Force
"配置: $target"

# 起動引数。-Hotkey 指定 > 既存ショートカットの引数 > なし（既定 Ctrl+Alt+P）
$appArgs = ''
$shell = New-Object -ComObject WScript.Shell
if ($Hotkey) {
    $appArgs = "--hotkey $Hotkey"
} elseif (Test-Path $shortcut) {
    $appArgs = $shell.CreateShortcut($shortcut).Arguments
}

if ($Startup -or (Test-Path $shortcut)) {
    $lnk = $shell.CreateShortcut($shortcut)
    $lnk.TargetPath = $target
    $lnk.Arguments = $appArgs
    $lnk.WorkingDirectory = $installDir
    $lnk.Description = 'PowerModeTray'
    $lnk.Save()
    "スタートアップ登録: $shortcut $appArgs"
}

if (-not $NoLaunch) {
    if ($appArgs) { Start-Process -FilePath $target -ArgumentList $appArgs } else { Start-Process -FilePath $target }
    Start-Sleep -Milliseconds 1500
    if (Get-Process PowerModeTray -ErrorAction SilentlyContinue) { "起動: $target $appArgs" } else { Write-Warning '起動を確認できませんでした' }
}

if ($firstInstall) {
    ''
    '初回はタスクバー右端の「^」の中に入ります。常時表示にするには'
    '  設定 > 個人用設定 > タスクバー > その他のシステム トレイ アイコン で PowerModeTray をオンにしてください。'
    'この設定は今後 install.ps1 で更新しても維持されます。'
}
