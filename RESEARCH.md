# Open Source Tools Research & Technology Recommendation

This document evaluates existing open-source network monitoring tools against the requirements for fine-grained internet connection instability detection, specifically for diagnosing upload issues on cable internet.

## Research Criteria

### Must-Have Requirements
1. ✓ 1-second or finer test granularity
2. ✓ Upload-specific testing capability
3. ✓ Packet loss detection
4. ✓ Historical data storage
5. ✓ Can run continuously
6. ✓ Active maintenance (commits within last year)

### Nice-to-Have Features
- Bufferbloat detection
- Multiple simultaneous targets
- Low resource usage
- Easy deployment (single binary or simple setup)
- Built-in visualization
- Export capabilities

---

## Tool Evaluations

### 1. Smokeping

**Website**: https://oss.oetiker.ch/smokeping/  
**Language**: Perl  
**License**: GPL

**Description**:
Latency monitoring tool that creates graphs showing network latency over time. It sends test packets at regular intervals and visualizes the results.

**Capabilities**:
- ✓ Continuous monitoring
- ✓ Multiple targets
- ✓ Historical data (RRD database)
- ✓ Packet loss detection
- ✓ Beautiful latency graphs
- ✓ Long-term trending

**Limitations**:
- ✗ Default 5-minute intervals (can be configured to 20 seconds minimum)
- ✗ Primarily focused on latency, not throughput
- ✗ No upload/download testing
- ✗ No bufferbloat detection
- ✗ Complex setup (web server, RRD tools)
- ✗ Heavy for simple use case

**Verdict**: Good for latency/loss monitoring but lacks throughput testing and 1-second granularity. **Not suitable as standalone solution.**

**Relevance Score**: 4/10

---

### 2. LibreSpeed

**Website**: https://github.com/librespeed/speedtest  
**Language**: JavaScript (PHP backend)  
**License**: LGPL-3.0

**Description**:
Self-hosted HTML5 speedtest. Lightweight, web-based speed testing tool.

**Capabilities**:
- ✓ Upload and download testing
- ✓ Latency measurement
- ✓ Self-hosted (control your test server)
- ✓ Lightweight

**Limitations**:
- ✗ Manual/on-demand testing (not continuous)
- ✗ No historical data storage
- ✗ No automated monitoring
- ✗ Web-based only (needs server)
- ✗ Not designed for 1-second intervals

**Verdict**: Good for manual speed testing but not for continuous monitoring. **Not suitable.**

**Relevance Score**: 2/10

---

### 3. Netdata

**Website**: https://github.com/netdata/netdata  
**Language**: C  
**License**: GPL-3.0

**Description**:
Real-time performance monitoring for systems and applications. Comprehensive monitoring solution with web dashboard.

**Capabilities**:
- ✓ Real-time monitoring (per-second)
- ✓ Beautiful web interface
- ✓ Historical data
- ✓ Low resource usage
- ✓ Active development
- ✓ Monitors many system metrics

**Limitations**:
- ✗ Primarily system monitoring, not network quality
- ✗ Network monitoring is basic (bytes in/out)
- ✗ No specific upload/download testing
- ✗ No packet loss detection
- ✗ No latency monitoring built-in
- ⚠ Can add custom plugins but significant work

**Verdict**: Excellent for system monitoring but lacks network quality testing features. **Not suitable as primary tool**, but could complement custom solution for system metrics.

**Relevance Score**: 3/10

---

### 4. iperf3

**Website**: https://github.com/esnet/iperf  
**Language**: C  
**License**: BSD-3-Clause

**Description**:
Active measurement tool for maximum network throughput. Client-server architecture for bandwidth testing.

**Capabilities**:
- ✓ Accurate throughput testing
- ✓ Upload and download testing
- ✓ Detailed statistics
- ✓ UDP testing with loss reporting
- ✓ Per-second interval reporting
- ✓ JSON output

**Limitations**:
- ✗ Requires server setup (need control of remote endpoint)
- ✗ Not designed for continuous monitoring
- ✗ No historical data storage
- ✗ No integrated latency monitoring
- ✗ Manual invocation

