#############################################################################
# Windows OpenSSH Server Setup with WSL Auto-Login
# Run this in PowerShell as Administrator on Bryan's Windows machine
#############################################################################

Write-Host "============================================" -ForegroundColor Blue
Write-Host "  Setting Up Windows OpenSSH Server" -ForegroundColor Blue
Write-Host "============================================" -ForegroundColor Blue
Write-Host ""

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "ERROR: This script must be run as Administrator!" -ForegroundColor Red
    Write-Host "Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Yellow
    exit 1
}

Write-Host "[1/6] Installing OpenSSH Server..." -ForegroundColor Cyan
try {
    $sshInstalled = Get-WindowsCapability -Online | Where-Object Name -like 'OpenSSH.Server*'
    if ($sshInstalled.State -eq "Installed") {
        Write-Host "  ✓ OpenSSH Server already installed" -ForegroundColor Green
    } else {
        Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0
        Write-Host "  ✓ OpenSSH Server installed" -ForegroundColor Green
    }
} catch {
    Write-Host "  ✗ Failed to install OpenSSH Server: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "[2/6] Starting SSH service..." -ForegroundColor Cyan
try {
    Start-Service sshd
    Write-Host "  ✓ SSH service started" -ForegroundColor Green
} catch {
    Write-Host "  ✓ SSH service already running" -ForegroundColor Green
}

Write-Host ""
Write-Host "[3/6] Setting SSH to auto-start..." -ForegroundColor Cyan
Set-Service -Name sshd -StartupType 'Automatic'
Write-Host "  ✓ SSH will start automatically on boot" -ForegroundColor Green

Write-Host ""
Write-Host "[4/6] Configuring Windows Firewall..." -ForegroundColor Cyan
try {
    $existingRule = Get-NetFirewallRule -Name "OpenSSH-Server-In-TCP" -ErrorAction SilentlyContinue
    if ($existingRule) {
        Write-Host "  ✓ Firewall rule already exists" -ForegroundColor Green
    } else {
        New-NetFirewallRule -Name 'OpenSSH-Server-In-TCP' -DisplayName 'OpenSSH Server (sshd)' -Enabled True -Direction Inbound -Protocol TCP -Action Allow -LocalPort 22
        Write-Host "  ✓ Firewall rule created" -ForegroundColor Green
    }
} catch {
    Write-Host "  ⚠ Firewall rule might already exist or need manual configuration" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "[5/6] Configuring auto-login to WSL..." -ForegroundColor Cyan
$sshConfigPath = "C:\ProgramData\ssh\sshd_config"

# Backup original config
if (Test-Path $sshConfigPath) {
    $backupPath = "$sshConfigPath.backup.$(Get-Date -Format 'yyyyMMdd-HHmmss')"
    Copy-Item $sshConfigPath $backupPath
    Write-Host "  ✓ Backed up config to: $backupPath" -ForegroundColor Green
}

# Get current Windows username
$currentUser = $env:USERNAME

# Check if WSL config already exists
$configContent = Get-Content $sshConfigPath -Raw
if ($configContent -notmatch "ForceCommand wsl\.exe") {
    # Add WSL auto-login configuration
    $wslConfig = @"

# Auto-login to WSL for user $currentUser
Match User $currentUser
    ForceCommand wsl.exe
"@
    Add-Content -Path $sshConfigPath -Value $wslConfig
    Write-Host "  ✓ WSL auto-login configured for user: $currentUser" -ForegroundColor Green
} else {
    Write-Host "  ✓ WSL auto-login already configured" -ForegroundColor Green
}

Write-Host ""
Write-Host "[6/6] Restarting SSH service..." -ForegroundColor Cyan
Restart-Service sshd
Write-Host "  ✓ SSH service restarted" -ForegroundColor Green

Write-Host ""
Write-Host "============================================" -ForegroundColor Blue
Write-Host "  ✅ SETUP COMPLETE" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Blue
Write-Host ""

# Get public IP
Write-Host "Getting public IP address..." -ForegroundColor Cyan
try {
    $publicIP = (Invoke-WebRequest -Uri "https://api.ipify.org" -UseBasicParsing).Content
    Write-Host "  Public IP: $publicIP" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "You can now SSH from remote machine:" -ForegroundColor Green
    Write-Host "  ssh $currentUser@$publicIP" -ForegroundColor White
    Write-Host ""
    Write-Host "This will automatically log you into WSL Ubuntu" -ForegroundColor Green
} catch {
    Write-Host "  Could not determine public IP" -ForegroundColor Yellow
    Write-Host "  Run: curl ifconfig.me" -ForegroundColor White
}

Write-Host ""
Write-Host "SSH Server Status:" -ForegroundColor Cyan
Get-Service sshd | Format-Table -AutoSize

Write-Host ""
Write-Host "NOTE: If connecting from outside your network, you may need to:" -ForegroundColor Yellow
Write-Host "  1. Forward port 22 on your router to this PC" -ForegroundColor White
Write-Host "  2. Or use a VPN/Tailscale for secure remote access" -ForegroundColor White
Write-Host ""
