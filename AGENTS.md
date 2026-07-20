# LaunchPadX — Agent Instructions

## Project Overview

Rust GUI/CLI application with a React + Vite web UI. The Rust code provides:
- **GUI mode** (`launchpadx`, default) — desktop webview app via `wry`/`tao`; no flag required
- **CLI mode** (`launchpadx --launch`, `--list-models`, `--diagnose`, etc.) — headless actions require an explicit flag
- **Embedded HTTP server** (`web_backend.rs`) — serves the web UI and exposes RPC endpoints

The web UI is a standalone Vite + React + Tailwind app that gets bundled and embedded into the Rust binary.

## Build System

- **Rust**: `cargo build --release --bin launchpadx` (includes `desktop` by default — Windows GUI PE, no console)
- **Windows console CLI**: `cargo build --release --bin launchpadx --no-default-features` (release stages this as `launchpadx-cli.exe`)
- **Web UI**: `cd web && npm run build` (Vite → `web/dist/`)
- **Build script**: `build.rs` auto-runs `npm ci` + `npm run build` if needed (all OSes)
- **Build checker**: `launchpadx --build-check` (or `build-check.sh` / `build-check.ps1` wrappers)
- **Full build**: `cargo build --release`, or `./build.sh` / `build.cmd` → `scripts\build.ps1`
- **Tests**: `test.cmd` → `cargo fmt -- --check && cargo test && cargo clippy --all-targets -- -D warnings`

### Common build commands

```powershell
cargo build                    # Debug build
cargo build --release          # Release build
cargo build --release --bin launchpadx  # Specific binary
cd web && npm run build        # Build web UI only
```

## Config

- Local config: `config.json` (gitignored)
- Template: `config.example.json`
- Key fields: `ollamaIp`, `ollamaPort`, `ollamaScheme`, `apiKey`, `codexCommand`, `codexApiKeyMode`

When modifying config-related code, always check `config.example.json` for the full schema and update it if new fields are added.

## Testing

Run tests:
```powershell
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

Rust unit tests live under `src/`. Playwright E2E tests live in `web/e2e/` (`cd web && npm run test:e2e`).

## Web UI

Located in `web/`. Built with Vite + React + TypeScript + Tailwind CSS.
- Source: `web/src/` (components, pages, context, types)
- Output: `web/dist/` (gitignored)
- Dev: `cd web && npm run dev`
- Build: `cd web && npm run build`

## Code Style

- Rust: use `cargo fmt`, `cargo clippy -- -D warnings`
- TypeScript: standard Prettier-style formatting (check `web/package.json` scripts)
- PowerShell: snake_case for functions, explicit error handling with `try/catch`, `Set-ErrorActionPreference = "Stop"`

## Common Tasks

### Adding a new config field
1. Update `src/config.rs` (serde struct) to include the field
2. Update `config.example.json` with the new field
3. Update UI components if the field is user-editable
4. Run `cargo clippy -- -D warnings` to verify

### Adding a new UI component
1. Create component in `web/src/components/launcher/`
2. Export from `web/src/main.tsx` or relevant layout
3. Follow existing naming: PascalCase, `tsx` extension
4. Use Tailwind for styling; primitives from `primitives.tsx`

### Changing the build system
1. Edit `build.rs` for Cargo-level automation
2. Edit `src/build_check.rs` for smart timestamp-based rebuilding
3. Run `build.cmd` to verify the full build pipeline works

## File Map

| File | Purpose |
|---|---|
| `src/main.rs` | Entry point: default GUI, CLI flags for headless actions |
| `src/web_backend.rs` | HTTP server, UI serving, model cache |
| `web/src/App.tsx` | Main React app component |
| `web/src/main.tsx` | React entry point |
| `web/src/types.ts` | Shared TypeScript types |
| `web/src/components/launcher/` | UI components |
| `web/src/pages/` | Page components |
| `web/src/context/` | React context providers |
| `scripts/lib.ps1` | Shared PS helpers (`Get-CargoCommand`) |
| `scripts/run-gui.ps1` | GUI launch script |
| `scripts/run-cli.ps1` | CLI launch script |
| `scripts/refresh-models.ps1` | Refresh model cache |
| `scripts/restore.ps1` | Restore Codex config |
| `launchpadx --build-check` | Smart incremental build + staging |
| `build-check.sh` / `build-check.ps1` | Thin wrappers around `--build-check` |
| `launch-codex.ps1` | Standalone Codex launcher |
| `launchpadx --diagnose` | Cross-platform health check |
| `diagnose.sh` / `diagnose.ps1` | Thin wrappers around `--diagnose` |