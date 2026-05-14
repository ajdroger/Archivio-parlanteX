#!/bin/bash
set -e

###############################################################################
# Oracle Cloud Account Setup & VM Creation - FULLY AUTOMATED
# Archivio Parlante - Zero-Cost Infrastructure
###############################################################################

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}"
cat << "EOF"
╔═══════════════════════════════════════════════════════════╗
║   Oracle Cloud Free Tier - Automated Setup                ║
║   Creating VMs for Archivio Parlante                      ║
╚═══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# ============================================================================
# Configuration
# ============================================================================

# Oracle Cloud Details (will be prompted or read from file)
ORACLE_USER="ajmeer03@gmail.com"
CLOUD_ACCOUNT="ajmeer03"

# VM Configuration
VM1_NAME="archivio-k3s-master"
VM2_NAME="archivio-k3s-worker"
VM_SHAPE="VM.Standard.A1.Flex"
VM_OCPU=2
VM_MEMORY=12  # GB
VM_IMAGE="Ubuntu 22.04 Minimal"  # Will get exact OCID later
VCN_NAME="archivio-vcn"
SUBNET_NAME="archivio-subnet"

# Region (EU compliance)
OCI_REGION="eu-frankfurt-1"  # Germany datacenter

# ============================================================================
# Step 1: Check if Oracle CLI is installed
# ============================================================================

echo -e "${YELLOW}📋 Step 1: Checking Oracle CLI...${NC}"

if ! command -v oci &> /dev/null; then
  echo -e "${YELLOW}Oracle CLI not found. Installing...${NC}"

  # Install Oracle CLI
  if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux installation
    bash -c "$(curl -L https://raw.githubusercontent.com/oracle/oci-cli/master/scripts/install/install.sh)" -- --accept-all-defaults

    # Add to PATH for current session
    export PATH="$HOME/bin:$PATH"
  else
    echo -e "${RED}❌ Please install Oracle CLI manually:${NC}"
    echo "   https://docs.oracle.com/en-us/iaas/Content/API/SDKDocs/cliinstall.htm"
    exit 1
  fi
fi

echo -e "${GREEN}✅ Oracle CLI installed${NC}"

# ============================================================================
# Step 2: Configure Oracle CLI
# ============================================================================

echo ""
echo -e "${YELLOW}🔐 Step 2: Configuring Oracle CLI...${NC}"

# Check if already configured
if [ -f ~/.oci/config ]; then
  echo -e "${GREEN}✅ Oracle CLI already configured${NC}"
  echo -e "${YELLOW}Using existing configuration...${NC}"
else
  echo -e "${YELLOW}Setting up Oracle CLI configuration...${NC}"
  echo ""
  echo -e "${BLUE}📝 You'll need to provide:${NC}"
  echo "  1. Your Oracle Cloud tenancy OCID"
  echo "  2. Your user OCID"
  echo "  3. Region: ${OCI_REGION}"
  echo ""
  echo -e "${YELLOW}Get these from Oracle Cloud Console:${NC}"
  echo "  https://cloud.oracle.com → Profile → Tenancy"
  echo ""

  # Interactive setup
  oci setup config

  echo ""
  echo -e "${GREEN}✅ Oracle CLI configured${NC}"
fi

# Test connection
echo -e "${YELLOW}Testing connection to Oracle Cloud...${NC}"
if oci iam region list &> /dev/null; then
  echo -e "${GREEN}✅ Connected to Oracle Cloud${NC}"
else
  echo -e "${RED}❌ Cannot connect to Oracle Cloud${NC}"
  echo "Please check your configuration: ~/.oci/config"
  exit 1
fi

# ============================================================================
# Step 3: Get Compartment OCID
# ============================================================================

echo ""
echo -e "${YELLOW}📦 Step 3: Getting compartment information...${NC}"

# Get root compartment (tenancy)
TENANCY_OCID=$(grep tenancy ~/.oci/config | cut -d'=' -f2 | tr -d ' ')
COMPARTMENT_OCID="$TENANCY_OCID"  # Use root compartment for simplicity

echo -e "${GREEN}✅ Using compartment: $COMPARTMENT_OCID${NC}"

# ============================================================================
# Step 4: Get Availability Domain
# ============================================================================

echo ""
echo -e "${YELLOW}🌍 Step 4: Getting availability domain...${NC}"