**Verdict**: Excellent for bandwidth testing but requires server infrastructure and not designed for continuous monitoring. **Could be used as component** in larger solution.

**Relevance Score**: 5/10 (good for specific testing, not complete solution)

---

### 5. MTR (My Traceroute)

**Website**: https://github.com/traviscross/mtr  
**Language**: C  
**Language**: GPL-2.0

**Description**:
Combines functionality of traceroute and ping. Shows network path and latency to each hop.

**Capabilities**:
- ✓ Real-time latency monitoring
- ✓ Per-hop visibility
- ✓ Packet loss detection per hop
- ✓ Can run continuously
- ✓ Lightweight
- ✓ Excellent for diagnosing where issues occur

**Limitations**:
- ✗ No throughput testing
- ✗ No built-in data storage (can export)
- ✗ No upload/download distinction
- ✗ Basic reporting

**Verdict**: Excellent diagnostic tool for understanding where packet loss/latency occurs, but incomplete for full monitoring. **Useful complementary tool**.

**Relevance Score**: 6/10

---

### 6. Gping

**Website**: https://github.com/orf/gping  
**Language**: Rust  
**License**: MIT

**Description**:
Ping tool with live graphing. Modern alternative to ping with visual feedback.

**Capabilities**:
- ✓ Real-time latency visualization
- ✓ Multiple targets simultaneously
- ✓ Beautiful terminal graphs
- ✓ Easy to use
- ✓ Written in Rust (good performance)

**Limitations**:
- ✗ Only ICMP ping (no throughput)
- ✗ No data storage
- ✗ No packet loss statistics
- ✗ Visualization only (not logging)
- ✗ Not designed for unattended operation

**Verdict**: Great for interactive latency monitoring but lacks features for comprehensive testing. **Good for quick diagnostics only**.

**Relevance Score**: 4/10

---

### 7. Flent

**Website**: https://github.com/tohojo/flent  
**Language**: Python  
**License**: GPL-3.0

**Description**:
The FLExible Network Tester. Comprehensive network testing with multiple flows and detailed plots.

**Capabilities**:
- ✓ Multiple test types (RRUL, TCP upload/download)
- ✓ Bufferbloat detection (RRUL test)
- ✓ Detailed plots and statistics
- ✓ Can use netperf, iperf
- ✓ Export data (JSON, CSV)
- ✓ Good for detailed analysis

**Limitations**:
- ✗ Requires test server setup
- ⚠ Designed for test runs, not continuous monitoring
- ⚠ Can be complex to set up
- ✗ Less suitable for 24/7 operation
- ⚠ Python performance for continuous ops

**Verdict**: Excellent for detailed network testing and bufferbloat detection, but more suited for periodic comprehensive tests than continuous monitoring. **Could inspire test methodology**.

**Relevance Score**: 7/10 (best bufferbloat detection, but not continuous)

---

### 8. IRTT (Isochronous Round-Trip Tester)

**Website**: https://github.com/heistp/irtt  
**Language**: Go  
**License**: GPL-2.0

**Description**:
Round-trip time and network quality measurement tool with precise timing and detailed statistics.

**Capabilities**:
- ✓ Sub-millisecond precision
- ✓ Detailed latency statistics
- ✓ One-way delay measurement
- ✓ Packet loss detection
- ✓ Excellent for QoS testing
- ✓ JSON output
- ✓ Can run continuously
- ✓ Low overhead

**Limitations**:
- ✗ Requires server setup
- ⚠ No throughput testing (only probes)
- ✗ No historical data storage built-in
- ⚠ Less active maintenance (last update 2 years ago)

**Verdict**: Excellent for precise latency and jitter measurement with minimal overhead. **Strong candidate for latency/jitter component**.

**Relevance Score**: 8/10

---

### 9. NetPerfMeter

**Website**: https://github.com/dreibh/netperfmeter  
**Language**: C++  
**License**: GPL-3.0

**Description**:
Network performance meter for testing various transport protocols with detailed statistics.

