# Bufferbane - Implementation Phases Summary

## Quick Reference

This document provides a quick overview of the 4 implementation phases for Bufferbane.

## Phase Overview

```
Phase 1: Client Only          → 1-2 weeks
Phase 2: + Server             → 2-3 weeks  
Phase 3: + Multiple Servers   → 1-2 weeks
Phase 4: + Multi-Interface    → 2-3 weeks
────────────────────────────────────────────
Total: 6-10 weeks for complete implementation
```

---

## Phase 1: Client Only (Standalone Mode)

### Goal
Basic but functional latency monitoring using ICMP.

### What You Get
```
✅ 1-second interval ICMP ping testing
✅ Latency (RTT), jitter, basic packet loss
✅ DNS resolution monitoring
✅ SQLite database for historical data
✅ Real-time console output
✅ Alert detection and logging
✅ Basic export (CSV, JSON)
✅ Basic chart export (PNG with statistical visualization)
```

**Chart Features (Phase 1)**:
- Line chart showing latency over time
- **Min line** (lower bound, thin line)
- **Max line** (upper bound, thin line)
- **Avg line** (bold, primary metric)
- **P95/P99 lines** (95th and 99th percentile, dashed)
- **Shaded area** between min/max (shows variance)
- Multiple targets on same plot (color-coded)
- PNG export (1920x1080, configurable)

**Example chart output**:
```
Latency Over Time (Last 24 Hours)

Latency (ms)
 50 ┤    ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ P99
 40 ┤   ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄    P95
 30 ┤  ━━━━━━━━━━━━━━━━━━━━━━   Avg (bold)
 20 ┤ [████ Shaded variance ████]
 10 ┤ ─────────────────────────  Min
  0 └┬────┬────┬────┬────┬────┬
    00:00 06:00 12:00 18:00 24:00

Visual diagnosis: Pattern shows evening latency spike (18:00-22:00)
```

Command: `bufferbane chart --last 24h --output latency.png`

### Limitations
```
❌ No throughput testing
❌ No bufferbloat detection
❌ Limited packet loss analysis (ICMP only)
❌ Dependent on external infrastructure (8.8.8.8, 1.1.1.1)
❌ Only one basic chart type (advanced charts in Phase 4)
```

### Use Case
Perfect for users who want to:
- Monitor latency and jitter over time
- Detect patterns in connection instability
- Basic diagnosis of ISP issues

### Effort
1-2 weeks

---

## Phase 2: Client + Server

### Goal
Full-featured monitoring with throughput and bufferbloat detection.

### New Features
```
✅ Upload/download throughput testing
✅ Bufferbloat detection (RRUL-style)
✅ Accurate bidirectional packet loss tracking
✅ Connection stability monitoring
✅ One-way delay measurement
✅ Encrypted communication (ChaCha20-Poly1305)
✅ Port knocking security
```

### Server Component
- Lightweight UDP service
- Echo service for latency/packet loss
- Upload/download throughput testing
- Bufferbloat test coordination
- Session management and rate limiting

### Use Case
**This is the core value proposition** for cable internet diagnosis:
- Detect upload instabilities (common on cable)
- Identify bufferbloat (latency spikes under load)
- Comprehensive connection quality monitoring

### Deployment
```bash
# Client (home network)
bufferbane --config client.conf

# Server (VPS in Austria/Germany)
bufferbane-server --config server.conf
```

### Cost
- VPS: €3-5/month (Hetzner, Netcup)
- Bandwidth: ~100 GB/month

### Effort
2-3 weeks

---

## Phase 3: Multiple Servers

### Goal
Geographic diversity for routing issue detection.

### New Features
```
✅ Test to multiple servers simultaneously
✅ Geographic comparison (e.g., Vienna vs Frankfurt)
✅ Detect routing-specific issues
✅ Server failover (redundancy)
✅ Compare peering point quality
✅ Per-server metrics and analysis
```

### Configuration
```toml
[[servers]]
name = "vienna"
host = "vienna.example.com"
port = 9876
shared_secret = "..."

[[servers]]
name = "frankfurt"
host = "frankfurt.example.com"
port = 9876
shared_secret = "..."
```