AVAILABILITY_DOMAIN=$(oci iam availability-domain list \
  --compartment-id "$COMPARTMENT_OCID" \
  --query 'data[0].name' \
  --raw-output)

echo -e "${GREEN}✅ Availability Domain: $AVAILABILITY_DOMAIN${NC}"

# ============================================================================
# Step 5: Get ARM Image OCID
# ============================================================================

echo ""
echo -e "${YELLOW}🖼️  Step 5: Finding Ubuntu ARM64 image...${NC}"

IMAGE_OCID=$(oci compute image list \
  --compartment-id "$COMPARTMENT_OCID" \
  --operating-system "Canonical Ubuntu" \
  --operating-system-version "22.04" \
  --shape "$VM_SHAPE" \
  --limit 1 \
  --query 'data[0].id' \
  --raw-output)

if [ -z "$IMAGE_OCID" ]; then
  echo -e "${RED}❌ Ubuntu ARM64 image not found${NC}"
  exit 1
fi

echo -e "${GREEN}✅ Image OCID: $IMAGE_OCID${NC}"

# ============================================================================
# Step 6: Create or Get VCN
# ============================================================================

echo ""
echo -e "${YELLOW}🌐 Step 6: Setting up Virtual Cloud Network...${NC}"

# Check if VCN already exists
VCN_OCID=$(oci network vcn list \
  --compartment-id "$COMPARTMENT_OCID" \
  --display-name "$VCN_NAME" \
  --query 'data[0].id' \
  --raw-output 2>/dev/null || echo "")

if [ -z "$VCN_OCID" ]; then
  echo -e "${YELLOW}Creating new VCN: $VCN_NAME${NC}"

  VCN_OCID=$(oci network vcn create \
    --compartment-id "$COMPARTMENT_OCID" \
    --display-name "$VCN_NAME" \
    --cidr-block "10.0.0.0/16" \
    --dns-label "archiviov" \
    --wait-for-state AVAILABLE \
    --query 'data.id' \
    --raw-output)

  echo -e "${GREEN}✅ VCN created: $VCN_OCID${NC}"
else
  echo -e "${GREEN}✅ Using existing VCN: $VCN_OCID${NC}"
fi

# Create Internet Gateway if not exists
IGW_OCID=$(oci network internet-gateway list \
  --compartment-id "$COMPARTMENT_OCID" \
  --vcn-id "$VCN_OCID" \
  --query 'data[0].id' \
  --raw-output 2>/dev/null || echo "")

if [ -z "$IGW_OCID" ]; then
  echo -e "${YELLOW}Creating Internet Gateway...${NC}"

  IGW_OCID=$(oci network internet-gateway create \
    --compartment-id "$COMPARTMENT_OCID" \
    --vcn-id "$VCN_OCID" \
    --is-enabled true \
    --display-name "archivio-igw" \
    --wait-for-state AVAILABLE \
    --query 'data.id' \
    --raw-output)

  # Add route to default route table
  ROUTE_TABLE_OCID=$(oci network vcn list \
    --compartment-id "$COMPARTMENT_OCID" \
    --display-name "$VCN_NAME" \
    --query 'data[0]."default-route-table-id"' \
    --raw-output)

  oci network route-table update \
    --rt-id "$ROUTE_TABLE_OCID" \
    --route-rules "[{\"destination\": \"0.0.0.0/0\", \"networkEntityId\": \"$IGW_OCID\"}]" \
    --force &> /dev/null

  echo -e "${GREEN}✅ Internet Gateway created${NC}"
fi

# Create subnet if not exists
SUBNET_OCID=$(oci network subnet list \
  --compartment-id "$COMPARTMENT_OCID" \
  --vcn-id "$VCN_OCID" \
  --display-name "$SUBNET_NAME" \
  --query 'data[0].id' \
  --raw-output 2>/dev/null || echo "")

if [ -z "$SUBNET_OCID" ]; then
  echo -e "${YELLOW}Creating subnet...${NC}"

  SUBNET_OCID=$(oci network subnet create \
    --compartment-id "$COMPARTMENT_OCID" \
    --vcn-id "$VCN_OCID" \
    --cidr-block "10.0.0.0/24" \
    --display-name "$SUBNET_NAME" \
    --dns-label "archiviosub" \
    --wait-for-state AVAILABLE \
    --query 'data.id' \
    --raw-output)

  echo -e "${GREEN}✅ Subnet created${NC}"
fi