**Capabilities**:
- ✓ Multiple protocols (TCP, UDP, SCTP)
- ✓ Detailed statistics
- ✓ Vector output for plotting
- ✓ Active/passive modes

**Limitations**:
- ✗ Requires server setup
- ✗ Complex configuration
- ✗ More academic/research focused
- ⚠ Less suitable for simple deployment

**Verdict**: Powerful but complex. **Overkill for this use case**.

**Relevance Score**: 4/10

---

### 10. Speedtest-CLI (Ookla)

**Website**: https://github.com/sivel/speedtest-cli  
**Language**: Python  
**License**: Apache-2.0

**Description**:
Command-line interface for testing internet bandwidth using speedtest.net infrastructure.

**Capabilities**:
- ✓ Easy to use
- ✓ No server setup needed
- ✓ Upload and download testing
- ✓ Widely available servers

**Limitations**:
- ✗ Not designed for high-frequency testing
- ✗ Rate limited by Ookla
- ✗ No continuous monitoring
- ✗ Large data consumption (full speed tests)
- ✗ No packet loss detection
- ✗ Inconsistent results

**Verdict**: Good for occasional speed testing but not for continuous monitoring. **Not suitable**.

**Relevance Score**: 2/10

---

### 11. Prometheus + Blackbox Exporter

**Website**: https://github.com/prometheus/blackbox_exporter  
**Language**: Go  
**License**: Apache-2.0

**Description**:
Prometheus exporter for probing endpoints (HTTP, TCP, ICMP, DNS) with time-series database storage.

**Capabilities**:
- ✓ Continuous monitoring
- ✓ ICMP/TCP/DNS probing
- ✓ Historical data (Prometheus TSDB)
- ✓ Multiple targets
- ✓ Grafana visualization
- ✓ Alert system
- ✓ Active development
- ✓ Industry standard

**Limitations**:
- ✗ No throughput testing
- ⚠ Complex setup (Prometheus + Blackbox + Grafana)
- ⚠ No built-in upload/download testing
- ⚠ Default scrape interval 15-60 seconds (configurable to 1s)
- ⚠ Heavy infrastructure for single-purpose monitoring

**Verdict**: Enterprise-grade monitoring but complex setup and lacks throughput testing. **Good for latency/loss if infrastructure already exists**.

**Relevance Score**: 5/10 (7/10 if Prometheus already in use)

---

### 12. Pingdom / Uptime Kuma

**Website**: https://github.com/louislam/uptime-kuma  
**Language**: JavaScript (Node.js)  
**License**: MIT

**Description**:
Self-hosted monitoring tool similar to Uptime Robot. Monitors uptime with notifications.

**Capabilities**:
- ✓ Web-based interface
- ✓ Multiple monitor types (HTTP, TCP, Ping)
- ✓ Historical data
- ✓ Notifications
- ✓ Easy setup

**Limitations**:
- ✗ Focused on uptime, not quality
- ✗ No throughput testing
- ✗ Limited network metrics
- ⚠ Default intervals 60 seconds (can go to 20s)

**Verdict**: Good for uptime monitoring, not for detailed network quality analysis. **Not suitable**.

**Relevance Score**: 3/10

---

## Gap Analysis

### What Existing Tools Provide

**Latency Monitoring (Good Coverage)**:
- Smokeping, MTR, Gping, IRTT, Blackbox Exporter
- Most can achieve 1-second intervals
- Packet loss detection available

**Throughput Testing (Limited)**:
- iperf3, Flent, LibreSpeed
- Require separate server infrastructure
- Not designed for continuous operation

**Bufferbloat Detection (Very Limited)**:
- Flent (RRUL test) - best option
- No continuous bufferbloat monitoring found

**Historical Data (Mixed)**:
- Smokeping (RRD)
- Prometheus (TSDB)
- Most others require custom solution

### What's Missing

1. **Continuous 1-second throughput testing**
   - No tool found that continuously tests upload/download at 1-second intervals
   - Would require significant bandwidth

2. **Integrated solution**
   - No single tool combines latency + throughput + packet loss + bufferbloat
   - Would need to combine multiple tools

