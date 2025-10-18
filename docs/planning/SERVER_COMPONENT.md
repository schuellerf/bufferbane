# Bufferbane Server Component - Quick Reference

**Bufferbane**: The bane of bufferbloat

This document summarizes the server component of Bufferbane and its benefits.

## Why Add a Server?

The server component transforms the monitor from basic latency checking into a comprehensive network diagnostics tool:

| Feature | Without Server | With Server |
|---------|---------------|-------------|
| **Latency** | ✅ ICMP only | ✅ Enhanced with timestamps |
| **Packet Loss** | ⚠️ Basic detection | ✅ Bidirectional with sequence tracking |
| **Upload Speed** | ❌ Not available | ✅ Accurate measurement |
| **Download Speed** | ❌ Not available | ✅ Accurate measurement |
| **Bufferbloat** | ❌ Not possible | ✅ Full RRUL-style detection |
| **Connection Stability** | ⚠️ Limited | ✅ Comprehensive |

## Security Model: Port Knocking + Encryption + Authentication

### Why This Approach?

Instead of complex authentication (OAuth, JWT, user databases), we use:

1. **Port Knocking**: Client sends encrypted packet to "unlock" the server
2. **Shared Secret**: 32-byte secret configured on both sides
3. **ChaCha20-Poly1305 AEAD**: Every packet encrypted and authenticated
4. **Silent Drops**: Invalid packets ignored (server appears closed)
5. **Nanosecond Nonces**: Prevents replay attacks and adds entropy

### Advantages

- ✅ Simple to implement (no TLS, certificates, or databases)
- ✅ Hidden from port scanners
- ✅ Prevents abuse and unauthorized usage
- ✅ **Prevents eavesdropping** (all payloads encrypted)
- ✅ **Prevents pattern analysis** (encrypted + random padding)
- ✅ Low overhead (ChaCha20 is very fast, ~3 cycles/byte)
- ✅ Replay protection via nonce tracking
- ✅ Supports multiple clients with different secrets
- ✅ AEAD provides both encryption and authentication (no separate MAC)

### Setup

```bash
# 1. Copy configuration templates
cp client.conf.template client.conf
cp server.conf.template server.conf

# 2. Generate shared secret
openssl rand -hex 32
# Output: a7b3c9d8e1f4a2b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9

# 3. Edit client.conf
#    In [server] section:
#    - enabled = true
#    - host = "monitor.example.com"
#    - port = 9876
#    - shared_secret = "a7b3c9d8e1f4a2b5..."

# 4. Edit server.conf
#    In [general] section:
#    - bind_address = "0.0.0.0"
#    - bind_port = 9876
#    - shared_secret = "a7b3c9d8e1f4a2b5..."  (same as client)
```

See `client.conf.template` and `server.conf.template` for complete configuration options with documentation.

## Protocol Overview

All communication uses UDP with custom binary protocol:

### Packet Structure
```
[24 bytes]  Cleartext Header (magic: BFBN, version, type, payload length, client ID, nonce timestamp)
[variable]  Encrypted Payload (ChaCha20-Poly1305 AEAD)
            - Ciphertext (packet-specific data + optional random padding)
            - Auth Tag (16 bytes, provides authentication and integrity)

Nonce: 12 bytes = client_id[0:4] || nonce_timestamp[0:8]
Key: 32-byte shared secret
Associated Data: Cleartext header (authenticated but not encrypted)
```

### Key Packet Types

| Type | Name | Direction | Purpose |
|------|------|-----------|---------|
| 0x01 | KNOCK | Client→Server | Authenticate |
| 0x02 | KNOCK_ACK | Server→Client | Confirm auth |
| 0x10 | ECHO_REQUEST | Client→Server | Latency test |
| 0x11 | ECHO_REPLY | Server→Client | Latency response |
| 0x20-0x23 | THROUGHPUT_* | Both | Upload tests |
| 0x30-0x32 | DOWNLOAD_* | Both | Download tests |
| 0x40-0x41 | BUFFERBLOAT_* | Client→Server | Bufferbloat tests |

