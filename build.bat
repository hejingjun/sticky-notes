@echo off
chcp 65001 >nul
cd /d "%~dp0"

setlocal enabledelayedexpansion

echo ========================================
echo  Sticky Notes - Production Build
echo ========================================

set "NODE_DIR=C:\Users\hexin\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin"
set "PNPM_DIR=C:\Users\hexin\.cache\codex-runtimes\codex-primary-runtime\dependencies\bin"
set "PATH=%NODE_DIR%;%PNPM_DIR%;%PATH%"

echo  Step 1/3: Installing dependencies...
call pnpm install
if %errorlevel% neq 0 (
    echo [ERROR] pnpm install failed
    pause
    exit /b 1
)

echo  Step 2/3: Building frontend + Rust backend...
call pnpm tauri build
if %errorlevel% neq 0 (
    echo [ERROR] tauri build failed
    pause
    exit /b 1
)

echo  Step 3/3: Copying to dist...
copy /y "src-tauri\target\release\sticky-notes.exe" "dist\sticky-notes.exe" >nul

echo.
echo ========================================
echo  Build complete!
echo.
echo  Binary: %~dp0dist\sticky-notes.exe
echo ========================================
pause
exit /b 0
