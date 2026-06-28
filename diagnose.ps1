[CmdletBinding()]
param(
    [string]$ConfigPath = $(Join-Path $PSScriptRoot "config.json")
)

$ErrorActionPreference = "Stop"

$root = if ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }
$releaseBinary = Join-Path $root "target\release\codex-launchpad.exe"
$debugBinary = Join-Path $root "target\debug\codex-launchpad.exe"

if (Test-Path -LiteralPath $releaseBinary) {
    & $releaseBinary --diagnose --config $ConfigPath
    exit $LASTEXITCODE
}

if (Test-Path -LiteralPath $debugBinary) {
    & $debugBinary --diagnose --config $ConfigPath
    exit $LASTEXITCODE
}

Push-Location $root
try {
    cargo run --release -- --diagnose --config $ConfigPath
    exit $LASTEXITCODE
} finally {
    Pop-Location
}