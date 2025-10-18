# RRD vs SQLite Analysis for Bufferbane

Quick evaluation of Round-Robin Database (RRD) vs SQLite for Bufferbane network monitoring.

## Round-Robin Database (RRD)

### What is RRD?
- Fixed-size database that stores time-series data
- Used by Smokeping, Cacti, Munin, Collectd
- Data structure: circular buffer with automatic aggregation
- File size never grows beyond initial allocation

### Advantages
✅ **Fixed disk usage**: Database size is predictable and constant  
✅ **Automatic aggregation**: Built-in downsampling (1s → 1m → 1h)  
✅ **Optimized for time-series**: Fast writes for regular-interval data  
✅ **Built-in retention**: Old data automatically overwritten  
✅ **Well-tested**: Decades of production use

### Disadvantages
❌ **Fixed schema**: Must define all metrics upfront, hard to change  
❌ **No flexible queries**: Can't do "SELECT * WHERE condition"  
❌ **No arbitrary time ranges**: Fixed retention periods only  
❌ **Can't store events**: Only numeric time-series data  
❌ **No full resolution forever**: Old data is aggregated/lost  
❌ **Limited export**: Harder to export raw data to CSV  
❌ **Complexity**: Requires understanding RRA (Round-Robin Archive) configuration

### Code Complexity
**RRD crate**: `rrd` (not very active) or call `rrdtool` binary

```rust
// Creating RRD database
rrdtool create bufferbane.rrd \
    --step 1 \
    DS:latency:GAUGE:2:0:10000 \
    DS:loss:GAUGE:2:0:100 \
    RRA:AVERAGE:0.5:1:86400 \      # 1 day at 1s resolution
    RRA:AVERAGE:0.5:60:43200 \     # 30 days at 1min resolution
    RRA:AVERAGE:0.5:3600:8760      # 1 year at 1h resolution
```

**Problem**: Need separate storage for events (alerts, anomalies)

## SQLite

### What is SQLite?
- Embedded relational database
- Self-contained, serverless, zero-configuration
- Full SQL query support
- Most widely deployed database

### Advantages
✅ **Flexible schema**: Easy to add new metrics or change structure  
✅ **Full SQL**: Complex queries, JOINs, aggregations  
✅ **Store everything**: Measurements, events, alerts, metadata  
✅ **Arbitrary queries**: "Find all packet loss events on Tuesdays"  
✅ **Easy export**: Direct CSV/JSON export via SQL  
✅ **Well-supported**: Excellent Rust crates (rusqlite, sqlx)  
✅ **Full resolution available**: Can query raw data anytime  
✅ **Simple backup**: Just copy .db file

### Disadvantages
⚠️ **Growing database**: Size increases over time (manageable)  
⚠️ **Manual cleanup**: Need to implement retention policies  
⚠️ **Slightly slower**: Marginally slower than RRD for pure time-series

### Code Complexity
**SQLite crate**: `rusqlite` or `sqlx` (both excellent)

```rust
// Creating tables
CREATE TABLE measurements (
    timestamp INTEGER PRIMARY KEY,
    target TEXT,
    rtt_ms REAL,
    loss_pct REAL,
    ...
);

CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER,
    event_type TEXT,
    severity TEXT,
    details JSON
);
```

**Benefit**: Single database for everything

## Use Case Analysis

### Our Requirements

| Requirement | RRD | SQLite |
|-------------|-----|--------|
| Store per-second measurements | ✅ Perfect | ✅ Good |
| Store variable events (alerts) | ❌ Need 2nd storage | ✅ Same DB |
| Query specific time ranges | ⚠️ Limited | ✅ Perfect |
| Export to CSV | ⚠️ Complex | ✅ Trivial |
| Add new metrics later | ❌ Rebuild DB | ✅ ALTER TABLE |
| Keep raw data >30 days | ❌ Aggregated | ✅ Yes |
| Find patterns (e.g., "all packet loss during peak hours") | ❌ Can't do | ✅ Easy |
| Backup/restore | ⚠️ Special tools | ✅ Copy file |

### Queries We Need

**Example 1**: "Show all upload degradation events last week"
```sql
-- SQLite: Easy
SELECT * FROM events 
WHERE event_type = 'upload_degradation' 
AND timestamp > strftime('%s', 'now', '-7 days');

-- RRD: Not possible, would need separate event storage anyway
```