### Communication Flow

```
1. Client sends KNOCK with HMAC
2. Server validates, sends KNOCK_ACK
3. Client unlocked for 5 minutes
4. Client can now send test packets
5. All packets include HMAC for validation
```

## Server Deployment

### Recommended VPS

- **Provider**: Hetzner Cloud, Netcup (EU providers)
- **Location**: Vienna or Frankfurt (low latency to Austria)
- **Specs**: 1 vCPU, 512 MB RAM, 10 GB disk
- **Cost**: €3-5/month
- **Bandwidth**: ~100 GB/month estimated

### Deployment Options

**Option 1: Binary**
```bash
# Build on dev machine
cargo build --release --bin bufferbane-server

# Copy to server
scp target/release/bufferbane-server user@server:~/

# Run
./bufferbane-server --config server.conf
```

**Option 2: Docker** (planned)
```bash
docker run -d \
  -p 9876:9876/udp \
  -v ./server.conf:/server.conf \
  bufferbane-server
```

**Option 3: Systemd Service**
```ini
[Unit]
Description=Bufferbane Server
After=network.target

[Service]
Type=simple
User=bufferbane
ExecStart=/usr/local/bin/bufferbane-server --config /etc/bufferbane/server.conf
Restart=always

[Install]
WantedBy=multi-user.target
```

## Testing Capabilities

### Latency/Jitter (Basic → Enhanced)

**Standalone (ICMP only)**:
- Basic RTT measurement
- Limited by ICMP rate limits
- No server-side timestamps

**With Server**:
- Nanosecond-precision timestamps on both ends
- One-way delay calculation
- Sequence number tracking
- No rate limits

### Packet Loss (Basic → Comprehensive)

**Standalone**:
- ICMP timeouts only
- Can't distinguish upload vs download loss

**With Server**:
- UDP stream with sequence numbers
- Server echoes received sequences
- Separate upload/download loss tracking
- Burst pattern detection (consecutive losses)

### Throughput (Not Available → Full Testing)

**Standalone**: ❌ Not possible

**With Server**:
- Upload: Client sends data, server measures rate
- Download: Server sends data, client measures rate
- Configurable test sizes (100KB, 1MB, 10MB)
- Per-second reporting
- Packet loss during throughput test

### Bufferbloat (Not Available → RRUL-Style)

**Standalone**: ❌ Not possible

**With Server** (This is the killer feature!):
```
1. Measure baseline latency (idle)
2. Start upload test
3. Continue latency tests during upload
4. Measure latency increase
5. Detect bufferbloat if latency >200ms increase
```

Example result:
```
Baseline latency: 15ms
Latency under load: 185ms
Bufferbloat detected: +170ms
```

## Architecture Changes

### Before (Standalone Only)

```
bufferbane/
└── src/
    ├── main.rs
    ├── monitor/
    │   └── icmp.rs      # Only ICMP
    └── storage/
```

### After (Client + Server)

```
bufferbane/
├── client/              # Client application
│   ├── src/
│   │   ├── monitor/
│   │   │   ├── icmp.rs           # ICMP tests
│   │   │   └── server_tests.rs   # Throughput, bufferbloat
│   │   └── protocol/             # Client-side protocol
│
├── server/              # Server application
│   ├── src/
│   │   ├── handlers/             # Echo, throughput
│   │   ├── protocol/             # Server-side protocol
│   │   └── session/              # Auth, rate limiting
│
└── protocol/            # Shared library
    └── src/
        └── lib.rs       # Packet types, constants
```

## Configuration: Making It Optional

### Client Config

See `client.conf.template` for full configuration options.

