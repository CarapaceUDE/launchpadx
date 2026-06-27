#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

WEB_DIR="$ROOT/web"

if [[ ! -d "$WEB_DIR/node_modules" ]]; then
  echo "Installing web UI dependencies (first-time setup)..."
  if [[ -f "$WEB_DIR/package-lock.json" ]]; then
    (cd "$WEB_DIR" && npm ci)
  else
    (cd "$WEB_DIR" && npm install)
  fi
fi

echo "Building web UI..."
(cd "$WEB_DIR" && npm run build)

echo "Building Rust binaries..."
cargo build --bins

echo "Build complete."