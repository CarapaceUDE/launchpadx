#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Always compile first so we never invoke a stale binary that ignores --build-check
# and accidentally falls through to the default Codex-launch code path.
exec cargo run --release --bin codex-launchpad -- --build-check