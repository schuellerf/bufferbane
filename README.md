# Bufferbane - Planning Documents

**Bufferbane**: The bane of bufferbloat - a network quality monitoring tool for cable internet.

This repository contains comprehensive planning and research documentation for building Bufferbane, a network quality monitoring application specifically designed to detect fine-grained network instabilities on cable internet (such as Magenta Austria, co-ax connection), with a particular focus on bufferbloat detection and upload monitoring.

## Overview

The goal is to create a client-server application that performs high-frequency (1-second interval) network quality tests to detect subtle issues that don't completely break connectivity but cause problems for applications like video conferencing, VoIP, file uploads, and online gaming.

**Project Name**: Bufferbane  
**Magic Bytes**: `BFBN` (0x4246424E)  
**Architecture**: The system consists of two components:
- **Client**: Runs on your local network, performs tests, stores data, generates alerts
- **Server** (optional): Lightweight VPS-based service that enables throughput and bufferbloat testing

The client can operate standalone (ICMP latency monitoring only) or with the server (full feature set including bufferbloat detection).

## Documents

### 📋 [SPECIFICATION.md](SPECIFICATION.md)
Complete technical specification covering:
- **Operating modes**: Standalone (ICMP only) vs. with server (full features)
- **Network metrics to monitor**: Latency, jitter, packet loss, throughput (upload/download), DNS resolution, connection stability, bufferbloat
- **Detection methods**: ICMP, UDP streams, TCP tests, multiple targets, parallel testing
- **Server architecture**: Lightweight echo and throughput service
- **Security model**: Port knocking + ChaCha20-Poly1305 AEAD encryption + authentication
- **Protocol specification**: Complete packet format with encryption and communication flow
- **Data collection & storage**: SQLite database with per-second measurements (client side)
- **Alert conditions**: Thresholds and detection logic for various issues
- **Configuration**: Client and server config file formats with examples

### 🎯 [SCENARIOS.md](SCENARIOS.md)
Detailed descriptions of network instability scenarios specific to cable internet:
1. **Intermittent Upload Degradation**: Periodic upload speed drops
2. **Bufferbloat**: High latency under load
3. **Packet Loss Bursts**: Short periods of consecutive packet loss
4. **Asymmetric Issues**: Upload fails while download works (or vice versa)
5. **ISP Routing Issues**: Problems to specific destinations
6. **Peak Hour Degradation**: Evening slowdowns due to neighborhood congestion
7. **Micro-Disconnections**: Brief complete disconnections

Each scenario includes symptoms, detection methods, data signatures, and root causes.

### 🔍 [RESEARCH.md](RESEARCH.md)
Comprehensive evaluation of existing open-source tools:
- **Tools evaluated**: Smokeping, LibreSpeed, netdata, iperf3, MTR, Gping, Flent, IRTT, NetPerfMeter, and more
- **Gap analysis**: What exists vs. what's needed
- **Technology recommendation**: **Rust** selected as optimal choice
- **Architecture proposal**: Detailed crate recommendations and implementation strategy

### 🔒 [ENCRYPTION_SECURITY.md](ENCRYPTION_SECURITY.md)
Complete analysis of the encryption and security model:
- **Encryption scheme**: ChaCha20-Poly1305 AEAD explained
- **Nonce construction**: Nanosecond timestamps for replay protection
- **Security properties**: What's protected and what's not
- **Pattern analysis resistance**: How encryption + padding hides traffic patterns
- **Threat model**: Detailed protection analysis
- **Performance impact**: Negligible overhead (<0.1% of RTT)
- **Comparison**: vs. TLS, vs. no encryption, vs. VPN

### 🌐 [MULTI_INTERFACE.md](MULTI_INTERFACE.md)
Multi-interface monitoring documentation (Phase 4):
- **Simultaneous testing**: WiFi + Ethernet at the same time
- **Real-time comparison**: Isolate WiFi vs ISP issues definitively
- **Linux implementation**: Interface binding with SO_BINDTODEVICE
- **Use cases**: Diagnose WiFi, A/B testing, failover monitoring
- **Advantages**: Answer in minutes instead of weeks
- **Export & visualization**: Comparison charts and time series

### 🗄️ [RRD_VS_SQLITE.md](RRD_VS_SQLITE.md)
Database technology decision analysis:
- **Comparison**: Round Robin Database vs SQLite
- **Verdict**: SQLite chosen for simplicity and flexibility
- **Rationale**: Less code, more query flexibility, better tooling

