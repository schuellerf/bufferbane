# Phase 2.2 Complete: One-Way Latency Visualization

## ✅ Implemented

### **Visual Breakdown for Server Targets**

For server-based tests (like `schueller.pro`), the charts now show **3 separate lines**:

1. **↑ Upload Latency** (lighter color, 50% opacity)
   - Time for packet to travel: Client → Server
   - Identifies upload/WiFi send issues

2. **↓ Download Latency** (medium color, 70% opacity) 
   - Time for packet to travel: Server → Client
   - Identifies download/WiFi receive issues

3. **RTT (Round Trip Time)** (full color, 100% opacity, thicker)
   - Total time: upload + download + server processing
   - Overall connection quality

For ICMP targets (like `1.1.1.1`, `8.8.8.8`), the charts show:
- **ICMP RTT** (single line, full color)
   - Standard ping round-trip time

---

## 📊 How It Looks

### Example Chart Legend:
```
━━━ schueller.pro ↑ Upload    (Light Red, thin, 50% opacity)
━━━ schueller.pro ↓ Download  (Medium Red, thin, 70% opacity)
━━━ schueller.pro RTT         (Full Red, thick, 100% opacity)
━━━ 1.1.1.1 ICMP              (Blue, medium)
━━━ 8.8.8.8 ICMP              (Green, medium)
```

### What You Can See:
- **Asymmetric issues**: If upload is consistently higher than download
- **WiFi problems**: If both upload/download are high (but ICMP is low)
- **ISP issues**: If ICMP is also high (everyone affected)
- **Server issues**: If processing time is significant

---

## 🚀 How to View

### Step 1: Rebuild and Deploy Server

The protocol changed (added timestamp field), so you need to redeploy the server:

```bash
# Build static server binary
source "$HOME/.cargo/env" && make build-server-static

# Deploy to Debian server
scp target/x86_64-unknown-linux-musl/release/bufferbane-server user@schueller.pro:/opt/bufferbane/

# Restart server (on Debian)
ssh user@schueller.pro
cd /opt/bufferbane
pkill bufferbane-server  # Stop old server
./bufferbane-server --config server.conf &  # Start new server
```

### Step 2: Run Client

```bash
# Start collecting new data
./target/release/bufferbane

# Let it run for 5-10 minutes to collect data with the new fields
```

### Step 3: Generate Chart

```bash
# Generate chart for last 10 minutes
./target/release/bufferbane --chart --interactive --last 10m

# Open in browser
firefox latency_*.html
# or
xdg-open latency_*.html
```

---

## 🔍 What to Look For

### Scenario 1: Upload Issues (Your Suspicion)
If you see:
- **Upload line** consistently higher than download (e.g., 15ms vs 5ms)
- **Spikes in upload** line while download stays stable

**Diagnosis**: WiFi upload saturation, interference, or weak signal

**Action**: Check WiFi channel congestion, move closer to router, or test with ethernet

### Scenario 2: Download Issues
If you see:
- **Download line** consistently higher than upload
- **Spikes in download** line

**Diagnosis**: WiFi download problems or ISP download throttling

### Scenario 3: Symmetric Issues  
If you see:
- **Both upload and download** high
- **ICMP to 1.1.1.1 also high**

**Diagnosis**: ISP/Internet connection problem (not WiFi-specific)

### Scenario 4: Normal Operation
If you see:
- **Upload ≈ Download** (both ~5-10ms)
- **RTT ≈ upload + download** (±1-2ms)
- **Low jitter** (lines are smooth)

**Diagnosis**: Everything working well!

---

## 📈 Chart Features

### Interactive Features:
- **Hover** over any point to see:
  - Exact timestamp
  - Window statistics (min, max, avg, P95, P99)
  - Sample count in that window
- **Statistics Panel** (bottom):
  - Overall weighted averages across entire time range
  - Total samples collected
- **Responsive**: Adjusts to browser window size

