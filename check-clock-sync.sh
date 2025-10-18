#!/bin/bash
#
# Check Clock Synchronization Status
#

SERVER="${1:-schueller.pro}"

echo "====================================="
echo "Clock Sync Diagnostic"
echo "====================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Local Machine (Client):${NC}"
echo "----------------------------------------"
timedatectl status | grep -E "(NTP service|System clock|Time zone)"

if systemctl is-active --quiet systemd-timesyncd; then
    echo -e "${GREEN}✓ NTP is active${NC}"
    timedatectl timesync-status 2>/dev/null || echo "Timesync status not available"
else
    echo -e "${RED}✗ NTP is NOT active${NC}"
    echo "  Fix with: sudo timedatectl set-ntp true"
fi

echo ""
echo -e "${YELLOW}Server ($SERVER):${NC}"
echo "----------------------------------------"
ssh "$SERVER" "
    timedatectl status | grep -E '(NTP service|System clock|Time zone)'
    
    if systemctl is-active --quiet systemd-timesyncd 2>/dev/null; then
        echo -e '✓ NTP is active'
        timedatectl timesync-status 2>/dev/null || echo 'Timesync status not available'
    else
        echo -e '✗ NTP is NOT active'
        echo '  Fix with: sudo timedatectl set-ntp true'
    fi
"

echo ""
echo -e "${YELLOW}Clock Offset Check:${NC}"
echo "----------------------------------------"
LOCAL_TIME=$(date +%s)
REMOTE_TIME=$(ssh "$SERVER" "date +%s")
OFFSET=$((LOCAL_TIME - REMOTE_TIME))

echo "Local time:  $(date -d @$LOCAL_TIME '+%Y-%m-%d %H:%M:%S')"
echo "Server time: $(date -d @$REMOTE_TIME '+%Y-%m-%d %H:%M:%S')"
echo "Offset: ${OFFSET}s"

if [ ${OFFSET#-} -gt 1 ]; then
    echo -e "${RED}✗ Clock offset > 1 second!${NC}"
    echo "  This will cause measurement issues."
    echo "  Enable NTP on both machines."
elif [ ${OFFSET#-} -eq 1 ]; then
    echo -e "${YELLOW}⚠ Clock offset = 1 second${NC}"
    echo "  Should be OK, but NTP recommended."
else
    echo -e "${GREEN}✓ Clocks are synchronized (< 1s offset)${NC}"
fi

echo ""
echo -e "${YELLOW}Recent Measurements:${NC}"
echo "----------------------------------------"
sqlite3 ~/git/connection_check/bufferbane.db "
SELECT 
  datetime(timestamp, 'unixepoch', 'localtime') as time,
  round(rtt_ms,2) as rtt,
  round(upload_latency_ms,2) as up,
  round(download_latency_ms,2) as down,
  server_processing_us as proc,
  CASE 
    WHEN abs(upload_latency_ms - download_latency_ms) < 0.01 THEN '⚠ SYMMETRIC'
    ELSE '✓ ASYMMETRIC'
  END as status
FROM measurements 
WHERE test_type='server_echo' 
ORDER BY timestamp DESC 
LIMIT 3;
" 2>/dev/null || echo "No measurements found"

echo ""
echo "====================================="
echo "Recommendation:"
echo "====================================="

if systemctl is-active --quiet systemd-timesyncd && ssh "$SERVER" "systemctl is-active --quiet systemd-timesyncd 2>/dev/null"; then
    if [ ${OFFSET#-} -lt 1 ]; then
        echo -e "${GREEN}✓ Everything looks good!${NC}"
        echo "  Clocks are synced, NTP is active."
    else
        echo -e "${YELLOW}⚠ NTP is active but offset is high${NC}"
        echo "  Wait 1-2 minutes for NTP to stabilize."
        echo "  Or restart NTP: ssh $SERVER 'sudo systemctl restart systemd-timesyncd'"
    fi
else
    echo -e "${RED}✗ Enable NTP on both machines:${NC}"
    echo ""
    echo "  Local:  sudo timedatectl set-ntp true"
    echo "  Server: ssh $SERVER 'sudo timedatectl set-ntp true'"
    echo ""
    echo "  Then wait 1-2 minutes and re-run this script."
fi

echo ""