### 🔧 [SERVER_COMPONENT.md](SERVER_COMPONENT.md)
Quick reference for server architecture:
- **Benefits**: Throughput testing, bufferbloat detection, bidirectional packet loss
- **Security**: Port knocking + ChaCha20-Poly1305 encryption
- **Protocol**: Binary UDP with 10 packet types
- **Deployment**: VPS setup, systemd service, Docker option

## Key Findings

### Existing Tools Assessment

**No single existing tool meets all requirements.** The closest matches are:
- **IRTT** (8/10): Excellent for precise latency/jitter, but no throughput testing
- **Flent** (7/10): Great for bufferbloat detection, but not for continuous monitoring
- **MTR** (6/10): Good for diagnostics, but incomplete feature set

### Technology Decision: Rust (Client + Server)

**Primary recommendation: Build custom client-server solution in Rust**

**Rationale**:
- ✅ Microsecond-precision timing (no GC pauses)
- ✅ Raw socket access for packet-level control
- ✅ Single binary deployment for both client and server
- ✅ Efficient async runtime (Tokio)
- ✅ Memory safety for 24/7 operation
- ✅ Low CPU/memory footprint
- ✅ Simple security (port knocking + ChaCha20-Poly1305 encryption, no complex auth)

**Server Benefits**:
- ✅ Accurate throughput testing (critical for upload monitoring)
- ✅ Bufferbloat detection (RRUL-style testing)
- ✅ Bidirectional packet loss tracking
- ✅ Connection stability monitoring

**Alternative**: Go (second choice) or Python (rapid prototyping only)

## Implementation Roadmap

Bufferbane development is organized into 4 major phases:

### Phase 1: Client Only (1-2 weeks)
**Goal**: Basic but functional latency monitoring

- Shared protocol library (encryption, packet structures)
- Client CLI with configuration
- ICMP ping to multiple targets (1-second intervals)
- SQLite storage for measurements
- Console output with real-time stats
- DNS resolution monitoring
- Alert detection and logging
- Basic export (CSV, JSON)

**✅ Milestone: Immediately useful for latency diagnosis**

**Capabilities**: Latency, jitter, basic packet loss, DNS monitoring

---

### Phase 2: Client + Server (2-3 weeks)
**Goal**: Full-featured monitoring with throughput and bufferbloat

**Server:**
- Server CLI and configuration
- Port knocking + ChaCha20-Poly1305 encryption
- Echo service (improved latency/packet loss)
- Upload/download throughput testing
- Bufferbloat test coordination
- Session management and rate limiting

**Client additions:**
- Server communication (encrypted protocol)
- Upload/download test implementation
- Bufferbloat test orchestration
- Throughput metrics and alerts
- Bufferbloat detection and alerts

**✅ Milestone: Complete solution for cable internet diagnosis**

**Capabilities**: All Phase 1 + throughput testing + bufferbloat detection

---

### Phase 3: Multiple Servers (1-2 weeks)
**Goal**: Geographic diversity and routing issue detection

**Client additions:**
- Multi-server configuration support
- Parallel testing to multiple servers
- Per-server metrics storage
- Cross-server comparison analysis
- Routing issue detection
- Server failover logic

**✅ Milestone: Detect ISP routing problems**

**Capabilities**: All Phase 2 + multi-server comparison + routing diagnosis

---

### Phase 4: Multiple Interfaces + Multiple Servers (2-3 weeks)
**Goal**: Real-time WiFi vs Ethernet comparison + comprehensive export

**Client additions:**
- Multi-interface configuration support
- Interface binding (Linux SO_BINDTODEVICE)
- **Simultaneous testing across WiFi + Ethernet**
- Auto-detection of interface type
- Per-interface metrics storage
- Real-time interface comparison
- Interface-specific alerts

**Export enhancements:**
- **PNG chart generation** (8 chart types)
- Comprehensive reporting (Markdown, HTML)
- Interface comparison visualizations
- Heatmaps for time-of-day patterns

**✅ Milestone: Complete solution with all advanced features**

**Capabilities**: 
- All Phase 3 features
- **WiFi vs Ethernet simultaneous testing**
- **Visual reporting** (PNG charts)
- **Comprehensive export** (CSV, JSON, charts, reports)
- Definitive WiFi vs ISP diagnosis

---

### Phase Summary

| Phase | Effort | Key Feature | Use Case |
|-------|--------|-------------|----------|
| **1: Client Only** | 1-2 weeks | ICMP latency | Basic diagnosis |
| **2: + Server** | 2-3 weeks | Throughput + bufferbloat | Cable diagnosis |
| **3: + Multi-Server** | 1-2 weeks | Geographic diversity | Routing issues |
| **4: + Multi-Interface** | 2-3 weeks | WiFi vs Ethernet + export | Complete solution |

