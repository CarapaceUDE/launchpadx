<div align="center">

<img src="assets/logo.png" alt="Codex Launchpad" width="360" />

# Codex Launchpad

**Point [Codex](https://github.com/openai/codex) at any OpenAI-compatible API and manage providers, models, and launch settings from one desktop app.**

[![CI](https://github.com/CarapaceUDE/codex-launchpad/actions/workflows/ci.yml/badge.svg)](https://github.com/CarapaceUDE/codex-launchpad/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-22c55e?style=plastic&labelColor=555555)](LICENSE)
[![Official builds](https://img.shields.io/badge/Official%20builds-Patreon-f96854?style=plastic&labelColor=555555&logo=patreon&logoColor=white)](https://carapaceai.org/patreon)
[![Rust](https://img.shields.io/badge/Rust-1.75+-f97316?style=plastic&labelColor=555555&logo=rust&logoColor=white)](https://rustup.rs/)
[![React](https://img.shields.io/badge/UI-React-61dafb?style=plastic&labelColor=555555&logo=react&logoColor=black)](web/)
[![Platforms](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-2563eb?style=plastic&labelColor=555555)]()

[**Website**](https://carapaceai.org) · [**Patreon — official builds**](https://carapaceai.org/patreon) · [**Discord**](https://carapaceai.org/discord) · [**Issues**](https://github.com/CarapaceUDE/codex-launchpad/issues)

<br />

<img src="assets/readme-screenshot.png" alt="Codex Launchpad showing Local API provider selection and model picker" width="920" />

<sub>Switch between Codex cloud sign-in and any OpenAI-compatible <code>/v1</code> endpoint, pick a model, and launch.</sub>

</div>

## Features

- **Dual provider modes** — Codex cloud account or route through any OpenAI-compatible API (vLLM, LiteLLM, OpenRouter, your own gateway, etc.)
- **Model discovery** — fetch and cache models from your endpoint's API
- **Codex config sync** — writes and restores `~/.codex/config.toml` safely
- **Desktop GUI + CLI** — full UI or scriptable headless workflows
- **Cross-platform** — Windows, macOS, and Linux builds
- **Dark / light theme**

## Contents

- [Distribution](#distribution)
- [Quick Start](#quick-start-gui)
- [CLI usage](#cli-usage-headless--automation)
- [Config](#config)
- [Build system](#build-system)
- [Testing](#testing)
- [Security](#security)
- [License](#license)

---

## Distribution

| What | License / terms | How to get it |
| ---- | --------------- | ------------- |
| **Source code** | [MIT License](LICENSE) — free for everyone | This repository |
| **Official binaries** | [Official build terms](OFFICIAL_BUILDS.md) — personal use, no redistribution | [Patreon supporters](https://carapaceai.org/patreon) |
| **Self-built binary** | MIT (you compiled from source) | [Build instructions](#quick-start-gui) below |

**Why Patreon for official builds?** Carapace is early-stage and needs supporter revenue to keep building. The source is fully open under MIT, so anyone can compile and run the app for free. Official pre-built binaries are a convenience for [Patreon supporters](https://carapaceai.org/patreon) while we grow.

During this phase, that split is intentional—it doubles as an early-access filter. While things are still rough, we'd rather surface bugs through people who build from source and are comfortable debugging setup: developer-minded early testers who file useful issues and contribute fixes. That keeps us learning from actionable feedback instead of drowning in "it just broke" reports from users who downloaded a binary and expected polish on day one. Once the project is sustainable, publishing builds on [GitHub Releases](https://github.com/CarapaceUDE/codex-launchpad/releases) becomes a priority — but not yet.

Whether you build from source or support us on Patreon, we're happy to offer **limited community support** on our [Discord server](https://carapaceai.org/discord) — setup questions, build trouble, and bug reports are welcome there.

---

## Prerequisites

- **Rust / Cargo** — `rustc` 1.75+ (install via [rustup](https://rustup.rs/))
- **Node.js** 18+ and **npm** (for the web UI — run `cd web && npm ci` once after cloning; `build.rs` installs web dependencies automatically when `web/node_modules` is missing on any OS)
- **Codex CLI or Desktop App** — installed and discoverable on PATH (or set `codexCommand` in config)
- An **OpenAI-compatible API** reachable from your machine (local server, LAN host, or remote gateway)

**GUI builds** also need native webview dependencies (wry/tao):

| Platform | Packages / tools |
| -------- | ---------------- |
| **Linux** | GTK 3 + WebKitGTK — e.g. on Debian/Ubuntu: `sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev pkg-config` |
| **macOS** | Xcode Command Line Tools (`xcode-select --install`) |
| **Windows** | WebView2 (usually preinstalled on Windows 10/11) |

The project targets **Windows, macOS, and Linux**. You can build on any of them for the host OS. Cross-compiling for another OS is supported via `rustup target add` + `cargo build --target <triple>` (see [Build system](#build-system)).

## Quick Start (GUI)

**Official builds:** download the latest binary from [Patreon](https://carapaceai.org/patreon).

**From source:** build and run locally (works on any supported OS):

1. **Copy and edit the config:**
   ```sh
   cp config.example.json config.json
   # Edit config.json with your API host, port, and key
   ```

2. **Build and launch the GUI:**
   ```sh
   cd web && npm ci && npm run build && cd ..
   cargo build --release
   ./target/release/codex-launchpad --gui
   ```

   On Windows the binary is `target\release\codex-launchpad.exe`. If `web/dist/` is missing, `cargo build` runs `npm ci` (when `web/node_modules` is absent) and then `npm run build` via `build.rs`. You can still run `cd web && npm ci` yourself first — that is the most reliable path on a fresh clone.

   **Windows shortcut:** `.\run-gui.cmd` runs a stale-build check and launches the release GUI — convenience only, not required.

3. **Use the UI:**
   - **Launch tab** — select a model, launch Codex.
   - **Models tab** — discover, cache, and select models from your API.
   - **Settings tab** — configure provider, API key mode, Codex command path, etc.
   - **Logs tab** — view real-time launchpad logs.
   - **About tab** — version and help info.

## CLI Usage (headless / automation)

You can also operate the launchpad entirely from the command line on any platform.

### Building & Running

```sh
# Build everything (Rust + web UI)
cd web && npm ci && npm run build && cd ..
cargo build --release

# Run the CLI (same binary as the GUI)
./target/release/codex-launchpad --config config.json
```

The binary lives at `target/debug/codex-launchpad` (debug) or `target/release/codex-launchpad` (release). Add `.exe` on Windows.

### Common CLI commands

```sh
codex-launchpad --refresh-models          # discover and cache models
codex-launchpad --list-models             # print cached models
codex-launchpad --write-config-only       # write ~/.codex/config.toml only
codex-launchpad --launch                  # apply config and launch Codex
codex-launchpad --restore                 # restore previous Codex settings
codex-launchpad --diagnose                # setup and connectivity checks
codex-launchpad --help                    # full flag list
```

Pass `--config path/to/config.json` when not running from the repo root.

### Windows helper scripts (optional)

PowerShell wrappers in `scripts/` mirror the CLI flags above (`run-cli.ps1`, `refresh-models.ps1`, `restore.ps1`, `launch-codex.ps1`). They are **Windows-only conveniences** — the `codex-launchpad` binary is the portable interface.

## Config

Local settings live in `config.json` (gitignored). Public defaults are in `config.example.json`.

| Field | Type | Description |
|---|---|---|
| `ollamaIp` | string | Hostname or IP of the OpenAI-compatible API server |
| `ollamaPort` | int | API port (default `11434`; use whatever your server exposes) |
| `ollamaScheme` | string | `http` or `https` (default `http`) |
| `apiKey` | string | API key for the endpoint, if required |
| `persistCodexConfig` | bool | Write a provider into `~/.codex/config.toml` (default `true`) |
| `discoverOllamaModels` | bool | Auto-fetch models from `/v1/models` on startup (default `true`) |
| `codexModel` | string | Override Codex model; leave empty to use UI selection |
| `codexProviderId` | string | Provider identifier written to Codex config |
| `codexProviderName` | string | Display name for the provider |
| `codexApiKeyMode` | string | `experimentalBearerToken` / `envKey` / `none` (see below) |
| `codexConfigPath` | string | Override default `~/.codex/config.toml` path |
| `codexCommand` | string | Full path to Codex executable; leave empty to auto-detect |
| `codexArgs` | array | Extra arguments passed to Codex |
| `workingDirectory` | string | Working directory for launched Codex processes |

### `codexApiKeyMode` Options

- **`experimentalBearerToken`** — Writes the configured `apiKey` directly into the Codex provider config.
- **`envKey`** — Sets `env_key = "OPENAI_API_KEY"` so Codex reads from the environment variable instead.
- **`none`** — Writes no auth key; the endpoint must allow unauthenticated requests.

## Build System

### Universal build (any OS)

```sh
# Web UI
cd web && npm ci && npm run build

# Rust binary (GUI + CLI share one executable)
cargo build --release
```

`build.rs` runs `npm ci` (or `npm install` when no lockfile) when `web/node_modules` is missing, then runs `npm run build` during `cargo build` if `web/dist/index.html` is missing. This works on Windows, macOS, and Linux — no PowerShell required.

Optional shell wrappers mirror the same steps:

| Script | Platform | Purpose |
| ------ | -------- | ------- |
| `scripts/build.sh` / `./build.sh` | macOS, Linux, Git Bash | `npm ci` + web build + `cargo build --bins` |
| `codex-launchpad --build-check` | All platforms | Timestamp-based incremental rebuild + staging |
| `build-check.sh` / `build-check.ps1` | All (thin wrappers) | Run `--build-check` |

### Cross-compilation

Install a target triple, then build for it:

```sh
rustup target add aarch64-unknown-linux-gnu   # example
cargo build --release --target aarch64-unknown-linux-gnu
```

Output: `target/<triple>/release/codex-launchpad`. You need the appropriate linker and sysroot for the destination OS. The [Build Official Binaries](.github/workflows/build-official-binaries.yml) workflow shows the target triples we ship (Windows, macOS x86_64/arm64, Linux x86_64/arm64).

### Optional convenience scripts

Cross-platform logic lives in the Rust binary and `build.rs`. These wrappers are shortcuts only:

| Script | Platform | Purpose |
| ------ | -------- | ------- |
| `build.sh` / `scripts/build.sh` | Unix, Git Bash | Full build (web + Rust) |
| `codex-launchpad --build-check` | All platforms | Incremental rebuild + staging |
| `build-check.sh` / `build-check.ps1` | All (thin wrappers) | Run `--build-check` |
| `build.cmd` / `scripts/build.ps1` | Windows | `cargo build --bins` |
| `run-gui.cmd` | Windows | Build if stale, then `codex-launchpad --gui` |
| `test.cmd` | Windows | `cargo fmt --check`, `cargo test`, `cargo clippy` |
| `diagnose.sh` / `diagnose.ps1` | All (thin wrappers) | Run `codex-launchpad --diagnose` |

Stuck on a build or platform-specific dependency? Run `codex-launchpad --diagnose` first, then ask on [Discord](https://carapaceai.org/discord) — we provide limited community support there and are glad to help when we can.

## Testing

The **CI** badge runs [GitHub Actions](https://github.com/CarapaceUDE/codex-launchpad/actions/workflows/ci.yml) on every push to `master`: it checks Rust formatting, runs Clippy lints, and executes unit tests. It does not build release binaries (those are distributed via [Patreon](https://carapaceai.org/patreon) for now).

```sh
cargo fmt -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

On Windows, `.\test.cmd` runs the same three commands.

## Diagnostics

```sh
codex-launchpad --diagnose                # config, Codex launch probe, endpoint + API checks
codex-launchpad --health
codex-launchpad --list-models
```

`./diagnose.sh` and `.\diagnose.ps1` are thin wrappers around `--diagnose` (they run the built binary, or `cargo run` if you have not built yet).

## Security

> **API keys are stored in plaintext** in `config.json` and `~/.codex/config.toml`. Restrict file permissions on multi-user systems. Consider using `envKey` mode or an external secret manager for sensitive deployments.

To report a security vulnerability, see [SECURITY.md](SECURITY.md). Please do not file public GitHub issues for security reports.

## License

Source code is licensed under the [MIT License](LICENSE). Copyright (c) 2026 Carapace LLC.

Official pre-built binaries are distributed separately under the [Official Build terms](OFFICIAL_BUILDS.md).

## Trademark

This project is an independent tool and is not affiliated with, endorsed by, or sponsored by OpenAI. Codex is a trademark of OpenAI.

## Project Structure

```
├── src/                  # Rust source (GUI + CLI binary)
│   ├── main.rs           # CLI entry point
│   └── web_backend.rs    # HTTP server + UI serving
├── web/                  # Vite + React + Tailwind web UI
│   ├── src/              # React components & pages
│   ├── dist/             # Built output (gitignored)
│   └── package.json      # Frontend deps
├── scripts/              # Optional build/run helpers
│   ├── build.sh          # Unix/Git Bash full build
│   ├── build-check.sh    # Unix/Git Bash incremental build
│   ├── build.ps1         # Windows cargo build wrapper
│   ├── lib.ps1           # Shared PowerShell helpers
│   ├── run-gui.ps1       # GUI run script (Windows)
│   ├── run-cli.ps1       # CLI run script (Windows)
│   ├── refresh-models.ps1
│   └── restore.ps1
├── build.sh              # Unix/Git Bash → scripts/build.sh
├── build-check.ps1       # Thin wrapper → codex-launchpad --build-check
├── build.rs              # Cargo build script (auto-builds web UI on all OSes)
├── launch-codex.ps1      # Windows Codex launcher wrapper
├── diagnose.sh           # Unix/Git Bash → codex-launchpad --diagnose
├── diagnose.ps1          # Windows → codex-launchpad --diagnose
├── config.example.json   # Public config template
├── config.json           # Local config (gitignored)
├── run-gui.cmd           # Windows: build + launch GUI
├── build.cmd             # Windows: cargo build wrapper
├── test.cmd              # Windows: fmt + test + clippy wrapper
└── docs/
    └── architecture.md   # Architecture notes
```

## Resources

- [Architecture docs](docs/architecture.md)
- [Contributing guide](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security policy](SECURITY.md)
- [License](LICENSE)
- [Official build terms](OFFICIAL_BUILDS.md)
- [Release process](docs/release-process.md)