### Use Case
For diagnosing:
- ISP routing problems (one server fast, another slow)
- Peering point congestion
- Geographic latency patterns

### When to Use
- If you suspect ISP routing issues
- If one destination is consistently problematic
- For redundancy (multiple monitoring points)

### Optional?
Yes - skip Phase 3 if routing issues aren't a concern. Go directly from Phase 2 to Phase 4.

### Effort
1-2 weeks

---

## Phase 4: Multiple Interfaces + Comprehensive Export

### Goal
**Simultaneous WiFi + Ethernet testing** to definitively isolate WiFi vs ISP issues.

### New Features

**Multi-Interface:**
```
✅ Test multiple interfaces simultaneously (WiFi + Ethernet)
✅ Real-time quality comparison
✅ Auto-detection of interface types
✅ Per-interface metrics storage
✅ Interface-specific alerts
✅ Isolate WiFi vs ISP problems definitively
```

**Comprehensive Export:**
```
✅ Advanced chart types (extends Phase 1 basic chart)
✅ 8 total chart types:
   - Basic latency over time (from Phase 1)
   - Jitter over time
   - Packet loss over time
   - Throughput over time
   - Latency distribution histogram
   - Interface comparison bars
   - Daily heatmap (time-of-day patterns)
   - Bufferbloat impact visualization
✅ HTML reports with embedded charts
✅ Markdown reports for documentation
✅ Batch chart generation
```

### Configuration
```toml
[general]
# Test both WiFi and Ethernet simultaneously
interfaces = ["wlan0", "eth0"]

[export]
enable_charts = true
chart_width = 1920
chart_height = 1080
default_charts = [
    "latency_over_time",
    "jitter_over_time",
    "packet_loss_over_time",
    "throughput_over_time",
    "connection_comparison",
    "daily_heatmap",
]
```

### How It Works
```
┌─────────────────────────────────┐
│  Linux Machine                   │
│                                  │
│  Interface wlan0 (WiFi)          │
│      │                           │
│      ├─► Test every 1s           │
│      └─► Store to DB             │
│                                  │
│  Interface eth0 (Ethernet)       │
│      │                           │
│      ├─► Test every 1s (same)    │
│      └─► Store to DB             │
│                                  │
│  Result: Real-time comparison    │
└─────────────────────────────────┘
```

### Example Use Case

**Problem**: User suspects WiFi is causing lag in video calls.

**Solution**:
1. Configure `interfaces = ["wlan0", "eth0"]`
2. Run Bufferbane for 1 hour
3. Generate comparison chart:
   ```bash
   bufferbane charts --last 1h
   ```
4. **Result**:
   ```
   Interface Comparison:
   
              WiFi    Wired
   Latency    28ms    15ms    ← WiFi adds 13ms
   Jitter     8.5ms   0.8ms   ← WiFi unstable
   Loss       2.3%    0.1%    ← WiFi drops packets
   
   Verdict: WiFi is the problem!
   ```

**Answer in minutes, not weeks!**

### Requirements
- Linux (SO_BINDTODEVICE socket option)
- Multiple active network interfaces
- Both interfaces connected to same router

### Export Examples

**Generate all charts for problem period:**
```bash
bufferbane charts --start "2025-10-18 18:00" \
                  --end "2025-10-18 22:00" \
                  --output-dir ./diagnosis/
```

**Creates:**
- `latency_over_time.png` - Time series of latency
- `jitter_over_time.png` - Jitter spikes over time
- `connection_comparison.png` - WiFi vs Ethernet bar chart
- `daily_heatmap.png` - Quality by hour of day
- `latency_distribution.png` - Histogram of latencies
- `throughput_over_time.png` - Upload/download speeds

**Export raw data:**
```bash
bufferbane export --format csv --same-period --output raw.csv
```

**Generate report:**
```bash
bufferbane report --same-period --format html --output report.html
```

### Effort
2-3 weeks

---

## Recommended Implementation Order

### Standard Order
```
Phase 1 → Phase 2 → Phase 3 → Phase 4
```
**Total**: 6-10 weeks