**Total estimated effort**: 6-10 weeks for all phases

**Recommended order**: Phases 1→2→4 (skip Phase 3 unless routing issues are a concern)

## Use Case

This monitoring solution is specifically designed for:
- **ISP**: Magenta Austria (cable/co-ax)
- **Primary concern**: Unstable upload connection
- **Detection goal**: Identify issues that don't break connectivity but affect applications
- **Frequency**: 1-second test granularity
- **Operation**: Continuous background monitoring with historical data

## Key Metrics

The monitor will track:
1. **Latency (RTT)**: Round-trip time to multiple targets
2. **Jitter**: Variation in latency (stability indicator)
3. **Packet Loss**: Missing packets in percentage and patterns
4. **Upload Throughput**: Actual vs. expected upload speed
5. **Download Throughput**: Actual vs. expected download speed
6. **DNS Resolution**: Time to resolve hostnames
7. **Connection Stability**: TCP connection establishment and drops
8. **Bufferbloat**: Latency increase under load

## Alert Types

Automatic detection and alerting for:
- Latency spikes (>3x baseline)
- High jitter (>30ms sustained)
- Packet loss (>1% or bursts)
- Upload/download degradation (<70% baseline)
- DNS issues (slow or failing resolutions)
- Connection instability (drops/resets)
- Bufferbloat (>200ms latency increase under load)

## Data Storage

- **Primary**: SQLite database (single file, no server)
- **Per-second**: Raw measurements
- **Aggregations**: 1-minute and 1-hour rollups
- **Events**: All detected issues with full details
- **Retention**: 30 days raw, 90 days aggregated, 1 year hourly
- **Export**: CSV and JSON formats available

## Next Steps

1. ✅ **Specification completed** - Technical requirements documented
2. ✅ **Scenarios documented** - Test cases and patterns identified
3. ✅ **Research completed** - Technology stack selected
4. ⏭️ **Implementation** - Ready to begin development

## Project Structure (Proposed)

```
bufferbane/
├── client/                  # Client application
│   ├── src/
│   │   ├── main.rs          # Entry point & CLI
│   │   ├── config.rs        # Configuration management
│   │   ├── monitor/         # Test implementations
│   │   │   ├── icmp.rs      # ICMP ping
│   │   │   ├── server_tests.rs  # Tests requiring server
│   │   │   └── dns.rs       # DNS resolution
│   │   ├── protocol/        # Client-side protocol
│   │   │   ├── packets.rs   # Packet encode/decode
│   │   │   └── security.rs  # HMAC, knock sequence
│   │   ├── storage/         # Database operations
│   │   ├── analysis/        # Statistics & alerts
│   │   └── output/          # Console & export
│   └── Cargo.toml
│
├── server/                  # Server application (optional)
│   ├── src/
│   │   ├── main.rs          # Entry point
│   │   ├── config.rs        # Server configuration
│   │   ├── protocol/        # Server-side protocol
│   │   │   ├── packets.rs   # Packet encode/decode
│   │   │   └── security.rs  # HMAC verification
│   │   ├── handlers/        # Request handlers
│   │   │   ├── echo.rs      # Echo service
│   │   │   ├── throughput.rs  # Upload/download
│   │   │   └── bufferbloat.rs # Bufferbloat tests
│   │   └── session/         # Session & rate limiting
│   ├── Cargo.toml
│   └── Dockerfile           # Docker deployment
│
├── protocol/                # Shared protocol library
│   ├── src/
│   │   ├── lib.rs           # Protocol constants
│   │   ├── types.rs         # Packet types
│   │   └── constants.rs     # Magic bytes, versions
│   └── Cargo.toml
│
├── docs/                    # Documentation
│   ├── SPECIFICATION.md     # Technical specification
│   ├── SCENARIOS.md         # Network scenarios
│   ├── RESEARCH.md          # Technology research
│   ├── ENCRYPTION_SECURITY.md # Encryption analysis
│   ├── MULTI_INTERFACE.md   # Multi-interface monitoring
│   ├── SERVER_COMPONENT.md  # Server quick reference
│   ├── RRD_VS_SQLITE.md     # Database decision
│   └── README.md            # This file
│
├── client.conf.template     # Client config template
├── server.conf.template     # Server config template
├── .gitignore              # Ignore *.conf, *.db, *.log
└── README.md               # Project overview
```

## Requirements

### Client System Requirements
- Linux (primary), macOS, or Windows
- Raw socket capability (CAP_NET_RAW on Linux for ICMP)
- ~10-50 MB RAM
- ~1 GB disk space per month of monitoring (SQLite database)
- Minimal bandwidth (~100-500 KB/s standalone, ~1-2 MB/s with server tests)

