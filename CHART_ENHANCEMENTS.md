# Chart Enhancements - Phase 2+ Improvements

## Current Status (Phase 2)

✅ **Implemented:**
- Charts show both ICMP and server-based (`server_echo`) measurements
- Basic RTT (Round Trip Time) visualization
- Min/Max/Avg/P95/P99 statistics per time window
- Gap detection (no data periods shown as breaks)
- Interactive tooltips with detailed statistics

## Requested Enhancements

### 1. Enhanced Server-Based Statistics

**Problem:** Current server tests only show RTT, but the protocol supports much more detailed timing information.

**Solution:** Store and visualize one-way latencies and distinguish upload vs download:

#### Database Schema Enhancement
```sql
-- Add new columns to measurements table
ALTER TABLE measurements ADD COLUMN upload_latency_ms REAL;
ALTER TABLE measurements ADD COLUMN download_latency_ms REAL;
ALTER TABLE measurements ADD COLUMN server_processing_us INTEGER;
ALTER TABLE measurements ADD COLUMN client_to_server_ms REAL;
ALTER TABLE measurements ADD COLUMN server_to_client_ms REAL;
```

#### Protocol Enhancement
The ECHO_REPLY already includes timestamps:
- `client_send_timestamp` - When client sent request
- `server_recv_timestamp` - When server received request  
- `server_send_timestamp` - When server sent reply
- `client_recv_timestamp` - When client received reply

**Calculations:**
```
upload_latency = server_recv_timestamp - client_send_timestamp
download_latency = client_recv_timestamp - server_send_timestamp
server_processing = server_send_timestamp - server_recv_timestamp
rtt = client_recv_timestamp - client_send_timestamp
```

**Benefits:**
- Identify if issues are upload-specific (WiFi → Internet)
- Identify if issues are download-specific (Internet → WiFi)
- Separate network latency from server processing time
- More accurate diagnosis for asymmetric connections

#### Chart Visualization
For server targets, show **three lines**:
- 🔵 **Upload Latency** (client → server)
- 🟢 **Download Latency** (server → client)
- 🔴 **Total RTT** (upload + download + processing)

For ICMP targets, show **one line**:
- 🟠 **ICMP RTT**

---

### 2. Packet Loss Visualization

**Problem:** Packet loss events are stored but not visualized in charts.

**Solution:** Add visual indicators for packet loss:

#### Data Collection
Already tracked in `measurements` table:
- `status = 'timeout'` → Packet lost
- `status = 'error'` → Test failed
- `packet_loss_pct` → Loss percentage (for bulk tests)

#### Chart Enhancement

**Option A: Scatter Plot Overlay**
- Add red dots/markers at timestamps where packets were lost
- Size of marker = severity (single loss vs multiple)
- Hover tooltip: "Packet loss at 22:15:30 (3 consecutive packets)"

**Option B: Background Shading**
- Shade time periods with packet loss in light red
- Intensity = loss percentage
- Useful for showing "problem periods"

**Option C: Separate Track** (Recommended)
- Add a separate track below the latency chart
- Show packet loss as a bar chart (0-100% scale)
- Color-coded: Green (0%), Yellow (1-5%), Orange (5-10%), Red (>10%)

**Implementation Priority:** Option C (most informative)

---

### 3. Alert Visualization

**Problem:** Alerts are logged but not shown in charts.

**Solution:** Mark alert events on the timeline:

#### Alert Types to Visualize
1. **High Latency Alert** - Threshold exceeded (e.g., >100ms)
2. **High Jitter Alert** - Unstable connection (e.g., stddev >50ms)
3. **Packet Loss Alert** - Drops detected (e.g., >5%)
4. **Connection Timeout** - Complete failure
5. **Server Unreachable** - Server down or network issue

#### Chart Enhancement

**Visual Design:**
- **Vertical lines** at alert timestamps
- Color-coded by severity:
  - 🟡 Yellow = Warning (threshold approaching)
  - 🟠 Orange = Alert (threshold exceeded)
  - 🔴 Red = Critical (severe/prolonged issue)
