# Assure cargo/rustup même si le terminal a été ouvert avant l'installation Rust.
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $cargoBin) {
  $env:Path = "$cargoBin;$env:Path"
}

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  Write-Error @"
Cargo introuvable. Installez Rust : https://rustup.rs/
Puis redémarrez Cursor ou exécutez : npm run tauri:dev
"@
  exit 1
}

if (-not (Get-Command link -ErrorAction SilentlyContinue)) {
  Write-Warning @"
link.exe (MSVC) introuvable — la compilation Rust va échouer.
Installez les Build Tools C++ : voir docs/WINDOWS_SETUP.md
"@
}

npm run tauri -- dev