**Example 2**: "Export per-second data for time range to CSV"
```sql
-- SQLite: Easy
SELECT timestamp, target, rtt_ms, loss_pct 
FROM measurements 
WHERE timestamp BETWEEN ? AND ?
ORDER BY timestamp;

-- RRD: Complex, requires rrdtool fetch + parsing
```

**Example 3**: "Find all packet loss bursts (5+ consecutive)"
```sql
-- SQLite: Doable with window functions
-- RRD: Impossible
```

## Code Complexity Comparison

### RRD Implementation

**Estimated LOC**: ~800-1000 lines
- RRD database creation and configuration: 100 lines
- Writing measurements to RRD: 150 lines
- Separate event storage (SQLite anyway?): 200 lines
- Reading from RRD for queries: 200 lines
- Export functionality: 150 lines
- Aggregation handling: 100 lines

**Dependencies**: `rrd` crate or shell out to `rrdtool`

### SQLite Implementation

**Estimated LOC**: ~400-600 lines
- Database schema creation: 50 lines
- Writing measurements: 100 lines
- Writing events: 50 lines
- Queries and analysis: 150 lines
- Export functionality: 50 lines
- Cleanup/retention: 100 lines

**Dependencies**: `rusqlite` or `sqlx`

## Performance Comparison

### Write Performance
- **RRD**: Slightly faster (~10-20% better for pure time-series)
- **SQLite**: Fast enough for 1/sec writes (trivial load)

### Disk Usage (30 days of 1-second data)
- **RRD**: Fixed ~50-100 MB (depending on RRA config)
- **SQLite**: ~200-500 MB (with indexes, depends on metrics count)

### Query Performance
- **RRD**: Fast for predefined aggregations
- **SQLite**: Fast for indexed queries, flexible

## Recommendation

**Use SQLite** for these reasons:

### 1. **Simpler Implementation** (-40% code)
- Single database for everything
- No need to learn RRD concepts
- Standard SQL queries

### 2. **Much More Flexible**
- Can add metrics without rebuilding
- Complex queries for analysis
- Store events with full context
- Easy export to any format

### 3. **Better for This Use Case**
We need:
- Event storage (alerts, anomalies) → SQLite needed anyway
- Flexible queries for analysis → Only SQLite can do
- CSV export → Much easier with SQLite
- Pattern detection → Only SQLite can do

### 4. **Disk Space Not a Concern**
- 1 GB per month is acceptable
- Storage is cheap
- Can implement aggressive cleanup if needed

### 5. **Excellent Rust Support**
```toml
[dependencies]
rusqlite = "0.30"  # Synchronous, simple
# OR
sqlx = { version = "0.7", features = ["sqlite"] }  # Async, more features
```

### 6. **Code Reduction Estimate**
- RRD: ~800-1000 lines + separate event storage
- SQLite: ~400-600 lines for everything
- **Reduction**: ~40-50% less code with SQLite

## What if Disk Space Becomes an Issue?

If 1 GB/month is too much:

### Option 1: Aggressive Cleanup
```sql
-- Keep only 7 days raw, rest aggregated
DELETE FROM measurements 
WHERE timestamp < strftime('%s', 'now', '-7 days');
```

### Option 2: Compression
```bash
# SQLite with compression
sqlite3 monitor.db "VACUUM;"  # Rebuilds DB, reclaims space
```

### Option 3: Archive to Files
```rust
// Export old data to compressed files
export_to_json("2024-01.json.gz");
delete_from_db("2024-01-01", "2024-01-31");
```

### Option 4: Hybrid (if really needed)
- SQLite for events and recent data (7 days)
- Archive older data to RRD or compressed files
- Only implement if disk space proves to be a problem

## Conclusion

**Stick with SQLite**. The implementation is:
- ✅ 40-50% less code than RRD
- ✅ More flexible and powerful
- ✅ Better for our specific requirements
- ✅ Easier to maintain and extend

RRD doesn't "drastically reduce implementation length" - it actually increases it because we'd need separate event storage anyway, and adds complexity with limited benefits for our use case.

---

**Decision**: Use SQLite as specified in SPECIFICATION.md

**Rationale**: 
- Simpler implementation (~400-600 LOC vs ~800-1000 LOC)
- Single storage solution for measurements + events
- Flexible queries essential for pattern detection
- Easy CSV export for analysis
- Disk space is not a constraint

