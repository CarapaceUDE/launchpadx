$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
if (-not (Test-Path (Join-Path $root "target\debug\codex-local-launcher.exe"))) {
    Set-Location (Join-Path $root "..\codex-launchpad")
    npm run build
    Set-Location $root
    cargo build --bin codex-local-launcher --manifest-path (Join-Path $root "Cargo.toml")
}

