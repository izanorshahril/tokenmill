@echo off
setlocal
cd /d "%~dp0"
rem ponytail: reuse the local build; rebuild with cargo after source changes.
if exist "%~dp0target\debug\tokenmill-tray.exe" goto launch
echo Building Tokenmill...
cargo build --locked --offline -p tokenmill-cli --bins
if errorlevel 1 (
    echo Build failed. Check Rust is installed and dependencies are cached.
    pause
    exit /b 1
)
:launch
start "" "%~dp0target\debug\tokenmill-tray.exe"
