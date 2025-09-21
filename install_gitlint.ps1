# PowerShell script to install go-gitlint
# Equivalent to the bash commands from install_windows.sh lines 85-92

# Helper functions for colored output
function Write-Message {
    param([string]$Message)
    Write-Host "AppFlowy : $Message" -ForegroundColor Yellow
}

function Write-Success {
    param([string]$Message)
    Write-Host "AppFlowy : $Message" -ForegroundColor Green
}

function Write-Error {
    param([string]$Message)
    Write-Host "AppFlowy : $Message" -ForegroundColor Red
}

# Install go-gitlint
Write-Message "Installing go-gitlint."

$GOLINT_FILENAME = "go-gitlint_1.1.0_windows_x86_64.tar.gz"
$downloadUrl = "https://github.com/llorllale/go-gitlint/releases/download/1.1.0/$GOLINT_FILENAME"

try {
    # Download the file using Invoke-WebRequest (PowerShell equivalent of curl)
    Write-Message "Downloading go-gitlint from $downloadUrl"
    Invoke-WebRequest -Uri $downloadUrl -OutFile $GOLINT_FILENAME -UseBasicParsing
    
    # Create .githooks directory if it doesn't exist
    if (-not (Test-Path ".githooks")) {
        New-Item -ItemType Directory -Path ".githooks" -Force
    }
    
    # Extract gitlint.exe from the tar.gz file to .githooks directory
    # Note: PowerShell 5.1+ has built-in support for tar
    Write-Message "Extracting gitlint.exe to .githooks directory"
    tar -zxv -C .githooks -f $GOLINT_FILENAME gitlint.exe
    
    # Remove the downloaded tar.gz file
    Remove-Item $GOLINT_FILENAME -Force
    
    Write-Success "go-gitlint installed successfully"
}
catch {
    Write-Error "Failed to install go-gitlint: $($_.Exception.Message)"
    exit 1
}
