# MSIX 用のロゴ PNG を Windows 標準アイコンフォントのゲージグリフから生成する（画像ファイルをリポジトリに置かないため）。
param(
    [string]$OutDir = (Join-Path $PSScriptRoot '..\obj\msix\Assets')
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$installed = (New-Object System.Drawing.Text.InstalledFontCollection).Families | ForEach-Object { $_.Name }
$family = if ($installed -contains 'Segoe Fluent Icons') { 'Segoe Fluent Icons' }
          elseif ($installed -contains 'Segoe MDL2 Assets') { 'Segoe MDL2 Assets' }
          else { throw 'Segoe Fluent Icons / Segoe MDL2 Assets が見つかりません' }

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$sizes = [ordered]@{
    'Square44x44Logo.png'   = @(44, 44)
    'Square71x71Logo.png'   = @(71, 71)
    'Square150x150Logo.png' = @(150, 150)
    'Square310x310Logo.png' = @(310, 310)
    'Wide310x150Logo.png'   = @(310, 150)
    'StoreLogo.png'         = @(50, 50)
}
$background = [System.Drawing.Color]::FromArgb(0x1F, 0x6F, 0xEB)  # AppxManifest.xml の BackgroundColor と同じ
$glyph = [string][char]0xEC49                                     # ゲージ（針が中央）

foreach ($name in $sizes.Keys) {
    $w, $h = $sizes[$name]
    $bmp = New-Object System.Drawing.Bitmap $w, $h
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAlias
    $g.Clear($background)
    $font = New-Object System.Drawing.Font($family, [single]([math]::Min($w, $h) * 0.62), [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
    $format = New-Object System.Drawing.StringFormat
    $format.Alignment = [System.Drawing.StringAlignment]::Center
    $format.LineAlignment = [System.Drawing.StringAlignment]::Center
    $g.DrawString($glyph, $font, [System.Drawing.Brushes]::White, (New-Object System.Drawing.RectangleF([single]0, [single]0, [single]$w, [single]$h)), $format)
    $g.Dispose(); $font.Dispose(); $format.Dispose()
    $bmp.Save((Join-Path $OutDir $name), [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}
"ロゴ生成 ($family): $OutDir"
