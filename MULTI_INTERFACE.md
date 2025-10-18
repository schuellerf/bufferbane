# Multi-Interface Monitoring - Phase 4

## Overview

Phase 4 of Bufferbane adds the ability to **simultaneously monitor multiple network interfaces** on the same machine. This enables real-time comparison of WiFi vs Ethernet connection quality, definitively isolating WiFi issues from ISP problems.

## The Problem

**Question**: "Is my poor connection due to WiFi or my ISP?"

**Traditional approach** (sequential testing):
1. Test on WiFi for a week
2. Switch to Ethernet, test for another week
3. Compare results

**Problems**:
- ❌ Two weeks of testing needed
- ❌ Network conditions change week-to-week
- ❌ Can't prove WiFi was the issue (maybe ISP improved?)

**Bufferbane Phase 4** (simultaneous testing):
1. Test WiFi **and** Ethernet at the same time
2. Same ISP conditions for both
3. Immediate, definitive comparison
4. Answer in minutes, not weeks

## How It Works

### Architecture

```
┌─────────────────────────────────────────────────────┐
│  Linux Machine Running Bufferbane Client             │
│                                                       │
│  ┌─────────────────┐      ┌─────────────────┐      │
│  │ Interface wlan0 │      │ Interface eth0  │      │
│  │    (WiFi)       │      │   (Ethernet)    │      │
│  └────────┬────────┘      └────────┬────────┘      │
│           │                         │                │
│           │  Tests every 1 second   │                │
│           │  to same targets        │                │
│           ▼                         ▼                │
│  ┌──────────────────────────────────────────┐      │
│  │         SQLite Database                   │      │
│  │  Stores: interface, connection_type, ... │      │
│  └──────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────┘
           │                         │
           │        Router           │
           └───────────┬─────────────┘
                       │
                       ▼
                  ISP Network
                       │
                       ▼
               Server / Targets
```

### Interface Binding (Linux)

**Technical implementation**:
```rust
use nix::sys::socket::{setsockopt, sockopt::BindToDevice};

// Bind socket to specific interface
let socket = /* create socket */;
setsockopt(socket, BindToDevice, &"wlan0".as_bytes())?;

// Now all traffic from this socket uses wlan0 exclusively
```

**Why Linux-only?**
- `SO_BINDTODEVICE` is a Linux-specific socket option
- Windows and macOS have different/limited approaches
- Linux is the primary target for Bufferbane anyway

### Database Schema

```sql
CREATE TABLE measurements (
    timestamp INTEGER,
    interface TEXT NOT NULL,       -- "wlan0", "eth0"
    connection_type TEXT NOT NULL, -- "wifi", "wired"
    target TEXT,
    rtt_ms REAL,
    jitter_ms REAL,
    packet_loss_pct REAL,
    throughput_kbps REAL,
    -- ... other fields
);

-- Query: Compare WiFi vs Ethernet
SELECT 
    interface,
    connection_type,
    AVG(rtt_ms) as avg_latency,
    AVG(jitter_ms) as avg_jitter,
    AVG(packet_loss_pct) as avg_loss
FROM measurements
WHERE timestamp > strftime('%s', 'now', '-1 hour')
  AND target = '8.8.8.8'
GROUP BY interface;
```

## Configuration

### Automatic (Recommended)

```toml
[general]
# Test both WiFi and Ethernet simultaneously
interfaces = ["wlan0", "eth0"]

# Connection types auto-detected from interface names:
# - wlan* → "wifi"
# - eth*, enp* → "wired"
# - ww* → "cellular"
```

### Manual Override

```toml
[general]
# Explicitly specify interface types
[[interfaces]]
name = "wlan0"
connection_type = "wifi"

[[interfaces]]
name = "eth0"
connection_type = "wired"
```

### Single Interface (Phase 1-3 compatibility)

```toml
[general]
# Empty = use default interface
interfaces = []

# Manual tag for single interface
connection_type = "wifi"
```

## Use Cases

### Use Case 1: Diagnose WiFi Issues

**Scenario**: User suspects WiFi is causing lag in video calls

**Setup**:
```toml
interfaces = ["wlan0", "eth0"]
```

