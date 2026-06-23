@echo off
chcp 65001 >nul
cd /d "%~dp0"

echo ========================================
echo  Sticky Notes - Building...
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

echo  Step 2/3: Building frontend + backend...
call pnpm tauri build
if %errorlevel% neq 0 (
    echo [ERROR] tauri build failed
    pause
    exit /b 1
)

echo.
echo ========================================
echo  Build complete!
echo.
echo  Output:
echo  %~dp0src-tauri\target\release\sticky-notes.exe
echo ========================================
pause
