# Installation Guide - Bufferbane

This guide covers building, installing, and setting up Bufferbane as a systemd service.

---

## Quick Installation (Recommended)

### 1. Build and Install Binary

```bash
# Clone repository
git clone https://github.com/schuellerf/bufferbane.git
cd bufferbane

# Build and install (installs to /usr/local/bin)
sudo make install
```

This installs:
- Binary: `/usr/local/bin/bufferbane`
- Config template: `/usr/local/share/bufferbane/client.conf.template`

### 2. Create Configuration

```bash
# Create config directory
sudo mkdir -p /etc/bufferbane

# Copy template
sudo cp /usr/local/share/bufferbane/client.conf.template /etc/bufferbane/client.conf

# Edit configuration
sudo nano /etc/bufferbane/client.conf
```

**Important settings to configure:**
- `targets` - Add your ping targets (default: dns.google, 1.1.1.1, 8.8.8.8)
- `database_path` - Default is fine for service: `~/.local/share/bufferbane/bufferbane.db`
- `test_interval_ms` - Default 1000 (1 second) is recommended

### 3. Install and Start Service

```bash
# Install systemd service
sudo make install-service

# Enable service to start on boot
sudo systemctl enable bufferbane

# Start service now
sudo systemctl start bufferbane

# Check status
sudo systemctl status bufferbane
```

### 4. View Logs

```bash
# Follow logs in real-time
sudo journalctl -u bufferbane -f

# View last 100 lines
sudo journalctl -u bufferbane -n 100

# View logs from today
sudo journalctl -u bufferbane --since today
```

In **quiet mode** (default for service), you'll see hourly statistics like:

```
Jan 18 14:00:00 bufferbane[1234]: ═══ Hourly Statistics ═══
Jan 18 14:00:00 bufferbane[1234]: Total measurements: 3600 (failed: 5)
Jan 18 14:00:00 bufferbane[1234]:   dns.google: 3600 tests, 0.1% loss
Jan 18 14:00:00 bufferbane[1234]:     RTT: min=8.23ms avg=10.45ms max=25.67ms p95=15.34ms
Jan 18 14:00:00 bufferbane[1234]:     Jitter: avg=1.23ms
Jan 18 14:00:00 bufferbane[1234]:   1.1.1.1: 3600 tests, 0.2% loss
Jan 18 14:00:00 bufferbane[1234]:     RTT: min=5.12ms avg=7.89ms max=20.34ms p95=12.45ms
Jan 18 14:00:00 bufferbane[1234]:     Jitter: avg=0.89ms
Jan 18 14:00:00 bufferbane[1234]: ═══════════════════════
```

---

## Manual Installation (Alternative)

If you prefer not to use the Makefile:

### 1. Build

```bash
cargo build --release
```

Binary will be at: `target/release/bufferbane`

### 2. Manual Service Setup

```bash
# Copy binary
sudo cp target/release/bufferbane /usr/local/bin/

# Set CAP_NET_RAW capability (allows ICMP without root)
sudo setcap cap_net_raw+ep /usr/local/bin/bufferbane

# Create config directory
sudo mkdir -p /etc/bufferbane

# Copy and edit config
sudo cp client.conf.template /etc/bufferbane/client.conf
sudo nano /etc/bufferbane/client.conf

# Create data directory for user
mkdir -p ~/.local/share/bufferbane

# Create systemd service (replace USERNAME with your username)
sudo tee /etc/systemd/system/bufferbane.service > /dev/null << 'EOF'
[Unit]
Description=Bufferbane Network Quality Monitoring
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=USERNAME
WorkingDirectory=/home/USERNAME/.local/share/bufferbane
ExecStartPre=/usr/bin/mkdir -p /home/USERNAME/.local/share/bufferbane
ExecStart=/usr/local/bin/bufferbane --config /etc/bufferbane/client.conf --quiet
Restart=always
RestartSec=10
AmbientCapabilities=CAP_NET_RAW
CapabilityBoundingSet=CAP_NET_RAW
NoNewPrivileges=true
StandardOutput=journal
StandardError=journal
SyslogIdentifier=bufferbane

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd
sudo systemctl daemon-reload

# Enable and start
sudo systemctl enable --now bufferbane
```

---

## Custom Installation Prefix

Install to a custom location (e.g., `/opt/bufferbane`):

```bash
# Build and install with custom prefix
sudo make install PREFIX=/opt/bufferbane

# Binary will be at: /opt/bufferbane/bin/bufferbane
# Config template: /opt/bufferbane/share/bufferbane/client.conf.template
```

---

## Running Without Systemd (Manual Mode)

For testing or interactive use:

```bash
# Normal mode (shows every ping)
./target/release/bufferbane --config client.conf

# Quiet mode (hourly statistics)
./target/release/bufferbane --config client.conf --quiet

# Short flag
./target/release/bufferbane -c client.conf -q
```

---

## Uninstallation

### Remove Service Only

```bash
sudo make uninstall-service
```

### Remove Everything

```bash
sudo make uninstall
```

