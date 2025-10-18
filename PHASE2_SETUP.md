# Phase 2 Setup Guide - Client-Server Deployment

This guide walks you through setting up Bufferbane in Phase 2 mode with a remote server for enhanced network testing.

## Overview

**Phase 2** adds a server component that enables:
- **Enhanced latency testing** with server timestamps (more accurate than ICMP)
- **Bidirectional packet loss tracking**
- **Encrypted communication** (all packets encrypted with ChaCha20-Poly1305)
- **Session management** with authentication
- Future: Upload/download throughput and bufferbloat testing

## Prerequisites

### Local Machine (Client)
- Linux system (for ICMP `CAP_NET_RAW`)
- Rust toolchain
- SSH client
- `openssl` (for generating shared secret)

### Remote Server
- Linux system with public IP or accessible hostname
- SSH access with sudo/root permissions
- Open UDP port (default: 9876)
- No other dependencies needed (Rust binary is statically linked)

---

## Quick Setup (Recommended)

### Automated Setup Script

The easiest way to set up Phase 2 is using the provided setup script:

```bash
# 1. Navigate to project root
cd /path/to/bufferbane

# 2. Run setup assistant
./setup-server.sh
```

**The script will**:
1. ✅ Prompt for server hostname/IP
2. ✅ Generate a cryptographically secure shared secret
3. ✅ Create matching `server.conf` and `client.conf`
4. ✅ Build the server binary
5. ✅ Deploy to your remote server via SCP
6. ✅ Show you commands to start server and client

**Example interaction**:
```
Enter server hostname or IP: monitor.example.com
Enter server port [9876]: 9876
Enter SSH user for deployment [root]: admin

✓ Generated secure shared secret
✓ Created server.conf
✓ Created client.conf
✓ Server binary built
✓ Deployment complete

Next Steps:
1. Start SERVER: ssh admin@monitor.example.com
   cd /opt/bufferbane
   ./bufferbane-server --config server.conf

2. Start CLIENT: ./target/release/bufferbane --config client.conf
```

---

## Manual Setup (Advanced)

If you prefer manual configuration:

### Step 1: Build the Server

```bash
# Build server binary
make build-server

# Or: cargo build --release -p bufferbane-server

# Binary location: target/release/bufferbane-server
```

### Step 2: Generate Shared Secret

```bash
# Generate 32-byte (64 hex character) secret
openssl rand -hex 32

# Example output:
# a7b3c9d8e1f4a2b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9
```

⚠️ **Keep this secret secure!** It's used for authentication and encryption.

### Step 3: Create Server Configuration

Copy the template and edit:

```bash
cp server.conf.template server.conf
nano server.conf
```

**Key settings** to configure:

```toml
[general]
bind_address = "0.0.0.0"      # Listen on all interfaces
bind_port = 9876               # UDP port for client connections

[security]
shared_secret = "YOUR_GENERATED_SECRET_HERE"  # From Step 2
session_timeout_sec = 3600     # How long authenticated sessions last

[logging]
level = "info"                 # debug, info, warn, error
```

### Step 4: Create Client Configuration

Copy the template and edit:

```bash
cp client.conf.template client.conf
nano client.conf
```

**Key settings** to configure:

```toml
[targets]
# Include your server for enhanced testing
hosts = ["monitor.example.com", "1.1.1.1", "8.8.8.8"]

[server]
enabled = true                            # Enable Phase 2
host = "monitor.example.com"              # Your server hostname
port = 9876                                # Must match server bind_port
shared_secret = "YOUR_GENERATED_SECRET_HERE"  # SAME as server
client_id = 12345678901234567              # Random 64-bit ID
enable_echo_test = true                    # Enable server latency tests
```

⚠️ **The `shared_secret` must be identical** in both files!

### Step 5: Deploy Server

