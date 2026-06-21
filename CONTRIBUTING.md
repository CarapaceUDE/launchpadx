# Contributing to Codex Local Launcher

Thanks for your interest in contributing! This project is a small Rust tool, and contributions are welcome.

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (latest stable)
- PowerShell 5+ (for build/launch scripts on Windows)

### Building

```powershell
.\build.cmd
```

Or directly:

```powershell
cargo build --bins
```

### Running

| Script | Purpose |
|---|---|
| `.\run-gui.cmd` | Launch the desktop UI |
| `.\launch-codex.cmd` | Write config and launch Codex |
| `.\test.cmd` | Run `cargo fmt --check`, `cargo test`, `cargo clippy` |

### CLI Options

```
codex-local-launcher --config <path>
codex-local-launcher --write-config-only
codex-local-launcher --refresh-models
codex-local-launcher --list-models
codex-local-launcher --restore
```

## How to Contribute

1. **Fork and clone** this repo.
2. **Create a feature branch**: `git checkout -b feature/my-change`
3. **Make your changes** — keep them focused and small.
4. **Run checks** before committing: `.\test.cmd`
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
scripts/           # PowerShell helper scripts
```

## Reporting Issues

Please include:
- OS and version
- Launcher version (check `cargo pkgid`)
- Steps to reproduce
- Relevant log output (`~/.codex-local-launcher/error.log` or `codex-local-launcher-gui.error.log`)