3. **Upload-specific focus**
   - No tool specializes in upload quality monitoring
   - Most treat upload/download equally

4. **Lightweight continuous operation**
   - Tools are either heavyweight (Smokeping, Prometheus) or not continuous (iperf3, Flent)

5. **Cable-specific diagnostics**
   - No tools understand cable internet characteristics
   - Would need custom analysis

### Potential Tool Combinations

**Option A: Hybrid Approach**
```
IRTT (latency/jitter) + iperf3 (periodic throughput) + custom wrapper
```
- Pros: Leverage existing quality tools
- Cons: Requires custom orchestration, separate server setup

**Option B: Extended MTR**
```
MTR (continuous) + custom throughput module
```
- Pros: MTR is solid, well-maintained
- Cons: Would need significant development

**Option C: Prometheus Stack**
```
Blackbox Exporter + Custom Exporter (throughput) + Prometheus + Grafana
```
- Pros: Enterprise-grade, excellent visualization
- Cons: Heavy infrastructure for single use case

### Conclusion

**No existing tool meets all requirements.** A custom solution is necessary, but can leverage:
- ICMP libraries from existing tools
- Test methodologies from Flent (bufferbloat)
- Data patterns from Smokeping
- Precision timing from IRTT

---

## Technology Stack Recommendation

Based on requirements for:
- **Packet-level precision**: Need raw socket access, microsecond timing
- **1-second intervals**: Precise scheduling, minimal overhead
- **Continuous operation**: Stable, low resource usage
- **Complex concurrent operations**: Multiple simultaneous tests
- **Data integrity**: Reliable storage, no data loss

### Primary Recommendation: Rust

**Why Rust?**

1. **Performance & Precision**
   - Zero-cost abstractions
   - No garbage collection pauses (critical for precise timing)
   - Direct system call access
   - Nanosecond-precision timing (`std::time::Instant`)
   - Efficient async runtime (Tokio)

2. **Network Capabilities**
   - Excellent crates for raw socket access
   - Mature networking ecosystem
   - Built-in async networking
   - Low-level packet control

3. **Reliability**
   - Memory safety without garbage collection
   - Fearless concurrency (no race conditions)
   - Robust error handling
   - Less likely to crash during long-running operation

4. **Deployment**
   - Compiles to single static binary
   - No runtime dependencies
   - Cross-platform (Linux, Windows, macOS)
   - Small binary size

5. **Resource Efficiency**
   - Minimal CPU usage
   - Low memory footprint
   - No garbage collection overhead
   - Efficient async I/O

**Recommended Rust Crates**:

```toml
[dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# ICMP ping
surge-ping = "0.8"  # Pure Rust ICMP
# Alternative: pnet = "0.34"  # More low-level control

# Raw sockets and packet crafting
socket2 = "0.5"  # Cross-platform socket library
pnet = "0.34"  # Packet construction and manipulation

# Database (client only)
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }
rusqlite = "0.30"  # Alternative: synchronous SQLite

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration
toml = "0.8"
config = "0.13"

# Logging
tracing = "0.1"  # Structured logging
tracing-subscriber = "0.3"

# CLI
clap = { version = "4.4", features = ["derive"] }

# Time handling
chrono = "0.4"

# Statistics (client only)
statrs = "0.16"  # Statistical functions

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Networking
reqwest = { version = "0.11", features = ["blocking"] }  # HTTP client (optional)
trust-dns-resolver = "0.23"  # DNS resolution

# Security (both client and server)
chacha20poly1305 = "0.10"  # AEAD encryption (ChaCha20-Poly1305)
hex = "0.4"  # Hex encoding/decoding for config
rand = "0.8"  # Random padding generation

# Server-specific
dashmap = "5.5"  # Concurrent HashMap for client session tracking
```

**Example Architecture**:

```
bufferbane/
├── client/                  # Client application
│   ├── src/
│   │   ├── main.rs          # Entry point, CLI
│   │   ├── config.rs        # Configuration management
│   │   ├── monitor/
│   │   │   ├── mod.rs       # Monitor orchestration
│   │   │   ├── icmp.rs      # ICMP ping tests
│   │   │   ├── server_tests.rs  # Tests requiring server
│   │   │   └── dns.rs       # DNS resolution tests
│   │   ├── protocol/        # Client-side protocol handling
│   │   │   ├── mod.rs       # Protocol implementation
│   │   │   ├── packets.rs   # Packet encode/decode
│   │   │   └── security.rs  # HMAC, knock sequence
│   │   ├── storage/
│   │   │   ├── mod.rs       # Storage abstraction
│   │   │   ├── database.rs  # SQLite operations
│   │   │   └── models.rs    # Data models
│   │   ├── analysis/
│   │   │   ├── mod.rs       # Analysis engine
│   │   │   ├── statistics.rs  # Statistical calculations
│   │   │   └── alerts.rs    # Alert detection
│   │   └── output/
│   │       ├── mod.rs       # Output formatting
│   │       ├── console.rs   # Terminal UI
│   │       └── export.rs    # CSV/JSON export
│   ├── Cargo.toml
│   └── config.toml          # Example client config
│
├── server/                  # Server application
│   ├── src/
│   │   ├── main.rs          # Entry point
│   │   ├── config.rs        # Server configuration
│   │   ├── protocol/        # Server-side protocol handling
│   │   │   ├── mod.rs       # Protocol implementation
│   │   │   ├── packets.rs   # Packet encode/decode
│   │   │   └── security.rs  # HMAC verification, auth
│   │   ├── handlers/
│   │   │   ├── mod.rs       # Request handlers
│   │   │   ├── echo.rs      # Echo service
│   │   │   ├── throughput.rs  # Upload/download tests
│   │   │   └── bufferbloat.rs # Bufferbloat coordination
│   │   ├── session/
│   │   │   ├── mod.rs       # Session management
│   │   │   └── rate_limit.rs  # Rate limiting
│   │   └── stats/
│   │       ├── mod.rs       # Optional server statistics
│   │       └── metrics.rs   # Metrics tracking
│   ├── Cargo.toml
│   └── config.toml          # Example server config
│
├── protocol/                # Shared protocol library
│   ├── src/
│   │   ├── lib.rs           # Protocol constants
│   │   ├── types.rs         # Packet type definitions
│   │   └── constants.rs     # Magic bytes, versions
│   └── Cargo.toml
│
└── docs/
    ├── SPECIFICATION.md
    ├── SCENARIOS.md
    ├── RESEARCH.md
    └── README.md
```

**Key Implementation Advantages**:

1. **Async Tokio Runtime**:
   ```rust
   // Multiple tests running concurrently
   tokio::select! {
       _ = icmp_monitor.run() => {},
       _ = udp_monitor.run() => {},
       _ = throughput_monitor.run() => {},
   }
   ```

2. **Precise Timing**:
   ```rust
   let mut interval = tokio::time::interval(Duration::from_secs(1));
   interval.set_missed_tick_behavior(MissedTickBehavior::Burst);
   
   loop {
       interval.tick().await;
       let start = Instant::now();
       // Perform test
       let duration = start.elapsed();
       // Compensate for test duration
   }
   ```

3. **Raw Socket ICMP**:
   ```rust
   use surge_ping::Pinger;
   
   let pinger = Pinger::new()
       .timeout(Duration::from_secs(1))
       .build()?;
   
   let (packet, duration) = pinger.ping(target_ip).await?;
   ```

4. **Efficient Database Writes**:
   ```rust
   // Batch writes every 10 seconds
   let mut buffer = Vec::with_capacity(10);
   
   while let Some(measurement) = rx.recv().await {
       buffer.push(measurement);
       if buffer.len() >= 10 {
           db.insert_batch(&buffer).await?;
           buffer.clear();
       }
   }
   ```

### Alternative: Python

**When to consider Python**:
- Rapid prototyping needed
- Team more familiar with Python
- Timing precision <10ms acceptable
- Willing to accept higher resource usage