```bash
# Create directory on remote server
ssh user@monitor.example.com "sudo mkdir -p /opt/bufferbane"

# Copy binary and config
scp target/release/bufferbane-server user@monitor.example.com:/opt/bufferbane/
scp server.conf user@monitor.example.com:/opt/bufferbane/

# Secure the configuration
ssh user@monitor.example.com "sudo chmod 600 /opt/bufferbane/server.conf"
```

### Step 6: Configure Firewall (if needed)

```bash
# On the server, open UDP port 9876
# UFW (Ubuntu/Debian):
sudo ufw allow 9876/udp

# firewalld (Fedora/RHEL):
sudo firewall-cmd --permanent --add-port=9876/udp
sudo firewall-cmd --reload

# iptables:
sudo iptables -A INPUT -p udp --dport 9876 -j ACCEPT
```

---

## Running

### Start the Server

```bash
# SSH to your server
ssh user@monitor.example.com

# Navigate to bufferbane directory
cd /opt/bufferbane

# Start server (foreground with logs)
./bufferbane-server --config server.conf

# Or run in background (recommended for production)
nohup ./bufferbane-server --config server.conf > bufferbane-server.log 2>&1 &
```

**Expected output**:
```
INFO bufferbane_server: Starting Bufferbane server v0.1.0
INFO bufferbane_server: Loaded configuration from: server.conf
INFO bufferbane_server: Server listening on 0.0.0.0:9876
INFO bufferbane_server: Max concurrent clients: 100
INFO bufferbane_server: Session timeout: 3600 seconds
```

### Start the Client

```bash
# On your local machine
./target/release/bufferbane --config client.conf
```

**Expected output**:
```
INFO bufferbane: Starting network monitoring...
INFO bufferbane: Testing targets: monitor.example.com, 1.1.1.1, 8.8.8.8
INFO bufferbane: Server mode enabled: monitor.example.com:9876
INFO bufferbane: ICMP test to 1.1.1.1: RTT=15.2ms
INFO bufferbane: Server echo test to monitor.example.com: RTT=8.3ms
```

---

## Verification

### Check Server is Running

```bash
# On server, check if process is running
ps aux | grep bufferbane-server

# Check if port is listening
sudo ss -ulnp | grep 9876

# Expected output:
# UNCONN 0 0 0.0.0.0:9876 0.0.0.0:* users:(("bufferbane-serve",pid=1234,fd=3))
```

### Check Client Can Connect

```bash
# On client, watch logs for server authentication
./target/release/bufferbane --config client.conf

# Look for messages like:
# "Authenticated with server monitor.example.com"
# "Server echo test completed: RTT=..."
```

### Monitor Server Logs

```bash
# On server
tail -f bufferbane-server.log

# Look for:
# "Received valid KNOCK from client_id=..."
# "Created session ... for client ..."
# "Received ECHO_REQUEST seq=..."
```

---

## Troubleshooting

### Issue: Server Not Listening

**Check**:
```bash
# Verify binary has correct permissions
ls -l /opt/bufferbane/bufferbane-server

# Try running with debug logging
RUST_LOG=debug ./bufferbane-server --config server.conf
```

**Common causes**:
- Port already in use: `sudo ss -ulnp | grep 9876`
- Permission denied: Need sudo or different port
- Config file not found: Check path in `--config`

### Issue: Client Can't Connect

**Check**:
1. **Network connectivity**:
   ```bash
   ping monitor.example.com
   nc -u -v monitor.example.com 9876
   ```

2. **Firewall**:
   - Server firewall blocks UDP 9876
   - ISP/router blocks outbound UDP
   
3. **Shared secret mismatch**:
   - Compare `shared_secret` in both configs
   - Must be exactly the same 64 hex characters

4. **Server not running**:
   ```bash
   ssh user@monitor.example.com "ps aux | grep bufferbane-server"
   ```

### Issue: Authentication Fails

**Symptoms**: Client logs show "authentication failed" or no server responses

