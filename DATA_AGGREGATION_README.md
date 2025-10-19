# Data Aggregation and Cleanup

This document describes the new data aggregation and cleanup features implemented in Bufferbane.

## Overview

Bufferbane now implements automatic data aggregation after 30 days to reduce database size while preserving long-term historical trends. Raw per-second measurements are aggregated into hourly statistics that are kept forever.

## Automatic Aggregation

### How It Works

1. **Raw Data Storage**: Measurements are stored per-second for the first 30 days
2. **Automatic Aggregation**: Daily at configured time (default: 03:00), data older than 30 days is:
   - Aggregated into hourly statistics (min, max, avg, P50, P95, P99)
   - Stored in the `aggregations_hourly` table
   - Original raw measurements are deleted after successful aggregation
3. **Forever Retention**: Hourly aggregations are kept forever

### Configuration

In `client.conf`:

```toml
[retention]
# Raw measurements are kept for 30 days
measurements_days = 30

# Hourly aggregations kept forever (0 = infinite)
aggregations_days = 0

# Events kept forever
events_days = 0

# Daily aggregation time (HH:MM format, 24h)
aggregation_time = "03:00"
```

### What is Aggregated

For each hour, the following statistics are calculated:

**Latency (RTT)**:
- Minimum RTT
- Maximum RTT
- Average RTT
- 50th percentile (median)
- 95th percentile
- 99th percentile

**Jitter**:
- Minimum jitter
- Maximum jitter
- Average jitter

**Other Metrics**:
- Packet loss percentage
- Average throughput
- Average DNS resolution time

### Monitoring

The aggregation process logs its progress:

```
[INFO] Next aggregation scheduled for: 2025-01-20 03:00:00
[INFO] Starting automatic data aggregation
[INFO] Aggregating data from 2024-12-20 to 2024-12-21
[INFO] Aggregated 24 hourly records for this period
[INFO] Deleted 86400 raw measurements
[INFO] Aggregation complete: 720 records aggregated, 2592000 raw measurements deleted
```

## Manual Cleanup

### Cleanup Command

Delete data older than a specific date:

```bash
# Delete all data (raw + aggregations) before 2024-01-01
bufferbane cleanup --before 2024-01-01

# Delete only raw measurements, keep aggregations
bufferbane cleanup --before 2024-01-01 --keep-aggregations
```

### Confirmation Prompt

The command shows what will be deleted and requires confirmation:

```
╔════════════════════════════════════════════════════╗
║           DATA CLEANUP CONFIRMATION               ║
╚════════════════════════════════════════════════════╝

  Delete all data before: 2024-01-01

  This will delete:
    • 2592000 raw measurements
    • 720 hourly aggregations

  Continue? [y/N]:
```

### Examples

**Cleanup old data to save space:**
```bash
# Keep only last year of data
bufferbane cleanup --before 2024-01-01
```

**Cleanup raw data but keep statistics:**
```bash
# Remove raw data but keep hourly aggregations
bufferbane cleanup --before 2024-06-01 --keep-aggregations
```

**Check what would be deleted (say 'N' to cancel):**
```bash
bufferbane cleanup --before 2024-01-01
# Review the counts, then type 'N' to cancel
```

## Database Schema

### New Table: aggregations_hourly

```sql
CREATE TABLE aggregations_hourly (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hour_timestamp INTEGER NOT NULL,  -- Unix timestamp of hour start
    interface TEXT NOT NULL,
    connection_type TEXT NOT NULL,
    test_type TEXT NOT NULL,
    target TEXT NOT NULL,
    server_name TEXT,
    count INTEGER NOT NULL,           -- Number of measurements
    min_rtt_ms REAL,
    max_rtt_ms REAL,
    avg_rtt_ms REAL,
    p50_rtt_ms REAL,
    p95_rtt_ms REAL,
    p99_rtt_ms REAL,
    min_jitter_ms REAL,
    max_jitter_ms REAL,
    avg_jitter_ms REAL,
    packet_loss_pct REAL,
    avg_throughput_kbps REAL,
    avg_dns_time_ms REAL,
    UNIQUE(hour_timestamp, interface, test_type, target, server_name)
);
```

## Benefits

1. **Reduced Database Size**: Aggregating 30-day-old data reduces database size by ~99%
   - 2,592,000 per-second measurements → 720 hourly aggregations (for 30 days)
   
2. **Long-term Trends**: Keep historical statistics forever without storage concerns

3. **Fast Queries**: Hourly aggregations are much faster to query than raw measurements

4. **Flexible Cleanup**: Manual control over data retention when needed

## Implementation Details

### Aggregation Process

1. Runs daily at configured time in the monitoring loop
2. Finds data older than `measurements_days` (default: 30)
3. Processes data in daily chunks to avoid memory issues
4. Calculates percentiles using sorted values
5. Uses `INSERT OR REPLACE` to handle re-aggregation
6. Deletes raw measurements after successful aggregation
7. Runs `VACUUM` to reclaim disk space

### Cleanup Process

1. Parses date from `--before` argument (YYYY-MM-DD format)
2. Counts records that would be deleted
3. Shows confirmation prompt with details
4. Deletes data if user confirms with 'y'
5. Runs `VACUUM` to optimize database
6. Reports results

## Backwards Compatibility

All existing commands continue to work:

```bash
# Old-style flags still work
bufferbane --export --last 24h --output data.csv
bufferbane --chart --last 7d --interactive --output chart.html

# New subcommand style (recommended)
bufferbane export --last 24h --output data.csv
bufferbane chart --last 7d --interactive --output chart.html
bufferbane monitor --quiet
bufferbane cleanup --before 2024-01-01
```

## Notes

- Chart generation will automatically use aggregated data for old time ranges (future enhancement)
- Aggregation is designed to be safe: data is only deleted after successful aggregation
- Multiple aggregation runs on the same data are safe (uses INSERT OR REPLACE)
- Database is automatically optimized (VACUUM) after cleanup operations

