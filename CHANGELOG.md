# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
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