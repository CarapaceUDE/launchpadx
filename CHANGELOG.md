# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] — 2026-07-10

### Changed
- Renamed the application to LaunchPadX and clarified that it launches Codex against OpenAI-compatible endpoints, not only local models.
- Added public Windows, macOS, and Linux release archives built from tags by GitHub Actions.
- Removed generated compiler probes, runtime discovery logs, obsolete helper scripts, and the Patreon-only binary distribution policy.
- Fixed `run-gui.cmd` / `--build-check` invoking a stale release binary that ignored the flag and tried to launch Codex instead of building.
- `launchpadx` with no arguments now prints help instead of auto-launching Codex; use `--launch` explicitly.
- Launch no longer errors when Codex is already running — it reports the detected process and leaves it alone.
- Added cross-platform `launchpadx --diagnose` and `launchpadx --build-check`; `diagnose.ps1` and `build-check.ps1` are now thin wrappers around the Rust CLI.
- `build.rs` and `build-check.ps1` now run `npm ci` when `web/node_modules` is missing before building the web UI.
- Windows Codex launch detection hardened: Microsoft Store shims are no longer launched directly, Start menu AppID lookup is broader, and `explorer.exe` is preferred for packaged app activation.
- Launch RPC responses now include `launchTarget` for easier troubleshooting when Codex does not appear.
- Linux launcher now searches PATH for `codex` first (removed Windows-specific `codex-app` from non-Windows platforms).
- macOS launcher now searches PATH for `codex` before checking `.app` bundles.
- GUI error log path now uses `dirs::config_dir()` or `dirs::cache_dir()` as fallback instead of the current working directory.

## [0.1.0] — 2026-06-19

### Added
- Launch Codex against a local Ollama-compatible endpoint.
- GUI for editing provider configuration, refreshing Ollama model lists, and launching Codex.
- CLI for writing config, refreshing/listing models, restoring previous Codex settings.
- Platform-specific launch support: Windows (path + Store AppID), macOS (.app bundles), Linux (PATH + AppImage).
- Model cache for Ollama model discovery, persisted in the OS cache directory.
- Config backup/restore before applying provider changes.
- Three API key modes: `experimentalBearerToken`, `envKey`, and `none`.
