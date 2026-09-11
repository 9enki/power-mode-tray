<#
.SYNOPSIS
  PowerModeTray のロゴ画像を生成する。
.DESCRIPTION
  ロゴは GDI+ の図形描画で作る。画像ファイルをリポジトリに置かずに済み、
  どのサイズでも輪郭がくっきり出る。意匠はトレイアイコンと同じ「ゲージ」。
.PARAMETER OutDir
  MSIX パッケージ用のロゴを書き出す場所。
.PARAMETER StoreLogoDir
  指定するとストア掲載用の大きいロゴもここに書き出す。
#>
param(
    [string]$OutDir = (Join-Path $PSScriptRoot '..\obj\msix\Assets'),
    [string]$StoreLogoDir
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

# AppxManifest.xml の BackgroundColor と合わせること
$ColorFrom = [System.Drawing.Color]::FromArgb(0x2E, 0x7C, 0xF6)
$ColorTo   = [System.Drawing.Color]::FromArgb(0x14, 0x45, 0x9E)

function New-Logo {
    <#
      ゲージのロゴを 1 枚描く。
      $Width と $Height が違う場合（ワイドタイル）は中央に正方形として描く。
      $Padding は図形の周囲に空ける余白の割合。タイルは余白を広めに取る。
    #>
    param([int]$Width, [int]$Height, [double]$Padding = 0.18)

    $bmp = New-Object System.Drawing.Bitmap $Width, $Height
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias

    $full = New-Object System.Drawing.Rectangle 0, 0, $Width, $Height
    $bg = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        $full, $ColorFrom, $ColorTo, [System.Drawing.Drawing2D.LinearGradientMode]::ForwardDiagonal)
    $g.FillRectangle($bg, $full)
    $bg.Dispose()

    # 図形は短辺を基準にした正方形の中に収める
    $s = [Math]::Min($Width, $Height) * (1.0 - $Padding * 2)
    $cx = $Width / 2.0
    $cy = $Height / 2.0 + $s * 0.14   # 目盛りの下に針の軸が来るので少し下げる
    $r = $s * 0.46

    $white = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)

    # 目盛りの弧。0 度が 3 時方向で時計回りなので、190 度から 160 度ぶんで上半分を描く
    $pen = New-Object System.Drawing.Pen ([System.Drawing.Color]::White), ([single]($s * 0.085))
    $pen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $arc = New-Object System.Drawing.RectangleF (
        [single]($cx - $r), [single]($cy - $r), [single]($r * 2), [single]($r * 2))
    $g.DrawArc($pen, $arc, 190, 160)
    $pen.Dispose()

    # 針。右上（300 度）を指して「上げた」状態を表す
    $angle = 300.0 * [Math]::PI / 180.0
    $tipLen = $r * 0.80
    $halfW = $s * 0.052
    $tip = New-Object System.Drawing.PointF (
        [single]($cx + [Math]::Cos($angle) * $tipLen), [single]($cy + [Math]::Sin($angle) * $tipLen))
    $left = New-Object System.Drawing.PointF (
        [single]($cx + [Math]::Cos($angle + [Math]::PI / 2) * $halfW),
        [single]($cy + [Math]::Sin($angle + [Math]::PI / 2) * $halfW))
    $right = New-Object System.Drawing.PointF (
        [single]($cx + [Math]::Cos($angle - [Math]::PI / 2) * $halfW),
        [single]($cy + [Math]::Sin($angle - [Math]::PI / 2) * $halfW))
    $g.FillPolygon($white, @($tip, $left, $right))

    # 針の軸
    $hub = $s * 0.075
    $g.FillEllipse($white, [single]($cx - $hub), [single]($cy - $hub), [single]($hub * 2), [single]($hub * 2))

    $white.Dispose()
    $g.Dispose()
    return $bmp
}

function Save-Logo {
    param([string]$Path, [int]$Width, [int]$Height, [double]$Padding)
    $bmp = New-Logo -Width $Width -Height $Height -Padding $Padding
    $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$tiles = [ordered]@{
    'Square44x44Logo.png'   = @(44, 44)
    'Square71x71Logo.png'   = @(71, 71)
    'Square150x150Logo.png' = @(150, 150)
    'Square310x310Logo.png' = @(310, 310)
    'Wide310x150Logo.png'   = @(310, 150)
    'StoreLogo.png'         = @(50, 50)
}
foreach ($name in $tiles.Keys) {
    $w, $h = $tiles[$name]
    Save-Logo -Path (Join-Path $OutDir $name) -Width $w -Height $h -Padding 0.18
}
"パッケージ用ロゴ: $OutDir"

if ($StoreLogoDir) {
    New-Item -ItemType Directory -Force -Path $StoreLogoDir | Out-Null
    # ストア掲載用。300x300 が基本、1080x1080 は大きく表示される場所で使われる
    foreach ($size in 300, 1080) {
        Save-Logo -Path (Join-Path $StoreLogoDir "store-logo-${size}.png") -Width $size -Height $size -Padding 0.16
    }
    "ストア掲載用ロゴ: $StoreLogoDir"
}
