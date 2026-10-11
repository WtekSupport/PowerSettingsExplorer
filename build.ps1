# Clean release build into .\target\ (needs Rust in PATH)
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
$env:CARGO_TARGET_DIR = Join-Path $PSScriptRoot 'target'

$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    $local = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
    if (Test-Path $local) { $cargo = Get-Item $local }
}
if (-not $cargo) {
    Write-Error 'Rust/cargo not found. Install: https://www.rust-lang.org/tools/install'
}

if (Test-Path $env:CARGO_TARGET_DIR) {
    Write-Host "Removing old target: $($env:CARGO_TARGET_DIR)"
    Remove-Item -Recurse -Force $env:CARGO_TARGET_DIR
}
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $PSScriptRoot 'scripts\__pycache__')

python (Join-Path $PSScriptRoot 'scripts\generate_catalog.py')
& $cargo.Source build --release
& (Join-Path $PSScriptRoot 'scripts\verify_build.ps1')

$exe = Join-Path $env:CARGO_TARGET_DIR 'release\power-settings-explorer.exe'
Write-Host ""
Write-Host "Run this build:"
Write-Host "  $exe"
