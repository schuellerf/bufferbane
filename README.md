# Bufferbane

**High-precision network quality monitoring with bufferbloat detection**

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)

Bufferbane detects fine-grained network issues on cable internet connections (DOCSIS). It performs per-second measurements to identify latency spikes, jitter, packet loss, and bufferbloat that traditional tools miss.

> **Bufferbloat**: Excessive buffering in network equipment that causes high latency during heavy traffic. Common on cable internet, it makes real-time applications (gaming, video calls) unusable even when bandwidth is available.

---

## Features

✅ **Continuous Monitoring**
- Per-second ICMP latency tests to multiple targets
- Microsecond-precision timestamps
- Real-time console output or quiet mode (hourly statistics)

✅ **Data Collection & Analysis**
- SQLite database with efficient indexing
- Alert detection (latency, jitter, packet loss)
- Query by time range, target, or connection type

✅ **Export & Visualization**
- **Interactive HTML charts** with hover tooltips and clickable legends
- **Static PNG charts** with min/max/avg/P95/P99 lines
- **CSV export** for spreadsheet analysis
- Configurable time ranges and aggregation levels

✅ **Server Mode** (Optional - Phase 2)
- Encrypted UDP protocol for accurate measurements
- One-way latency tracking (upload vs download)
- Built-in clock synchronization
- Easy deployment with setup script

---

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/schuellerf/bufferbane.git
cd bufferbane

# Build and install as systemd service (recommended)
make install
sudo make install-service

# Or just build locally
make
```

### Configuration

```bash
# Copy and edit configuration template
cp client.conf.template client.conf
nano client.conf
```

### Run

```bash
# Run locally (verbose)
./target/release/bufferbane

# Run as systemd service (quiet mode with hourly stats)
sudo systemctl start bufferbane
sudo journalctl -u bufferbane -f
```

---

## Usage

### Monitoring

```bash
# Start continuous monitoring
./target/release/bufferbane
```

### Export Data

```bash
# Generate interactive HTML chart (last 24h)
make chart-interactive

# Generate PNG chart (last 24h)
make chart

# Export to CSV
make export
```

**Time ranges**:
```bash
# Last 6 hours
./target/release/bufferbane --chart --interactive --last 6h

# Specific date range
./target/release/bufferbane --chart --interactive \
  --start "2025-10-18 18:00" \
  --end "2025-10-18 22:00"
```

**Chart features**:
- Hover to see detailed statistics (min/max/avg/P95/P99)
- Click legend or stat panels to hide/show series
- Aggregated into 100 time windows (configurable with `--segments`)
- Line breaks at data gaps > 5 minutes

### Server Setup (Optional)

```bash
# Automated server deployment
./setup-server.sh your-server-hostname

# Or manual setup
make build-server-static
# ... copy to server, configure, run
```

See **[INSTALL.md](INSTALL.md)** for detailed installation guide.

---

## Use Cases

### Diagnose Evening Slowdowns

```bash
# Run continuously
./target/release/bufferbane &

# Next day, visualize the evening period
./target/release/bufferbane --chart --interactive \
  --start "2025-10-18 18:00" --end "2025-10-18 23:00"
```

**Result**: Visual proof of ISP congestion during peak hours.

### Prove Connection Issues to ISP

```bash
# Collect data for a week
sudo systemctl enable --now bufferbane

# Generate report
./target/release/bufferbane --export --last 7d --output isp_complaint.csv
./target/release/bufferbane --chart --last 7d --output latency_proof.png
```

**Result**: Hard evidence of consistent latency spikes or packet loss.

---

## Makefile Targets

```bash
make                    # Build client
make install            # Install binary to /usr/local/bin
make install-service    # Install and enable systemd service

make clean              # Remove build artifacts
make clean-data         # Remove generated charts, CSV, and database

make chart              # Generate PNG chart (last 24h)
make chart-interactive  # Generate HTML chart (last 24h)
make export             # Export CSV (last 24h)

make build-server       # Build server (dynamic linking)
make build-server-static # Build static server (for deployment)
make windows            # Cross-compile for Windows
```

---

## Roadmap

- [x] **Phase 1**: Standalone ICMP monitoring with chart export
- [x] **Phase 2**: Server component with encrypted communication
  - [x] Server infrastructure
  - [x] Encrypted protocol (ChaCha20-Poly1305)
  - [x] Built-in clock synchronization
  - [x] One-way latency tracking
  - [x] Setup automation
  - [ ] Active throughput/bufferbloat testing (future)
- [ ] **Phase 3**: Multiple servers for geographic testing
- [ ] **Phase 4**: Multi-interface monitoring (WiFi vs Ethernet)

---

## Documentation

- **[INSTALL.md](INSTALL.md)** - Complete installation guide with troubleshooting
- **[UBUNTU_SETUP.md](UBUNTU_SETUP.md)** - Ubuntu-specific ICMP setup (no sudo required)
- **[DATA_AGGREGATION_README.md](DATA_AGGREGATION_README.md)** - Data retention and cleanup
- **[CHANGELOG.md](CHANGELOG.md)** - Version history and changes
- **[docs/USAGE.md](docs/USAGE.md)** - Usage examples and command reference
- **[docs/planning/](docs/planning/)** - Technical specifications and research

---

## Project Structure

```
bufferbane/
├── client/              # Main application
│   ├── src/
│   │   ├── charts/      # Chart generation (PNG & HTML)
│   │   ├── config/      # Configuration management
│   │   ├── testing/     # ICMP and server testing
│   │   ├── storage/     # SQLite database
│   │   ├── analysis/    # Alert detection
│   │   └── output/      # Console & CSV export
│   └── templates/       # HTML chart template
├── server/              # Optional server component
├── protocol/            # Shared protocol library
├── docs/planning/       # Planning documents
├── Makefile            # Build automation
└── *.conf.template     # Configuration templates
```

---

## Troubleshooting

### ICMP Permission Denied

```bash
# Set capability (recommended)
sudo setcap cap_net_raw+ep ./target/release/bufferbane

# Or run with sudo
sudo ./target/release/bufferbane
```

### Database Locked

Only run one instance at a time, or use separate database files via configuration.

### No Data in Charts

Ensure bufferbane has been running and collecting data for the specified time range.

---

## Contributing

Contributions welcome! Please ensure:
1. Code follows Rust best practices
2. Tests pass: `cargo test`
3. Builds without warnings: `cargo build --release`
4. Documentation is updated

---

## License

MIT License - see [LICENSE](LICENSE)

Copyright (c) 2025 Florian Schüller ([@schuellerf](https://github.com/schuellerf))

---

## Links

- **Repository**: https://github.com/schuellerf/bufferbane
- **Issues**: https://github.com/schuellerf/bufferbane/issues
- **Bufferbloat Info**: https://www.bufferbloat.net/
