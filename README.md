# Codex Local Launcher

Launch [Codex](https://github.com/openai/codex) against an Ollama-compatible endpoint and manage the Codex desktop app config from a small local UI.

[![CI](https://github.com/codex-local-launcher/codex-local-launcher/actions/workflows/ci.yml/badge.svg)](https://github.com/codex-local-launcher/codex-local-launcher/actions/workflows/ci.yml)

---

## Prerequisites

- **Rust / Cargo** – `rustc` 1.75+ (install via [rustup](https://rustup.rs/))
- **Node.js** 18+ (for the web UI)
- **Windows 10+** (tested on Windows; other platforms may work for the CLI)
- **Codex CLI or Desktop App** – installed and discoverable on PATH (or set `codexCommand` in config)
- An **Ollama server** or any OpenAI-compatible endpoint running on your network

## Quick Start (GUI)

The easiest way to use the launcher is the bundled GUI:

1. **Copy and edit the config:**
   ```powershell
   Copy-Item config.example.json config.json
   # Edit config.json with your Ollama IP / API key
   notepad config.json
   ```

2. **Run the launcher:**
   ```powershell
   .\run-gui.cmd
   ```

   This script runs a build check first (Rust + web UI) and launches `target\release\codex-local-launcher.exe --gui`.

3. **Use the UI:**
   - **Launch tab** – select a model, launch Codex.
   - **Models tab** – discover, cache, and select Ollama models.
   - **Settings tab** – configure provider, API key mode, Codex command path, etc.
   - **Logs tab** – view real-time launcher logs.
   - **About tab** – version and help info.

## CLI Usage (headless / automation)

You can also operate the launcher entirely from the command line.

### Building & Running

```powershell
# Build everything (Rust + web UI)
.\build.cmd

# Or the release binary directly
cargo build --release

# Run the CLI
.\scripts\run-cli.ps1 --config config.json
```

The CLI binary is `target\debug\codex-local-launcher.exe` (debug) or `target\release\codex-local-launcher.exe` (release).

### Refreshing Models

```powershell
# Discover and cache models from your Ollama endpoint
.\scripts\refresh-models.ps1

# Or with a specific config
.\scripts\run-cli.ps1 --config config.json --refresh-models
```

### Launching Codex from CLI

```powershell
.\launch-codex.cmd
# or
.\scripts\launch-codex.ps1
```

These scripts read `config.json`, set `OPENAI_BASE_URL` and `OPENAI_API_KEY` environment variables, then launch Codex. They auto-detect the Codex executable (including Microsoft Store packaged apps).

### Restoring Previous Config

The launcher backs up the previous Codex root model/provider before applying its own. Restore it via:

```powershell
.\scripts\restore.ps1
```

## Config

Local settings live in `config.json` (gitignored). Public defaults are in `config.example.json`.

| Field | Type | Description |
|---|---|---|
| `ollamaIp` | string | IP or hostname of the Ollama server |
| `ollamaPort` | int | Port (default `11434`) |
| `ollamaScheme` | string | `http` or `https` (default `http`) |
| `apiKey` | string | API key for the Ollama-compatible endpoint |
| `persistCodexConfig` | bool | Write a provider into `~/.codex/config.toml` (default `true`) |
| `discoverOllamaModels` | bool | Auto-fetch model list on startup (default `true`) |
| `codexModel` | string | Override Codex model; leave empty to use UI selection |
| `codexProviderId` | string | Provider identifier written to Codex config |
| `codexProviderName` | string | Display name for the provider |
| `codexApiKeyMode` | string | `experimentalBearerToken` / `envKey` / `none` (see below) |
| `codexConfigPath` | string | Override default `~/.codex/config.toml` path |
| `codexCommand` | string | Full path to Codex executable; leave empty to auto-detect |
| `codexArgs` | array | Extra arguments passed to Codex |
| `workingDirectory` | string | Working directory for launched Codex processes |

### `codexApiKeyMode` Options

- **`experimentalBearerToken`** – Writes the configured `apiKey` directly into the Codex provider config.
- **`envKey`** – Sets `env_key = "OPENAI_API_KEY"` so Codex reads from the environment variable instead.
- **`none`** – Writes no auth key; the endpoint must allow unauthenticated requests.

## Build System

### `build.cmd` – Full Build

Runs a conditional build script (`scripts\build.ps1`) that invokes `cargo build --bins`. This compiles the Rust binaries (GUI and CLI).

### `build-check.ps1` – Smart Build Check

Used by `run-gui.cmd`. Checks whether the release binary or web UI bundle is stale and rebuilds only what's needed:
- Compares source file timestamps against the existing binary and web bundle.
- Builds Rust if any `.rs` source is newer than `target\release\codex-local-launcher.exe`.
- Runs `npm run build` in `web/` if any web source is newer than `web\dist\assets\index.js`.
- Stages the web build output and `config.json` next to the release binary.

### `run-gui.cmd` – Launch GUI

1. Runs `build-check.ps1` to ensure everything is built.
2. Launches `target\release\codex-local-launcher.exe --gui`.

### `build.rs` – Cargo Build Script

Runs `npm run build` automatically during `cargo build` if `web/dist/index.html` is missing. Ensures the web UI is always present alongside the binary.

## Testing

```powershell
# Format check, unit tests, and clippy
.\test.cmd
```

This runs:
```powershell
cargo fmt -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

## Diagnostics

```powershell
# Health-check diagnostic script
.\diagnose.ps1
```

Tests RPC endpoint, Ollama health, and Ollama model list. Helpful for troubleshooting connectivity issues.

## Security

> **API keys are stored in plaintext** in `config.json` and `~/.codex/config.toml`. Restrict file permissions on multi-user systems. Consider using `envKey` mode or an external secret manager for sensitive deployments.

## Project Structure

```
+-- src/                  # Rust source (GUI + CLI binaries)
¦   +-- main.rs           # CLI entry point
¦   +-- web_backend.rs    # HTTP server + UI serving
+-- web/                  # Vite + React + Tailwind web UI
¦   +-- src/              # React components & pages
¦   +-- dist/             # Built output (gitignored)
¦   +-- package.json      # Frontend deps
+-- scripts/              # PowerShell scripts
¦   +-- lib.ps1           # Shared helpers (Get-CargoCommand)
¦   +-- run-gui.ps1       # GUI run script
¦   +-- run-cli.ps1       # CLI run script
¦   +-- refresh-models.ps1
¦   +-- restore.ps1
¦   +-- build.ps1
+-- build-check.ps1       # Smart build checker (used by run-gui.cmd)
+-- build.rs              # Cargo build script (auto-builds web UI)
+-- launch-codex.ps1      # Standalone Codex launcher
+-- diagnose.ps1          # Health check diagnostic
+-- config.example.json   # Public config template
+-- config.json           # Local config (gitignored)
+-- run-gui.cmd           # Windows launcher for the GUI
+-- build.cmd             # Windows launcher for cargo build
+-- test.cmd              # Windows launcher for cargo test
+-- docs/
    +-- architecture.md   # Architecture notes
```

## Resources

- [Architecture docs](docs/architecture.md)
- [Contributing guide](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)