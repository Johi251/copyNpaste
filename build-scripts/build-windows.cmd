@echo off
REM copyNpaste — Windows-Produktionsbuild (NSIS-Installer + eigenstaendige EXE)
REM Voraussetzungen: Node.js, Rust (MSVC-Toolchain), WebView2 Runtime
setlocal
cd /d "%~dp0.."
echo === copyNpaste: npm install ===
call npm install --no-fund --no-audit || goto :err
echo === copyNpaste: tauri build ===
call npm run tauri build || goto :err
echo.
echo FERTIG. Artefakte:
echo   src-tauri\target\release\copyNpaste.exe
echo   src-tauri\target\release\bundle\nsis\copyNpaste_1.0.0_x64-setup.exe
exit /b 0
:err
echo FEHLER: Build abgebrochen.
exit /b 1