```toml
# Server section is optional
[server]
enabled = false  # Default: works without server

# To enable server features:
enabled = true
host = "monitor.example.com"
port = 9876
shared_secret = "..."

# Tests automatically enable based on server availability
[tests.throughput]
enabled = true  # Only works if server.enabled = true

[tests.bufferbloat]
enabled = true  # Only works if server.enabled = true
```

### Client Behavior

```rust
if config.server.enabled {
    // Full feature set
    run_icmp_tests();
    run_server_echo_tests();
    run_throughput_tests();
    run_bufferbloat_tests();
} else {
    // Standalone mode
    run_icmp_tests();
    log("Server disabled - throughput and bufferbloat tests unavailable");
}
```

## Implementation Phases

### Phase 1: Protocol Library (1-2 days)
- Shared packet structures
- HMAC functions
- Serialization/deserialization

### Phase 2: Client Standalone (2-3 days)
- Works without server
- ICMP monitoring
- SQLite storage
- **Milestone**: Usable for basic monitoring

### Phase 3: Server Core (2-3 days)
- Port knocking
- Echo service
- Session management
- **Milestone**: Enhanced latency testing

### Phase 4: Throughput (2-3 days)
- Upload/download tests
- **Milestone**: Full speed testing

### Phase 5: Bufferbloat (1-2 days)
- RRUL-style testing
- **Milestone**: Complete diagnostics

## Security Considerations

### Threat Model

**Protected Against**:
- ✅ Port scanning (appears closed)
- ✅ Unauthorized usage (encryption required)
- ✅ Eavesdropping (all payloads encrypted)
- ✅ Pattern analysis (encrypted + random padding)
- ✅ Traffic fingerprinting (variable packet sizes)
- ✅ Replay attacks (nonce tracking)
- ✅ Packet tampering (auth tag validation)
- ✅ Abuse/DoS (rate limiting per IP)
- ✅ Secret guessing (256-bit secret = 2^256 possibilities)

**Not Protected Against** (by design):
- DDoS with valid secret (but rate limited)
- Server compromise (no data to steal, stateless)
- Network-level attacks (use firewall rules)
- Traffic metadata analysis (packet timing, frequency)

**Why Not Use TLS/DTLS?**
- Simpler: No certificate management
- Faster: No TLS handshake overhead
- Stealthier: Custom protocol harder to fingerprint
- Sufficient: AEAD provides same crypto as TLS 1.3

### Hardening Recommendations

1. **Firewall**: Only open UDP port 9876
2. **Monitoring**: Log authentication failures
3. **Rate Limiting**: Default 10 req/s per IP
4. **Secret Rotation**: Change shared secret periodically
5. **Multiple Clients**: Use different secrets per client

## Cost Estimate

### Server Costs (Monthly)

- VPS: €3-5
- Bandwidth: Included in VPS
- Domain (optional): €1-2
- **Total**: €3-7/month

### Bandwidth Usage

**Per client per day**:
- Echo tests: ~50 KB
- Small throughput: ~10 MB
- Medium throughput: ~100 MB
- Bufferbloat: ~50 MB

**Total per client**: ~160 MB/day = ~5 GB/month

**10 clients**: ~50 GB/month (well within VPS limits)

## Summary

The server component:
- ✅ **Optional**: Client works standalone
- ✅ **Essential**: Enables critical features (throughput, bufferbloat)
- ✅ **Simple**: Port knocking + HMAC, no complex auth
- ✅ **Secure**: Hidden, authenticated, rate-limited
- ✅ **Affordable**: €3-5/month VPS
- ✅ **Lightweight**: 512 MB RAM, minimal CPU

**Recommendation**: Plan for server from start, implement incrementally.

## Further Reading

- **SPECIFICATION.md**: Complete protocol specification and packet formats
- **RESEARCH.md**: Technology choices and implementation strategy
- **SCENARIOS.md**: How server enables detection of all 7 scenarios

---

**Document Version**: 1.0  
**Last Updated**: 2025-10-18