**Result after 1 hour**:
```
Interface Comparison (8.8.8.8):
─────────────────────────────────────────
            WiFi (wlan0)   Wired (eth0)
Avg Latency    28 ms         15 ms      ← WiFi adds 13ms
Avg Jitter     8.5 ms        0.8 ms     ← WiFi unstable
Packet Loss    2.3%          0.1%       ← WiFi drops packets
Upload Speed   3.2 Mbps      5.0 Mbps   ← WiFi throttled

Verdict: WiFi is the problem!
         - Switch to Ethernet or improve WiFi setup
```

### Use Case 2: A/B Test WiFi Changes

**Scenario**: User upgraded WiFi router, wants to verify improvement

**Setup**:
- Old router: Test for 1 day
- New router: Test for 1 day
- Compare WiFi performance (same wlan0 interface)

### Use Case 3: Monitor Failover Connection

**Scenario**: User has Ethernet primary + cellular backup

**Setup**:
```toml
interfaces = ["eth0", "wwan0"]
```

**Result**: Monitor both connections 24/7, know when failover is needed

### Use Case 4: Diagnose "Evening Lag"

**Scenario**: Connection gets bad at 7 PM every night. WiFi or ISP?

**Test**:
- Monitor both WiFi and Ethernet during evening
- If both degrade: ISP congestion
- If only WiFi degrades: Neighbor WiFi interference

## Export & Visualization

### Chart: Interface Comparison

```bash
bufferbane charts --last 24h --output-dir ./comparison
```

**Generates**:
```
connection_comparison.png:
┌─────────────────────────────────────────┐
│ WiFi vs Ethernet Comparison (24h avg)   │
├─────────────────────────────────────────┤
│                                          │
│ Latency (ms)                             │
│   WiFi    ████████████░ 28ms             │
│   Wired   ████░ 15ms                     │
│                                          │
│ Jitter (ms)                              │
│   WiFi    ████████░ 8.5ms                │
│   Wired   ░ 0.8ms                        │
│                                          │
│ Packet Loss (%)                          │
│   WiFi    ██░ 2.3%                       │
│   Wired   ░ 0.1%                         │
│                                          │
│ Upload (Mbps)                            │
│   WiFi    ████████░ 3.2                  │
│   Wired   ████████████░ 5.0              │
└─────────────────────────────────────────┘
```

### Chart: Time Series Comparison

```bash
bufferbane chart latency_over_time --interfaces all --last 6h
```

**Generates**:
```
latency_over_time.png:

Latency (ms)
60 ┤                                    
50 ┤  ╭╮                  ╭──╮          WiFi (wlan0)
40 ┤  │╰╮               ╭─╯  ╰─╮        
30 ┤╭─╯ ╰─╮           ╭─╯      ╰─╮     
20 ┤│     ╰───────────╯          ╰──   
10 ┤╰────────────────────────────────   Wired (eth0)
 0 └┬────┬────┬────┬────┬────┬────┬
   12:00  14:00  16:00  18:00  20:00
   
Clear pattern: WiFi latency spikes in evening (interference)
               Ethernet remains stable
```

### CSV Export

```csv
timestamp,interface,connection_type,target,rtt_ms,jitter_ms,loss_pct
1697654321,wlan0,wifi,8.8.8.8,28.5,8.2,2.1
1697654321,eth0,wired,8.8.8.8,15.2,0.7,0.0
1697654322,wlan0,wifi,8.8.8.8,29.1,8.5,2.3
1697654322,eth0,wired,8.8.8.8,15.3,0.8,0.0
...
```

## Implementation Details

### Test Orchestration

```rust
// Pseudo-code for multi-interface testing

async fn run_multi_interface_monitoring(config: Config) {
    let mut interface_handles = Vec::new();
    
    for interface_config in config.interfaces {
        let handle = tokio::spawn(async move {
            let monitor = InterfaceMonitor::new(
                interface_config.name,
                interface_config.connection_type,
            );
            
            // Bind all sockets to this specific interface
            monitor.bind_to_interface()?;
            
            // Run monitoring loop
            loop {
                let measurements = monitor.run_tests(&config.targets).await?;
                
                // Tag measurements with interface info
                for mut m in measurements {
                    m.interface = interface_config.name.clone();
                    m.connection_type = interface_config.connection_type.clone();
                    database.store(m).await?;
                }
                
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
        
        interface_handles.push(handle);
    }
    
    // Wait for all interfaces (runs in parallel)
    futures::future::join_all(interface_handles).await;
}
```

