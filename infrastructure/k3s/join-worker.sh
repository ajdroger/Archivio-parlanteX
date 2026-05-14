#!/bin/bash
set -e

###############################################################################
# k3s Worker Join Script
# Archivio Parlante - Zero-Cost Kubernetes
###############################################################################

echo "🚀 Joining k3s Cluster as Worker Node..."
echo "================================================"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check arguments
if [ $# -ne 2 ]; then
  echo -e "${RED}Usage: $0 <K3S_URL> <K3S_TOKEN>${NC}"
  echo ""
  echo "Example:"
  echo "  $0 https://10.0.0.10:6443 K10abc123..."
  echo ""
  echo "Get these values from master node:"
  echo "  cat ~/k3s-join-command.txt"
  exit 1
fi

K3S_URL=$1
K3S_TOKEN=$2

# Checks
if [ "$EUID" -eq 0 ]; then
  echo -e "${RED}❌ Do not run as root. Run as ubuntu user.${NC}"
  exit 1
fi

echo -e "${YELLOW}📋 Pre-flight checks...${NC}"

# Check RAM
TOTAL_RAM=$(free -g | awk '/^Mem:/{print $2}')
if [ "$TOTAL_RAM" -lt 10 ]; then
  echo -e "${RED}❌ Insufficient RAM: ${TOTAL_RAM}GB (need 10GB+)${NC}"
  exit 1
fi
echo -e "${GREEN}✅ RAM: ${TOTAL_RAM}GB${NC}"

# Update system
echo -e "${YELLOW}📦 Updating system packages...${NC}"
sudo apt-get update -qq
sudo apt-get install -y curl wget

# Disable swap
echo -e "${YELLOW}💾 Disabling swap...${NC}"
sudo swapoff -a
sudo sed -i '/ swap / s/^/#/' /etc/fstab

# Configure firewall
echo -e "${YELLOW}🔥 Configuring firewall...${NC}"
sudo ufw --force enable
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 10250/tcp # Kubelet
sudo ufw allow from 10.0.0.0/8  # Internal traffic
sudo ufw reload

# Install k3s agent
echo -e "${YELLOW}🎯 Installing k3s agent and joining cluster...${NC}"
curl -sfL https://get.k3s.io | K3S_URL="${K3S_URL}" \
  K3S_TOKEN="${K3S_TOKEN}" \
  INSTALL_K3S_EXEC="agent \
    --kubelet-arg system-reserved=memory=1Gi \
    --kubelet-arg kube-reserved=memory=512Mi" sh -

# Wait for k3s agent to start
echo -e "${YELLOW}⏳ Waiting for k3s agent to start...${NC}"
sleep 10

# Check status
sudo systemctl status k3s-agent --no-pager || true

echo ""
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}✅ Worker Node Joined Successfully!${NC}"
echo -e "${GREEN}================================================${NC}"
echo ""
echo -e "${YELLOW}📋 Verify on master node:${NC}"
echo "   kubectl get nodes"
echo ""
echo -e "${YELLOW}Expected output:${NC}"
echo "   NAME                    STATUS   ROLES                  AGE"
echo "   archivio-k3s-master     Ready    control-plane,master   10m"
echo "   archivio-k3s-worker     Ready    <none>                 1m"
echo ""
