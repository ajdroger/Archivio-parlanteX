#!/bin/bash
set -e

###############################################################################
# k3s Installation Script - Master Node
# Archivio Parlante - Zero-Cost Kubernetes
###############################################################################

echo "🚀 Installing k3s on Master Node..."
echo "================================================"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

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

# Check connectivity
if ! ping -c 1 8.8.8.8 &> /dev/null; then
  echo -e "${RED}❌ No internet connectivity${NC}"
  exit 1
fi
echo -e "${GREEN}✅ Internet connection OK${NC}"

# Update system
echo -e "${YELLOW}📦 Updating system packages...${NC}"
sudo apt-get update -qq
sudo apt-get install -y curl wget git

# Disable swap (k8s requirement)
echo -e "${YELLOW}💾 Disabling swap...${NC}"
sudo swapoff -a
sudo sed -i '/ swap / s/^/#/' /etc/fstab

# Configure firewall (UFW)
echo -e "${YELLOW}🔥 Configuring firewall...${NC}"
sudo ufw --force enable
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 6443/tcp  # Kubernetes API
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 10250/tcp # Kubelet
sudo ufw allow from 10.0.0.0/8  # Internal VCN traffic
sudo ufw reload
echo -e "${GREEN}✅ Firewall configured${NC}"

# Install k3s
echo -e "${YELLOW}🎯 Installing k3s (lightweight Kubernetes)...${NC}"
curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC="server \
  --disable traefik \
  --disable servicelb \
  --write-kubeconfig-mode 644 \
  --kube-apiserver-arg enable-admission-plugins=NodeRestriction,NamespaceLifecycle,ServiceAccount \
  --kubelet-arg system-reserved=memory=1Gi \
  --kubelet-arg kube-reserved=memory=1Gi" sh -

# Wait for k3s to be ready
echo -e "${YELLOW}⏳ Waiting for k3s to start...${NC}"
sleep 10

# Check k3s status
sudo systemctl status k3s --no-pager || true

# Verify installation
echo -e "${YELLOW}🔍 Verifying k3s installation...${NC}"
if sudo k3s kubectl get nodes; then
  echo -e "${GREEN}✅ k3s installed successfully!${NC}"
else
  echo -e "${RED}❌ k3s installation failed${NC}"
  exit 1
fi

# Setup kubectl for ubuntu user
echo -e "${YELLOW}⚙️  Setting up kubectl for ubuntu user...${NC}"
mkdir -p $HOME/.kube
sudo cp /etc/rancher/k3s/k3s.yaml $HOME/.kube/config
sudo chown $(id -u):$(id -g) $HOME/.kube/config
chmod 600 $HOME/.kube/config

# Add kubectl alias
if ! grep -q "alias k=" ~/.bashrc; then
  echo "alias k='kubectl'" >> ~/.bashrc
  echo "source <(kubectl completion bash)" >> ~/.bashrc
  echo "complete -F __start_kubectl k" >> ~/.bashrc
fi

# Test kubectl
echo -e "${YELLOW}🧪 Testing kubectl...${NC}"
if kubectl get nodes; then
  echo -e "${GREEN}✅ kubectl configured successfully!${NC}"
else
  echo -e "${RED}❌ kubectl configuration failed${NC}"
  exit 1
fi

# Get node token for workers
echo -e "${YELLOW}🔑 Saving node token for worker nodes...${NC}"
NODE_TOKEN=$(sudo cat /var/lib/rancher/k3s/server/node-token)
MASTER_IP=$(hostname -I | awk '{print $1}')

# Save token to file
echo "K3S_TOKEN=${NODE_TOKEN}" > ~/k3s-join-command.txt
echo "K3S_URL=https://${MASTER_IP}:6443" >> ~/k3s-join-command.txt
chmod 600 ~/k3s-join-command.txt

echo ""
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}✅ k3s Master Node Installation Complete!${NC}"
echo -e "${GREEN}================================================${NC}"
echo ""
echo -e "${YELLOW}📋 Next Steps:${NC}"
echo ""
echo "1. Copy the join command to worker nodes:"
echo -e "   ${GREEN}cat ~/k3s-join-command.txt${NC}"
echo ""
echo "2. On worker node, run:"
echo -e "   ${GREEN}./join-worker.sh <K3S_URL> <K3S_TOKEN>${NC}"
echo ""
echo "3. Verify nodes:"
echo -e "   ${GREEN}kubectl get nodes${NC}"
echo ""
echo -e "${YELLOW}📊 Cluster Info:${NC}"
echo -e "   Master IP: ${GREEN}${MASTER_IP}${NC}"
echo -e "   API Server: ${GREEN}https://${MASTER_IP}:6443${NC}"
echo ""
echo -e "${YELLOW}💡 Useful Commands:${NC}"
echo "   kubectl get pods -A    # List all pods"
echo "   kubectl get nodes      # List nodes"
echo "   k describe node        # Node details (k is alias)"
echo ""
