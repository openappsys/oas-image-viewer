@echo off
chcp 65001 >nul
:: Image-Viewer Windows Installer
:: This script installs Image-Viewer and registers it in the context menu

setlocal EnableDelayedExpansion

echo =========================================
echo Image-Viewer Windows Installation
echo =========================================
echo.

:: Check for admin privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo Error: Administrator privileges required.
    echo Please run this script as Administrator.
    pause
    exit /b 1
)

:: Set installation directory
set "INSTALL_DIR=%PROGRAMFILES%\Image-Viewer"
set "SOURCE_DIR=%~dp0"

echo Installation directory: %INSTALL_DIR%
echo.

:: Create installation directory
if not exist "%INSTALL_DIR%" (
    echo Creating installation directory...
    mkdir "%INSTALL_DIR%"
)

:: Copy executable (assuming it's built)
if exist "%SOURCE_DIR%\..\..\target\release\image-viewer.exe" (
    echo Copying image-viewer.exe...
    copy /Y "%SOURCE_DIR%\..\..\target\release\image-viewer.exe" "%INSTALL_DIR%\" >nul
) else if exist "%SOURCE_DIR%\image-viewer.exe" (
    echo Copying image-viewer.exe...
    copy /Y "%SOURCE_DIR%\image-viewer.exe" "%INSTALL_DIR%\" >nul
) else (
    echo Warning: image-viewer.exe not found.
    echo Please build the project first with: cargo build --release
    pause
    exit /b 1
)

:: Register context menu
echo Registering context menu entries...
regedit /s "%SOURCE_DIR%\register-context-menu.reg"

if %errorLevel% equ 0 (
    echo.
    echo =========================================
    echo Installation completed successfully!
    echo =========================================
    echo.
    echo Image-Viewer installed to: %INSTALL_DIR%
    echo.
    echo You can now right-click on image files to open with Image-Viewer.
    echo.
) else (
    echo.
    echo Error: Failed to register context menu.
    echo.
)

pause
