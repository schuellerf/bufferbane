# Gap Handling in Charts - Update Summary

## Problem Identified

When Bufferbane wasn't running (due to system reboot, crash, manual stop, etc.), the charts would draw continuous lines connecting the last measurement before the gap to the first measurement after the gap. This created misleading visualizations that made it appear as though:

- Latency gradually changed across the downtime period
- The connection had issues during times when it wasn't even being monitored
- The monitoring was continuous when it actually had gaps

### Visual Problem (Before)

```
Latency (ms)
    30 ─
       │    ╱╲
    20 ─  ╱    ╲              ╱──╲
       │╱        ──────────  ╱    ╲
    10 ─               (misleading slope)
       │
     0 ─└─────────────────────────────────
        08:00        10:00        12:00
        
    Last measurement     First measurement
    before app stopped   after app restarted
              ↓                  ↓
              •──────────────────• ← This line is WRONG!
```

The sloped line between 09:00-11:00 suggests the connection was degrading, but actually **Bufferbane wasn't even running** during that time!

---

## Solution Implemented

### Detection Logic

The chart generation now detects gaps in measurement data:
- **Threshold**: 5 minutes (300 seconds)
- **Action**: When two consecutive measurements are more than 5 minutes apart, the line is broken

### Implementation Details

#### 1. PNG Charts (`generate_latency_chart`)

Added `split_into_segments()` helper function:
```rust
fn split_into_segments(points: &[(i64, f64)], max_gap_seconds: i64) -> Vec<Vec<(i64, f64)>>
```

- Splits time series data into continuous segments
- Each segment is drawn separately
- No lines connect across segment boundaries

#### 2. Interactive HTML Charts (`generate_interactive_chart`)

Updated JavaScript rendering code:
```javascript
const MAX_GAP_SECONDS = 300;  // 5 minutes

// Check gap between consecutive points
if (gap > MAX_GAP_SECONDS) {
    ctx.stroke();      // Finish current line
    ctx.beginPath();   // Start new line
    ctx.moveTo(x, y);  // Move without drawing
}
```

---

## Visual Result (After)

```
Latency (ms)
    30 ─
       │    ╱╲                    ╱──╲
    20 ─  ╱    ╲                ╱    ╲
       │╱        ──╲        ╱──        ╲
    10 ─              ──  ──            ──
       │               (clear gap)
     0 ─└─────────────────────────────────
        08:00        10:00        12:00
        
        ← monitored → | ← gap → | ← monitored →
```

**Clear visual indication**: The gap makes it obvious when monitoring wasn't active.

---

## Why 5 Minutes?

### Reasoning

1. **Normal test intervals**: 1 second (default)
   - Expected gap between measurements: 1-2 seconds
   - Even with network issues: < 60 seconds

2. **Short outages**: 30-60 seconds
   - Brief system hangs, network drops
   - Should still show continuous line (these are real issues)

3. **Monitoring stopped**: Typically > 5 minutes
   - System reboots: 2-10 minutes
   - Manual stops: Variable, usually > 5 minutes
   - Crashes: Detection and restart takes time

4. **Balance**:
   - **Too small** (e.g., 1 min): False gaps during legitimate network issues
   - **Too large** (e.g., 30 min): Small downtimes not visible
   - **5 minutes**: Good middle ground

### Configurable?

Currently hardcoded at 300 seconds. Could be made configurable in future:

```toml
[export]
gap_threshold_seconds = 300  # Break lines at gaps > this value
```

---

## Technical Changes

### Files Modified

1. **`client/src/charts/mod.rs`**
   - Added `split_into_segments()` function (35 lines)
   - Updated `generate_latency_chart()` to use segments
   - Updated `generate_interactive_chart()` JavaScript code

### Code Changes Summary

#### New Helper Function

```rust
/// Split time series data into continuous segments, breaking when gap > max_gap_seconds
fn split_into_segments(points: &[(i64, f64)], max_gap_seconds: i64) -> Vec<Vec<(i64, f64)>> {
    // ... implementation ...
}
```

**Logic**:
1. Iterate through sorted points
2. Compare each timestamp with previous
3. If gap > threshold, start new segment
4. Return vector of segments

#### PNG Chart Update

```rust
// OLD: Process all points as one series
let segments = vec![sorted_points];

// NEW: Split into segments
let segments = split_into_segments(&sorted_points, 300);

// Draw each segment separately
for (segment_idx, segment) in segments.iter().enumerate() {
    // ... calculate windowed stats for this segment ...
    // ... draw min/max/avg/percentile lines ...
}
```

#### HTML Chart Update