### Server System Requirements (Optional)
- Linux VPS (recommended: Hetzner, Netcup in EU)
- Location: Vienna or Frankfurt preferred (low latency to Austria)
- 1 vCPU, 512 MB RAM minimum
- 10 GB disk
- 100 Mbps network
- ~100 GB/month bandwidth
- Cost: €3-5/month

### Development Requirements
- Rust 1.70+ (stable)
- SQLite 3.x (client only)
- OpenSSL (for secret generation)
- Standard build tools (cargo, gcc/clang)

## Configuration

Configuration uses TOML files. Fully documented templates are provided:

- **`client.conf.template`** - Client configuration (all options documented)
- **`server.conf.template`** - Server configuration (all options documented)

### Quick Setup

```bash
# 1. Copy template to config file (in .gitignore)
cp client.conf.template client.conf

# 2. For standalone mode: Leave server.enabled = false
# 3. For server mode:
#    a. Generate shared secret
openssl rand -hex 32

#    b. Add secret to client.conf [server] section
#    c. Enable server: server.enabled = true
#    d. Set server host/port

# 4. Run client
bufferbane --config client.conf
```

### Server Setup

```bash
# 1. Copy template
cp server.conf.template server.conf

# 2. Add same shared secret as client
# 3. Configure bind address/port
# 4. Run server
bufferbane-server --config server.conf
```

See template files for complete documentation of all options.

## References

- **Cable Internet Architecture**: DOCSIS 3.0/3.1 specifications
- **Network Testing**: RFC 2544, RFC 6349 (TCP throughput), RFC 8175 (queue management)
- **Bufferbloat**: https://www.bufferbloat.net/
- **RRUL Test**: Realtime Response Under Load (Flent methodology)

## License

To be determined based on implementation.

## Contributing

This is currently in the planning phase. Once implementation begins, contributions following the documented specification will be welcome.

---

## Summary

This project provides a complete specification for building a professional-grade network monitoring solution specifically designed for diagnosing cable internet issues. The 4-phase architecture enables:

- **Phase 1 - Basic monitoring** (standalone): Latency, jitter, basic packet loss via ICMP
- **Phase 2 - Advanced monitoring** (with server): Throughput testing, bufferbloat detection, detailed packet loss analysis
- **Phase 3 - Geographic diversity** (multiple servers): Routing issue detection, redundancy
- **Phase 4 - Multi-interface** (WiFi + Ethernet): **Simultaneous testing** to isolate WiFi vs ISP issues
- **Comprehensive export**: CSV, JSON, PNG charts (8 chart types), HTML reports
- **Simple security**: Port knocking + ChaCha20-Poly1305 encryption avoids complex authentication
- **Privacy**: All payloads encrypted, pattern analysis resistance
- **Production-ready**: Designed for 24/7 operation with low resource usage

The planning phase is complete with detailed specifications for:
- ✅ Network metrics and detection methods
- ✅ Client-server protocol with encryption (ChaCha20-Poly1305)
- ✅ Multi-interface monitoring (WiFi + Ethernet simultaneously)
- ✅ Multi-server support (geographic diversity)
- ✅ Export capabilities (CSV, JSON, PNG charts, reports)
- ✅ Real-world scenarios and test patterns
- ✅ Technology stack and 4-phase implementation strategy
- ✅ Configuration templates with full documentation
- ✅ Database technology decision (SQLite)

**Planning Documents**:
- `SPECIFICATION.md` - Complete technical specification (1115+ lines)
- `SCENARIOS.md` - Network instability scenarios (584 lines)
- `RESEARCH.md` - Open source tools research and phases (905+ lines)
- `ENCRYPTION_SECURITY.md` - Encryption and security analysis (428 lines)
- `MULTI_INTERFACE.md` - Multi-interface monitoring (Phase 4) (515 lines)
- `SERVER_COMPONENT.md` - Server quick reference (397 lines)
- `RRD_VS_SQLITE.md` - Database decision analysis (266 lines)
- `client.conf.template` - Client configuration with multi-interface support (220+ lines)
- `server.conf.template` - Server configuration (225 lines)

---

**Status**: 📝 Planning Complete - Ready for Implementation  
**Last Updated**: 2025-10-18  
**Version**: 3.0 (Planning Documentation - Multi-Interface + Export)  
**Architecture**: Client-Server (4 phases: standalone → +server → +multi-server → +multi-interface)  
**Database**: SQLite (simpler, more flexible than RRD)  
**Total Documentation**: ~4650+ lines across 9 files

