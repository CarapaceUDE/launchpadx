# Architecture

`launchpadx` is a desktop-first launcher with optional headless CLI flags:

- **Default:** bare `launchpadx` (or double-click) opens the desktop GUI via `web_backend.rs` (`wry`/`tao` + embedded HTTP/RPC).
- **CLI:** headless actions require explicit flags (`--launch`, `--list-models`, `--diagnose`, …).
- `src/config.rs` reads the local JSON config and normalizes the OpenAI-compatible endpoint.
- `src/lpad_config.rs` updates `~/.codex/config.toml` for persistent provider settings and creates backups before writing.
- `src/ollama.rs` discovers models from the OpenAI-compatible models API and stores a cache for UI selection.
- `src/launcher/` resolves and starts Codex using the best platform-specific launch target.

Launch targets:

- `Path`: a directly executable Codex binary. Environment variables are passed to the child process.
- `WindowsStartApp`: a Microsoft Store packaged app identity from `Get-StartApps`. This uses `shell:AppsFolder\<AppID>`, matching Ollama's Codex App launch strategy.
- `MacAppBundle`: a `.app` bundle, preferring `Contents/MacOS/Codex` when present so the environment can be passed directly.

The future UI should call into the same config and launcher modules rather than duplicating launch detection.

Persistent Codex config is intentionally separate from process launching. That keeps the future UI free to offer actions like "save config", "test endpoint", and "launch" independently.