```javascript
// OLD: Always lineTo() for all points
points.forEach((point, i) => {
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);  // Always connect
});

// NEW: Check gaps and break lines
points.forEach((point, i) => {
    if (i === 0) {
        ctx.moveTo(x, y);
    } else {
        const gap = point[0] - points[i-1][0];
        if (gap > MAX_GAP_SECONDS) {
            ctx.stroke();
            ctx.beginPath();
            ctx.moveTo(x, y);  // Start new line
        } else {
            ctx.lineTo(x, y);  // Continue line
        }
    }
});
```

---

## Testing

### Test Scenarios

1. **Continuous monitoring**: No gaps
   - ✅ Chart shows unbroken lines

2. **Brief network issue** (< 5 min): 2-minute outage
   - ✅ Line continues (this is expected behavior)

3. **Monitoring stopped** (> 5 min): 20-minute gap
   - ✅ Line breaks, gap clearly visible

4. **Multiple segments**: Start-Stop-Start-Stop
   - ✅ Each continuous period shows as separate line segment

### Test Commands

```bash
# Generate PNG chart
./target/release/bufferbane --chart --last 24h --output test_gaps.png

# Generate HTML chart
./target/release/bufferbane --chart --interactive --last 24h --output test_gaps.html
```

**Results**:
- Both charts compile and generate successfully
- File sizes: PNG ~300KB, HTML ~14KB
- Visual inspection: Gaps render correctly

---

## Documentation Updates

### Files Updated

1. **`docs/INTERACTIVE_CHARTS.md`**
   - Added gap handling to "Data Points & Gap Handling" section
   - Added visual example showing gap handling
   - Explained before/after comparison

2. **`docs/USAGE.md`**
   - Added gap detection to PNG chart features
   - Added gap detection to Interactive HTML features

3. **`README.md`**
   - Added gap detection to Visual Chart Export features

---

## User-Visible Changes

### What Users Will Notice

1. **Charts now show gaps**: When Bufferbane wasn't running, lines break
2. **More accurate representation**: No misleading slopes across downtime
3. **Easier diagnosis**: Can immediately see when monitoring was active

### No Configuration Required

- Feature is automatic
- Works for both PNG and HTML charts
- No breaking changes to existing commands

### Example Use Cases

#### Use Case 1: System Reboot Identification

```
Latency (ms)
    30 ─     ╱╲
       │   ╱    ╲
    20 ─╱        ──╲     ← gap here = system was down
       │              ╱──╲
    10 ─            ╱    ──
       │
     0 ─└──────────────────────
        22:00   23:00   00:00
        
User: "Oh, the system rebooted at 23:15 for updates"
```

#### Use Case 2: Manual Testing

```
Latency (ms)
    30 ─  ╱──╲  (gap)  ╱──╲  (gap)  ╱──╲
       │╱      ──    ──    ──    ──    ╲
    20 ─
       │
    10 ─
     0 ─└──────────────────────────────
        10:00   11:00   12:00   13:00
        
User: "I ran three separate 20-minute tests"
```

#### Use Case 3: ISP Issue vs App Issue

```
Latency (ms)
    50 ─                ╱╲╱╲╱╲╱╲
       │              ╱          ╲
    30 ─            ╱              ╲
       │╱──────────                 ──────╲
    10 ─
     0 ─└────────────────────────────────────
        08:00   09:00   10:00   11:00   12:00
        
← normal → ← ISP issue (continuous line = we were monitoring) → ← normal →

If there were gaps, we couldn't be sure the issue was ISP-related!
```

---

## Future Enhancements (Possible)

### 1. Configurable Threshold

Allow users to set their own gap threshold:

```toml
[export]
gap_threshold_seconds = 300  # Default: 5 minutes
```

**Use cases**:
- Aggressive: 60 seconds (show even brief stops)
- Conservative: 600 seconds (only show major gaps)

### 2. Gap Annotations

Add text labels on charts showing gap duration:

```
Latency (ms)
    30 ─  ╱──╲         ╱──╲
       │╱      ──   ──    ╲
    20 ─        [Gap: 23 min]
       │
```

### 3. Gap Statistics

In stats panel, show:
- Number of gaps in time range
- Total gap time
- Longest gap duration

### 4. Color Coding

Different colors for different gap reasons:
- Red: Long gaps (> 1 hour)
- Yellow: Medium gaps (5-60 min)
- No break: Short gaps (< 5 min)

---

## Summary

**Problem**: Charts connected points across monitoring gaps, creating misleading visualizations.

**Solution**: Automatically detect gaps > 5 minutes and break lines at those points.

**Implementation**: 
- Added `split_into_segments()` helper function
- Updated both PNG and HTML chart generation
- ~100 lines of code changes

**Result**: Charts now accurately represent when monitoring was active vs inactive.

**User Impact**: 
- ✅ More accurate visualizations
- ✅ Easier problem diagnosis
- ✅ No configuration required
- ✅ Works for both chart types

**Status**: ✅ Implemented, tested, documented

