#!/bin/bash
# Bufferbane Server Setup & Deployment Script
# This script automates server deployment and config generation

set -e

echo "=========================================="
echo "Bufferbane Server Setup & Deployment"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if we're in the project root
if [ ! -f "Cargo.toml" ] || [ ! -f "client.conf.template" ]; then
    echo -e "${RED}Error: Please run this script from the project root directory${NC}"
    exit 1
fi

echo "This script will:"
echo "  1. Generate a shared secret"
echo "  2. Create server.conf and client.conf from templates"
echo "  3. Build the server binary"
echo "  4. Deploy to your remote server (optional)"
echo ""

# Get server hostname
echo -e "${YELLOW}Step 1: Server Configuration${NC}"
read -p "Enter server hostname or IP (e.g., monitor.example.com): " SERVER_HOST
if [ -z "$SERVER_HOST" ]; then
    echo -e "${RED}Error: Server hostname is required${NC}"
    exit 1
fi

read -p "Enter server port [9876]: " SERVER_PORT
SERVER_PORT=${SERVER_PORT:-9876}

read -p "Enter SSH user for deployment (leave empty to skip deployment): " SSH_USER

echo ""
echo -e "${YELLOW}Step 2: Generate Shared Secret${NC}"

# Generate shared secret (32 bytes = 64 hex characters)
if command -v openssl &> /dev/null; then
    SHARED_SECRET=$(openssl rand -hex 32)
    echo "✓ Generated secure shared secret using openssl"
elif command -v xxd &> /dev/null && [ -f /dev/urandom ]; then
    SHARED_SECRET=$(head -c 32 /dev/urandom | xxd -p -c 32)
    echo "✓ Generated secure shared secret using /dev/urandom"
else
    echo -e "${RED}Error: Cannot generate random secret (openssl or xxd not found)${NC}"
    exit 1
fi

echo "Shared secret: $SHARED_SECRET"
echo ""

# Generate unique client ID
if command -v openssl &> /dev/null; then
    CLIENT_ID=$(printf "%d" 0x$(openssl rand -hex 8))
else
    CLIENT_ID=$(date +%s)$(shuf -i 1000-9999 -n 1)
fi

# Create server.conf from template
echo -e "${YELLOW}Step 3: Create Configuration Files${NC}"

if [ ! -f "server.conf.template" ]; then
    echo -e "${RED}Error: server.conf.template not found${NC}"
    exit 1
fi

if [ ! -f "client.conf.template" ]; then
    echo -e "${RED}Error: client.conf.template not found${NC}"
    exit 1
fi

# Create server.conf
cp server.conf.template server.conf

# Replace placeholders in server.conf
# Note: server.conf.template should have placeholders like:
# bind_port = 9876
# shared_secret = "GENERATED_SECRET_HERE"

sed -i "s/bind_port = .*/bind_port = $SERVER_PORT/" server.conf
sed -i "s/shared_secret = \".*\"/shared_secret = \"$SHARED_SECRET\"/" server.conf

echo "✓ Created server.conf from template"

# Create client.conf
cp client.conf.template client.conf

# Replace placeholders in client.conf
# Add/update server section
sed -i "s/host = \".*\"/host = \"$SERVER_HOST\"/" client.conf
sed -i "s/port = .*/port = $SERVER_PORT/" client.conf
sed -i "s/client_id = \".*\"/client_id = \"$CLIENT_ID\"/" client.conf
sed -i "/\[server\]/,/^\[/ s/shared_secret = \".*\"/shared_secret = \"$SHARED_SECRET\"/" client.conf

# Make sure server is enabled
sed -i "/\[server\]/,/^\[/ s/enabled = false/enabled = true/" client.conf

echo "✓ Created client.conf from template"
echo ""

# Build server
echo -e "${YELLOW}Step 4: Build Server Binary${NC}"

# Check for rustup and musl target
if command -v rustup &> /dev/null; then
    if ! rustup target list 2>/dev/null | grep -q "x86_64-unknown-linux-musl (installed)"; then
        echo "Installing musl target for static builds..."
        rustup target add x86_64-unknown-linux-musl
    fi
    echo "Building bufferbane-server with musl (fully static, works on any Linux)..."
    cargo build --release --target x86_64-unknown-linux-musl -p bufferbane-server
    SERVER_BINARY="target/x86_64-unknown-linux-musl/release/bufferbane-server"
    echo "✓ Server binary built (static musl): $SERVER_BINARY"
    echo "  This binary works on any Linux (no GLIBC dependency)"
elif command -v musl-gcc &> /dev/null; then
    echo "Warning: rustup not found, using musl-gcc..."
    echo "Building bufferbane-server with musl..."
    cargo build --release --target x86_64-unknown-linux-musl -p bufferbane-server
    SERVER_BINARY="target/x86_64-unknown-linux-musl/release/bufferbane-server"
    echo "✓ Server binary built (static musl): $SERVER_BINARY"