# Configure security list (firewall rules)
SECLIST_OCID=$(oci network vcn list \
  --compartment-id "$COMPARTMENT_OCID" \
  --display-name "$VCN_NAME" \
  --query 'data[0]."default-security-list-id"' \
  --raw-output)

echo -e "${YELLOW}Configuring firewall rules...${NC}"

oci network security-list update \
  --security-list-id "$SECLIST_OCID" \
  --ingress-security-rules '[
    {"protocol": "6", "source": "0.0.0.0/0", "tcpOptions": {"destinationPortRange": {"min": 22, "max": 22}}},
    {"protocol": "6", "source": "0.0.0.0/0", "tcpOptions": {"destinationPortRange": {"min": 80, "max": 80}}},
    {"protocol": "6", "source": "0.0.0.0/0", "tcpOptions": {"destinationPortRange": {"min": 443, "max": 443}}},
    {"protocol": "6", "source": "0.0.0.0/0", "tcpOptions": {"destinationPortRange": {"min": 6443, "max": 6443}}},
    {"protocol": "all", "source": "10.0.0.0/16"}
  ]' \
  --force &> /dev/null

echo -e "${GREEN}✅ Firewall rules configured${NC}"

# ============================================================================
# Step 7: Generate SSH Key
# ============================================================================

echo ""
echo -e "${YELLOW}🔑 Step 7: Setting up SSH keys...${NC}"

SSH_KEY_PATH="$HOME/.ssh/oracle_archivio"

if [ ! -f "$SSH_KEY_PATH" ]; then
  echo -e "${YELLOW}Generating new SSH key...${NC}"
  ssh-keygen -t rsa -b 4096 -f "$SSH_KEY_PATH" -N "" -C "archivio-parlante"
  echo -e "${GREEN}✅ SSH key generated: $SSH_KEY_PATH${NC}"
else
  echo -e "${GREEN}✅ Using existing SSH key: $SSH_KEY_PATH${NC}"
fi

SSH_PUBLIC_KEY=$(cat "${SSH_KEY_PATH}.pub")

# ============================================================================
# Step 8: Create VM1 (Master Node)
# ============================================================================

echo ""
echo -e "${YELLOW}🖥️  Step 8: Creating VM1 (Master Node)...${NC}"
echo -e "${BLUE}This may take 2-5 minutes...${NC}"

# Check if VM already exists
VM1_OCID=$(oci compute instance list \
  --compartment-id "$COMPARTMENT_OCID" \
  --display-name "$VM1_NAME" \
  --query 'data[0].id' \
  --raw-output 2>/dev/null || echo "")

if [ -z "$VM1_OCID" ]; then
  # Create VM1
  VM1_OCID=$(oci compute instance launch \
    --compartment-id "$COMPARTMENT_OCID" \
    --availability-domain "$AVAILABILITY_DOMAIN" \
    --shape "$VM_SHAPE" \
    --shape-config "{\"ocpus\": $VM_OCPU, \"memoryInGBs\": $VM_MEMORY}" \
    --display-name "$VM1_NAME" \
    --image-id "$IMAGE_OCID" \
    --subnet-id "$SUBNET_OCID" \
    --assign-public-ip true \
    --ssh-authorized-keys-file "${SSH_KEY_PATH}.pub" \
    --wait-for-state RUNNING \
    --query 'data.id' \
    --raw-output)

  echo -e "${GREEN}✅ VM1 created: $VM1_OCID${NC}"
else
  echo -e "${GREEN}✅ VM1 already exists: $VM1_OCID${NC}"
fi

# Get VM1 public IP
echo -e "${YELLOW}Getting VM1 public IP...${NC}"
sleep 5  # Wait for network to attach

VM1_VNIC_ID=$(oci compute instance list-vnics \
  --instance-id "$VM1_OCID" \
  --query 'data[0].id' \
  --raw-output)

VM1_PUBLIC_IP=$(oci network vnic get \
  --vnic-id "$VM1_VNIC_ID" \
  --query 'data."public-ip"' \
  --raw-output)

echo -e "${GREEN}✅ VM1 Public IP: ${VM1_PUBLIC_IP}${NC}"

# ============================================================================
# Step 9: Create VM2 (Worker Node)
# ============================================================================

echo ""
echo -e "${YELLOW}🖥️  Step 9: Creating VM2 (Worker Node)...${NC}"
echo -e "${BLUE}This may take 2-5 minutes...${NC}"

