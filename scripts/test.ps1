$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
. "$PSScriptRoot\lib.ps1"

$cargo = Get-CargoCommand
& $cargo fmt -- --check
& $cargo test
& $cargo clippy --all-targets -- -D warnings
