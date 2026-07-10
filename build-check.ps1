$ErrorActionPreference = "Stop"

$root = if ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }

# Always compile first so we never invoke a stale binary that ignores --build-check
# and accidentally falls through to the default Codex-launch code path.
Push-Location $root
try {
    cargo run --release --bin launchpadx -- --build-check
    exit $LASTEXITCODE
} finally {
    Pop-Location
}