@echo off
echo Creating AppFlowy Windows Installer with Dynamic Version...
echo.

REM Extract version from pubspec.yaml
set VERSION=
for /f "tokens=2 delims=: " %%i in ('findstr /r "^version:" appflowy_flutter\pubspec.yaml') do (
    set VERSION=%%i
    goto :version_found
)

:version_found
if "%VERSION%"=="" (
    echo Warning: Could not extract version from pubspec.yaml
    set VERSION=0.9.20251022
    echo Using default version: %VERSION%
) else (
    echo Found version in pubspec.yaml: %VERSION%
)

REM Set environment variable for Inno Setup
set APP_VERSION=%VERSION%
echo Set APP_VERSION environment variable: %VERSION%

REM Check if Inno Setup is installed
where iscc >nul 2>&1
if %errorlevel% neq 0 (
    echo Inno Setup is not installed or not in PATH.
    echo Please download and install Inno Setup from: https://jrsoftware.org/isinfo.php
    echo After installation, add the installation directory to your PATH environment variable.
    echo.
    echo Alternatively, you can run this script with the full path to iscc.exe:
    echo "C:\Program Files (x86)\Inno Setup 6\iscc.exe" scripts\windows_installer\appflowy.iss
    pause
    exit /b 1
)

REM Create output directory if it doesn't exist
if not exist "appflowy_flutter\product\%VERSION%\windows" (
    mkdir "appflowy_flutter\product\%VERSION%\windows"
)

REM Compile the installer
echo Compiling installer with version: %VERSION%...
iscc scripts\windows_installer\appflowy.iss

if %errorlevel% equ 0 (
    echo.
    echo ✅ Installer created successfully!
    echo 📁 Location: appflowy_flutter\product\%VERSION%\windows\AppFlowy-Setup-%VERSION%.exe
    echo 📋 Version: %VERSION%
    echo.
    echo You can now distribute this installer to users.
) else (
    echo.
    echo ❌ Failed to create installer. Please check the error messages above.
)

pause