### Recommended Order (Skip Phase 3 initially)
```
Phase 1 → Phase 2 → Phase 4
```
**Total**: 5-8 weeks

**Rationale**: 
- Phase 3 (multiple servers) is most useful for routing issues
- Phase 4 (multi-interface + export) is more valuable for typical users
- Can add Phase 3 later if routing issues are discovered

---

## Feature Matrix

| Feature | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|---------|---------|---------|---------|---------|
| **ICMP Latency** | ✅ | ✅ | ✅ | ✅ |
| **Jitter** | ✅ | ✅ | ✅ | ✅ |
| **Basic Packet Loss** | ✅ | ✅ | ✅ | ✅ |
| **DNS Monitoring** | ✅ | ✅ | ✅ | ✅ |
| **SQLite Storage** | ✅ | ✅ | ✅ | ✅ |
| **Console Output** | ✅ | ✅ | ✅ | ✅ |
| **Alerts** | ✅ | ✅ | ✅ | ✅ |
| **Upload/Download Throughput** | ❌ | ✅ | ✅ | ✅ |
| **Bufferbloat Detection** | ❌ | ✅ | ✅ | ✅ |
| **Bidirectional Packet Loss** | ❌ | ✅ | ✅ | ✅ |
| **Encrypted Protocol** | ❌ | ✅ | ✅ | ✅ |
| **Multiple Servers** | ❌ | ❌ | ✅ | ✅ |
| **Routing Diagnosis** | ❌ | ❌ | ✅ | ✅ |
| **Basic Chart (PNG)** | ✅ | ✅ | ✅ | ✅ |
| **Advanced Charts (8 types)** | ❌ | ❌ | ❌ | ✅ |
| **Multi-Interface (WiFi + Ethernet)** | ❌ | ❌ | ❌ | ✅ |
| **HTML Reports** | ❌ | ❌ | ❌ | ✅ |
| **Interface Comparison** | ❌ | ❌ | ❌ | ✅ |

---

## Dependencies by Phase

### Phase 1 Crates
```toml
tokio = { version = "1.35", features = ["full"] }
surge-ping = "0.8"
rusqlite = "0.30"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
clap = { version = "4.4", features = ["derive"] }
chrono = "0.4"
anyhow = "1.0"
plotters = "0.3"  # Chart generation (basic time series)
```

### Phase 2 Additional Crates
```toml
chacha20poly1305 = "0.10"
socket2 = "0.5"
dashmap = "5.5"  # Server only
rand = "0.8"
```

### Phase 3 Additional Crates
- No new dependencies (reuse Phase 2)

### Phase 4 Additional Crates
```toml
nix = "0.27"  # Interface binding (SO_BINDTODEVICE)
# Note: plotters is already included from Phase 1
```

---

## When to Stop at Each Phase

### Stop at Phase 1 if:
- You only care about latency and jitter
- You don't need throughput monitoring
- You can't run a VPS
- Budget constraints
- Basic visual chart is sufficient for your needs

### Stop at Phase 2 if:
- You have one VPS location
- Routing issues aren't a concern
- WiFi diagnosis isn't needed
- You're satisfied with text/CSV output

### Stop at Phase 3 if:
- You don't need WiFi vs Ethernet comparison
- Visual charts aren't important
- You can't run Linux (multi-interface needs Linux)

### Complete Phase 4 for:
- ✅ Full diagnostic capabilities
- ✅ WiFi problem diagnosis
- ✅ Visual reporting
- ✅ Professional-grade monitoring

---

## Conclusion

**Phase 1**: Basic but immediately useful (latency monitoring)  
**Phase 2**: Core value (throughput + bufferbloat) ← **Recommended minimum**  
**Phase 3**: Optional (routing diagnosis)  
**Phase 4**: Ultimate (WiFi diagnosis + visual reporting) ← **Recommended target**

**Total effort**: 5-10 weeks depending on phases implemented

**Best path for most users**: Phase 1 → Phase 2 → Phase 4 (skip Phase 3 initially)

---

**Document Version**: 1.0  
**Last Updated**: 2025-10-18  
**Total Planning Documentation**: ~4650+ lines across 9 files

