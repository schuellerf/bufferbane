#!/bin/bash
#
# Test Built-In Offset Calculation
# Run this to verify the algorithm works
#

echo "====================================="
echo "Testing Built-In Clock Offset"
echo "====================================="
echo ""

# Run client for 30 seconds with debug logging
echo "Running client for 30 seconds..."
echo "(Watch for clock offset convergence)"
echo ""

timeout 30 ./target/release/bufferbane 2>&1 | grep -E "(Clock offset|Server ECHO test)" | head -20

echo ""
echo "====================================="
echo "Checking Database Results"
echo "====================================="
echo ""

sqlite3 bufferbane.db "
SELECT 
  datetime(timestamp, 'unixepoch', 'localtime') as time,
  round(rtt_ms,2) as rtt,
  round(upload_latency_ms,2) as up,
  round(download_latency_ms,2) as down,
  server_processing_us as proc,
  round(upload_latency_ms + download_latency_ms, 2) as sum,
  round(abs(rtt_ms - (upload_latency_ms + download_latency_ms + server_processing_us/1000.0)), 2) as error
FROM measurements 
WHERE test_type='server_echo' 
  AND timestamp > strftime('%s', 'now', '-1 minute')
ORDER BY timestamp DESC 
LIMIT 10;
" 2>/dev/null

echo ""
echo "====================================="
echo "Analysis"
echo "====================================="
echo ""
echo "✅ If 'error' column is < 1ms: Algorithm working perfectly!"
echo "⚠️  If 'error' column is > 5ms: Path might be asymmetric"
echo "✅ If up ≠ down: Real asymmetry detected (your goal!)"
echo ""
echo "Generate chart to visualize:"
echo "  ./target/release/bufferbane --chart --interactive --last 5m"
echo ""

