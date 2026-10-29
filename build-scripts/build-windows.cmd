@echo off
REM copyNpaste — Windows-Produktionsbuild (NSIS-Installer + eigenstaendige EXE)
REM Voraussetzungen: Node.js, Rust (MSVC-Toolchain), WebView2 Runtime
setlocal
cd /d "%~dp0.."
echo === copyNpaste: bun install ===
call bun install --frozen-lockfile || goto :err
echo === copyNpaste: tauri build ===
call npm run tauri build || goto :err
echo.
echo FERTIG. Artefakte:
echo   src-tauri\target\release\copynpaste.exe
echo   src-tauri\target\release\bundle\nsis\copyNpaste_1.2.0_x64-setup.exe
exit /b 0
:err
echo FEHLER: Build abgebrochen.
exit /b 1
