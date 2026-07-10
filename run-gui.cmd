@echo off
setlocal

set ROOT=%~dp0

:: Run conditional build script (Rust + Web UI)
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%ROOT%build-check.ps1"
if errorlevel 1 (
    echo Error: build failed. Check the output above.
    pause
    exit /b 1
)

:: Check binary exists (release)
if not exist "%ROOT%target\release\launchpadx.exe" (
    echo Error: release binary not built. Check build output above.
    pause
    exit /b 1
)

:: Keep the console attached so runtime errors remain visible.
"%ROOT%target\release\launchpadx.exe" --gui
if errorlevel 1 (
    echo Error: GUI exited unexpectedly. Check the output above.
    pause
    exit /b 1
)
