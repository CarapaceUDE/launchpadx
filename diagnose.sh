#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

CONFIG_PATH="${1:-$ROOT/config.json}"

if [[ "$(uname -s)" == "MINGW"* || "$(uname -s)" == "MSYS"* || "$(uname -s)" == "CYGWIN"* ]]; then
  BIN_NAME="codex-launchpad.exe"
else
  BIN_NAME="codex-launchpad"
fi

RELEASE_BIN="$ROOT/target/release/$BIN_NAME"
DEBUG_BIN="$ROOT/target/debug/$BIN_NAME"

if [[ -x "$RELEASE_BIN" ]]; then
  exec "$RELEASE_BIN" --diagnose --config "$CONFIG_PATH"
elif [[ -x "$DEBUG_BIN" ]]; then
  exec "$DEBUG_BIN" --diagnose --config "$CONFIG_PATH"
else
  exec cargo run --release -- --diagnose --config "$CONFIG_PATH"
fi