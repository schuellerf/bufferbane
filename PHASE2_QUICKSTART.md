# Phase 2 Quick Start - TL;DR

## What You Need

1. **Local machine**: Linux with Rust
2. **Remote server**: Any Linux VPS with public IP (€3-5/month)
3. **5 minutes**: Automated setup script does everything

## One-Command Setup

```bash
./setup-server.sh
```

**The script asks for**:
- Server hostname (e.g., `monitor.example.com`)
- SSH user (default: `root`)

**The script does**:
1. ✅ Generates cryptographic shared secret
2. ✅ Creates `server.conf` and `client.conf` (matching secrets)
3. ✅ Builds `bufferbane-server` binary
4. ✅ Deploys to your server via SCP

## Start Monitoring

### On Server:
```bash
ssh user@monitor.example.com
cd /opt/bufferbane
./bufferbane-server --config server.conf
```

### On Client:
```bash
./target/release/bufferbane --config client.conf
```

## What Phase 2 Adds

| Feature | Phase 1 (ICMP) | Phase 2 (Server) |
|---------|----------------|------------------|
| **Latency** | ✅ Good | ✅ Better (nanosecond precision) |
| **Security** | N/A | ✅ Encrypted (ChaCha20-Poly1305) |
| **Authentication** | N/A | ✅ Port knocking + shared secret |
| **Future** | N/A | Throughput + bufferbloat tests |

## Files Generated

After running `setup-server.sh`:

```
./server.conf          # Server configuration (deployed to server)
./client.conf          # Client configuration (use locally)
./target/release/
  ├── bufferbane       # Client binary (Phase 1 features)
  └── bufferbane-server # Server binary (deployed to server)
```

## Verify It Works

### Server logs should show:
```
INFO bufferbane_server: Server listening on 0.0.0.0:9876
INFO bufferbane_server: Received valid KNOCK from client_id=...
INFO bufferbane_server: Created session ... for client ...
```

### Client output should show:
```
INFO bufferbane: ICMP test to 1.1.1.1: RTT=15.2ms
INFO bufferbane: Server echo test to monitor.example.com: RTT=8.3ms
```

## Troubleshooting

**Client can't connect?**
```bash
# Check firewall on server
sudo ufw allow 9876/udp

# Verify server is running
ssh user@server "ps aux | grep bufferbane-server"

# Check port is listening
ssh user@server "ss -ulnp | grep 9876"
```

**Shared secret mismatch?**
```bash
# Compare secrets
grep shared_secret client.conf
ssh user@server "grep shared_secret /opt/bufferbane/server.conf"
# They must be IDENTICAL
```

## Next Steps

Once both are running:
1. Monitor for a few hours
2. Generate charts: `./target/release/bufferbane --chart --last 24h`
3. Compare ICMP vs server latency
4. Read [PHASE2_SETUP.md](PHASE2_SETUP.md) for advanced setup

## Manual Config Tweaking

### Server (`server.conf`):
```toml
[general]
bind_port = 9876              # Change if needed

[security]
session_timeout_sec = 3600    # How long clients stay authenticated

[logging]
level = "info"                # Or "debug" for troubleshooting
```

### Client (`client.conf`):
```toml
[targets]
hosts = ["monitor.example.com", "1.1.1.1", "8.8.8.8"]

[server]
enabled = true                # Set to false to disable Phase 2
host = "monitor.example.com"
enable_echo_test = true       # Enhanced latency tests
```

## Cost Estimate

**Server hosting** (examples):
- **Hetzner Cloud**: €3.79/month (CX11 - 1 vCPU, 2GB RAM)
- **Netcup**: €2.99/month (VPS 200 - 1 vCPU, 2GB RAM)  
- **DigitalOcean**: $4/month (Basic Droplet)

**Bandwidth**: ~100GB/month (more than enough for 1-second tests)

## Production Tips

1. **Systemd service** (server):
   ```bash
   # See PHASE2_SETUP.md for full systemd unit file
   sudo systemctl enable bufferbane-server
   sudo systemctl start bufferbane-server
   ```

2. **Secure configs**:
   ```bash
   chmod 600 client.conf
   ssh user@server "chmod 600 /opt/bufferbane/server.conf"
   ```

3. **Monitor server**:
   ```bash
   ssh user@server "journalctl -u bufferbane-server -f"
   ```

## Links

- Full setup guide: [PHASE2_SETUP.md](PHASE2_SETUP.md)
- Main README: [README.md](README.md)
- Technical spec: [docs/planning/SPECIFICATION.md](docs/planning/SPECIFICATION.md)

---

**tl;dr**: Run `./setup-server.sh`, follow prompts, start server and client. Done. 🎉

