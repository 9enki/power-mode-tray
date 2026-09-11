<#
.SYNOPSIS
  PowerModeTray を自分用にインストール・更新・アンインストールする。
.DESCRIPTION
  ビルド → 起動中のインスタンスを停止 → %LOCALAPPDATA%\Programs\PowerModeTray\ に上書きコピー → 起動。
  「^」の外に常時表示する設定（システムトレイの昇格）は exe のパス単位で保存されるため、
  毎回同じ場所へ上書きすることで更新後も維持される。

  自動起動は -Startup / -NoStartup で指定する。どちらも指定しない場合、初回インストール時のみ対話で尋ね、
  更新時は今の設定を保ったままにする。設定はアプリの右クリックメニューからいつでも変更できる。
.PARAMETER Hotkey
  起動時に渡す --hotkey の値（例: Ctrl+Shift+F9、none）。省略時は今の自動起動設定の値、それも無ければ既定の Ctrl+Alt+P。
.PARAMETER Startup
  ログイン時に自動起動する。
.PARAMETER NoStartup
  ログイン時に自動起動しない。
.PARAMETER NoBuild
  ビルドを省略し、既存の bin\PowerModeTray.exe をそのまま配置する。
.PARAMETER NoLaunch
  配置後に起動しない。
.PARAMETER Uninstall
  アプリを停止し、インストール先フォルダーと自動起動の設定を削除する。
.EXAMPLE
  .\install.ps1                                 # ビルドして入れ替え、起動（更新もこれ 1 回）
  .\install.ps1 -Startup                        # 自動起動を有効にして入れ替え
  .\install.ps1 -NoStartup                      # 自動起動を無効にして入れ替え
  .\install.ps1 -Hotkey Ctrl+Shift+F9 -Startup  # ホットキーを変えて自動起動を有効化
  .\install.ps1 -Uninstall
#>
param(
    [string]$Hotkey,
    [switch]$Startup,
    [switch]$NoStartup,
    [switch]$NoBuild,
    [switch]$NoLaunch,
    [switch]$Uninstall
)
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

if ($Startup -and $NoStartup) { throw '-Startup と -NoStartup は同時に指定できません' }

$installDir = Join-Path $env:LOCALAPPDATA 'Programs\PowerModeTray'
$target = Join-Path $installDir 'PowerModeTray.exe'
$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$approvedKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run'
$valueName = 'PowerModeTray'

function Stop-App {
    $procs = @(Get-Process PowerModeTray -ErrorAction SilentlyContinue)
    if ($procs.Count -gt 0) {
        $procs | Stop-Process -Force
        Start-Sleep -Milliseconds 800
        "停止: PowerModeTray ($($procs.Count) プロセス)"
    }
}

function Get-StartupCommand {
    (Get-ItemProperty $runKey -Name $valueName -ErrorAction SilentlyContinue).$valueName
}

function Get-StartupArgs {
    # "C:\...\PowerModeTray.exe" --hotkey X から引数部分だけを取り出す
    $command = Get-StartupCommand
    if (-not $command) { return '' }
    if ($command -match '^\s*"[^"]*"\s*(.*)$') { return $Matches[1].Trim() }
    return ''
}

function Set-Startup {
    param([bool]$Enabled, [string]$AppArgs)
    if ($Enabled) {
        $command = if ($AppArgs) { "`"$target`" $AppArgs" } else { "`"$target`"" }
        New-Item -Path $runKey -Force | Out-Null
        Set-ItemProperty -Path $runKey -Name $valueName -Value $command -Type String
        # タスク マネージャーで無効にされていた記録があれば消して有効に戻す
        if (Get-ItemProperty $approvedKey -Name $valueName -ErrorAction SilentlyContinue) {
            Remove-ItemProperty -Path $approvedKey -Name $valueName -ErrorAction SilentlyContinue
        }
        "自動起動: 有効  ($command)"
    } else {
        if (Get-StartupCommand) {
            Remove-ItemProperty -Path $runKey -Name $valueName
            "自動起動: 無効"
        }
    }
}

if ($Uninstall) {
    Stop-App
    Set-Startup -Enabled $false -AppArgs '' | Out-Null
    if (Get-StartupCommand) { Write-Warning '自動起動の設定を削除できませんでした' } else { "削除: 自動起動の設定" }
    # 以前のバージョンがスタートアップ フォルダーに置いていたショートカット
    $legacy = Join-Path ([Environment]::GetFolderPath('Startup')) 'PowerModeTray.lnk'
    if (Test-Path $legacy) { Remove-Item $legacy; "削除: $legacy" }
    if (Test-Path $installDir) { Remove-Item $installDir -Recurse; "削除: $installDir" }
    "アンインストール完了。"
    return
}

if (-not $NoBuild) {
    Stop-App | Out-Null   # 起動中だと bin\PowerModeTray.exe を上書きできない
    & .\build.ps1
    if ($LASTEXITCODE -ne 0) { throw 'ビルドに失敗しました' }
}
if (-not (Test-Path .\bin\PowerModeTray.exe)) { throw 'bin\PowerModeTray.exe がありません。先に build.ps1 を実行してください' }

$firstInstall = -not (Test-Path $target)
$startupWasOn = [bool](Get-StartupCommand)
# 起動引数。-Hotkey 指定 > 今の自動起動設定の値 > なし（既定 Ctrl+Alt+P）
$appArgs = if ($Hotkey) { "--hotkey $Hotkey" } else { Get-StartupArgs }

Stop-App
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item .\bin\PowerModeTray.exe $target -Force
"配置: $target"

# 自動起動をどうするか決める
$enableStartup = $startupWasOn
if ($Startup) {
    $enableStartup = $true
} elseif ($NoStartup) {
    $enableStartup = $false
} elseif ($firstInstall) {
    try {
        $answer = $Host.UI.PromptForChoice(
            'PowerModeTray',
            'Windows へのサインイン時に自動起動しますか？（あとからトレイアイコンの右クリックメニューで変更できます）',
            @([System.Management.Automation.Host.ChoiceDescription]::new('はい(&Y)', '自動起動する'),
              [System.Management.Automation.Host.ChoiceDescription]::new('いいえ(&N)', '自動起動しない')),
            1)
        $enableStartup = ($answer -eq 0)
    } catch {
        # 非対話（CI やリダイレクト実行）では既定の「しない」のまま進む
        Write-Warning '対話で確認できないため、自動起動は無効のままにします（-Startup で有効化できます）'
        $enableStartup = $false
    }
}
Set-Startup -Enabled $enableStartup -AppArgs $appArgs

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
