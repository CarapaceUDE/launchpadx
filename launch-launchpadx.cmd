@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0launch-codex.ps1" 2>&1
set "launchExit=%ERRORLEVEL%"
if not "%launchExit%"=="0" (
    echo.
    echo Codex launcher failed. See the error above.
    pause
)
exit /b %launchExit%
