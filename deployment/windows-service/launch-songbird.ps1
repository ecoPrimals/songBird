# Songbird PowerShell Launcher (Windows)
# genomeBin Phase 1 - Manual launcher for Windows
#
# Usage:
#   .\launch-songbird.ps1 [-FamilyID "my-game"] [-LogLevel "info"]

param(
    [string]$FamilyID = "default",
    [string]$LogLevel = "info",
    [string]$BinaryPath = ".\songbird.exe"
)

# Colors for output
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

# Print banner
Write-ColorOutput "╔══════════════════════════════════════════════════════════════╗" "Blue"
Write-ColorOutput "║  🌍 Songbird PowerShell Launcher (Windows)                  ║" "Blue"
Write-ColorOutput "║  genomeBin Phase 1 - Manual Deployment                      ║" "Blue"
Write-ColorOutput "╚══════════════════════════════════════════════════════════════╝" "Blue"
Write-Host ""

# Verify binary exists
if (-not (Test-Path $BinaryPath)) {
    Write-ColorOutput "[ERROR] Songbird binary not found: $BinaryPath" "Red"
    Write-ColorOutput "[HINT] Build with: cargo build --release --target x86_64-pc-windows-gnu" "Yellow"
    exit 1
}

# Verify binary is executable
$file = Get-Item $BinaryPath
if ($file.Extension -ne ".exe") {
    Write-ColorOutput "[ERROR] File is not a Windows executable (.exe)" "Red"
    exit 1
}

# Display configuration
Write-ColorOutput "[INFO] Configuration:" "Blue"
Write-Host "  • Family ID: $FamilyID"
Write-Host "  • Log Level: $LogLevel"
Write-Host "  • Binary: $BinaryPath"
Write-Host "  • Working Dir: $PWD"
Write-Host ""

# Set environment variables
$env:SONGBIRD_FAMILY_ID = $FamilyID
$env:RUST_LOG = $LogLevel
$env:SONGBIRD_MODE = "windows-manual"

# Display environment
Write-ColorOutput "[INFO] Environment:" "Blue"
Write-Host "  • SONGBIRD_FAMILY_ID = $env:SONGBIRD_FAMILY_ID"
Write-Host "  • RUST_LOG = $env:RUST_LOG"
Write-Host "  • SONGBIRD_MODE = $env:SONGBIRD_MODE"
Write-Host ""

# Launch Songbird
Write-ColorOutput "[START] Launching Songbird..." "Green"
Write-Host ""

try {
    # Start process
    $process = Start-Process -FilePath $BinaryPath `
                              -NoNewWindow `
                              -PassThru `
                              -Wait
    
    # Check exit code
    if ($process.ExitCode -eq 0) {
        Write-ColorOutput "[SUCCESS] Songbird exited gracefully." "Green"
    } else {
        Write-ColorOutput "[ERROR] Songbird exited with code: $($process.ExitCode)" "Red"
        exit $process.ExitCode
    }
}
catch {
    Write-ColorOutput "[ERROR] Failed to launch Songbird: $_" "Red"
    exit 1
}

Write-Host ""
Write-ColorOutput "[INFO] Songbird stopped." "Blue"