# Check if VM already exists
VM2_OCID=$(oci compute instance list \
  --compartment-id "$COMPARTMENT_OCID" \
  --display-name "$VM2_NAME" \
  --query 'data[0].id' \
  --raw-output 2>/dev/null || echo "")

if [ -z "$VM2_OCID" ]; then
  # Create VM2
  VM2_OCID=$(oci compute instance launch \
    --compartment-id "$COMPARTMENT_OCID" \
    --availability-domain "$AVAILABILITY_DOMAIN" \
    --shape "$VM_SHAPE" \
    --shape-config "{\"ocpus\": $VM_OCPU, \"memoryInGBs\": $VM_MEMORY}" \
    --display-name "$VM2_NAME" \
    --image-id "$IMAGE_OCID" \
    --subnet-id "$SUBNET_OCID" \
    --assign-public-ip true \
    --ssh-authorized-keys-file "${SSH_KEY_PATH}.pub" \
    --wait-for-state RUNNING \
    --query 'data.id' \
    --raw-output)

  echo -e "${GREEN}✅ VM2 created: $VM2_OCID${NC}"
else
  echo -e "${GREEN}✅ VM2 already exists: $VM2_OCID${NC}"
fi

# Get VM2 public IP
echo -e "${YELLOW}Getting VM2 public IP...${NC}"
sleep 5

VM2_VNIC_ID=$(oci compute instance list-vnics \
  --instance-id "$VM2_OCID" \
  --query 'data[0].id' \
  --raw-output)

VM2_PUBLIC_IP=$(oci network vnic get \
  --vnic-id "$VM2_VNIC_ID" \
  --query 'data."public-ip"' \
  --raw-output)

echo -e "${GREEN}✅ VM2 Public IP: ${VM2_PUBLIC_IP}${NC}"

# ============================================================================
# Step 10: Save Configuration
# ============================================================================

echo ""
echo -e "${YELLOW}💾 Step 10: Saving configuration...${NC}"

# Save to file for easy reference
cat > ~/oracle-vms-info.txt << EOF
# Archivio Parlante - Oracle Cloud VMs
# Created: $(date)

VM1 (Master):
  Name: $VM1_NAME
  OCID: $VM1_OCID
  Public IP: $VM1_PUBLIC_IP
  SSH: ssh -i $SSH_KEY_PATH ubuntu@$VM1_PUBLIC_IP

VM2 (Worker):
  Name: $VM2_NAME
  OCID: $VM2_OCID
  Public IP: $VM2_PUBLIC_IP
  SSH: ssh -i $SSH_KEY_PATH ubuntu@$VM2_PUBLIC_IP

Network:
  VCN: $VCN_NAME ($VCN_OCID)
  Subnet: $SUBNET_NAME ($SUBNET_OCID)
  Region: $OCI_REGION

SSH Key: $SSH_KEY_PATH
EOF

chmod 600 ~/oracle-vms-info.txt

echo -e "${GREEN}✅ Configuration saved to: ~/oracle-vms-info.txt${NC}"

# ============================================================================
# Summary
# ============================================================================

echo ""
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}✅ Oracle Cloud Setup Complete!${NC}"
echo -e "${GREEN}================================================${NC}"
echo ""
echo -e "${YELLOW}📊 Resources Created:${NC}"
echo "  • VCN: $VCN_NAME"
echo "  • VM1 (Master): $VM1_PUBLIC_IP"
echo "  • VM2 (Worker): $VM2_PUBLIC_IP"
echo ""
echo -e "${YELLOW}🔐 SSH Access:${NC}"
echo "  VM1: ${GREEN}ssh -i $SSH_KEY_PATH ubuntu@$VM1_PUBLIC_IP${NC}"
echo "  VM2: ${GREEN}ssh -i $SSH_KEY_PATH ubuntu@$VM2_PUBLIC_IP${NC}"
echo ""
echo -e "${YELLOW}📝 Next Steps:${NC}"
echo "  1. Test SSH connection to VM1"
echo "  2. Clone repository on VM1"
echo "  3. Run: ${GREEN}./infrastructure/k3s/install-k3s.sh${NC}"
echo ""
echo -e "${YELLOW}💡 Quick Connect:${NC}"
echo "  ${GREEN}ssh -i $SSH_KEY_PATH ubuntu@$VM1_PUBLIC_IP${NC}"
echo ""
