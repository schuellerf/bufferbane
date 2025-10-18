# Bufferbane - Technical Specification

## Overview

This specification defines Bufferbane, a network quality monitoring application designed to detect fine-grained network instabilities, particularly focusing on upload connection issues and bufferbloat for cable internet connections (co-ax, such as Magenta Austria). The application performs continuous, high-frequency (1-second interval) tests to identify subtle network problems that may not completely interrupt connectivity but cause application-level issues.

**Project Name**: Bufferbane ("Bane of bufferbloat" - destroyer of buffer bloat issues)  
**Magic Bytes**: `BFBN` (0x4246424E)

## Requirements

### Functional Requirements

1. **Continuous Monitoring**: Run indefinitely with 1-second test intervals
2. **Multi-Metric Collection**: Capture multiple network health indicators simultaneously
3. **Multi-Interface Support**: Test multiple network interfaces simultaneously (WiFi + Ethernet)
4. **Multi-Server Support**: Monitor connection quality to multiple servers
5. **Historical Data**: Store all measurements for trend analysis
6. **Real-time Analysis**: Provide immediate feedback on current connection state
7. **Export Capability**: Raw data (CSV, JSON) and visualizations (PNG charts)
8. **Low Impact**: Minimal bandwidth consumption to avoid self-induced congestion
9. **Optional Server Mode**: Can operate standalone (ICMP only) or with companion server (full features)

### Non-Functional Requirements

1. **Precision**: Microsecond-level timestamp accuracy
2. **Reliability**: Must not crash or miss test intervals
3. **Resource Efficiency**: Low CPU and memory footprint
4. **Deployment**: Single binary, no external dependencies (except optional test server)
5. **Security**: Simple but effective protection against unauthorized use of server component

## Network Metrics to Monitor

### 1. Round-Trip Time (RTT) / Latency

**Description**: Time for a packet to travel to destination and back

**Measurement Method**:
- ICMP Echo Request/Reply (ping)
- Multiple targets: ISP gateway, public DNS (8.8.8.8, 1.1.1.1), specific CDNs
- Frequency: 1 packet per second per target

**Metrics Captured**:
- Minimum RTT
- Average RTT
- Maximum RTT
- Standard deviation (jitter indicator)
- Median RTT
- 95th and 99th percentile

**Thresholds** (configurable):
- Warning: >50ms to ISP gateway, >100ms to public DNS
- Critical: >200ms sustained for >10 seconds

### 2. Jitter

**Description**: Variation in packet arrival times

**Measurement Method**:
- Calculate standard deviation of RTT over rolling windows:
  - 10-second window
  - 60-second window
  - 5-minute window
- Track consecutive RTT differences

**Metrics Captured**:
- Inter-packet delay variation (IPDV)
- Peak-to-peak jitter
- Running average jitter

**Thresholds**:
- Warning: >10ms jitter
- Critical: >30ms jitter sustained

### 3. Packet Loss

**Description**: Percentage of packets that fail to reach destination or return

**Measurement Method**:
- **ICMP**: Count missing echo replies
- **UDP Stream**: Send numbered UDP packets, server echoes sequence numbers
- Track both directions separately (upload vs download loss)

**Metrics Captured**:
- Loss percentage per second
- Loss percentage per minute
- Consecutive packet loss count
- Loss burst patterns

**Thresholds**:
- Warning: >0.5% loss
- Critical: >2% loss or any burst of 5+ consecutive losses

### 4. Upload Throughput

**Description**: Actual upload speed vs expected baseline

**Measurement Method**:
- Small continuous upload stream (100KB/s baseline traffic)
- Periodic larger uploads (1MB test every 60 seconds)
- Measure actual transfer rate vs expected

**Metrics Captured**:
- Instantaneous upload rate
- Average upload rate (1-min, 5-min, 15-min)
- Upload rate variance
- Time to transfer fixed-size payloads

**Thresholds**:
- Warning: <80% of baseline
- Critical: <50% of baseline sustained for >30 seconds

### 5. Download Throughput

**Description**: Actual download speed vs expected baseline

**Measurement Method**:
- Similar to upload, but download direction
- Small continuous stream + periodic larger tests

**Metrics Captured**:
- Same as upload throughput

**Thresholds**:
- Warning: <80% of baseline
- Critical: <50% of baseline sustained for >30 seconds

### 6. DNS Resolution Time

**Description**: Time to resolve hostnames to IP addresses

