# Verify release binary contains expected v0.2 features.
$ErrorActionPreference = "Stop"
$exe = (Resolve-Path (Join-Path $PSScriptRoot "..\target\release\power-settings-explorer.exe") -ErrorAction SilentlyContinue)
if (-not $exe) {
    Write-Error "Missing release exe. Run: .\build.ps1 or cargo build --release"
}
$exe = $exe.Path
$bytes = [System.IO.File]::ReadAllBytes($exe)
function Has([string]$s) {
    $needle = [System.Text.Encoding]::ASCII.GetBytes($s)
    for ($i = 0; $i -le $bytes.Length - $needle.Length; $i++) {
        $ok = $true
        for ($j = 0; $j -lt $needle.Length; $j++) {
            if ($bytes[$i + $j] -ne $needle[$j]) { $ok = $false; break }
        }
        if ($ok) { return $true }
    }
    return $false
}

$checks = @(
    @{ Name = "Version 0.2.0"; Ok = (Has "0.2.0") },
    @{ Name = "Preset saved (EN)"; Ok = (Has "Preset saved") },
    @{ Name = "Preset label (EN)"; Ok = (Has "Preset:") },
    @{ Name = "Builtin preset name"; Ok = (Has "Windows default") },
    @{ Name = "No old visibility boilerplate in binary"; Ok = -not (Has "Visibility in this program") },
    @{ Name = "Presets file format"; Ok = (Has "Power Settings Explorer preset v1") }
)
$all = $true
foreach ($c in $checks) {
    $mark = if ($c.Ok) { "OK" } else { "FAIL" }
    if (-not $c.Ok) { $all = $false }
    Write-Host "$mark  $($c.Name)"
}
$item = Get-Item $exe
Write-Host ""
Write-Host "Exe: $($item.FullName)"
Write-Host "Size: $($item.Length)"
Write-Host "Built: $($item.LastWriteTime)"
if (-not $all) { exit 1 }
