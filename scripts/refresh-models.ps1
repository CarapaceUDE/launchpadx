$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
. "$PSScriptRoot\lib.ps1"

& (Get-CargoCommand) run --bin codex-launchpad -- --config config.json --refresh-models