**Python Stack**:
```python
# Core
asyncio          # Async operations
scapy           # Packet crafting
icmplib         # Simpler ICMP alternative
speedtest-cli   # Basic speed testing

# Database
sqlite3         # Built-in
sqlalchemy      # ORM

# Analysis
numpy           # Numerical operations
pandas          # Data analysis

# Visualization (optional)
rich            # Terminal UI
matplotlib      # Graphs
```

**Python Drawbacks for This Use Case**:
- GIL (Global Interpreter Lock) limits true concurrency
- Garbage collection can cause timing jitter
- Higher CPU/memory usage
- Runtime dependency (Python interpreter)
- Slower packet processing
- Less precise timing (millisecond vs microsecond)

### Alternative: Go

**Pros**:
- Good async model (goroutines)
- Fast compilation
- Single binary deployment
- Good network libraries
- Garbage collected but fast GC

**Cons**:
- GC can cause minor timing jitter (better than Python)
- Less control over memory layout than Rust
- Smaller ecosystem for packet crafting

**Verdict**: Go would be second choice if Rust is too complex or team unfamiliar with Rust.

---

## Recommended Implementation Strategy

Bufferbane will be developed in 4 major phases, each building on the previous:

### Phase 1: Client Only (Standalone Mode)
**Goal**: Basic but functional latency monitoring

**Deliverables:**
1. Shared protocol library (packet structures, encryption)
2. Client CLI with configuration
3. ICMP ping to multiple targets (1-second intervals)
4. SQLite storage for measurements
5. Console output with real-time stats
6. DNS resolution monitoring
7. Alert detection and logging
8. Basic export (CSV, JSON)

**Capabilities:**
- ✅ Latency monitoring (RTT, jitter)
- ✅ Basic packet loss detection (ICMP)
- ✅ DNS monitoring
- ✅ Historical data storage
- ❌ No throughput testing
- ❌ No bufferbloat detection

**Estimated effort**: 1-2 weeks  
**Status**: Immediately useful for diagnosing latency issues

---

### Phase 2: Client + Server
**Goal**: Full-featured monitoring with throughput and bufferbloat detection

**Deliverables:**

**Server:**
1. Server CLI with configuration
2. Port knocking and ChaCha20-Poly1305 encryption
3. Echo service (ECHO_REQUEST/ECHO_REPLY)
4. Upload/download throughput testing
5. Bufferbloat test coordination
6. Session management and rate limiting
7. Server-side statistics

**Client additions:**
1. Server communication via encrypted protocol
2. Upload/download test implementation
3. Bufferbloat test orchestration
4. Throughput metrics storage and alerts
5. Bufferbloat detection and alerts

**Capabilities:**
- ✅ All Phase 1 features
- ✅ Accurate upload/download throughput testing
- ✅ Bidirectional packet loss tracking
- ✅ Bufferbloat detection (RRUL-style)
- ✅ Connection stability monitoring
- ❌ Single server only
- ❌ Single interface only

**Estimated effort**: 2-3 weeks  
**Status**: Complete solution for cable internet diagnosis

---

### Phase 3: Multiple Servers
**Goal**: Geographic diversity and routing issue detection

**Deliverables:**

**Client additions:**
1. Multi-server configuration support
2. Parallel testing to multiple servers
3. Per-server metrics storage
4. Cross-server comparison analysis
5. Routing issue detection
6. Server failover logic

**Server:**
- No changes (reuse Phase 2 server)

**Capabilities:**
- ✅ All Phase 2 features
- ✅ Test to multiple servers simultaneously
- ✅ Geographic comparison (e.g., Vienna vs Frankfurt)
- ✅ Detect routing-specific issues
- ✅ Redundancy (failover if one server down)
- ✅ Compare peering point quality
- ❌ Single interface only

**Estimated effort**: 1-2 weeks  
**Status**: Useful for detecting ISP routing problems

---

### Phase 4: Multiple Interfaces + Multiple Servers
**Goal**: Real-time WiFi vs Ethernet comparison

**Deliverables:**

**Client additions:**
1. Multi-interface configuration support
2. Interface binding (Linux SO_BINDTODEVICE)
3. Simultaneous testing across interfaces
4. Auto-detection of interface type (WiFi vs Wired)
5. Per-interface metrics storage
6. Real-time interface comparison
7. Interface-specific alerts

