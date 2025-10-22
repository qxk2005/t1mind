# AppFlowy Windows Installer Creation Script with Dynamic Version
# This script creates a Windows installer for AppFlowy with version from pubspec.yaml

Write-Host "Creating AppFlowy Windows Installer..." -ForegroundColor Green
Write-Host ""

# Function to extract version from pubspec.yaml
function Get-VersionFromPubspec {
    $pubspecPath = "appflowy_flutter\pubspec.yaml"
    if (-not (Test-Path $pubspecPath)) {
        Write-Host "❌ pubspec.yaml not found at: $pubspecPath" -ForegroundColor Red
        return $null
    }
    
    $content = Get-Content $pubspecPath
    foreach ($line in $content) {
        if ($line -match "^version:\s*(.+)$") {
            $version = $matches[1].Trim()
            Write-Host "📋 Found version in pubspec.yaml: $version" -ForegroundColor Cyan
            return $version
        }
    }
    
    Write-Host "❌ Version not found in pubspec.yaml" -ForegroundColor Red
    return $null
}

# Extract version from pubspec.yaml
$version = Get-VersionFromPubspec
if (-not $version) {
    Write-Host "Using default version: 0.9.20251022" -ForegroundColor Yellow
    $version = "0.9.20251022"
}

# Set environment variable for Inno Setup
$env:APP_VERSION = $version
Write-Host "🔧 Set APP_VERSION environment variable: $version" -ForegroundColor Green

# Check if Inno Setup is installed
$isccPath = Get-Command iscc -ErrorAction SilentlyContinue
if (-not $isccPath) {
    # Try common installation paths
    $commonPaths = @(
        "C:\Program Files (x86)\Inno Setup 6\iscc.exe",
        "C:\Program Files\Inno Setup 6\iscc.exe",
        "C:\Program Files (x86)\Inno Setup 5\iscc.exe",
        "C:\Program Files\Inno Setup 5\iscc.exe"
    )
    
    $isccExe = $null
    foreach ($path in $commonPaths) {
        if (Test-Path $path) {
            $isccExe = $path
            Write-Host "Found Inno Setup at: $path" -ForegroundColor Green
            break
        }
    }
    
    if (-not $isccExe) {
        Write-Host "Inno Setup is not installed or not in PATH." -ForegroundColor Red
        Write-Host "Please download and install Inno Setup from: https://jrsoftware.org/isinfo.php" -ForegroundColor Yellow
        Write-Host "After installation, add the installation directory to your PATH environment variable." -ForegroundColor Yellow
        Write-Host ""
        Write-Host "Alternatively, you can run this script with the full path to iscc.exe:" -ForegroundColor Yellow
        Write-Host '"C:\Program Files (x86)\Inno Setup 6\iscc.exe" scripts\windows_installer\appflowy.iss' -ForegroundColor Cyan
        Read-Host "Press Enter to exit"
        exit 1
    }
} else {
    $isccExe = "iscc"
}

# Create output directory if it doesn't exist
$outputDir = "appflowy_flutter\product\$version\windows"
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
    Write-Host "Created output directory: $outputDir" -ForegroundColor Yellow
}

# Compile the installer
Write-Host "Compiling installer with version: $version..." -ForegroundColor Yellow
$result = & $isccExe "scripts\windows_installer\appflowy.iss"

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✅ Installer created successfully!" -ForegroundColor Green
    Write-Host "📁 Location: $outputDir\AppFlowy-Setup-$version.exe" -ForegroundColor Cyan
    Write-Host "📋 Version: $version" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "You can now distribute this installer to users." -ForegroundColor Green
    
    # Open the output directory
    $installerPath = "$outputDir\AppFlowy-Setup-$version.exe"
    if (Test-Path $installerPath) {
        Write-Host ""
        $openDir = Read-Host "Would you like to open the output directory? (y/n)"
        if ($openDir -eq "y" -or $openDir -eq "Y") {
            Invoke-Item $outputDir
        }
    }
} else {
    Write-Host ""
    Write-Host "❌ Failed to create installer. Please check the error messages above." -ForegroundColor Red
}

Read-Host "Press Enter to exit"
