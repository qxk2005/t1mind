@echo off
REM Marker tool wrapper script for Windows
REM This script runs marker_single (for single PDF files) using the pipx installation
REM IMPORTANT: This wrapper MUST use marker_single, not marker, because marker expects a directory

setlocal enabledelayedexpansion

REM Try to find marker_single in various locations
set MARKER_PYTHON=
set MARKER_SCRIPT=

REM First, try to use marker_single from pipx if available (REQUIRED for single files)
if exist "%LOCALAPPDATA%\pipx\venvs\marker-pdf\Scripts\marker_single.exe" (
    set "MARKER_PYTHON=%LOCALAPPDATA%\pipx\venvs\marker-pdf\Scripts\python.exe"
    set "MARKER_SCRIPT=%LOCALAPPDATA%\pipx\venvs\marker-pdf\Scripts\marker_single.exe"
    goto :execute
)

REM Try USERPROFILE\.local\pipx
if exist "%USERPROFILE%\.local\pipx\venvs\marker-pdf\Scripts\marker_single.exe" (
    set "MARKER_PYTHON=%USERPROFILE%\.local\pipx\venvs\marker-pdf\Scripts\python.exe"
    set "MARKER_SCRIPT=%USERPROFILE%\.local\pipx\venvs\marker-pdf\Scripts\marker_single.exe"
    goto :execute
)

REM Try system PATH for marker_single
where marker_single.exe >nul 2>&1
if %ERRORLEVEL% == 0 (
    marker_single.exe %*
    exit /b %ERRORLEVEL%
)

REM marker-pdf not found, show installation instructions
echo Error: marker-pdf is not installed. >&2
echo. >&2
echo Marker 工具需要 marker-pdf 才能运行。 >&2
echo. >&2

REM Check if Python is available
where python >nul 2>&1
set PYTHON_AVAILABLE=%ERRORLEVEL%

REM Check if pipx is available
where pipx >nul 2>&1
set PIPX_AVAILABLE=%ERRORLEVEL%

if %PYTHON_AVAILABLE% neq 0 (
    echo 检测到您的系统未安装 Python 3。 >&2
    echo. >&2
    echo 首先需要安装 Python 3.8 或更高版本： >&2
    echo. >&2
    echo 方法 1：使用 WinGet 安装（推荐，Windows 10/11 自带） >&2
    echo     winget install Python.Python.3.12 >&2
    echo. >&2
    echo 方法 2：手动安装 >&2
    echo   1. 访问 https://www.python.org/downloads/ 下载并安装 Python >&2
    echo   2. 安装时勾选 "Add Python to PATH" >&2
    echo. >&2
    echo 安装 Python 后，再安装 marker-pdf： >&2
    echo   1. 打开命令提示符或 PowerShell >&2
    echo   2. 运行: pip install --user pipx >&2
    echo   3. 运行: pipx install marker-pdf >&2
    echo. >&2
    exit /b 1
)

if %PIPX_AVAILABLE% neq 0 (
    echo pipx 未安装。请先安装 pipx： >&2
    echo. >&2
    echo 方法 1：使用 WinGet 安装（推荐，Windows 10/11 自带） >&2
    echo    注意：WinGet 会自动安装 Python 作为 pipx 的依赖 >&2
    echo    winget install pipx >&2
    echo. >&2
    echo 方法 2：使用 pip 安装（需要先安装 Python） >&2
    echo   1. 打开命令提示符或 PowerShell >&2
    echo   2. 运行: pip install --user pipx >&2
    echo   3. 将 pipx 添加到 PATH（如果尚未添加） >&2
    echo. >&2
    echo 安装 pipx 后，运行: pipx install marker-pdf >&2
    echo. >&2
    exit /b 1
)

echo 安装方法： >&2
echo. >&2
echo 使用 pipx 安装 marker-pdf： >&2
echo   1. 打开命令提示符或 PowerShell（以管理员身份运行） >&2
echo   2. 运行: pipx install marker-pdf >&2
echo. >&2
echo 注意：Windows 下 Pillow 使用预编译包，通常不需要手动安装依赖库。 >&2
echo. >&2
echo 验证安装： >&2
echo   安装完成后，运行以下命令验证： >&2
echo   pipx list  # 应该看到 marker-pdf >&2
echo. >&2
echo 详细说明： >&2
echo   marker-pdf 是一个 Python 工具，用于将 PDF 转换为 Markdown。 >&2
echo   它需要 Python 3.8+ 和 pipx 来安装和管理。 >&2
echo. >&2
exit /b 1

:execute
if not defined MARKER_PYTHON (
    echo Error: Failed to find marker_single >&2
    exit /b 1
)

if not exist "!MARKER_SCRIPT!" (
    echo Error: marker_single script not found: !MARKER_SCRIPT! >&2
    exit /b 1
)

REM Set environment variables to force CPU usage and avoid Metal shader errors
REM (These are mainly for macOS, but won't hurt on Windows)
set PYTORCH_ENABLE_MPS_FALLBACK=1
set PYTORCH_MPS_HIGH_WATERMARK_RATIO=0.0
set PYTORCH_MPS_FORCE_CPU=1
set CUDA_VISIBLE_DEVICES=
set TORCH_DEVICE=cpu

REM Set model cache directories to avoid re-downloading models on each run
REM Hugging Face cache directory
if defined USERPROFILE (
    set "HF_CACHE_DIR=%USERPROFILE%\.cache\huggingface"
    REM Surya OCR model cache directory (used by marker-pdf)
    if defined LOCALAPPDATA (
        set "SURYA_CACHE_DIR=%LOCALAPPDATA%\datalab\models"
    ) else (
        set "SURYA_CACHE_DIR=%USERPROFILE%\AppData\Local\datalab\models"
    )
    
    REM Ensure cache directories exist
    if not exist "%HF_CACHE_DIR%" mkdir "%HF_CACHE_DIR%" 2>nul
    if not exist "%SURYA_CACHE_DIR%" mkdir "%SURYA_CACHE_DIR%" 2>nul
    
    REM Export environment variables
    set "HF_HOME=%HF_CACHE_DIR%"
    set "HF_HUB_CACHE=%HF_CACHE_DIR%"
    set "HUGGINGFACE_HUB_CACHE=%HF_CACHE_DIR%"
    set "TRANSFORMERS_CACHE=%HF_CACHE_DIR%"
    set "SURYA_MODEL_CACHE_DIR=%SURYA_CACHE_DIR%"
)

REM Execute marker_single
"!MARKER_PYTHON!" "!MARKER_SCRIPT!" %*
exit /b %ERRORLEVEL%