**Export enhancements:**
1. PNG chart generation (plotters crate)
2. 8 chart types (latency, jitter, throughput, comparison, heatmap, etc.)
3. Comprehensive reporting (Markdown, HTML)
4. Interface comparison charts

**Capabilities:**
- ✅ All Phase 3 features
- ✅ **Test WiFi and Ethernet simultaneously**
- ✅ Real-time quality comparison
- ✅ Isolate WiFi vs ISP issues definitively
- ✅ Comprehensive export (CSV, JSON, PNG charts)
- ✅ Visual reporting for easy diagnosis

**Estimated effort**: 2-3 weeks  
**Status**: Complete solution with all advanced features

---

## Phase Summary

| Phase | Effort | Key Feature | Primary Use Case |
|-------|--------|-------------|------------------|
| **1: Client Only** | 1-2 weeks | ICMP latency monitoring | Basic latency diagnosis |
| **2: Client + Server** | 2-3 weeks | Throughput + bufferbloat | Cable internet diagnosis |
| **3: Multiple Servers** | 1-2 weeks | Geographic diversity | ISP routing issues |
| **4: Multi-Interface** | 2-3 weeks | WiFi vs Ethernet comparison | Isolate WiFi problems |

**Total estimated effort**: 6-10 weeks for all phases

**Recommended development order**: Phases 1→2→4 (skip Phase 3 initially unless routing issues are a concern)

---

## Dependencies & Crates by Phase

### Phase 1 Dependencies
```toml
tokio = { version = "1.35", features = ["full"] }
surge-ping = "0.8"
rusqlite = "0.30"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
clap = { version = "4.4", features = ["derive"] }
chrono = "0.4"
anyhow = "1.0"
```

### Phase 2 Additional Dependencies
```toml
chacha20poly1305 = "0.10"
socket2 = "0.5"
dashmap = "5.5"  # Server only
```

### Phase 3 Additional Dependencies
- No new dependencies (reuse Phase 2)

### Phase 4 Additional Dependencies
```toml
plotters = "0.3"  # Chart generation
nix = "0.27"  # Interface binding (SO_BINDTODEVICE)
```

---

## Final Recommendation

**Build a custom Rust client-server application** that:
1. Client can operate standalone (ICMP only) or with server (full features)
2. Takes inspiration from Flent's bufferbloat tests
3. Uses ICMP ping approach similar to MTR/IRTT
4. Port knocking + HMAC for simple but effective security
5. Stores data in SQLite (client side only)
6. Focuses specifically on cable upload instabilities

**Reasoning**:
- No existing tool meets all requirements
- Rust provides necessary precision and reliability
- Single binary deployment for both client and server
- Server optional but enables critical features (throughput, bufferbloat)
- Simple security model avoids complexity of full authentication
- Can evolve to add features as needed
- Performance allows for continuous 1-second testing
- Memory safety critical for 24/7 operation

**Server Deployment**:
- Minimal VPS: €3-5/month (Hetzner, Netcup)
- Location: Vienna or Frankfurt (low latency to Austria)
- Resources: 1 vCore, 512MB RAM, 10GB disk
- Bandwidth: ~100GB/month (estimated)

**Estimated Effort**:
- Phase 1 (Protocol library): 1-2 days
- Phase 2 (Client standalone): 2-3 days
- Phase 3 (Server core): 2-3 days
- Phase 4 (Throughput tests): 2-3 days
- Phase 5 (Bufferbloat): 1-2 days
- Phase 6 (Advanced features): 2-3 days
- Phase 7 (Polish & deployment): 2-3 days

**Total**: ~3-4 weeks for fully-featured, polished application

**Development Strategy**:
1. Develop protocol library first (shared by client and server)
2. Client standalone mode next (immediately useful)
3. Server core (enables enhanced testing)
4. Iteratively add features to both sides

This custom solution will outperform any combination of existing tools for this specific use case while remaining maintainable and extensible. The optional server component provides the critical functionality needed to diagnose cable internet upload issues effectively.

