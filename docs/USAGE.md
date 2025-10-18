# Bufferbane Usage Guide

Complete usage guide for Bufferbane network monitoring tool.

## Table of Contents

- [Installation](#installation)
- [Configuration](#configuration)
- [Running Modes](#running-modes)
- [Common Workflows](#common-workflows)
- [Systemd Service](#systemd-service)
- [Tips & Tricks](#tips--tricks)

---

## Installation

### Build from Source

```bash
# Clone repository
git clone https://github.com/schuellerf/bufferbane.git
cd bufferbane

# Build release binary
cargo build --release

# Binary location
./target/release/bufferbane
```

### Set ICMP Permissions

Bufferbane requires raw socket access for ICMP pings:

```bash
# Option 1: Set capability (recommended)
sudo setcap cap_net_raw+ep ./target/release/bufferbane

# Option 2: Run with sudo
sudo ./target/release/bufferbane
```

### Create Configuration

```bash
# Copy template
cp client.conf.template client.conf

# Edit if needed (optional - defaults work fine)
vim client.conf
```

---

## Configuration

### Minimal Configuration

The template works out of the box! Just copy and run:

```bash
cp client.conf.template client.conf
./target/release/bufferbane
```

### Key Settings to Customize

#### Test Targets

```toml
[targets]
# Default public DNS servers
public_dns = ["8.8.8.8", "1.1.1.1"]

# Add your own targets (ISP gateway, router, etc.)
custom = ["192.168.1.1", "10.0.0.1"]
```

#### Test Interval

```toml
[general]
# How often to ping (milliseconds)
test_interval_ms = 1000  # 1 second

# For less frequent testing
test_interval_ms = 5000  # 5 seconds
```

#### Alert Thresholds

```toml
[alerts]
enabled = true
latency_threshold_ms = 100.0   # Alert if RTT > 100ms
jitter_threshold_ms = 50.0     # Alert if jitter > 50ms
packet_loss_threshold_pct = 5.0  # Alert if loss > 5%
```

#### Database Location

```toml
[general]
database_path = "./bufferbane.db"

# Or use absolute path
database_path = "/var/lib/bufferbane/measurements.db"
```

---

## Running Modes

### 1. Monitoring Mode (Default)

Continuously ping targets and store results:

```bash
# Normal mode - show every ping
./target/release/bufferbane --config client.conf

# Quiet mode - hourly statistics (recommended for systemd service)
./target/release/bufferbane --config client.conf --quiet
# Or short form:
./target/release/bufferbane -c client.conf -q
```

**Normal Mode Output** (every ping):
```
[14:23:59] 8.8.8.8 -> 18.28ms
[14:23:59] 1.1.1.1 -> 13.47ms
[14:24:00] 8.8.8.8 -> 23.21ms
[14:24:00] 1.1.1.1 -> 15.21ms
```

**Quiet Mode Output** (hourly statistics):
```
2025-10-18T15:00:00Z ═══ Hourly Statistics ═══
2025-10-18T15:00:00Z Total measurements: 7200 (failed: 12)
2025-10-18T15:00:00Z   dns.google: 3600 tests, 0.1% loss
2025-10-18T15:00:00Z     RTT: min=8.45ms avg=10.23ms max=45.67ms p95=15.89ms
2025-10-18T15:00:00Z     Jitter: avg=1.34ms
2025-10-18T15:00:00Z   1.1.1.1: 3600 tests, 0.2% loss
2025-10-18T15:00:00Z     RTT: min=5.23ms avg=7.89ms max=30.12ms p95=12.34ms
2025-10-18T15:00:00Z     Jitter: avg=0.98ms
2025-10-18T15:00:00Z ═══════════════════════
```

**Stop**: Press `Ctrl+C`

**Use cases**: 
- **Normal mode**: Interactive monitoring, immediate feedback
- **Quiet mode**: Systemd service, reduced log noise, long-term monitoring

---

### 2. Export Mode

Export measurements to CSV:

```bash
# Last 24 hours (default)
./target/release/bufferbane --export --output data.csv

# Last 7 days
./target/release/bufferbane --export --last 7d --output week.csv

# Last 6 hours
./target/release/bufferbane --export --last 6h --output tonight.csv

# Specific date/time range
./target/release/bufferbane --export \
  --start "2025-10-18 00:00" \
  --end "2025-10-18 23:59" \
  --output oct18.csv
```

**CSV Format**:
```csv
timestamp,interface,connection_type,test_type,target,rtt_ms,jitter_ms,packet_loss_pct,status,error
1760789929,default,unknown,icmp,8.8.8.8,24.34,,,success,
1760789929,default,unknown,icmp,1.1.1.1,16.50,,,success,
```

**Use case**: Analyze data in spreadsheets or scripts

---

### 3. Chart Mode

Generate **static PNG** or **interactive HTML** charts:

#### Static PNG Charts (Default)

```bash
# Last 24 hours (default)
./target/release/bufferbane --chart --output latency.png

# Last week
./target/release/bufferbane --chart --last 7d --output week.png

# Specific period
./target/release/bufferbane --chart \
  --start "2025-10-18 18:00" \
  --end "2025-10-18 22:00" \
  --output evening_problem.png

# Custom aggregation (more detail with 200 segments instead of default 100)
./target/release/bufferbane --chart --last 24h --segments 200 --output detailed.png
```

**PNG Chart Features**:
- Min/max lines (thin, showing bounds)
- Average line (bold, main metric)
- P95/P99 percentile lines (dashed)
- Shaded area between min/max (variance)
- Color-coded targets with legend
- **Large, readable fonts** for axis labels and legend
- **Gap detection**: Lines break when data gap > 5 minutes (shows monitoring downtime)
- 1920x1080 resolution (configurable)

#### Interactive HTML Charts (NEW! ✨)

```bash
# Generate interactive HTML chart
./target/release/bufferbane --chart --interactive

# With specific time range
./target/release/bufferbane --chart --interactive --last 24h --output report.html

# Custom time range
./target/release/bufferbane --chart --interactive \
  --start "2025-10-18 18:00" \
  --end "2025-10-18 22:00" \
  --output investigation.html

# High detail for long time ranges (e.g., 7 days with 300 segments)
./target/release/bufferbane --chart --interactive --last 7d --segments 300 --output week_detailed.html

# Low detail for quick overview (50 segments)
./target/release/bufferbane --chart --interactive --last 24h --segments 50 --output quick_overview.html
```

**Interactive HTML Features**:
- 📊 **Hover tooltips**: See exact timestamp and latency for each data point
- 📈 **Canvas-based rendering**: Smooth, responsive charts
- 📉 **Statistics panel**: Min/Max/Avg/P95/P99 for each target
- 🎨 **Modern UI**: Professional design with grid, legend, and styled cards
- 💾 **Standalone file**: No external dependencies, works offline
- 📏 **Smaller file size**: ~14KB HTML vs ~300KB PNG
- 🔍 **Gap detection**: Lines break when data gap > 5 minutes (shows monitoring downtime)

**Technology**: Pure HTML5 Canvas + JavaScript (no external libraries)

**Use cases**:
- Visual proof of connection issues
- Detailed forensic analysis with exact timestamps
- Sharing interactive reports with ISP support
- Lightweight storage compared to PNG files

---

## Common Workflows

### Workflow 1: Diagnose Intermittent Issues

**Goal**: Catch sporadic latency spikes or packet loss

```bash
# Step 1: Start monitoring
./target/release/bufferbane &
BFBN_PID=$!

# Step 2: Let it run for 24-48 hours
# (go about your normal activities)

# Step 3: Generate report when issue occurs
./target/release/bufferbane --chart --last 24h --output problem.png
./target/release/bufferbane --export --last 24h --output problem.csv

# Step 4: Stop monitoring
kill $BFBN_PID
```

**Result**: Chart showing when/how issues occurred

---

### Workflow 2: Prove ISP Issues

**Goal**: Collect evidence for ISP complaint

```bash
# Step 1: Monitor for a week
./target/release/bufferbane &

# Step 2: After a week, generate comprehensive report
./target/release/bufferbane --export --last 7d --output isp_report.csv
./target/release/bufferbane --chart --last 7d --output isp_proof.png

# Step 3: Calculate statistics
# In spreadsheet or script:
# - Average latency: Should be < 50ms
# - Packet loss: Should be < 1%
# - Jitter: Should be < 10ms
# - 95th percentile latency: Should be < 100ms
```

**Result**: Hard data showing consistent problems

---

### Workflow 3: Identify Peak Hour Congestion

**Goal**: Determine if issues happen at specific times

```bash
# Step 1: Monitor for several days
./target/release/bufferbane &

# Step 2: Compare different time periods
./target/release/bufferbane --chart \
  --start "2025-10-18 02:00" \
  --end "2025-10-18 04:00" \
  --output night.png

./target/release/bufferbane --chart \
  --start "2025-10-18 19:00" \
  --end "2025-10-18 22:00" \
  --output evening.png
```

**Result**: Visual comparison showing evening congestion

---

### Workflow 4: Monitor After ISP Intervention

**Goal**: Verify ISP fixes the issue

```bash
# Before fix: Collect baseline
./target/release/bufferbane &
sleep 86400  # 24 hours
./target/release/bufferbane --chart --last 24h --output before_fix.png

# ISP performs fix

# After fix: Collect new data
./target/release/bufferbane &
sleep 86400  # 24 hours
./target/release/bufferbane --chart --last 24h --output after_fix.png

# Compare the two charts
```

**Result**: Proof that fix worked (or didn't)

---

## Systemd Service

Run Bufferbane as a background service:

### Create Service File

```bash
sudo vim /etc/systemd/system/bufferbane.service
```

```ini
[Unit]
Description=Bufferbane Network Monitoring
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=YOUR_USERNAME
WorkingDirectory=/home/YOUR_USERNAME/bufferbane
ExecStart=/home/YOUR_USERNAME/bufferbane/target/release/bufferbane --config /home/YOUR_USERNAME/bufferbane/client.conf
Restart=always
RestartSec=10

# Capabilities for ICMP
AmbientCapabilities=CAP_NET_RAW
CapabilityBoundingSet=CAP_NET_RAW

[Install]
WantedBy=multi-user.target
```

### Enable and Start

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable (start on boot)
sudo systemctl enable bufferbane

# Start now
sudo systemctl start bufferbane

# Check status
sudo systemctl status bufferbane

# View logs
sudo journalctl -u bufferbane -f
```

### Manage Service

```bash
# Stop
sudo systemctl stop bufferbane

# Restart
sudo systemctl restart bufferbane

# Disable (don't start on boot)
sudo systemctl disable bufferbane
```

---

## Tips & Tricks

### Tip 1: Monitor Multiple Locations

Test to different geographic endpoints to isolate issues:

```toml
[targets]
public_dns = [
    "8.8.8.8",      # Google US
    "1.1.1.1",      # Cloudflare global
]
custom = [
    "185.43.135.1", # Digitalcourage Germany
]
```

### Tip 2: Adjust Test Frequency for Different Scenarios

**High-frequency (1 second)**: Catch brief spikes
```toml
test_interval_ms = 1000
```

**Medium-frequency (5 seconds)**: Long-term monitoring
```toml
test_interval_ms = 5000
```

**Low-frequency (60 seconds)**: Minimize bandwidth
```toml
test_interval_ms = 60000
```

### Tip 3: Clean Up Old Data

SQLite database grows over time. Clean up:

```bash
# Check database size
ls -lh bufferbane.db

# Export old data before cleaning
./target/release/bufferbane --export \
  --start "2025-01-01 00:00" \
  --end "2025-10-01 00:00" \
  --output archive_q1-q3.csv

# Delete old measurements (manual SQL)
sqlite3 bufferbane.db "DELETE FROM measurements WHERE timestamp < strftime('%s', '2025-10-01')"
sqlite3 bufferbane.db "VACUUM"
```

### Tip 4: Combine with Other Tools

Export and analyze:

```bash
# Export to CSV
./target/release/bufferbane --export --last 7d --output week.csv

# Analyze with Python/pandas
python3 << EOF
import pandas as pd
df = pd.read_csv('week.csv')
print(f"Average latency: {df['rtt_ms'].mean():.2f}ms")
print(f"Max latency: {df['rtt_ms'].max():.2f}ms")
print(f"99th percentile: {df['rtt_ms'].quantile(0.99):.2f}ms")
EOF
```

### Tip 5: Monitor Your Router Too

Add your router/gateway to targets:

```toml
[targets]
custom = ["192.168.1.1"]  # Your router IP
```

This helps distinguish between:
- Router issues (router ping high)
- ISP issues (router ping OK, internet ping high)

---

## Time Range Formats

### Relative Time

```bash
# Hours
--last 1h
--last 6h
--last 24h

# Days
--last 1d
--last 7d
--last 30d

# Minutes
--last 30m
--last 90m
```

### Absolute Time

```bash
# Date + time format: YYYY-MM-DD HH:MM
--start "2025-10-18 00:00" --end "2025-10-18 23:59"
--start "2025-10-15 18:00" --end "2025-10-15 22:00"
```

---

## Troubleshooting

### No Data in Charts

**Problem**: Chart generation fails with "No measurements found"

**Solutions**:
1. Check if monitoring has been running: `ls -l bufferbane.db`
2. Verify time range has data: `--last 1h` instead of `--last 7d`
3. Check database: `sqlite3 bufferbane.db "SELECT COUNT(*) FROM measurements"`

### High CPU Usage

**Problem**: Bufferbane uses too much CPU

**Solutions**:
1. Increase test interval: `test_interval_ms = 5000`
2. Reduce number of targets: Remove unnecessary custom targets
3. Check for runaway processes: `ps aux | grep bufferbane`

### Database Locked

**Problem**: `database is locked` error

**Solutions**:
1. Only run one instance: `killall bufferbane`
2. Wait a moment and try again
3. Check for stale processes: `ps aux | grep bufferbane`

---

## Next Steps

- Read [Planning Documentation](../docs/planning/) for future phases
- Check [GitHub Issues](https://github.com/schuellerf/bufferbane/issues) for known issues
- Contribute improvements or bug reports

---

**Last Updated**: October 2025  
**Version**: Phase 1 (v0.1.0)

