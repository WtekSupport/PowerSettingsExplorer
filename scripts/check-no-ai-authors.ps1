# Fails if git history contains Cursor / AI attribution trailers.
$ErrorActionPreference = 'Stop'

$pattern = '(?im)^(Co-authored-by:\s*Cursor(\s|<|$)|Co-authored-by:.*<cursoragent@cursor\.com>|Made-with:\s*Cursor|Made with Cursor|Generated-by:\s*(Cursor|ChatGPT|Claude|Copilot|Gemini|AI)|Assisted-by:\s*(Cursor|ChatGPT|Claude|Copilot|Gemini|AI))'

$range = if ($args.Count -ge 1) { $args[0] } else { 'HEAD' }
$log = git log --format='%H%n%an <%ae>%n%B---END---' $range 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error $log
    exit 1
}

$hits = [regex]::Matches([string]$log, $pattern)
if ($hits.Count -gt 0) {
    Write-Host 'ERROR: AI/Cursor attribution found in git history:' -ForegroundColor Red
    $hits | ForEach-Object { Write-Host ("  - " + $_.Value.Trim()) }
    Write-Host ''
    Write-Host 'Remove these trailers before pushing to GitHub.'
    exit 1
}

$authors = git log --format='%an <%ae>' $range | Sort-Object -Unique
Write-Host 'OK: no AI attribution trailers.'
Write-Host 'Authors:'
$authors | ForEach-Object { Write-Host ("  - " + $_) }
exit 0
