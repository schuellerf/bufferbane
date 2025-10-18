# Bufferbane

**Network quality monitoring for cable internet with bufferbloat detection**

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)
![Status](https://img.shields.io/badge/Status-Phase%201%20Complete-green.svg)

Bufferbane is a high-precision network monitoring tool designed to detect fine-grained network issues on cable internet connections (DOCSIS). It performs per-second measurements to identify latency spikes, jitter, packet loss, and bufferbloat that traditional tools miss.

**Project Name**: *Bufferbane* - "Bane of bufferbloat" (destroyer of buffer bloat issues)  
**Magic Bytes**: `BFBN` (0x4246424E)

---

## Features (Phase 1 - Current)

✅ **ICMP Latency Monitoring**
- Per-second ping tests to multiple targets
- RTT, jitter, and packet loss tracking
- Microsecond-precision timestamps

✅ **SQLite Database Storage**
- Historical data with efficient indexing
- Query by time range, target, or connection type
- Automatic schema management

✅ **Visual Chart Export**
- **Static PNG** charts with min/max/avg/P95/P99 lines
- **Interactive HTML** charts with hover tooltips and detailed statistics
- **Windowed aggregation**: 100 segments (default, configurable via `--segments`)
- **Gap detection**: Lines break when data gap > 5 minutes (shows monitoring downtime)
- **Shaded variance areas**: Visual representation of min/max spread
- Multiple targets on same plot
- Large, readable fonts for accessibility
- Configurable detail level (50-500+ segments)

✅ **CSV Data Export**
- Flexible time range selection (`--last 24h`, `--start`/`--end`)
- Spreadsheet-compatible format
- All measurement fields included

✅ **Real-time Console Output**
- Live latency display
- Timestamps for each measurement
- Color-coded output (optional)

✅ **Alert System**
- Threshold-based alerts (latency, jitter, packet loss)
- Configurable thresholds
- Alert logging to file

---

## Quick Start

### Prerequisites

- **Rust 1.70+** (edition 2024)
- **Linux** (for ICMP - requires `CAP_NET_RAW` capability)
- **SQLite 3.x** (bundled with rusqlite)

### Installation (Recommended - Systemd Service)

```bash
# Clone repository
git clone https://github.com/schuellerf/bufferbane.git
cd bufferbane

# Build and install
sudo make install

# Create configuration
sudo mkdir -p /etc/bufferbane
sudo cp /usr/local/share/bufferbane/client.conf.template /etc/bufferbane/client.conf
sudo nano /etc/bufferbane/client.conf

# Install and start service (runs in quiet mode with hourly stats)
sudo make install-service
sudo systemctl enable --now bufferbane

# View logs
sudo journalctl -u bufferbane -f
```

### Development / Manual Build

```bash
# Clone and build
git clone https://github.com/schuellerf/bufferbane.git
cd bufferbane
cargo build --release

# Configure
cp client.conf.template client.conf
nano client.conf

# Run locally (shows every ping)
./target/release/bufferbane --config client.conf

# Run in quiet mode (hourly statistics, like systemd service)
./target/release/bufferbane --config client.conf --quiet
```

### Configuration

Key configuration options:
- **Test interval**: How often to ping (default: 1000ms)
- **Database path**: Where to store measurements (default: `./bufferbane.db`)
- **Targets**: Public DNS servers (default: 8.8.8.8, 1.1.1.1, dns.google)
- **Alert thresholds**: Latency, jitter, packet loss limits

📖 **Full installation guide**: See **[INSTALL.md](INSTALL.md)** for detailed instructions, troubleshooting, and advanced configuration.

### Usage

#### Monitoring Mode (Continuous)

```bash
# Start monitoring (Ctrl+C to stop)
./target/release/bufferbane --config client.conf
```

Output:
```
[14:23:59] 8.8.8.8 -> 18.28ms
[14:23:59] 1.1.1.1 -> 13.47ms
[14:24:00] 8.8.8.8 -> 23.21ms
[14:24:00] 1.1.1.1 -> 15.21ms
```

#### Export to CSV

```bash
# Last 24 hours (default)
./target/release/bufferbane --export --output report.csv

# Last 7 days
./target/release/bufferbane --export --last 7d --output week.csv

# Specific date range
./target/release/bufferbane --export \
  --start "2025-10-18 00:00" \
  --end "2025-10-18 23:59" \
  --output oct18.csv
```

#### Generate Chart

```bash
# Last 24 hours (default)
./target/release/bufferbane --chart --output latency.png

# Last 6 hours
./target/release/bufferbane --chart --last 6h --output tonight.png

# Specific date range
./target/release/bufferbane --chart \
  --start "2025-10-18 18:00" \
  --end "2025-10-18 22:00" \
  --output problem_evening.png
```

The chart includes:
- **Min line** (lower bound, thin)
- **Max line** (upper bound, thin)
- **Avg line** (bold, primary metric)
- **P95/P99 lines** (95th/99th percentile, dashed)
- **Shaded area** between min/max showing variance
- **Color-coded targets** with legend

---

## Use Cases

### Diagnose Evening Slowdowns

```bash
# Monitor continuously
./target/release/bufferbane &

# Next day, check the evening period
./target/release/bufferbane --chart \
  --start "2025-10-18 18:00" \
  --end "2025-10-18 23:00" \
  --output evening_issue.png
```

**Result**: Visual proof of ISP congestion during peak hours.

### Prove Connection Issues to ISP

```bash
# Collect 7 days of data
./target/release/bufferbane &

# Generate report after a week
./target/release/bufferbane --export --last 7d --output isp_complaint.csv
./target/release/bufferbane --chart --last 7d --output latency_proof.png
```

**Result**: Hard evidence of consistent latency spikes or packet loss.

### Monitor Connection Stability

```bash
# Run as a systemd service (see docs/systemd-service.example)
sudo systemctl start bufferbane
sudo systemctl enable bufferbane

# Check live stats
./target/release/bufferbane --export --last 1h --output current.csv
```

---

## Project Structure

```
bufferbane/
├── client/                    # Main client application
│   ├── src/
│   │   ├── main.rs           # Entry point and CLI
│   │   ├── config/           # Configuration management
│   │   ├── testing/          # ICMP testing logic
│   │   ├── storage/          # SQLite database
│   │   ├── analysis/         # Alert detection
│   │   ├── output/           # Console output & CSV export
│   │   └── charts/           # PNG chart generation
│   └── Cargo.toml
│
├── protocol/                  # Shared protocol library (for Phase 2+)
│   ├── src/
│   │   ├── lib.rs            # Protocol constants
│   │   ├── constants.rs      # Packet types, magic bytes
│   │   └── error.rs          # Protocol errors
│   └── Cargo.toml
│
├── docs/
│   └── planning/             # Planning documents (spec, research)
│       ├── README.md         # Planning docs index
│       ├── SPECIFICATION.md  # Technical specification
│       ├── SCENARIOS.md      # Network scenarios
│       ├── RESEARCH.md       # Tool evaluation
│       ├── PHASE_SUMMARY.md  # Implementation roadmap
│       └── ... (more planning docs)
│
├── client.conf.template      # Configuration template
├── .gitignore               # Ignore *.conf, *.db, *.log, etc.
├── Cargo.toml               # Workspace configuration
├── LICENSE                  # MIT License
└── README.md                # This file
```

---

## Implementation Status

### ✅ Phase 1: Client Only (COMPLETED - October 2025)

**Features**:
- ICMP ping to multiple targets (1-second intervals)
- SQLite database storage
- Real-time console output
- CSV export with time range selection
- **PNG chart generation** with min/max/avg/percentile visualization
- Alert detection (latency, jitter, packet loss)
- TOML configuration

**Deliverables**:
- `bufferbane` binary (standalone client)
- Configuration template
- Complete source code

### 📋 Phase 2: Client + Server (Planned)

**Goals**: Full-featured monitoring with throughput and bufferbloat detection

**Features**:
- Companion server application
- Upload/download throughput testing
- Bufferbloat detection (RRUL-style)
- Bidirectional packet loss tracking
- ChaCha20-Poly1305 encrypted protocol
- Port knocking security

**Estimated effort**: 2-3 weeks

### 📋 Phase 3: Multiple Servers (Planned)

**Goals**: Geographic diversity for routing diagnosis

**Features**:
- Test to multiple servers simultaneously
- Routing issue detection
- Server failover and redundancy

**Estimated effort**: 1-2 weeks

### 📋 Phase 4: Multi-Interface + Advanced Export (Planned)

**Goals**: WiFi vs Ethernet comparison + comprehensive reporting

**Features**:
- **Simultaneous testing** of multiple interfaces (WiFi + Ethernet)
- Real-time interface comparison
- 8 chart types (jitter, throughput, heatmaps, etc.)
- HTML/Markdown reports

**Estimated effort**: 2-3 weeks

---

## Configuration

See [`client.conf.template`](client.conf.template) for full configuration documentation.

Key sections:

### General Settings
```toml
[general]
test_interval_ms = 1000
database_path = "./bufferbane.db"
client_id = "auto"
```

### Targets
```toml
[targets]
public_dns = ["8.8.8.8", "1.1.1.1"]
custom = []  # Add your own targets
```

### Alerts
```toml
[alerts]
enabled = true
latency_threshold_ms = 100.0
jitter_threshold_ms = 50.0
packet_loss_threshold_pct = 5.0
```

### Export
```toml
[export]
enable_charts = true
chart_width = 1920
chart_height = 1080
export_directory = "./exports"
```

---

## Development

### Build from Source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check for errors without building
cargo check
```

### Project Dependencies

- **tokio** - Async runtime
- **surge-ping** - ICMP ping implementation
- **rusqlite** - SQLite database
- **plotters** - Chart generation
- **clap** - CLI argument parsing
- **toml** - Configuration parsing
- **chrono** - Time handling

See [`Cargo.toml`](Cargo.toml) for complete dependency list.

---

## Planning Documentation

Comprehensive planning documentation (~5000+ lines) is available in [`docs/planning/`](docs/planning/):

- **[SPECIFICATION.md](docs/planning/SPECIFICATION.md)** - Complete technical specification
- **[PHASE_SUMMARY.md](docs/planning/PHASE_SUMMARY.md)** - 4-phase implementation roadmap
- **[RESEARCH.md](docs/planning/RESEARCH.md)** - Tool evaluation and technology decisions
- **[SCENARIOS.md](docs/planning/SCENARIOS.md)** - Network instability scenarios

**Note**: These documents represent the planning phase and may differ slightly from the actual implementation.

---

## Troubleshooting

### ICMP Permission Denied

**Error**: `Failed to create ICMP client (CAP_NET_RAW required)`

**Solution**:
```bash
# Option 1: Set capability (persistent)
sudo setcap cap_net_raw+ep ./target/release/bufferbane

# Option 2: Run with sudo
sudo ./target/release/bufferbane
```

### Database Lock Errors

**Error**: `database is locked`

**Solution**: Only run one instance of bufferbane at a time, or use separate database files.

### Chart Generation Fails

**Error**: `Failed to create chart`

**Solution**: Ensure you have write permissions to the output directory and there's data in the database for the specified time range.

---

## Roadmap

- [x] **Phase 1**: Standalone ICMP monitoring with chart export
- [ ] **Phase 2**: Server component for throughput and bufferbloat testing
- [ ] **Phase 3**: Multiple server support for routing diagnosis
- [ ] **Phase 4**: Multi-interface monitoring (WiFi vs Ethernet)
- [ ] **Future**: Web dashboard, mobile app, integration with monitoring systems

---

## Contributing

Contributions are welcome! Please ensure:

1. Code follows Rust best practices
2. Tests pass: `cargo test`
3. Code compiles without warnings: `cargo build --release`
4. Documentation is updated for new features
5. Commit messages are descriptive

---

## License

MIT License - see [LICENSE](LICENSE) for details.

Copyright (c) 2025 Florian Schüller

---

## Author

**Florian Schüller** - [@schuellerf](https://github.com/schuellerf)

---

## Acknowledgments

- Inspired by the need for better cable internet diagnostics
- Built with Rust for performance and reliability
- Uses the excellent `surge-ping` crate for ICMP
- Chart generation powered by `plotters`

---

## Links

- **Repository**: https://github.com/schuellerf/bufferbane
- **Issues**: https://github.com/schuellerf/bufferbane/issues
- **Bufferbloat Project**: https://www.bufferbloat.net/
