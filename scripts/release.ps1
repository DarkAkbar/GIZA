# Build release Windows signé pour l'updater Tauri
$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

$keyPath = Join-Path "src-tauri" "giza-updater.key"
if (-not (Test-Path $keyPath)) {
    Write-Host "Clé absente. Génération (CI)…" -ForegroundColor Yellow
    $env:CI = "true"
    npx tauri signer generate -w $keyPath -f --ci
}

$env:TAURI_SIGNING_PRIVATE_KEY_PATH = $keyPath
Write-Host "Build Giza (release)…" -ForegroundColor Cyan
npm run build:win

Write-Host "Terminé. Voir src-tauri\target\release\bundle\" -ForegroundColor Green