else
    echo "Warning: Neither rustup nor musl-gcc found - building with default target"
    echo "This may cause GLIBC compatibility issues on older systems."
    echo "To fix, install rustup:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    cargo build --release -p bufferbane-server
    SERVER_BINARY="target/release/bufferbane-server"
    echo "✓ Server binary built: $SERVER_BINARY"
fi
echo ""

# Deploy to server (if SSH_USER provided)
if [ -n "$SSH_USER" ]; then
    echo -e "${YELLOW}Step 5: Deploy to Remote Server${NC}"
    echo "This will:"
    echo "  1. Create /opt/bufferbane on $SERVER_HOST"
    echo "  2. Copy bufferbane-server binary"
    echo "  3. Copy server.conf"
    echo ""
    read -p "Proceed with deployment? [Y/n]: " DEPLOY
    DEPLOY=${DEPLOY:-Y}

    if [[ "$DEPLOY" =~ ^[Yy]$ ]]; then
        echo "Deploying to $SSH_USER@$SERVER_HOST..."
        
        # Create directory on remote
        ssh "$SSH_USER@$SERVER_HOST" "mkdir -p /opt/bufferbane"
        
        # Copy files
        scp "$SERVER_BINARY" "$SSH_USER@$SERVER_HOST:/opt/bufferbane/bufferbane-server"
        scp server.conf "$SSH_USER@$SERVER_HOST:/opt/bufferbane/"
        
        echo "✓ Deployment complete"
        
        DEPLOYED=true
    else
        echo "Skipping deployment."
        DEPLOYED=false
    fi
else
    echo -e "${YELLOW}Step 5: Deployment${NC}"
    echo "Skipping deployment (no SSH user provided)."
    echo "You can deploy manually later using:"
    echo "  scp $SERVER_BINARY user@$SERVER_HOST:/opt/bufferbane/bufferbane-server"
    echo "  scp server.conf user@$SERVER_HOST:/opt/bufferbane/"
    DEPLOYED=false
fi

echo ""
echo -e "${GREEN}=========================================="
echo "Setup Complete!"
echo -e "==========================================${NC}"
echo ""
echo -e "${YELLOW}Configuration Files Created:${NC}"
echo "  ✓ server.conf - Ready for deployment"
echo "  ✓ client.conf - Ready to use locally"
echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo ""

if [ "$DEPLOYED" = true ]; then
    echo "1. Start the SERVER on $SERVER_HOST:"
    echo "   ssh $SSH_USER@$SERVER_HOST"
    echo "   cd /opt/bufferbane"
    echo "   ./bufferbane-server --config server.conf"
    echo ""
    echo "   Or run in background:"
    echo "   nohup ./bufferbane-server --config server.conf > server.log 2>&1 &"
else
    echo "1. Deploy to server manually:"
    echo "   ssh user@$SERVER_HOST 'mkdir -p /opt/bufferbane'"
    echo "   scp target/release/bufferbane-server user@$SERVER_HOST:/opt/bufferbane/"
    echo "   scp server.conf user@$SERVER_HOST:/opt/bufferbane/"
    echo ""
    echo "2. Start the SERVER:"
    echo "   ssh user@$SERVER_HOST"
    echo "   cd /opt/bufferbane"
    echo "   ./bufferbane-server --config server.conf"
fi

echo ""
echo "2. Start the CLIENT locally:"
echo "   ./target/release/bufferbane --config client.conf"
echo ""
echo "3. Monitor server logs (on server):"
echo "   tail -f /opt/bufferbane/server.log"
echo "   # Or use: RUST_LOG=info ./bufferbane-server --config server.conf"
echo ""
echo "4. Open firewall on server (if needed):"
echo "   sudo ufw allow $SERVER_PORT/udp"
echo "   # Or: sudo firewall-cmd --add-port=$SERVER_PORT/udp --permanent"
echo ""
echo -e "${YELLOW}Security:${NC}"
echo "  Protect your configuration files (they contain the shared secret):"
echo "    chmod 600 client.conf"
if [ "$DEPLOYED" = true ]; then
    echo "    ssh $SSH_USER@$SERVER_HOST 'chmod 600 /opt/bufferbane/server.conf'"
fi
echo ""
echo -e "${YELLOW}Verification:${NC}"
echo "  Server should log: 'Server listening on 0.0.0.0:$SERVER_PORT'"
echo "  Client should show server tests alongside ICMP tests"
echo ""
echo -e "${GREEN}Happy monitoring with Bufferbane!${NC}"
echo ""
echo "For detailed setup instructions, see: PHASE2_SETUP.md"
