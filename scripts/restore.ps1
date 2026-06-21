$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
. "$PSScriptRoot\lib.ps1"

& (Get-CargoCommand) run --bin codex-local-launcher -- --config config.json --restore
