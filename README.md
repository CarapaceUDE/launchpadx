# Codex Local Launcher


[![CI](https://github.com/codex-local-launcher/codex-local-launcher/actions/workflows/ci.yml/badge.svg)](https://github.com/codex-local-launcher/codex-local-launcher/actions/workflows/ci.yml)
Launch Codex with an Ollama-compatible endpoint and manage the Codex desktop app config from a small local UI.

## Quick Start

Run the UI:

```powershell
.\run-gui.cmd
```

Build everything:

```powershell
.\build.cmd
```

Run checks:

```powershell
.\test.cmd
```

Local settings live in `config.json`, which is gitignored. Public defaults live in `config.example.json`.

## Scripts

```powershell
.\scripts\run-gui.ps1
.\scripts\run-cli.ps1 --config config.json --refresh-models
.\scripts\refresh-models.ps1
.\scripts\restore.ps1
```

The GUI binary is `target\debug\codex-local-launcher-gui.exe`. The CLI binary is `target\debug\codex-local-launcher.exe`. Do not use the CLI binary for the UI.

## What It Does

The launcher can:

- discover Ollama models from `GET /api/tags`
- cache model metadata locally for UI selection
- write a Codex provider into `~/.codex/config.toml`
- back up and restore the previous Codex root model/provider settings
- launch Codex through direct executable paths or the Windows Store AppID route

## Config

Example:

```json
{
  "ollamaIp": "100.64.0.10",
  "ollamaPort": 11434,
  "ollamaScheme": "http",
  "apiKey": "my-real-api-key",
  "persistCodexConfig": true,
  "discoverOllamaModels": true,
  "codexModel": "",
  "codexProviderId": "codex-local-launcher",
  "codexProviderName": "Local Ollama",
  "codexApiKeyMode": "experimentalBearerToken",
  "codexConfigPath": "",
  "codexCommand": "",
  "codexArgs": [],
  "workingDirectory": ""
}
```

`codexApiKeyMode` options:

- `experimentalBearerToken`: writes the configured API key into Codex config.
- `envKey`: writes `env_key = "OPENAI_API_KEY"`.
- `none`: writes no provider auth key.

## Restore

The first time the launcher applies its provider, it saves the previous root Codex settings. Restore from the UI, or run:

```powershell
.\scripts\restore.ps1
```

## Security

**API keys are stored in plaintext** in `config.json` and `~/.codex/config.toml`. Restrict file permissions on multi-user systems. Consider using `envKey` mode or an external secret manager for sensitive deployments.

## Development

Architecture notes live in `docs/architecture.md`.

