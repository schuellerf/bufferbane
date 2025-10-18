#!/bin/bash
#
# Quick Deploy and Test Script
# Deploys updated server and tests clock sync
#

set -e

SERVER_HOST="${1:-server.example.com}"
SERVER_USER="${2:-${USER}}"

echo "==================================="
echo "Bufferbane Server Deploy & Test"
echo "==================================="
echo ""
echo "Target: $SERVER_USER@$SERVER_HOST"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Step 1: Check NTP on local machine${NC}"
if systemctl is-active --quiet systemd-timesyncd; then
    echo -e "${GREEN}✓ NTP is active${NC}"
else
    echo -e "${YELLOW}! NTP not active, enabling...${NC}"
    sudo timedatectl set-ntp true
    sleep 2
    echo -e "${GREEN}✓ NTP enabled${NC}"
fi

echo ""
echo -e "${YELLOW}Step 2: Deploy server binary${NC}"
echo "Copying bufferbane-server to $SERVER_HOST..."
scp -q target/x86_64-unknown-linux-musl/release/bufferbane-server "$SERVER_USER@$SERVER_HOST:/opt/bufferbane/" || {
    echo -e "${RED}✗ Failed to copy binary${NC}"
    echo "Make sure you've built with: make build-server-static"
    exit 1
}
echo -e "${GREEN}✓ Server binary deployed${NC}"

echo ""
echo -e "${YELLOW}Step 3: Check/Enable NTP on server${NC}"
ssh "$SERVER_USER@$SERVER_HOST" '
    if systemctl is-active --quiet systemd-timesyncd 2>/dev/null; then
        echo "✓ NTP is active on server"
    else
        echo "! NTP not active, attempting to enable..."
        sudo timedatectl set-ntp true 2>/dev/null || echo "! Could not enable NTP (may need manual setup)"
    fi
    
    echo ""
    echo "Server time sync status:"
    timedatectl timesync-status 2>/dev/null || timedatectl status | grep -i ntp
'

echo ""
echo -e "${YELLOW}Step 4: Restart server${NC}"
ssh "$SERVER_USER@$SERVER_HOST" '
    cd /opt/bufferbane
    
    # Stop old server
    pkill -9 bufferbane-server 2>/dev/null || true
    sleep 1
    
    # Start new server
    nohup ./bufferbane-server --config server.conf > server.log 2>&1 &
    
    sleep 2
    
    # Check if running
    if pgrep bufferbane-server > /dev/null; then
        echo "✓ Server started successfully"
        echo ""
        tail -5 server.log
    else
        echo "✗ Server failed to start"
        echo "Last 10 lines of log:"
        tail -10 server.log
        exit 1
    fi
'

echo ""
echo -e "${GREEN}==================================="
echo "Deployment Complete!"
echo "===================================${NC}"
echo ""
echo "Next steps:"
echo "1. Run client: ./target/release/bufferbane"
echo "2. Wait 2-5 minutes"
echo "3. Generate chart: ./target/release/bufferbane --chart --interactive --last 5m"
echo "4. Open chart: firefox latency_*.html"
echo ""
echo "To check data:"
echo "  sqlite3 bufferbane.db 'SELECT datetime(timestamp, \"unixepoch\", \"localtime\") as time, round(rtt_ms,2) as rtt, round(upload_latency_ms,2) as upload, round(download_latency_ms,2) as download FROM measurements WHERE test_type=\"server_echo\" ORDER BY timestamp DESC LIMIT 5;'"
echo ""

