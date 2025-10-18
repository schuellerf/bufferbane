# Changelog

All notable changes to Bufferbane will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Phase 2: Server component for throughput and bufferbloat testing
- Phase 3: Multiple server support
- Phase 4: Multi-interface monitoring (WiFi vs Ethernet)

## [0.1.0] - 2025-10-18

### Added - Phase 1 Complete! 🎉

**Core Features**:
- ICMP ping monitoring with 1-second intervals
- Multiple target support (public DNS + custom targets)
- SQLite database for historical data storage
- Real-time console output with timestamps
- Alert system with configurable thresholds (latency, jitter, packet loss)

**Export Capabilities**:
- CSV export with flexible time range selection
- **PNG chart generation** with statistical visualization:
  - Min/max lines (thin, showing bounds)
  - Average line (bold, primary metric)
  - P95/P99 percentile lines (dashed)
  - Shaded variance area between min/max
  - Multiple targets on same plot with legend
  - Configurable resolution and style

**Configuration**:
- TOML-based configuration system
- Auto-generated client ID
- Comprehensive template with documentation
- Configuration validation

**CLI**:
- Monitoring mode (continuous)
- Export mode (`--export --last 24h --output data.csv`)
- Chart mode (`--chart --last 24h --output latency.png`)
- Time range selection (relative: `--last 7d`, absolute: `--start/--end`)

**Documentation**:
- Complete README with usage examples
- Detailed usage guide (docs/USAGE.md)
- Planning documentation (docs/planning/)
- Systemd service example
- MIT License

**Technical**:
- Rust 2024 edition
- Zero-warning compilation
- Efficient async runtime (Tokio)
- Microsecond-precision timestamps
- Proper error handling with anyhow/thiserror
- Database indexing for fast queries

### Infrastructure

**Project Structure**:
- Cargo workspace with `protocol` and `client` crates
- Modular architecture (config, testing, storage, analysis, output, charts)
- Planning documents moved to `docs/planning/`
- Clear separation between code and specs

**Dependencies**:
- surge-ping 0.8 - ICMP ping implementation
- rusqlite 0.30 - SQLite database
- plotters 0.3 - Chart generation
- tokio 1.35 - Async runtime
- clap 4.4 - CLI parsing
- chrono 0.4 - Time handling

### Author
- Florian Schüller (@schuellerf)

---

## Phase Roadmap

- [x] **Phase 1** (v0.1.0): Standalone ICMP monitoring with chart export - **COMPLETED**
- [ ] **Phase 2** (v0.2.0): Server component for throughput and bufferbloat testing
- [ ] **Phase 3** (v0.3.0): Multiple server support for routing diagnosis
- [ ] **Phase 4** (v0.4.0): Multi-interface monitoring (WiFi vs Ethernet simultaneous testing)

---

## Legend

- `Added` - New features
- `Changed` - Changes in existing functionality
- `Deprecated` - Soon-to-be removed features
- `Removed` - Removed features
- `Fixed` - Bug fixes
- `Security` - Vulnerability fixes

