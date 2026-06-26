# Contributing to Codex Launcher

Thanks for your interest in contributing! This project is a small Rust tool, and contributions are welcome.

**Source code** is MIT-licensed and open to everyone. **Official pre-built binaries** are distributed separately to Patreon supporters — see [OFFICIAL_BUILDS.md](OFFICIAL_BUILDS.md) and [docs/release-process.md](docs/release-process.md).

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) 18+ (web UI)
- Platform GUI deps if building the desktop app — see [README prerequisites](README.md#prerequisites)

### Building

```sh
cd web && npm ci && npm run build
cargo build --bins
```

PowerShell wrappers (`build.cmd`, `run-gui.cmd`, etc.) are **Windows-only conveniences**. On macOS or Linux, use `cargo` and `npm` directly.

### Running

| Command | Purpose |
|---|---|
| `codex-launchpad --gui` | Launch the desktop UI |
| `codex-launchpad --launch` | Write config and launch Codex |
| `cargo fmt --check && cargo test && cargo clippy --all-targets -- -D warnings` | Pre-commit checks |
| `cd web && npm run screenshot:readme` | Regenerate `assets/readme-screenshot.png` for the README |

On Windows you can use `.\run-gui.cmd`, `.\test.cmd`, and the scripts in `scripts/` instead.

### CLI Options

```
codex-launchpad --config <path>
codex-launchpad --write-config-only
codex-launchpad --refresh-models
codex-launchpad --list-models
codex-launchpad --restore
```

## How to Contribute

1. **Fork and clone** this repo.
2. **Create a feature branch**: `git checkout -b feature/my-change`
3. **Make your changes** — keep them focused and small.
4. **Run checks** before committing (see [Testing](README.md#testing) in the README)
5. **Commit** with a descriptive message (conventional commits preferred but not required).
6. **Open a pull request** with a clear description of what changes and why.

## Code Style

- Follow `cargo fmt` formatting (run `cargo fmt` before committing).
- `cargo clippy --all-targets -- -D warnings` should pass cleanly.
- Error handling uses `thiserror` for custom error types.
- Prefer descriptive variable names over abbreviations.

## Project Structure

```
src/
  main.rs          # CLI entry point
  lib.rs           # Library root (re-exports modules)
  config.rs        # Local JSON config reader/validator
  codex_config.rs  # ~/.codex/config.toml management
  ollama.rs        # Ollama model discovery + caching
  app_logic.rs     # Shared business logic (write/restore/refresh/launch)
  gui.rs           # egui/eframe desktop UI
  launcher/        # Platform-specific launch code
    mod.rs         # Unified resolve + launch dispatcher
    windows.rs     # Windows: PATH search + Store AppID
    macos.rs       # macOS: .app bundle search
    linux.rs       # Linux: PATH + AppImage search
scripts/           # Windows PowerShell helpers (optional)
```

## Reporting Issues

Please include:
- OS and version
- Launcher version (check `cargo pkgid`)
- Steps to reproduce
- Relevant log output (`~/.codex-launchpad/error.log` or `codex-launchpad-gui.error.log`)