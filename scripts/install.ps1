# xchecker installer for Windows
# Usage: irm https://raw.githubusercontent.com/EffortlessMetrics/xchecker/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "EffortlessMetrics/xchecker"
$Target = "x86_64-pc-windows-msvc"

# Get version from environment or fetch latest
$Version = if ($env:XCHECKER_VERSION) { $env:XCHECKER_VERSION } else { $null }

# Get install directory
$InstallDir = if ($env:XCHECKER_INSTALL_DIR) {
    $env:XCHECKER_INSTALL_DIR
} else {
    "$env:LOCALAPPDATA\xchecker\bin"
}

function Write-Info($Message) {
    Write-Host "[INFO] $Message" -ForegroundColor Green
}

function Write-Warn($Message) {
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Error($Message) {
    Write-Host "[ERROR] $Message" -ForegroundColor Red
    exit 1
}

function Get-LatestVersion {
    try {
        $Response = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
        return $Response.tag_name -replace '^v', ''
    }
    catch {
        Write-Error "Failed to fetch latest version: $_"
    }
}

function Install-Xchecker {
    Write-Info "Installing xchecker..."
    Write-Info "Platform: $Target"

    # Get version
    if (-not $Version) {
        Write-Info "Fetching latest version..."
        $Script:Version = Get-LatestVersion
    }
    Write-Info "Version: $Version"

    # Create install directory
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    }

    # Download
    $Url = "https://github.com/$Repo/releases/download/v$Version/xchecker-$Target.zip"
    Write-Info "Downloading from: $Url"

    $TempDir = Join-Path $env:TEMP "xchecker-install-$(Get-Random)"
    New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

    try {
        $ZipPath = Join-Path $TempDir "xchecker.zip"
        Invoke-WebRequest -Uri $Url -OutFile $ZipPath

        # Verify checksum if available
        $ChecksumUrl = "$Url.sha256"
        $ChecksumPath = Join-Path $TempDir "checksum.sha256"
        try {
            Invoke-WebRequest -Uri $ChecksumUrl -OutFile $ChecksumPath
            $ExpectedHash = (Get-Content $ChecksumPath).Split()[0].ToUpper()
            $ActualHash = (Get-FileHash $ZipPath -Algorithm SHA256).Hash

            if ($ExpectedHash -ne $ActualHash) {
                Write-Error "Checksum verification failed"
            }
            Write-Info "Checksum verified"
        }
        catch {
            Write-Warn "Checksum file not available, skipping verification"
        }

        # Extract
        Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

        # Install
        $ExePath = Join-Path $TempDir "xchecker.exe"
        $DestPath = Join-Path $InstallDir "xchecker.exe"
        Copy-Item $ExePath $DestPath -Force

        Write-Info "xchecker installed to $DestPath"

        # Check PATH
        $UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if ($UserPath -notlike "*$InstallDir*") {
            Write-Warn "$InstallDir is not in your PATH"
            Write-Host ""
            Write-Host "To add it to your PATH, run:"
            Write-Host "  `$env:PATH += `";$InstallDir`""
            Write-Host ""
            Write-Host "To permanently add it (requires admin):"
            Write-Host "  [Environment]::SetEnvironmentVariable('PATH', `$env:PATH + ';$InstallDir', 'User')"
            Write-Host ""
        }

        # Verify installation
        try {
            $VersionOutput = & $DestPath --version 2>&1
            Write-Info "Installation successful!"
            Write-Host $VersionOutput
        }
        catch {
            Write-Error "Installation verification failed: $_"
        }
    }
    finally {
        # Cleanup
        if (Test-Path $TempDir) {
            Remove-Item -Recurse -Force $TempDir
        }
    }
}

Install-Xchecker
