Add-Type -AssemblyName System.Drawing
$out = $PSScriptRoot

function Color($hex, $a = 255) { [System.Drawing.Color]::FromArgb($a, [Convert]::ToInt32($hex.Substring(0, 2), 16), [Convert]::ToInt32($hex.Substring(2, 2), 16), [Convert]::ToInt32($hex.Substring(4, 2), 16)) }

function RoundRect($g, $color, $x, $y, $w, $h, $r) {
    $p = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = [Math]::Min($r * 2, [Math]::Min($w, $h))
    if ($d -le 0) { $p.AddRectangle((New-Object System.Drawing.RectangleF $x, $y, $w, $h)) } else {
        $p.AddArc($x, $y, $d, $d, 180, 90); $p.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
        $p.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90); $p.AddArc($x, $y + $h - $d, $d, $d, 90, 90); $p.CloseFigure()
    }
    $b = New-Object System.Drawing.SolidBrush $color
    $g.FillPath($b, $p); $b.Dispose(); $p.Dispose()
}

function Logo($g, $x, $y, $size) {
    $u = $size / 22
    $sq, $track, $cut = 'EDECE8', 'B9B7B0', '0D0D0C'
    RoundRect $g (Color $sq) ($x + 0.5 * $u) ($y + 0.5 * $u) (21 * $u) (21 * $u) (5 * $u)
    RoundRect $g (Color $track) ($x + 5 * $u) ($y + 14 * $u) (12 * $u) (2 * $u) $u
    RoundRect $g (Color 'D9774F') ($x + 5 * $u) ($y + 14 * $u) (7 * $u) (2 * $u) $u
    RoundRect $g (Color $cut) ($x + 5 * $u) ($y + 6 * $u) (2 * $u) (5 * $u) $u
    RoundRect $g (Color $cut) ($x + 9 * $u) ($y + 8 * $u) (2 * $u) (3 * $u) $u
}

function Canvas($w, $h) {
    $bmp = New-Object System.Drawing.Bitmap $w, $h
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = 'AntiAlias'; $g.TextRenderingHint = 'AntiAliasGridFit'; $g.InterpolationMode = 'HighQualityBicubic'
    $bmp, $g
}

# Imagen lateral de las páginas de bienvenida y final (164x314 al 100 %).
foreach ($k in 1, 2) {
    $w, $h = (164 * $k), (314 * $k)
    $bmp, $g = Canvas $w $h
    $g.Clear((Color '121211'))
    Logo $g (24 * $k) (36 * $k) (40 * $k)
    $font = New-Object System.Drawing.Font 'Segoe UI Semibold', (19 * $k), ([System.Drawing.FontStyle]::Regular), ([System.Drawing.GraphicsUnit]::Pixel)
    $g.DrawString('Tally', $font, (New-Object System.Drawing.SolidBrush (Color 'EDECE8')), (20 * $k), (88 * $k))
    $small = New-Object System.Drawing.Font 'Segoe UI', (10.5 * $k), ([System.Drawing.FontStyle]::Regular), ([System.Drawing.GraphicsUnit]::Pixel)
    $g.DrawString("Tu límite de Claude`nen la bandeja.", $small, (New-Object System.Drawing.SolidBrush (Color '9E9D97')), (21 * $k), (116 * $k))
    $bars = @(@('Sesión', 0.42, '93BFA3'), @('Semana', 0.74, 'F2B04A'), @('Opus', 0.18, '86A8F0'))
    $y = 214
    foreach ($b in $bars) {
        $g.DrawString($b[0], $small, (New-Object System.Drawing.SolidBrush (Color '9E9D97')), (21 * $k), ($y * $k))
        RoundRect $g (Color '2B2B29') (24 * $k) (($y + 17) * $k) (116 * $k) (3 * $k) (1.5 * $k)
        RoundRect $g (Color $b[2]) (24 * $k) (($y + 17) * $k) (116 * $b[1] * $k) (3 * $k) (1.5 * $k)
        $y += 30
    }
    $bmp.Save("$out\wizard-$k.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
    $g.Dispose(); $bmp.Dispose()
}

# Imagen pequeña de la cabecera de las páginas interiores (55x55 al 100 %).
foreach ($k in 1, 2) {
    $s = 55 * $k
    $bmp, $g = Canvas $s $s
    $g.Clear((Color '121211'))
    Logo $g (8 * $k) (8 * $k) (39 * $k)
    $bmp.Save("$out\small-$k.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
    $g.Dispose(); $bmp.Dispose()
}

# Icono con varios tamaños en PNG dentro del .ico.
$sizes = 16, 20, 24, 32, 40, 48, 64, 256
$pngs = foreach ($s in $sizes) {
    $bmp, $g = Canvas $s $s
    $g.Clear([System.Drawing.Color]::Transparent)
    Logo $g 0 0 $s
    $ms = New-Object System.IO.MemoryStream
    if ($s -eq 256) { $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png) } else {
        # Tamaños pequeños como DIB de 32 bits: algunos lectores de .ico no aceptan PNG por debajo de 256.
        $bw = New-Object System.IO.BinaryWriter $ms
        $bw.Write([UInt32]40); $bw.Write([Int32]$s); $bw.Write([Int32]($s * 2)); $bw.Write([UInt16]1); $bw.Write([UInt16]32)
        $bw.Write([UInt32]0); $bw.Write([UInt32]($s * $s * 4)); $bw.Write([Int32]0); $bw.Write([Int32]0); $bw.Write([UInt32]0); $bw.Write([UInt32]0)
        for ($y = $s - 1; $y -ge 0; $y--) { for ($x = 0; $x -lt $s; $x++) { $bw.Write($bmp.GetPixel($x, $y).ToArgb()) } }
        $bw.Write((New-Object byte[] ([Math]::Ceiling($s / 32) * 4 * $s)))
    }
    $g.Dispose(); $bmp.Dispose()
    , $ms.ToArray()
}
$f = New-Object System.IO.MemoryStream
$wr = New-Object System.IO.BinaryWriter $f
$wr.Write([UInt16]0); $wr.Write([UInt16]1); $wr.Write([UInt16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
    $s = $sizes[$i] % 256
    $wr.Write([byte]$s); $wr.Write([byte]$s); $wr.Write([byte]0); $wr.Write([byte]0)
    $wr.Write([UInt16]1); $wr.Write([UInt16]32); $wr.Write([UInt32]$pngs[$i].Length); $wr.Write([UInt32]$offset)
    $offset += $pngs[$i].Length
}
foreach ($p in $pngs) { $wr.Write($p) }
[System.IO.File]::WriteAllBytes("$out\tally.ico", $f.ToArray())
