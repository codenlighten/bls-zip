#!/bin/bash
#############################################################################
# Setup SSH Access for Remote Administration
# Configures SSH server on WSL and adds authorized keys
#############################################################################

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}  Setting up SSH Access${NC}"
echo -e "${BLUE}============================================${NC}"

# Install OpenSSH server
echo -e "\n${YELLOW}[1/5]${NC} Installing OpenSSH server..."
sudo apt-get update -qq
sudo apt-get install -y openssh-server

# Configure SSH
echo -e "\n${YELLOW}[2/5]${NC} Configuring SSH server..."
sudo sed -i 's/#Port 22/Port 22/' /etc/ssh/sshd_config
sudo sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
sudo sed -i 's/#PubkeyAuthentication yes/PubkeyAuthentication yes/' /etc/ssh/sshd_config
sudo sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin no/' /etc/ssh/sshd_config

# Create .ssh directory if it doesn't exist
echo -e "\n${YELLOW}[3/5]${NC} Setting up authorized_keys..."
mkdir -p ~/.ssh
chmod 700 ~/.ssh
touch ~/.ssh/authorized_keys
chmod 600 ~/.ssh/authorized_keys

# Add your SSH public key
echo -e "\n${YELLOW}[4/5]${NC} Adding authorized SSH keys..."

# Add codenlighten's key (the one we generated earlier)
cat >> ~/.ssh/authorized_keys << 'EOF'
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOEbvkldSHAFg1cRrc/l2UarJZK7a8JVEv3un2l08MW4 codenlighten-bls-20251118
EOF

echo -e "${GREEN}✅ SSH key added${NC}"

# Start SSH service
echo -e "\n${YELLOW}[5/5]${NC} Starting SSH service..."
sudo service ssh start
sudo service ssh status | head -3

# Get network info
echo -e "\n${GREEN}============================================${NC}"
echo -e "${GREEN}  SSH Setup Complete!${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo -e "Local IP address:"
hostname -I | awk '{print $1}'
echo ""
echo -e "To connect from your machine, use:"
echo -e "  ${BLUE}ssh $(whoami)@<bryan-public-ip>${NC}"
echo ""
echo -e "To auto-start SSH on WSL boot, add to Windows startup:"
echo -e "  ${YELLOW}wsl -d Ubuntu -u root service ssh start${NC}"
echo ""
echo -e "Or configure in ${YELLOW}/etc/wsl.conf${NC}:"
cat << 'EOF'

[boot]
command = service ssh start
EOF

echo -e "\n${GREEN}Note: You'll need to forward port 22 on Bryan's router${NC}"
echo -e "${GREEN}to this WSL IP for external access.${NC}"