This removes:
- Binary: `/usr/local/bin/bufferbane`
- Data directory: `/usr/local/share/bufferbane`
- Systemd service: `/etc/systemd/system/bufferbane.service`

**Note**: Configuration in `/etc/bufferbane/` is preserved. To remove it:

```bash
sudo rm -rf /etc/bufferbane/
```

Database files in `~/.local/share/bufferbane/` are also preserved. To remove:

```bash
rm -rf ~/.local/share/bufferbane/
```

---

## Troubleshooting

### Service Won't Start

```bash
# Check status and errors
sudo systemctl status bufferbane

# View full logs
sudo journalctl -u bufferbane -e

# Common issues:
# 1. Config file missing or invalid
sudo ls -l /etc/bufferbane/client.conf

# 2. CAP_NET_RAW not set
sudo getcap /usr/local/bin/bufferbane
# Should show: /usr/local/bin/bufferbane cap_net_raw=ep

# 3. Database directory not writable
ls -ld ~/.local/share/bufferbane
```

### Permission Denied for ICMP

```bash
# Set capability on binary
sudo setcap cap_net_raw+ep /usr/local/bin/bufferbane

# Verify
sudo getcap /usr/local/bin/bufferbane

# Alternative: Run as root (not recommended)
# Edit service file and change User=root
```

### Service Stops Unexpectedly

```bash
# Check for errors
sudo journalctl -u bufferbane -n 50

# Check disk space
df -h ~/.local/share/bufferbane

# Check memory
free -h

# Restart service
sudo systemctl restart bufferbane
```

### Config File Errors

```bash
# Validate config syntax
./target/release/bufferbane --config /etc/bufferbane/client.conf --help

# Check for common issues:
# - TOML syntax errors
# - Missing required fields
# - Invalid values

# Test with minimal config
sudo cp client.conf.template /tmp/test.conf
./target/release/bufferbane --config /tmp/test.conf
```

---

## Advanced Configuration

### Change Service User

By default, the service runs as the user who installed it. To change:

```bash
# Reinstall with different user
sudo make install-service INSTALL_USER=bufferbane

# Or manually edit service file
sudo nano /etc/systemd/system/bufferbane.service
# Change: User=bufferbane
# Update paths accordingly

sudo systemctl daemon-reload
sudo systemctl restart bufferbane
```

### Multiple Instances

To run multiple Bufferbane instances (e.g., different configs):

```bash
# Copy service file
sudo cp /etc/systemd/system/bufferbane.service \
        /etc/systemd/system/bufferbane-wan.service

# Edit new service
sudo nano /etc/systemd/system/bufferbane-wan.service
# Change config path: --config /etc/bufferbane/wan.conf
# Change database path in config

# Create separate config
sudo cp /etc/bufferbane/client.conf /etc/bufferbane/wan.conf
sudo nano /etc/bufferbane/wan.conf

# Enable and start
sudo systemctl enable --now bufferbane-wan
```

---

## Upgrading

### Upgrade Installed Version

```bash
# Pull latest code
git pull

# Rebuild and reinstall
sudo make install

# Restart service to use new binary
sudo systemctl restart bufferbane
```

### Upgrade from Manual to Makefile Installation

If you installed manually and want to switch to Makefile:

```bash
# Stop and disable old service
sudo systemctl stop bufferbane
sudo systemctl disable bufferbane

# Remove old service file
sudo rm /etc/systemd/system/bufferbane.service

# Install via Makefile
sudo make install
sudo make install-service

# Start new service
sudo systemctl enable --now bufferbane
```

---

## Distribution-Specific Notes

### Fedora / RHEL / CentOS

```bash
# Install dependencies
sudo dnf install rust cargo make gcc sqlite-devel

# Standard installation
sudo make install
sudo make install-service
```

### Ubuntu / Debian

```bash
# Install dependencies
sudo apt install rustc cargo make gcc libsqlite3-dev

# Standard installation
sudo make install
sudo make install-service
```

### Arch Linux

```bash
# Install dependencies
sudo pacman -S rust make gcc sqlite

# Standard installation
sudo make install
sudo make install-service
```

---

## Verification

After installation, verify everything works:

```bash
# 1. Check binary
bufferbane --version
bufferbane --help

# 2. Check config
sudo ls -l /etc/bufferbane/client.conf

# 3. Check service
sudo systemctl status bufferbane

# 4. Check logs
sudo journalctl -u bufferbane -n 20

# 5. Generate test chart
bufferbane --chart --last 1h --output /tmp/test.png

# 6. Check database
ls -lh ~/.local/share/bufferbane/bufferbane.db
sqlite3 ~/.local/share/bufferbane/bufferbane.db "SELECT COUNT(*) FROM measurements;"
```

---

## Getting Help

If you encounter issues:

1. Check logs: `sudo journalctl -u bufferbane -e`
2. Verify config: `/etc/bufferbane/client.conf`
3. Test manually: `bufferbane --config /etc/bufferbane/client.conf`
4. Check permissions: `ls -l /usr/local/bin/bufferbane`
5. Open issue: https://github.com/schuellerf/bufferbane/issues

