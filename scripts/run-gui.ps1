$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
. "$PSScriptRoot\lib.ps1"

Set-Location $root
$binary = Join-Path $root "target\debug\launchpadx.exe"

if (-not (Test-Path -LiteralPath $binary)) {
    & (Get-CargoCommand) build --bin launchpadx --manifest-path (Join-Path $root "Cargo.toml")
    if ($LASTEXITCODE -ne 0) {
        throw "Rust build failed with exit code $LASTEXITCODE."
    }
}

# Bare launch opens the GUI by default; no --gui flag required.
& $binary @args
