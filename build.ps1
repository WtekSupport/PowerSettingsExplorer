# Сборка release (нужен Rust в PATH)
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    $local = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
    if (Test-Path $local) { $cargo = Get-Item $local }
}
if (-not $cargo) {
    Write-Error 'Rust/cargo не найден. Установите: https://www.rust-lang.org/tools/install'
}
& $cargo.Source build --release
Write-Host "Done: $PSScriptRoot\target\release\power-settings-explorer.exe"