- **Icons** at top of chart:
  - ⚠️ Warning
  - 🚨 Alert
  - ❌ Critical
- **Hover tooltip:** "High Latency Alert: 156ms at 22:45:12 (threshold: 100ms)"

**Alert Timeline Track:**
- Separate track below packet loss track
- Shows alert events as colored bars
- Duration = how long issue persisted
- Clicking alert shows detailed context (before/after measurements)

---

### 4. Anomaly Detection Visualization

**Problem:** Anomalies (unusual patterns) are not automatically detected or highlighted.

**Solution:** Implement basic anomaly detection and visualization:

#### Anomaly Types

1. **Latency Spikes**
   - Definition: RTT > 3× moving average
   - Visualization: Highlight data point with 🔺 marker
   - Color: Orange

2. **Micro-Outages**
   - Definition: 3+ consecutive timeouts in <10 seconds
   - Visualization: Red vertical band
   - Label: "Micro-outage: 5 seconds"

3. **Degradation Events**
   - Definition: Sustained increase (e.g., avg increases by 50% for >1 minute)
   - Visualization: Yellow background shading for duration
   - Label: "Degradation: +75% latency for 90 seconds"

4. **Jitter Events**
   - Definition: High variance in short window (stddev > 50ms over 10 seconds)
   - Visualization: Wavy line indicator or hatch pattern
   - Label: "High jitter: 78ms stddev"

5. **Clock Drift**
   - Definition: Timestamps out of order or gaps not explained by timeouts
   - Visualization: ⏰ icon
   - Label: "Clock issue detected"

#### Implementation
```rust
struct Anomaly {
    timestamp: i64,
    anomaly_type: AnomalyType,
    severity: Severity,
    description: String,
    duration_sec: Option<u64>,
    affected_measurements: Vec<i64>,
}

enum AnomalyType {
    LatencySpike,
    MicroOutage,
    DegradationEvent,
    HighJitter,
    ClockDrift,
}

enum Severity {
    Info,      // Unusual but not problematic
    Warning,   // Potentially problematic
    Alert,     // Definitely problematic
    Critical,  // Severe issue
}
```

---

### 5. Multi-Target Comparison

**Problem:** When monitoring multiple targets (e.g., 1.1.1.1, 8.8.8.8, schueller.pro), hard to compare them.

**Solution:** Enhanced comparison view:

#### Chart Modes

**Mode 1: Separate Targets (Current)**
- Each target gets its own colored line
- Good for overall view
- Can be cluttered with many targets

**Mode 2: Difference View**
- Show difference between targets
- Example: `schueller.pro latency - 1.1.1.1 latency`
- Identifies when specific target degrades vs baseline
- Useful for: "Is this my ISP or the server?"

**Mode 3: Stacked View**
- Separate subplot for each target
- Vertically stacked
- Same time axis, easier to see per-target patterns
- Good for detailed analysis

**Mode 4: Correlation View**
- Scatter plot: X=baseline (1.1.1.1), Y=target (schueller.pro)
- Shows if latencies are correlated (network issue) or independent (server issue)
- 45° line = perfect correlation

---

## Implementation Roadmap

### Phase 2.1 (Quick Wins - ~2-4 hours)
✅ Show server data in charts (DONE)
- [ ] Add packet loss track to charts
- [ ] Add alert markers to charts
- [ ] Store one-way latencies in database

### Phase 2.2 (Enhanced Server Stats - ~4-6 hours)
- [ ] Calculate upload/download latencies from timestamps
- [ ] Visualize separate upload/download lines for server targets
- [ ] Add server processing time to statistics
- [ ] Update tooltips with detailed timing breakdown

### Phase 2.3 (Anomaly Detection - ~6-8 hours)
- [ ] Implement latency spike detection
- [ ] Implement micro-outage detection
- [ ] Implement degradation event detection
- [ ] Implement high jitter detection
- [ ] Visualize anomalies with markers/shading