### Visual Features:
- **Line breaks** for data gaps >5 minutes (shows when client wasn't running)
- **Color coding**: Each target gets unique color family
- **Transparency**: Upload/download lines are semi-transparent for easy comparison
- **Legend**: Shows all series with their colors

---

## 💾 Data Collection Status

### Old Data (Before Phase 2.1):
- ❌ No upload/download breakdown
- ❌ Columns are NULL in database
- ✅ Still shows RTT (works normally)

### New Data (After Phase 2.1 & Server Update):
- ✅ Full timing breakdown
- ✅ Upload, download, processing times stored
- ✅ Chart shows all 3 lines for server targets

**Important**: You need to run the client with the **new server** to collect the enhanced data. Old measurements won't have the breakdown.

---

## 🔬 Verify Data Collection

Check if new data is being collected:

```bash
sqlite3 bufferbane.db "
SELECT 
  datetime(timestamp, 'unixepoch', 'localtime') as time,
  target,
  round(rtt_ms, 2) as rtt,
  round(upload_latency_ms, 2) as upload,
  round(download_latency_ms, 2) as download,
  server_processing_us as proc_us
FROM measurements 
WHERE test_type='server_echo' 
  AND timestamp > strftime('%s', 'now', '-10 minutes')
ORDER BY timestamp DESC 
LIMIT 10;
"
```

Expected output:
```
2025-10-18 23:00:15|schueller.pro|12.34|6.12|6.15|75
2025-10-18 23:00:14|schueller.pro|11.89|5.98|5.84|68
2025-10-18 23:00:13|schueller.pro|13.45|7.23|6.15|72
```

If `upload`, `download`, `proc_us` are NULL, the server hasn't been updated yet.

---

## 📚 Technical Details

### Color Scheme:
- **Base color** per target (from palette)
- **Upload**: base_color + 50% transparency (`#FF6B6B80`)
- **Download**: base_color + 70% transparency (`#FF6B6BB0`)
- **RTT**: base_color + 100% opacity (`#FF6B6B`)

### Data Format:
Each window in the chart contains:
- `window_start`: Start timestamp (seconds)
- `window_end`: End timestamp (seconds)
- `count`: Number of samples in window
- `min`: Minimum latency in window
- `max`: Maximum latency in window
- `avg`: Average latency (main line)
- `p95`: 95th percentile
- `p99`: 99th percentile

### Line Rendering:
- Upload/Download: 2px lines, dashed (semi-transparent)
- RTT: 3px line, solid (full opacity)
- ICMP: 2px line, solid (full opacity)

---

## 🐛 Troubleshooting

### Chart shows only 2 lines for server target
- **Cause**: Old measurements without upload/download data
- **Fix**: Generate chart for recent time period: `--last 5m`

### All lines are NULL/missing
- **Cause**: Server not updated with new protocol
- **Fix**: Redeploy server binary (see Step 1 above)

### Colors look wrong
- **Cause**: Browser cached old version
- **Fix**: Hard refresh (Ctrl+Shift+R) or delete old HTML files

### Chart is empty
- **Cause**: No data collected yet
- **Fix**: Run client for 5-10 minutes, then generate chart

---

## 📝 What's Next?

Phase 2.2 is **complete**! The remaining enhancements are:

### Phase 2.3: Anomaly Detection (~3-4 hours)
- Auto-detect spikes, outages, degradation
- Mark on charts automatically

### Phase 2.4: Advanced Visualization (~4-6 hours)
- Packet loss track (separate subplot)
- Alert timeline track
- Anomaly markers and shading
- Zoom/pan controls

**Recommendation**: Test Phase 2.2 first! Run for a few hours and see if the visualization gives you the insights you need. Then decide if anomaly detection would add value.

---

## 🎯 Quick Test

**5-minute verification**:

```bash
# 1. Update server
make build-server-static
scp target/x86_64-unknown-linux-musl/release/bufferbane-server user@schueller.pro:/opt/bufferbane/
ssh user@schueller.pro "cd /opt/bufferbane && pkill bufferbane-server && ./bufferbane-server --config server.conf &"

# 2. Run client for 5 minutes
timeout 5m ./target/release/bufferbane

# 3. Generate chart
./target/release/bufferbane --chart --interactive --last 5m

# 4. Open and analyze
firefox latency_*.html
```

Look at the `schueller.pro` lines - can you see upload vs download differences? 🔍

---

**Status**: ✅ Phase 2.2 Complete - Ready to visualize asymmetric latencies!

