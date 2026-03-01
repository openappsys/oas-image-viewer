@echo off
chcp 65001 >nul
:: Image-Viewer Windows Installer
:: Universal installer for Windows 7/8/10/11 - no admin rights needed

echo =========================================
echo Image-Viewer Windows Installation
echo =========================================
echo.

:: Get script directory
set "SCRIPT_DIR=%~dp0"
set "SCRIPT_DIR=%SCRIPT_DIR:~0,-1%"

echo Script directory: %SCRIPT_DIR%
echo.

:: Try to find image-viewer.exe in various locations
set "EXE_PATH="

:: Check 1: Parent of parent directory (for install\windows\ structure)
echo Checking: %SCRIPT_DIR%\..\..\image-viewer.exe
if exist "%SCRIPT_DIR%\..\..\image-viewer.exe" (
    for %%F in ("%SCRIPT_DIR%\..\..") do set "EXE_PATH=%%~fF\image-viewer.exe"
    echo [OK] Found
    goto :found_exe
)

:: Check 2: Same directory as script
echo Checking: %SCRIPT_DIR%\image-viewer.exe
if exist "%SCRIPT_DIR%\image-viewer.exe" (
    set "EXE_PATH=%SCRIPT_DIR%\image-viewer.exe"
    echo [OK] Found
    goto :found_exe
)

:: Check 3: Standard LocalAppData location
echo Checking: %LOCALAPPDATA%\Image-Viewer\image-viewer.exe
if exist "%LOCALAPPDATA%\Image-Viewer\image-viewer.exe" (
    set "EXE_PATH=%LOCALAPPDATA%\Image-Viewer\image-viewer.exe"
    echo [OK] Found
    goto :found_exe
)

:: Not found
echo.
echo [ERROR] image-viewer.exe not found!
echo.
echo Please ensure you have downloaded the correct package.
echo.
pause
exit /b 1

:found_exe
echo.
echo Executable: %EXE_PATH%
echo.

:: Register context menu using PowerShell
echo Registering context menu...
echo.

:: Create PowerShell script file
set "PS_SCRIPT=%TEMP%\iv-register.ps1"
(
echo $exePath = '%EXE_PATH%';
echo $formats = @('png', 'jpg', 'jpeg', 'gif', 'webp', 'tiff', 'tif', 'bmp', 'ico', 'heic', 'heif', 'avif');
echo foreach ($fmt in $formats) {
echo     $regPath = "HKCU:\Software\Classes\.$fmt\shell\OpenWithImageViewer";
echo     New-Item -Path $regPath -Force ^| Out-Null;
echo     Set-ItemProperty -Path $regPath -Name '(Default)' -Value 'Open with Image-Viewer';
echo     Set-ItemProperty -Path $regPath -Name 'Icon' -Value $exePath;
echo     $cmdPath = "$regPath\command";
echo     New-Item -Path $cmdPath -Force ^| Out-Null;
echo     Set-ItemProperty -Path $cmdPath -Name '(Default)' -Value "`"$exePath`" \"%1\"";
echo }
echo Write-Host 'Context menu registered successfully.';
) > "%PS_SCRIPT%"

:: Run PowerShell script
powershell -NoProfile -ExecutionPolicy Bypass -File "%PS_SCRIPT%"

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Failed to register context menu.
    del "%PS_SCRIPT%" 2>nul
    pause
    exit /b 1
)

:: Clean up
del "%PS_SCRIPT%" 2>nul

echo.
echo =========================================
echo Installation completed successfully!
echo =========================================
echo.
echo You can now right-click on image files and select:
echo   "Open with Image-Viewer"
echo.
pause