**Check**:
1. **Shared secret matches**:
   ```bash
   # On client
   grep shared_secret client.conf
   
   # On server
   ssh user@server "grep shared_secret /opt/bufferbane/server.conf"
   ```

2. **Client ID**:
   - Must be a valid 64-bit integer
   - Different for each client instance

3. **Server logs**:
   ```bash
   # On server with debug logging
   RUST_LOG=debug ./bufferbane-server --config server.conf
   
   # Look for "Knock decryption failed" or "Invalid knock payload"
   ```

---

## Security Best Practices

### 1. Protect Configuration Files

```bash
# On client
chmod 600 client.conf

# On server
ssh user@server "sudo chmod 600 /opt/bufferbane/server.conf"
```

### 2. Use Firewall

Only allow UDP 9876 from known client IPs if possible:

```bash
# Allow from specific IP
sudo ufw allow from CLIENT_IP to any port 9876 proto udp
```

### 3. Monitor Failed Authentication

```bash
# On server, watch for failed knocks
grep "KNOCK failed" bufferbane-server.log

# Multiple failures from same IP may indicate attack
```

### 4. Rotate Secrets

Periodically regenerate the shared secret:

```bash
# 1. Generate new secret
openssl rand -hex 32

# 2. Update both server.conf and client.conf
# 3. Restart server
# 4. Restart client
```

---

## Production Deployment

### Systemd Service (Server)

Create `/etc/systemd/system/bufferbane-server.service`:

```ini
[Unit]
Description=Bufferbane Network Monitoring Server
After=network.target

[Service]
Type=simple
User=bufferbane
WorkingDirectory=/opt/bufferbane
ExecStart=/opt/bufferbane/bufferbane-server --config /opt/bufferbane/server.conf
Restart=on-failure
RestartSec=10s

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/bufferbane

[Install]
WantedBy=multi-user.target
```

```bash
# Create user
sudo useradd -r -s /bin/false bufferbane

# Set permissions
sudo chown -R bufferbane:bufferbane /opt/bufferbane

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable bufferbane-server
sudo systemctl start bufferbane-server

# Check status
sudo systemctl status bufferbane-server
sudo journalctl -u bufferbane-server -f
```

### Client as Systemd Service

Use the existing `make install-service` for the client.

---

## Next Steps

Once Phase 2 is running:

1. **Monitor for a few hours** - Verify stable operation
2. **Compare ICMP vs Server tests** - Server tests should show lower latency
3. **Review charts** - Use `--chart` to visualize both test types
4. **Phase 3** (optional) - Add multiple servers for geographic diversity
5. **Phase 4** (future) - Multi-interface testing (WiFi vs Ethernet)

---

## Summary

**Setup with script** (recommended):
```bash
./setup-server.sh
# Follow prompts
# Start server and client as instructed
```

**Manual setup**:
```bash
# 1. Generate secret
openssl rand -hex 32

# 2. Configure server.conf and client.conf with same secret
# 3. Build and deploy
make build-server
scp target/release/bufferbane-server user@server:/opt/bufferbane/
scp server.conf user@server:/opt/bufferbane/

# 4. Start server
ssh user@server "cd /opt/bufferbane && ./bufferbane-server --config server.conf"

# 5. Start client
./target/release/bufferbane --config client.conf
```

**Verify**:
- Server logs show "Created session"
- Client logs show server test results
- Both ICMP and server tests appear in output

---

## Support

**Common Commands**:
```bash
# Build everything
make build

# Build only server
make build-server

# View server help
./target/release/bufferbane-server --help

# Debug mode
RUST_LOG=debug ./bufferbane-server --config server.conf
```

**Files**:
- Setup script: `setup-server.sh`
- Templates: `server.conf.template`, `client.conf.template`
- Documentation: `PHASE2_SETUP.md` (this file)
- Specification: `docs/planning/SPECIFICATION.md`