### Phase 2.4 (Advanced Visualization - ~4-6 hours)
- [ ] Add packet loss track
- [ ] Add alert timeline track
- [ ] Implement chart mode selector (separate/difference/stacked/correlation)
- [ ] Add zoom/pan controls
- [ ] Add time range selector

### Phase 3+ (Future)
- [ ] Real-time chart updates (WebSocket)
- [ ] Export anomaly report (PDF/JSON)
- [ ] Machine learning anomaly detection
- [ ] Predictive alerts ("latency increasing, alert in 5 minutes")
- [ ] Correlation with external events (ISP outages, weather, etc.)

---

## Configuration

Add to `client.conf`:

```toml
[charts]
# Chart enhancements
show_packet_loss = true        # Show packet loss track
show_alerts = true             # Mark alerts on timeline
show_anomalies = true          # Highlight detected anomalies
detect_anomalies = true        # Enable automatic anomaly detection

# Anomaly detection thresholds
anomaly_spike_multiplier = 3.0 # Spike if > 3x moving average
anomaly_jitter_threshold_ms = 50.0
anomaly_degradation_percent = 50.0
anomaly_degradation_duration_sec = 60

# Chart modes
default_chart_mode = "separate"  # separate, difference, stacked, correlation
show_one_way_latencies = true   # Show upload/download for server tests
```

---

## Example: Enhanced Chart Output

```
┌─────────────────────────────────────────────────────────────────────┐
│ Latency Chart - Last 24 Hours                           [Zoom] [⚙️] │
├─────────────────────────────────────────────────────────────────────┤
│                                        🚨                           │
│ 100ms ┤     ╭─╮           ⚠️     ╭──╮    │                         │
│       ├╮   ╭╯ ╰╮        ╭─╮   ╭─╯  ╰╮   │                         │
│ 50ms  ├╯───╯   ╰────────╯ ╰───╯     ╰───┤ 1.1.1.1 (ICMP)          │
│       │                                  │ 8.8.8.8 (ICMP)          │
│ 0ms   └──────────────────────────────────┤ schueller.pro (Server)  │
│        00:00   06:00   12:00   18:00     │                         │
├─────────────────────────────────────────────────────────────────────┤
│ Packet Loss                                                         │
│ 10%   ┤       ██                     ██  │                         │
│ 0%    └──────────────────────────────────┤                         │
├─────────────────────────────────────────────────────────────────────┤
│ Alerts                                                              │
│       │  ⚠️    🚨        ⚠️           🚨  │                         │
│       └──────────────────────────────────┤                         │
└─────────────────────────────────────────────────────────────────────┘

Legend:
  📶 Normal operation    ⚠️ Warning    🚨 Alert    🔺 Spike    ▓ Degradation
```

---

## Testing Plan

1. **Test packet loss visualization**
   - Unplug ethernet for 5 seconds
   - Verify chart shows red band with "Micro-outage: 5s"

2. **Test alert markers**
   - Set low latency threshold (e.g., 20ms)
   - Verify alerts appear on chart

3. **Test one-way latencies**
   - Compare upload vs download to server
   - Verify asymmetry is visible

4. **Test anomaly detection**
   - Simulate traffic spike (e.g., download large file)
   - Verify latency spike is highlighted

---

## Benefits Summary

| Enhancement | Benefit | User Value |
|-------------|---------|------------|
| One-way latencies | Identify asymmetric issues | "My upload is slow, but download is fine" |
| Packet loss track | See loss patterns over time | "Loss happens every evening at 6pm" |
| Alert markers | Quick problem identification | "What happened at 3am?" |
| Anomaly detection | Automatic issue discovery | "Didn't notice the micro-outages" |
| Multi-target comparison | Identify issue source | "Is it my ISP or the server?" |

---

**Status:** Specification complete, ready for implementation prioritization.
**Next Steps:** Implement Phase 2.1 quick wins, then evaluate user feedback before Phase 2.2.

