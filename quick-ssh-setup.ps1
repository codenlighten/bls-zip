#############################################################################
# One-Command Windows OpenSSH + WSL Node Setup
# Downloads and runs the SSH setup script
# Run in PowerShell as Administrator
#############################################################################

# Enable TLS 1.2 for downloads
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

Write-Host "Downloading and running Windows OpenSSH setup..." -ForegroundColor Cyan

# Download and execute the setup script
Invoke-Expression (Invoke-WebRequest -Uri "https://raw.githubusercontent.com/codenlighten/bls-zip/main/setup-windows-ssh.ps1" -UseBasicParsing).Content
