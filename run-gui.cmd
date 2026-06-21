@echo off
setlocal

set ROOT=%~dp0

:: Run conditional build script (Rust + Web UI)
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%ROOT%build-check.ps1"

:: Check binary exists (release)
if not exist "%ROOT%target\release\codex-local-launcher.exe" (
    echo Error: release binary not built. Check build output above.
    pause
    exit /b 1
)

:: Launch the GUI
start "" "%ROOT%target\release\codex-local-launcher.exe" --gui