### Interface Type Detection

```rust
fn detect_connection_type(interface_name: &str) -> String {
    if interface_name.starts_with("wlan") || interface_name.starts_with("wlp") {
        "wifi".to_string()
    } else if interface_name.starts_with("eth") || interface_name.starts_with("enp") {
        "wired".to_string()
    } else if interface_name.starts_with("ww") || interface_name.starts_with("usb") {
        "cellular".to_string()
    } else if interface_name.starts_with("tun") || interface_name.starts_with("tap") {
        "vpn".to_string()
    } else {
        "unknown".to_string()
    }
}
```

## Bandwidth Considerations

**Question**: Won't testing multiple interfaces consume too much bandwidth?

**Answer**: No, tests are lightweight:

```
Single interface bandwidth:
- ICMP ping: ~100 bytes/second
- UDP echo: ~200 bytes/second
- Small throughput test (100KB): ~100 KB / 10 seconds = 10 KB/s average
- Total: ~10-15 KB/s per interface

Two interfaces:
- Total: ~20-30 KB/s = 0.24 Mbps
- Negligible on modern connections (5+ Mbps)
```

**During throughput tests**:
- Upload test saturates one interface temporarily (~5 seconds)
- Tests staggered by a few seconds per interface
- Acceptable for diagnosis purposes

## Advantages Over Sequential Testing

| Aspect | Sequential (Phase 1-3) | Simultaneous (Phase 4) |
|--------|----------------------|------------------------|
| **Test Duration** | 2 weeks (1 week per connection) | 1 hour (real-time) |
| **Accuracy** | ❌ Different network conditions | ✅ Same conditions |
| **ISP Variables** | ❌ ISP might change | ✅ ISP same for both |
| **Proof** | ⚠️ Correlation only | ✅ Direct causation |
| **Convenience** | ❌ Physical cable swap | ✅ No intervention |
| **Cost** | ❌ High (time) | ✅ Low (automatic) |

## Limitations

1. **Linux Only**: `SO_BINDTODEVICE` is Linux-specific
   - Windows: Would need different approach (routing tables)
   - macOS: Limited support
   - **Verdict**: Document as Linux-only feature

2. **Requires Multiple Active Interfaces**: 
   - Must have both WiFi and Ethernet connected
   - Both must be able to reach the internet
   - May require routing table configuration

3. **Slightly Higher Bandwidth**:
   - 2x the bandwidth of single interface
   - Still negligible (~20-30 KB/s total)

4. **Routing Complexity**:
   - Linux routing can be tricky with multiple interfaces
   - May need to set up policy routing
   - Document common configurations

## Prerequisites

### Hardware
- ✅ Machine with multiple network interfaces (laptop with WiFi + Ethernet)
- ✅ Both interfaces connected to same router
- ✅ Both interfaces can reach internet

### Linux Configuration

**Check interfaces**:
```bash
ip link show
# Should show: wlan0, eth0, etc.
```

**Verify both interfaces have IPs**:
```bash
ip addr show
```

**Optional: Configure routing** (if both interfaces on different subnets):
```bash
# Usually not needed if both connect through same router
# But may be required in complex setups
```

## Conclusion

Multi-interface monitoring (Phase 4) is the **definitive solution** for diagnosing WiFi vs ISP issues:

- ✅ **Simultaneous testing**: Real-time comparison
- ✅ **Accurate**: Same network conditions for both
- ✅ **Fast**: Answer in minutes, not weeks
- ✅ **Visual**: Clear charts showing the difference
- ✅ **Proof**: Direct evidence of where problem lies

**Implementation effort**: 2-3 weeks  
**Value**: Very high for users with WiFi issues  
**Complexity**: Moderate (Linux socket binding)  

**Recommendation**: Implement as Phase 4 after Phases 1-2 are stable.

---

**Document Version**: 1.0  
**Last Updated**: 2025-10-18  
**Phase**: 4 (Multi-Interface + Multi-Server + Export)

