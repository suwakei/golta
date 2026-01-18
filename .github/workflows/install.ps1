$ErrorActionPreference = "Stop"

# --- Configuration ---
$GitHubRepo = "suwakei/golta"
$BinaryName = "golta"
$InstallDir = Join-Path $env:USERPROFILE ".golta\bin"
# ---------------------

Write-Host "Installing $BinaryName latest for Windows (amd64)..."

# Construct download URL
# Based on release.yml configuration, Windows asset name is golta-windows-amd64.zip
$AssetName = "golta-windows-amd64.zip"
$DownloadUrl = "https://github.com/$GitHubRepo/releases/latest/download/$AssetName"

# Create temporary directory
$TempDir = Join-Path $env:TEMP ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null
$ZipPath = Join-Path $TempDir $AssetName

# Download
Write-Host "Downloading $DownloadUrl..."
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath
}
catch {
    Write-Error "Failed to download: $_"
    exit 1
}

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Extract
Write-Host "Extracting to $InstallDir..."
Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force

# Remove temporary files
Remove-Item -Path $TempDir -Recurse -Force

Write-Host "Installed to $InstallDir"

# Configure PATH (User environment variable)
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -split ';' -notcontains $InstallDir) {
    $NewPath = "$InstallDir;$UserPath"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    Write-Host "Added $InstallDir to your User Path."
    Write-Host "Please restart your terminal/PowerShell to apply the changes."
}
else {
    Write-Host "$InstallDir is already in your PATH."
}