**Measurement Method**:
- Query multiple DNS servers (ISP's DNS, 8.8.8.8, 1.1.1.1)
- Test common domains + random subdomains (to avoid caching)
- Frequency: Every 10 seconds

**Metrics Captured**:
- Resolution time per query
- Success/failure rate
- Timeout count

**Thresholds**:
- Warning: >100ms resolution time
- Critical: >500ms or resolution failures

### 7. Connection Stability

**Description**: TCP connection establishment and maintenance

**Measurement Method**:
- Attempt TCP connections to test servers
- Monitor existing connection states
- Track RST/FIN packets

**Metrics Captured**:
- Connection establishment time (SYN to SYN-ACK)
- Connection drops/resets
- Retransmission rate (if accessible)

**Thresholds**:
- Warning: Connection establishment >500ms
- Critical: Any unexpected connection drops

### 8. Bufferbloat

**Description**: Latency increase under load conditions

**Measurement Method**:
- Baseline: Measure latency during idle
- Under load: Measure latency during upload/download tests
- Calculate latency delta

**Metrics Captured**:
- Idle latency
- Loaded latency (upload, download, both)
- Latency increase percentage
- Queue depth estimate

**Thresholds**:
- Warning: >100ms latency increase under load
- Critical: >500ms latency increase

## Architecture: Client and Server

### Operating Modes

**Mode 1: Standalone (No Server)**
- Uses public infrastructure (8.8.8.8, 1.1.1.1, ISP gateway)
- ICMP ping for latency and packet loss
- Basic DNS testing
- **Limitations**: No throughput testing, no bufferbloat detection, limited packet loss analysis

**Mode 2: With Companion Server (Recommended)**
- All standalone features plus:
- Accurate upload/download throughput testing
- Bidirectional packet loss detection with sequence tracking
- Bufferbloat detection (RRUL-style testing)
- Connection stability monitoring
- One-way delay measurement

**Mode 3: Multiple Servers**
- All Mode 2 features plus:
- Geographic diversity (e.g., Vienna + Frankfurt)
- Routing issue detection
- Redundancy and failover
- Comparison across locations

**Mode 4: Multiple Interfaces + Multiple Servers**
- All Mode 3 features plus:
- **Simultaneous testing** of multiple network interfaces (WiFi + Ethernet)
- Real-time comparison of connection quality
- Automatic interface detection and tagging
- Isolate WiFi vs ISP issues

### Multi-Interface Monitoring (Phase 4)

**Purpose**: Test multiple network interfaces on the same machine simultaneously to isolate WiFi vs wired connection quality.

**How it works**:
```rust
// Client binds to specific interfaces and tests each independently
Interface wlan0 (WiFi):
  → Tests to all configured targets
  → Tagged as "wifi" in database

Interface eth0 (Wired):
  → Tests to same targets at same time
  → Tagged as "wired" in database

Result: Real-time comparison of WiFi vs Ethernet quality
```

**Use Cases**:
1. **Diagnose WiFi Issues**: Is lag from WiFi or ISP?
2. **A/B Testing**: Compare two connections side-by-side
3. **Failover Monitoring**: Monitor backup connection
4. **Mobile Testing**: Compare cellular vs WiFi

**Requirements**:
- Linux (required for interface binding)
- Multiple active interfaces
- Sufficient bandwidth (tests run in parallel)

**Configuration**:
```toml
[general]
interfaces = ["wlan0", "eth0"]  # Test both simultaneously

# Alternative: Single interface
interfaces = []  # Use default interface
connection_type = "wifi"  # Manual tag
```

### Server Component Design

**Purpose**: Lightweight echo and throughput test service

**Features**:
- UDP echo with sequence numbers and timestamps
- TCP throughput testing (upload/download)
- Bufferbloat test coordination
- No persistent storage (stateless design)
- Minimal resource usage

**Security Model**: Port Knocking + Encryption + Authentication

Rather than complex authentication, the server uses a simple but effective security approach:

1. **Port Knocking Sequence**
   - Server listens on configured port (e.g., 9876)
   - Client must send "knock sequence" before server responds
   - Knock = specific UDP packet with encrypted payload
   - Invalid knocks are silently dropped (no response)

2. **Shared Secret**
   - 32-byte random secret configured on both client and server
   - Used for authenticated encryption (ChaCha20-Poly1305 AEAD)
   - Prevents unauthorized usage, tampering, and eavesdropping

3. **Encryption (ChaCha20-Poly1305)**
   - All packet payloads are encrypted
   - AEAD provides both encryption AND authentication
   - Nonce: client_id (8 bytes) + nanosecond timestamp (8 bytes)
   - Associated Data: unencrypted header
   - Auth tag: 16 bytes appended to ciphertext

4. **Rate Limiting per Client**
   - Track source IP addresses
   - Limit: 10 requests/second per IP
   - Exceeding limit = temporary ignore (5 minutes)

5. **No Response to Invalid Packets**
   - Invalid decryption: silent drop
   - No knock sequence: silent drop
   - Makes server appear closed to port scanners

**Knock Protocol**:
```
Step 1: Client sends UDP knock packet
  Cleartext header:
  - Magic bytes: 0x4246424E ("BFBN" = Bufferbane)
  - Protocol version: 1
  - Packet type: 0x01 (KNOCK)
  - Payload length: encrypted payload size
  - Client ID: 8 random bytes (generated once, reused)
  - Nonce timestamp: nanoseconds since epoch (16 bytes: client_id + nano_ts)
  
  Encrypted payload (ChaCha20-Poly1305):
  - Unix timestamp (8 bytes, prevents replay)
  - Random padding (variable, makes packets harder to fingerprint)
  - Auth tag (16 bytes, appended by AEAD)

Step 2: Server validates
  - Decrypt payload using shared secret + nonce
  - Check unix timestamp is within ±60 seconds
  - Track (client_id, nonce) to prevent replay
  - If decryption fails: silent drop

Step 3: Server unlocks port for this client
  - Client IP + client_id added to allowlist
  - Valid for 5 minutes
  - Sends encrypted KNOCK_ACK response
  - Client can now send test packets

Step 4: Client performs tests
  - All test packets encrypted with ChaCha20-Poly1305
  - Server decrypts and validates each packet
  - Responds only to valid packets
```

**Why This Approach**:
- ✅ Simple to implement (no TLS, no user database, no complex auth)
- ✅ Hidden from casual scanning (appears closed)
- ✅ Prevents abuse (rate limiting + encryption)
- ✅ Prevents eavesdropping (all payloads encrypted)
- ✅ Prevents pattern analysis (encrypted + random padding)
- ✅ Low overhead (ChaCha20 is very fast)
- ✅ Replay attack protection (nonce + timestamp tracking)
- ✅ Multiple clients supported (different secrets per client)
- ✅ AEAD provides both encryption and authentication (no separate HMAC needed)

## Detection Methods

### Parallel Testing Strategy

Tests run concurrently to capture real-world conditions:

1. **Continuous Background Tests** (every second):
   - ICMP ping to 3 public targets (always available)
   - UDP packet stream to server (if server configured)
   - DNS queries (every 10 seconds)

2. **Periodic Throughput Tests** (requires server):
   - Small upload/download (100KB) every 10 seconds
   - Medium upload/download (1MB) every 60 seconds
   - Large upload/download (10MB) every 300 seconds (optional)

3. **Load Tests** (requires server, every 300 seconds):
   - Full bandwidth test while monitoring latency
   - Bufferbloat detection (RRUL methodology)

### Target Selection

**Primary Targets**:
1. ISP Gateway (first hop, obtain via traceroute)
2. Google DNS: 8.8.8.8
3. Cloudflare DNS: 1.1.1.1
4. Magenta-specific endpoint (if available)

**Selection Criteria**:
- Geographic proximity (Austria/EU)
- High availability
- Consistent response behavior
- Representative of real-world usage

### Timing Precision

- Use monotonic clock for interval scheduling
- Timestamp each measurement with:
  - Unix epoch time (milliseconds)
  - Monotonic time (nanoseconds since start)
- Compensate for test execution time to maintain 1-second intervals

## Data Collection & Storage

### Data Model

#### Measurement Record
```
{
  "timestamp": "2025-10-18T14:23:45.123456Z",
  "monotonic_ns": 1234567890123456,
  "test_type": "icmp_ping|udp_stream|throughput|dns",
  "target": "8.8.8.8",
  "metrics": {
    "rtt_ms": 25.123,
    "jitter_ms": 2.5,
    "packet_loss_pct": 0.0,
    "throughput_kbps": 5000,
    "dns_time_ms": 15.2,
    ...
  },
  "status": "success|timeout|error",
  "error_detail": null
}
```

### Storage Backend

**Primary: SQLite**
- Single file database
- Tables:
  - `measurements`: Raw per-second data
  - `aggregations_1min`: 1-minute rollups
  - `aggregations_1hour`: 1-hour rollups
  - `events`: Detected anomalies/issues
  - `configuration`: Test parameters

**Schema Example**:
```sql
CREATE TABLE measurements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,  -- Unix ms
    monotonic_ns INTEGER NOT NULL,
    interface TEXT NOT NULL,  -- Network interface (e.g., "wlan0", "eth0")
    connection_type TEXT NOT NULL,  -- Type tag (e.g., "wifi", "wired")
    test_type TEXT NOT NULL,
    target TEXT NOT NULL,
    server_name TEXT,  -- Server identifier (NULL for ICMP/DNS, name for server tests)
    rtt_ms REAL,
    jitter_ms REAL,
    packet_loss_pct REAL,
    throughput_kbps REAL,
    dns_time_ms REAL,
    status TEXT NOT NULL,
    error_detail TEXT,
    INDEX idx_timestamp (timestamp),
    INDEX idx_interface (interface),
    INDEX idx_connection_type (connection_type),
    INDEX idx_test_type (test_type),
    INDEX idx_target (target),
    INDEX idx_server_name (server_name)
);

CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    event_type TEXT NOT NULL,  -- 'latency_spike', 'packet_loss', etc.
    severity TEXT NOT NULL,  -- 'warning', 'critical'
    description TEXT,
    metrics JSON,
    duration_ms INTEGER,
    INDEX idx_timestamp (timestamp),
    INDEX idx_event_type (event_type)
);
```

### Data Retention

- Raw measurements: Keep for 30 days
- 1-minute aggregations: Keep for 90 days
- 1-hour aggregations: Keep for 1 year
- Events: Keep indefinitely
- Automatic cleanup job runs daily

### Real-time Statistics

Maintain in-memory rolling windows:
- Last 60 seconds (per-second granularity)
- Last 60 minutes (per-minute granularity)
- Last 24 hours (per-5-minute granularity)

Calculate on-the-fly:
- Min, max, mean, median
- Standard deviation
- 95th and 99th percentiles
- Event counts by type

### Export Formats

Bufferbane provides comprehensive export capabilities for analysis and reporting.

**CSV Export** (Spreadsheet Analysis):
```bash
bufferbane export --format csv --start "2025-10-18 00:00" --end "2025-10-18 23:59"
```
- Time series data with all measurements
- Configurable time range and metrics
- Optional filtering by interface, target, connection_type
- Output: `bufferbane_export_YYYYMMDD_HHMMSS.csv`

**JSON Export** (Programmatic Analysis):
```bash
bufferbane export --format json --last 7d --output data.json
```
- Complete measurement records with metadata
- Structured for programmatic analysis
- Includes aggregations and statistics
- Nested format for easy parsing

**Chart Export** (Visual Analysis):

**Phase 1: Basic Time Series Chart**
```bash
# Generate basic latency chart (available in Phase 1)
bufferbane chart --last 24h --output latency.png

# Generate interactive HTML chart with detailed tooltips
bufferbane chart --last 24h --interactive --output latency.html
```

Single chart showing latency over time with statistical visualization:
- **Min line**: Minimum latency (lower bound, thin line)
- **Max line**: Maximum latency (upper bound, thin line)
- **Avg line**: Average latency (bold, primary metric)
- **P95/P99 lines**: 95th and 99th percentile (dashed lines)
- **Shaded area**: Light fill between min and max (shows variance)
- **Multiple targets**: Color-coded lines for each target
- Export as PNG (1920x1080, configurable) or HTML (interactive)

**Data Aggregation**:
Charts aggregate raw measurements into time windows for clarity and performance:
- **Default segments**: 100 (configurable via `--segments` flag)
- **Window size**: `(time_range) / segments` (e.g., 24h / 100 = ~14.4 minutes per window)
- **Custom segments**: `--segments 50` (less detail), `--segments 200` (more detail)
- **Statistics per window**: min, max, avg, P95, P99
- **Interactive tooltips**: Hover to see all statistics for each time window
- **Benefits**: 
  - Handles large datasets (3600+ measurements/hour)
  - Reduces visual clutter
  - Shows statistical trends clearly
  - Flexible detail level for different use cases
  - Enables meaningful zoom/pan (Phase 4)

**Usage Examples**:
```bash
# Default (100 segments)
bufferbane chart --last 24h --output latency.png

# High detail (200 segments) for detailed analysis
bufferbane chart --last 24h --segments 200 --output detailed.png

# Low detail (50 segments) for quick overview
bufferbane chart --last 7d --segments 50 --output overview.png

# Very high detail (500 segments) for short time ranges
bufferbane chart --last 1h --segments 500 --output minute_by_minute.png
```

**Example visual layout**:
```
Latency Over Time (Last 24 Hours)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Latency (ms)
100 ┤                            ╭─╮ Max
 90 ┤                          ╭─╯ ╰─╮
 80 ┤                        ╭─╯     ╰─╮
 70 ┤                      ╭─╯         ╰─╮
 60 ┤                    ╭─╯             ╰─╮
 50 ┤    ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ P99
 40 ┤   ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄    P95
 30 ┤  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  Avg (bold)
 20 ┤ [████████ Shaded area shows variance ████████]
 10 ┤ ─────────────────────────────────────────── Min
  0 └┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬
    00:00 04:00 08:00 12:00 16:00 20:00 24:00
    
Legend:
  8.8.8.8 (Google)  ━━━ (blue)
  1.1.1.1 (Cloudflare) ─── (green)
  ISP Gateway ─·─·─ (orange)
```

**Purpose**: Immediately visualize latency patterns and variance without needing external tools.

**Phase 4: Advanced Charts**
```bash
# Generate all advanced charts (Phase 4)
bufferbane charts --last 24h --output-dir ./charts

# Generate specific chart
bufferbane chart latency_over_time --interface wlan0 --last 7d --output latency.png
```

**Available Chart Types (Phase 4)**:

1. **latency_over_time**: Enhanced line chart with interface comparison
   - Multiple targets and interfaces on same plot
   - Color-coded by target and interface
   - Shaded regions for jitter
   
2. **jitter_over_time**: Jitter (latency variation) over time
   - Shows stability of connection
   - Spikes indicate instability
   
3. **packet_loss_over_time**: Packet loss percentage
   - Bar chart or line chart
   - Highlights problem periods
   
4. **throughput_over_time**: Upload/download speeds
   - Dual-axis line chart
   - Baseline indicators
   
5. **latency_distribution**: Histogram of latency values
   - Shows typical vs outlier latencies
   - Percentile markers (50th, 95th, 99th)
   
6. **connection_comparison**: Bar chart comparing interfaces
   - Side-by-side comparison of WiFi vs Wired
   - Multiple metrics (latency, jitter, loss, speed)
   
7. **daily_heatmap**: Heatmap showing quality by hour/day
   - Y-axis: Hour of day (00-23)
   - X-axis: Day of week or date
   - Color: Average latency or packet loss
   - Reveals time-of-day patterns

8. **bufferbloat_impact**: Latency under load vs idle
   - Scatter plot or before/after comparison
   - Shows bufferbloat severity

**Chart Configuration** (Phase 1):
```toml
[export]
enable_charts = true
chart_width = 1920
chart_height = 1080
chart_dpi = 100
chart_style = "darkgrid"  # darkgrid, whitegrid, dark, white

# Phase 1: Basic time series chart
# Phase 4: All advanced chart types
```

**Chart Library**: Uses `plotters` Rust crate for PNG generation
- No external dependencies (Python, gnuplot, etc.)
- Fast rendering
- High-quality output
- Customizable styles
- Available from Phase 1 for basic charts

**Summary Report** (Human-Readable):
```bash
bufferbane report --last 7d --format markdown --output report.md
```
- Human-readable text/markdown/HTML
- Key statistics and detected issues
- Embedded charts (for HTML output)
- Recommendations based on findings

**Example Export Workflow**:
```bash
# Quick diagnosis: Generate all charts for problem period
bufferbane charts --start "2025-10-18 18:00" --end "2025-10-18 22:00" \
                  --output-dir ./problem-evening/

# Creates:
#   ./problem-evening/latency_over_time.png
#   ./problem-evening/jitter_over_time.png
#   ./problem-evening/packet_loss_over_time.png
#   ./problem-evening/throughput_over_time.png
#   ./problem-evening/connection_comparison.png
#   ./problem-evening/daily_heatmap.png

# Export raw data for detailed analysis
bufferbane export --format csv --same-period --output raw_data.csv

# Generate comprehensive report
bufferbane report --same-period --format html --output report.html
```

## Alert Conditions

### Alert Types

1. **Latency Spike**
   - Trigger: RTT >3x baseline for >5 consecutive seconds
   - Data: Affected targets, duration, max RTT

2. **High Jitter**
   - Trigger: Jitter >30ms sustained for >10 seconds
   - Data: Jitter value, affected period

3. **Packet Loss**
   - Trigger: >1% loss over 60-second window OR any 5 consecutive drops
   - Data: Loss percentage, pattern (burst/scattered)

4. **Upload Degradation**
   - Trigger: Upload speed <70% baseline for >30 seconds
   - Data: Expected vs actual speed, duration

5. **Download Degradation**
   - Trigger: Download speed <70% baseline for >30 seconds
   - Data: Expected vs actual speed, duration

6. **DNS Issues**
   - Trigger: Resolution time >200ms or >10% failures
   - Data: Affected DNS servers, failure rate

7. **Connection Instability**
   - Trigger: >2 connection drops in 5 minutes
   - Data: Connection targets, error types

8. **Bufferbloat Detected**
   - Trigger: Latency increases >200ms under load
   - Data: Idle vs loaded latency, load type

### Alert Actions

- **Logging**: Write to events table with full details
- **Console Output**: Display alert with color coding
- **File Output**: Append to alerts.log
- **Optional**: System notification, webhook, email (future enhancement)

### Alert Deduplication

- Suppress duplicate alerts within 5-minute window
- Track alert state transitions (start, ongoing, resolved)
- Generate summary of ongoing issues every 5 minutes

## Output & Reporting

### Console Output (Real-time)

```
[2025-10-18 14:23:45] Bufferbane v1.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Current Status: GOOD
Uptime: 2h 34m 12s

Latency (last 60s):
  ISP Gateway:    avg=5.2ms   jitter=0.8ms   loss=0.0%
  8.8.8.8:        avg=15.3ms  jitter=1.2ms   loss=0.0%
  1.1.1.1:        avg=12.8ms  jitter=0.9ms   loss=0.0%

Throughput (last 60s):
  Upload:         avg=4.8 Mbps  (95% of baseline)
  Download:       avg=48.5 Mbps (97% of baseline)

DNS Resolution:  avg=18.5ms

Events (last 5 min): 0

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[!] ALERT: Upload degradation detected at 14:21:30
    Duration: 45s | Speed: 2.1 Mbps (42% of baseline)
```

### Historical Queries

CLI commands for data analysis:
```bash
# Show summary for last 24 hours
bufferbane report --last 24h

# Show all events today
bufferbane events --date 2025-10-18

# Export data to CSV
bufferbane export --start "2025-10-18 00:00" --end "2025-10-18 23:59" --output data.csv

# Show stats for specific time range
bufferbane stats --start "14:00" --end "15:00"
```

## Configuration

Configuration is managed via TOML files. Template files with full documentation are provided:

- **`client.conf.template`** - Client configuration with all options documented
- **`server.conf.template`** - Server configuration with all options documented

### Setup Instructions

**Client Setup:**
```bash
# 1. Copy template to actual config file
cp client.conf.template client.conf

# 2. Edit client.conf:
#    - Leave server.enabled = false for standalone mode
#    - OR set server.enabled = true and configure server connection
#    - Adjust thresholds for your connection

# 3. Generate shared secret if using server
openssl rand -hex 32

# 4. Add shared secret to client.conf [server] section
```

**Server Setup:**
```bash
# 1. Copy template to actual config file
cp server.conf.template server.conf

# 2. Edit server.conf:
#    - Set bind_address (usually "0.0.0.0")
#    - Set bind_port (must match client)
#    - Add the same shared secret as client

# 3. Adjust security and resource limits
```

**Note**: `client.conf` and `server.conf` are in `.gitignore` and will not be committed (they contain secrets).

### Key Configuration Sections

**Client (`client.conf.template`)**:
- `[general]` - Test interval, database path
- `[targets]` - ISP gateway, DNS servers to monitor
- `[server]` - Optional server connection (enabled = false for standalone)
- `[thresholds]` - Alert thresholds for all metrics
- `[tests.throughput]` - Throughput test configuration (requires server)
- `[tests.bufferbloat]` - Bufferbloat test configuration (requires server)
- `[alerts]` - Alert output configuration
- `[data_retention]` - How long to keep data

**Server (`server.conf.template`)**:
- `[general]` - Bind address/port, shared secret
- `[security]` - Port knocking, rate limiting, session management
- `[tests]` - Test size limits and configuration
- `[limits]` - Resource limits (bandwidth, memory, connections)
- `[logging]` - Server logging configuration
- `[statistics]` - Optional server-side statistics

### Generating Shared Secret

```bash
# Generate a secure 32-byte shared secret
openssl rand -hex 32

# Example output:
# a7b3c9d8e1f4a2b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9

# Add this exact string to both:
# - client.conf: [server] section, shared_secret field
# - server.conf: [general] section, shared_secret field

# IMPORTANT: Keep this secret secure!
# Anyone with this secret can use your server.
```

### Minimal Configuration Examples

**Client (Standalone Mode)**:
```toml
[general]
test_interval_ms = 1000

[targets]
public_dns = ["8.8.8.8", "1.1.1.1"]

[server]
enabled = false  # Standalone mode
```

**Client (With Server)**:
```toml
[general]
test_interval_ms = 1000

[server]
enabled = true
host = "monitor.example.com"
port = 9876
shared_secret = "a7b3c9d8e1f4a2b5..."  # From openssl rand -hex 32
```

**Server (Minimal)**:
```toml
[general]
bind_address = "0.0.0.0"
bind_port = 9876
shared_secret = "a7b3c9d8e1f4a2b5..."  # Must match client
```

For complete configuration options and detailed documentation, see the template files.

## Server Protocol Specification

### Packet Format

All packets use a common structure with cleartext header and encrypted payload:

```
Cleartext Header (24 bytes):
[0-3]    Magic: 0x4246424E ("BFBN" = Bufferbane)
[4]      Protocol Version: 0x01
[5]      Packet Type: see below
[6-7]    Encrypted Payload Length: uint16 big-endian (includes auth tag)
[8-15]   Client ID: 8 random bytes (persistent per client)
[16-23]  Nonce Timestamp: nanoseconds since epoch, uint64 big-endian

Encrypted Payload (variable length):
  ChaCha20-Poly1305 AEAD encryption using:
  - Key: 32-byte shared secret
  - Nonce: 12 bytes = client_id[0:4] || nonce_timestamp[0:8]
  - Associated Data: cleartext header (24 bytes)
  
  Plaintext payload structure (before encryption):
    [0-N]   Packet-specific data (see packet types below)
    [N+1-M] Random padding (0-255 bytes, optional, prevents size fingerprinting)
  
  After encryption:
    [0-M]   Ciphertext (encrypted plaintext)
    [M+1-M+16] Auth Tag (16 bytes, provided by ChaCha20-Poly1305)

Total Packet: 24 bytes (header) + encrypted_payload_length
```

**Security Properties**:
- **Confidentiality**: Payload contents hidden from eavesdroppers
- **Authenticity**: Auth tag proves packet came from holder of shared secret
- **Integrity**: Any tampering detected via auth tag validation
- **Replay Protection**: Nonce (client_id + nano timestamp) must be unique
- **Pattern Hiding**: Random padding makes packets harder to fingerprint

### Packet Types

**Note**: All payload structures below are **before encryption**. In actual transmission, these are encrypted with ChaCha20-Poly1305 and have a 16-byte auth tag appended.

**0x01: KNOCK** (Client → Server)
```
Purpose: Authenticate and unlock port for this client
Plaintext Payload (before encryption):
  [0-7]   Unix timestamp: uint64 (for replay protection)
  [8-N]   Random padding: 0-255 bytes (optional)
Response: KNOCK_ACK or silent drop if invalid
```

**0x02: KNOCK_ACK** (Server → Client)
```
Purpose: Confirm client is authenticated
Payload (8 bytes):
  [0-3]  Session ID: uint32
  [4-7]  Valid until: Unix timestamp, uint32
```

**0x10: ECHO_REQUEST** (Client → Server)
```
Purpose: Latency measurement with packet loss tracking
Payload (16 bytes):
  [0-3]  Sequence number: uint32
  [4-7]  Client send timestamp: nanoseconds part, uint32
  [8-15] Random data for packet identification
```

**0x11: ECHO_REPLY** (Server → Client)
```
Purpose: Echo back with server timestamps
Payload (32 bytes):
  [0-3]  Sequence number: uint32 (echoed)
  [4-7]  Client send timestamp: uint32 (echoed)
  [8-15] Random data: uint64 (echoed)
  [16-23] Server receive timestamp: uint64 nanoseconds since epoch
  [24-31] Server send timestamp: uint64 nanoseconds since epoch
```

**0x20: THROUGHPUT_START** (Client → Server)
```
Purpose: Begin upload throughput test
Payload (12 bytes):
  [0-3]  Test ID: uint32 (random, identifies this test)
  [4-7]  Target size bytes: uint32
  [8-11] Expected duration ms: uint32
```

**0x21: THROUGHPUT_DATA** (Client → Server)
```
Purpose: Upload test data chunks
Payload (variable, max 65KB):
  [0-3]  Test ID: uint32
  [4-7]  Chunk sequence: uint32
  [8+]   Data: random bytes
```

**0x22: THROUGHPUT_END** (Client → Server)
```
Purpose: Signal end of upload test
Payload (8 bytes):
  [0-3]  Test ID: uint32
  [4-7]  Total bytes sent: uint32
```

**0x23: THROUGHPUT_STATS** (Server → Client)
```
Purpose: Report server-side throughput measurements
Payload (24 bytes):
  [0-3]  Test ID: uint32
  [4-7]  Bytes received: uint32
  [8-11] Duration ms: uint32
  [12-15] Throughput kbps: uint32
  [16-19] Packets received: uint32
  [20-23] Packets lost: uint32
```

**0x30: DOWNLOAD_REQUEST** (Client → Server)
```
Purpose: Request download throughput test
Payload (12 bytes):
  [0-3]  Test ID: uint32
  [4-7]  Requested bytes: uint32
  [8-11] Rate limit kbps: uint32 (0 = unlimited)
```

**0x31: DOWNLOAD_DATA** (Server → Client)
```
Purpose: Download test data chunks
Payload (variable):
  [0-3]  Test ID: uint32
  [4-7]  Chunk sequence: uint32
  [8+]   Data: random bytes
```

**0x32: DOWNLOAD_END** (Server → Client)
```
Purpose: Signal end of download
Payload (8 bytes):
  [0-3]  Test ID: uint32
  [4-7]  Total bytes sent: uint32
```

**0x40: BUFFERBLOAT_START** (Client → Server)
```
Purpose: Begin coordinated bufferbloat test
Payload (8 bytes):
  [0-3]  Test ID: uint32
  [4]    Test type: 0x01=upload, 0x02=download, 0x03=both
  [5-7]  Reserved
```

**0x41: BUFFERBLOAT_END** (Client → Server)
```
Purpose: End bufferbloat test
Payload (4 bytes):
  [0-3]  Test ID: uint32
```

**0xFF: ERROR** (Server → Client)
```
Purpose: Error response
Payload (variable):
  [0]    Error code: see below
  [1-N]  Error message: UTF-8 string

Error codes:
  0x01: Authentication required (knock first)
  0x02: Invalid HMAC
  0x03: Rate limit exceeded
  0x04: Invalid packet format
  0x05: Test size exceeds maximum
  0x06: Server overloaded
```

### Communication Flow Examples

**Example 1: Initial Connection and Echo Test**
```
Client → Server: KNOCK
Server → Client: KNOCK_ACK (session ID: 12345, valid until: +300s)
Client → Server: ECHO_REQUEST (seq: 1)
Server → Client: ECHO_REPLY (seq: 1, timestamps)
Client → Server: ECHO_REQUEST (seq: 2)
Server → Client: ECHO_REPLY (seq: 2, timestamps)
...
```

**Example 2: Upload Throughput Test**
```
Client → Server: THROUGHPUT_START (test_id: 7890, size: 1MB)
Client → Server: THROUGHPUT_DATA (test_id: 7890, seq: 0, data)
Client → Server: THROUGHPUT_DATA (test_id: 7890, seq: 1, data)
...
Client → Server: THROUGHPUT_DATA (test_id: 7890, seq: 15, data)
Client → Server: THROUGHPUT_END (test_id: 7890, bytes: 1048576)
Server → Client: THROUGHPUT_STATS (received: 1048576, duration: 2154ms, 
                                    throughput: 3891 kbps, lost: 0)
```

**Example 3: Bufferbloat Test**
```
# Phase 1: Baseline latency
Client → Server: ECHO_REQUEST (seq: 1-10)
Server → Client: ECHO_REPLY (seq: 1-10)
# Client calculates baseline: 15ms average

# Phase 2: Under load
Client → Server: BUFFERBLOAT_START (test_id: 1111, type: upload)
Client → Server: THROUGHPUT_START (test_id: 2222, size: 10MB)
# Client starts uploading
Client → Server: THROUGHPUT_DATA (continuous stream)
# While uploading, client continues echo tests
Client → Server: ECHO_REQUEST (seq: 11-20)
Server → Client: ECHO_REPLY (seq: 11-20)
# Client measures latency during load: 185ms average
# Bufferbloat detected: 170ms increase

Client → Server: THROUGHPUT_END (test_id: 2222)
Client → Server: BUFFERBLOAT_END (test_id: 1111)
Server → Client: THROUGHPUT_STATS (...)

# Phase 3: Recovery
Client → Server: ECHO_REQUEST (seq: 21-30)
Server → Client: ECHO_REPLY (seq: 21-30)
# Client verifies latency returns to baseline
```

### Security Implementation Details

**Encryption/Decryption (ChaCha20-Poly1305)**:
```rust
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce
};

// Encrypt payload
fn encrypt_payload(
    secret: &[u8; 32],
    client_id: &[u8; 8],
    nonce_timestamp: u64,
    header: &[u8; 24],
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    let cipher = ChaCha20Poly1305::new(secret.into());
    
    // Construct 12-byte nonce: client_id[0:4] + nonce_timestamp[0:8]
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[0..4].copy_from_slice(&client_id[0..4]);
    nonce_bytes[4..12].copy_from_slice(&nonce_timestamp.to_be_bytes());
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Encrypt with associated data (header)
    let payload = Payload {
        msg: plaintext,
        aad: header,  // Associated data: authenticated but not encrypted
    };
    
    cipher.encrypt(nonce, payload)
        .map_err(|_| Error::EncryptionFailed)
}

// Decrypt payload
fn decrypt_payload(
    secret: &[u8; 32],
    client_id: &[u8; 8],
    nonce_timestamp: u64,
    header: &[u8; 24],
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    let cipher = ChaCha20Poly1305::new(secret.into());
    
    // Construct same nonce
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[0..4].copy_from_slice(&client_id[0..4]);
    nonce_bytes[4..12].copy_from_slice(&nonce_timestamp.to_be_bytes());
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Decrypt and verify
    let payload = Payload {
        msg: ciphertext,
        aad: header,  // Must match encryption
    };
    
    cipher.decrypt(nonce, payload)
        .map_err(|_| Error::DecryptionFailed)
}
```

**Timestamp Validation**:
```rust
fn validate_timestamp(packet_timestamp: u64, tolerance_secs: u64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let diff = if now > packet_timestamp {
        now - packet_timestamp
    } else {
        packet_timestamp - now
    };
    
    diff <= tolerance_secs
}
```

**Nonce Tracking (Replay Protection)**:
```rust
use std::collections::HashSet;

struct NonceCache {
    // Store recently used nonces: (client_id, nonce_timestamp)
    // Keep for 2 * knock_timeout_window to prevent replay
    cache: HashSet<([u8; 8], u64)>,
    cleanup_interval: Duration,
}

impl NonceCache {
    fn is_nonce_used(&self, client_id: [u8; 8], nonce_ts: u64) -> bool {
        self.cache.contains(&(client_id, nonce_ts))
    }
    
    fn mark_nonce_used(&mut self, client_id: [u8; 8], nonce_ts: u64) {
        self.cache.insert((client_id, nonce_ts));
    }
    
    fn cleanup_old_nonces(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        // Remove nonces older than 2 * tolerance window
        self.cache.retain(|(_, nonce_ts)| {
            now - nonce_ts < 120_000_000_000  // 120 seconds in nanoseconds
        });
    }
}
```

**Client Session Management**:
```rust
struct ClientSession {
    client_id: [u8; 8],
    ip_address: IpAddr,
    session_id: u32,
    valid_until: SystemTime,
    request_count: u32,
    last_request: Instant,
    nonce_cache: NonceCache,  // Track nonces for this client
}

// Server maintains HashMap<(IpAddr, [u8; 8]), ClientSession>
// Clean up expired sessions every minute
```

## Implementation Considerations

### Privileges

- ICMP requires raw socket access (root/CAP_NET_RAW on Linux)
- Options:
  1. Run as root (not recommended)
  2. Set capability: `sudo setcap cap_net_raw+ep ./bufferbane`
  3. Use unprivileged ICMP sockets (Linux 3.x+)

### Error Handling

- Network timeouts: Treat as measurement failure, record as such
- Test server unreachable: Retry with exponential backoff, alert if sustained
- Database errors: Critical, should halt program with clear error
- Clock adjustments: Use monotonic time for intervals

### Performance Optimization

- Async I/O for all network operations
- Batch database writes (every 10 seconds or 10 measurements)
- Connection pooling for test targets
- Memory-mapped files for high-frequency logging (optional)

### Platform Compatibility

- Primary target: Linux (Fedora, Debian, Ubuntu)
- Secondary: Windows (some limitations on raw sockets)
- Should work: macOS (with permissions)

## Success Criteria

1. Can detect latency spikes >100ms that last <5 seconds
2. Can identify packet loss bursts as small as 3 consecutive packets
3. Can distinguish between upload and download issues
4. Maintains precise 1-second test intervals (±10ms)
5. Runs for >24 hours without crashes or memory leaks
6. Database size <100MB per week of continuous monitoring
7. CPU usage <5% on modern hardware
8. Accurately detects bufferbloat conditions

