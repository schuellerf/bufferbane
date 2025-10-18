# Network Instability Scenarios for Bufferbane

This document describes specific network instability scenarios that Bufferbane is designed to detect, along with their symptoms, detection methods, and typical patterns observed with co-ax cable internet connections.

## Overview

Cable internet (co-ax) connections like those provided by Magenta Austria can exhibit various instability patterns due to:
- Shared bandwidth in the neighborhood
- Signal degradation
- Modem/CMTS issues
- Interference on the cable line
- ISP congestion or routing issues

These issues often manifest as subtle problems that don't completely break connectivity but cause application-level failures.

---

## Scenario 1: Intermittent Upload Degradation

### Description
Upload speeds periodically drop below expected levels while download remains stable. This is particularly common in cable systems where upload bandwidth is significantly smaller than download and more susceptible to noise.

### Symptoms
- **Video conferencing**: Camera freezes, but you can still see/hear others
- **File uploads**: Stall or restart repeatedly
- **VPN**: Disconnects or becomes very slow (VPN is sensitive to upload)
- **Online gaming**: High upload lag, inability to send position updates
- **Cloud backups**: Fail or take extremely long
- **SSH/Remote desktop**: Commands take long to register, but responses come back

### Detection Method
```
Primary Indicators:
- Upload throughput drops >30% below baseline
- Download throughput remains stable (±10% of baseline)
- RTT may increase slightly due to buffer congestion
- Pattern: Often occurs in bursts or at specific times

Measurement Strategy:
1. Continuous small upload stream (100KB/s)
2. Measure actual transfer rate every second
3. Compare to baseline established during known-good periods
4. Track time-of-day patterns

Alert Threshold:
- Warning: Upload <80% baseline for >30 seconds
- Critical: Upload <50% baseline for >30 seconds
```

### Timing Patterns
- **Peak hours**: 18:00-23:00 (evening, neighborhood congestion)
- **Weekends**: More consistent degradation
- **Random bursts**: 5-60 second durations, possibly due to interference
- **Cyclical**: May repeat with predictable intervals (e.g., every 10 minutes)

### Data Signature
```
Example Detection:
Time     Upload(Mbps)  Download(Mbps)  RTT(ms)  Status
14:30:00    5.2          48.5           15       OK
14:30:01    5.1          48.2           16       OK
14:30:02    2.3          48.0           22       DEGRADED
14:30:03    1.8          47.8           28       DEGRADED
14:30:04    1.5          48.1           35       CRITICAL
...
14:30:45    4.8          48.3           17       RECOVERING
14:30:46    5.0          48.5           16       OK
```

### Root Causes (Cable-Specific)
- High upstream signal noise (ingress)
- Failing upstream amplifiers
- Loose connectors causing intermittent signal loss
- CMTS port congestion
- Neighbor with faulty equipment causing interference

---

## Scenario 2: Bufferbloat

### Description
Latency increases dramatically when upload or download bandwidth is being utilized, even if the utilization is well below the connection's rated capacity. This occurs when network equipment (modem, router, ISP equipment) has excessive buffering.

### Symptoms
- **Gaming**: Massive lag spikes when someone starts a download/upload
- **Video streaming**: Starts fine, then everything else becomes slow
- **Voice calls**: Audio delays and echo when transferring files
- **Web browsing**: Pages load slowly when backup is running
- **Smart home**: Devices become unresponsive during high traffic

### Detection Method
```
Primary Indicators:
- Idle latency: 15ms
- Loaded latency: 200ms+ (during upload/download)
- Latency spike duration matches load duration
- Latency returns to normal when load stops

Measurement Strategy:
1. Establish baseline idle latency (60-second average)
2. Start upload test
3. Continue ICMP pings during upload
4. Measure latency increase
5. Repeat for download and simultaneous up+down

Alert Threshold:
- Warning: Loaded latency >100ms increase vs idle
- Critical: Loaded latency >500ms increase vs idle
```

### Test Protocol
```
Phase 1: Baseline (30 seconds)
- Measure idle latency
- No bandwidth tests running

Phase 2: Upload Load (60 seconds)
- Start sustained upload at 80% capacity
- Continue latency measurements
- Record latency increase

Phase 3: Download Load (60 seconds)
- Start sustained download at 80% capacity
- Continue latency measurements
- Record latency increase

Phase 4: Both (60 seconds)
- Simultaneous upload + download
- Record maximum latency impact

Phase 5: Recovery (30 seconds)
- Stop all load tests
- Verify latency returns to baseline
```

### Data Signature
```
Example Detection:
Time     Phase      Upload(Mbps)  RTT_8.8.8.8(ms)  RTT_Gateway(ms)
15:00:00 Idle          0.1            15               5
15:00:30 Upload_Start  4.5            18               7
15:00:31 Upload        4.8            45              28
15:00:32 Upload        4.9            92              67
15:00:33 Upload        4.9           156             134
15:00:34 Upload        5.0           189             167  BUFFERBLOAT
...
15:01:30 Upload_Stop   0.1           198             175
15:01:31 Recovery      0.1            87              68
15:01:32 Recovery      0.1            35              22
15:01:33 Recovery      0.1            17               8
15:01:34 Normal        0.1            15               5
```

### Root Causes
- Excessive buffering in cable modem
- ISP CMTS buffer settings
- Router queue management issues
- Multiple levels of buffering (modem + router + ISP)

### Notes
- Bufferbloat is NOT packet loss, packets arrive but with huge delays
- Can exist even with high-speed connections
- SQM/QoS at router can help mitigate
- Often worse in upload direction on cable

---

## Scenario 3: Packet Loss Bursts

### Description
Short periods where multiple consecutive packets are lost, rather than scattered random losses. This often indicates signal issues or brief equipment failures.

### Symptoms
- **VoIP/Video calls**: Audio cuts out, robotic voice, video freezes
- **Online gaming**: Character teleports, position rollbacks
- **Streaming**: Brief video artifacts or buffering
- **SSH sessions**: Commands get lost, need to be retyped
- **TCP connections**: Retransmissions, slow recovery

### Detection Method
```
Primary Indicators:
- 3+ consecutive packets lost
- Loss occurs in clusters rather than randomly
- May coincide with latency spikes
- More common during certain times or conditions

Measurement Strategy:
1. Send UDP packets with sequence numbers every 100ms
2. Test server echoes back received sequence numbers
3. Detect gaps in sequence
4. Classify as: random (scattered) vs burst (consecutive)
5. ICMP ping may show timeouts

Alert Threshold:
- Warning: >0.5% loss over 60 seconds OR 3+ consecutive losses
- Critical: >2% loss OR 5+ consecutive losses
```

### Burst Pattern Types

**Type A: Short Bursts (1-3 seconds)**
```
Time       Seq   Status   Pattern
10:15:00   1000  OK       ████████████████
10:15:01   1001  OK       
10:15:02   1002  OK       
10:15:03   1003  LOST     ░░░░░░░░  <- Burst starts
10:15:04   1004  LOST     ░░░░░░░░
10:15:05   1005  LOST     ░░░░░░░░
10:15:06   1006  OK       ████████████████
10:15:07   1007  OK       

Cause: Brief signal interruption, interference spike
```

**Type B: Periodic Losses**
```
Pattern repeats every 10-30 seconds
Time       Loss%  Pattern
10:00:00   0.0%   ████████████████
10:00:10   5.0%   ██░░████████████
10:00:20   0.0%   ████████████████
10:00:30   4.0%   ██░░████████████
10:00:40   0.0%   ████████████████

Cause: Periodic interference, neighbor equipment, failing active device
```

**Type C: Scattered Losses**
```
Random, non-consecutive losses
10:00:00  OK   ████████████████
10:00:01  OK   ████████████████
10:00:02  LOST ░░██████████████
10:00:03  OK   ████████████████
10:00:04  OK   ████████████████
10:00:05  OK   ████████████████
10:00:06  LOST ██░░████████████

Cause: Low-level RF noise, weak signal
```

### Data Signature
```
Example Burst Detection:
Time     Seq#   Status  Consecutive_Loss
10:15:00  1000  OK      0
10:15:01  1001  OK      0
10:15:02  1002  OK      0
10:15:03  1003  LOST    1  <- Burst begins
10:15:04  1004  LOST    2
10:15:05  1005  LOST    3  ALERT: Burst detected
10:15:06  1006  LOST    4
10:15:07  1007  OK      0  <- Burst ends, duration: 4 seconds
10:15:08  1008  OK      0

Event Record:
- Type: Packet Loss Burst
- Start: 10:15:03
- Duration: 4 seconds
- Packets Lost: 4
- Loss Rate: 100% during burst
```

### Root Causes (Cable-Specific)
- Signal level drops (cable damage, weather)
- Loose or corroded connectors
- Splitter issues
- Upstream noise burst
- CMTS equipment glitches
- Channel bonding failures

---

## Scenario 4: Asymmetric Issues

### Description
Upload and download behave completely differently - one direction works perfectly while the other has problems. This is especially relevant for cable systems where upload and download use different frequency ranges and equipment.

### Symptoms
- **Variant A (Upload fails, download OK)**:
  - Can browse web and watch videos fine
  - Cannot send emails with attachments
  - Video calls: see/hear others, but they can't hear you
  - Can download files, but not upload

- **Variant B (Download fails, upload OK)**:
  - Less common in cable systems
  - Web pages time out or load slowly
  - Can send emails but not receive attachments
  - Streams buffer constantly

### Detection Method
```
Primary Indicators:
- One direction shows: high latency, loss, or low throughput
- Other direction remains within normal parameters
- Pattern is sustained, not brief

Measurement Strategy:
1. Separate monitoring of upload vs download
2. Independent throughput tests for each direction
3. Bidirectional packet loss testing
4. Compare metrics side-by-side

Alert Threshold:
- Upload problem + Download OK: Likely upload-specific issue
- Download problem + Upload OK: Likely download-specific issue
```

### Data Signature
```
Example: Upload Problem Only
Time     Upload_Mbps  Download_Mbps  Upload_Loss%  Download_Loss%  RTT_ms
14:00:00    5.0           48.5            0.0            0.0          15
14:05:00    2.1           48.2            2.5            0.0          25  <- Upload degrades
14:10:00    1.8           48.0            5.0            0.1          35
14:15:00    1.5           47.8            8.0            0.0          42
14:20:00    4.5           48.3            0.5            0.0          18  <- Recovers

Analysis:
- Download: Stable ~48 Mbps, ~0% loss
- Upload: Dropped to 30% capacity, up to 8% loss
- RTT increased (likely due to upload congestion)
- Conclusion: Upload-specific issue
```

### Root Causes

**Upload-Specific Issues**:
- Upstream frequency issues (cable uses 5-42 MHz or 5-85 MHz for upstream)
- Upstream amplifier problems
- High noise in upstream spectrum
- Lower upstream power budget
- More susceptible to ingress noise
- CMTS upstream port congestion

**Download-Specific Issues** (less common):
- Downstream frequency issues (cable uses 54-1002 MHz for downstream)
- Downstream power too high/low
- Specific downstream channel problems
- MER (Modulation Error Ratio) issues

---

## Scenario 5: ISP Routing Issues

### Description
Connection to some destinations works perfectly while others are problematic. This suggests routing issues within the ISP network or at peering points, rather than last-mile problems.

### Symptoms
- **Geographic patterns**: European sites work, US sites don't (or vice versa)
- **Provider patterns**: Google services fast, everything else slow
- **Specific services**: Gaming servers lag, but web browsing is fine
- **Traceroute shows**: Problems at specific hops within ISP network
- **Time-based**: Issues appear during peak hours only

### Detection Method
```
Primary Indicators:
- Latency/loss varies significantly by target
- ISP gateway latency normal
- Problems appear at specific hops in traceroute
- Issues correlate with time-of-day

Measurement Strategy:
1. Monitor multiple diverse targets simultaneously:
   - ISP gateway (should always be good)
   - National CDN nodes
   - International targets
   - Different ASNs
2. Compare metrics across targets
3. Periodic traceroutes to identify problem hop
4. Track peering point behavior

Alert Threshold:
- Warning: One target shows issues, others OK
- Critical: Multiple targets via same path affected
```

### Target Comparison Matrix
```
Example Detection:
Time     Gateway(ISP)  Google_AT  Cloudflare  AWS_EU  Target_US
14:00:00    5ms  0%      12ms 0%    13ms 0%    18ms 0%   95ms 0%
14:05:00    5ms  0%      12ms 0%    13ms 0%    45ms 2%   95ms 0%  <- AWS problem
14:10:00    6ms  0%      13ms 0%    14ms 0%    78ms 5%   94ms 0%
14:15:00    5ms  0%      12ms 0%    13ms 0%    92ms 8%   96ms 0%
14:20:00    5ms  0%      12ms 0%    13ms 0%    21ms 0%   95ms 0%  <- Recovered

Analysis:
- ISP gateway: Normal (5-6ms, 0% loss)
- Google, Cloudflare: Normal
- AWS EU: Problem (4x latency, packet loss)
- US targets: Normal
- Conclusion: Routing issue to AWS, not general connection problem
```

### Routing Problem Patterns

**Pattern A: Peering Congestion**
```
Symptoms:
- High latency to specific networks
- Packet loss during peak hours
- Traceroute shows congestion at peering point

Example:
  1  192.168.1.1 (Gateway)           1ms
  2  10.x.x.x (ISP)                  5ms
  3  10.x.x.x (ISP core)             8ms
  4  peer.vie1.at (Peering)         15ms  <- Normal
  5  target.net                     185ms  <- 10x expected! Congestion!
```

**Pattern B: BGP Route Flapping**
```
Symptoms:
- Route changes frequently
- Latency varies wildly
- Intermittent connectivity

Detection:
- Same target shows 20ms, then 150ms, then 20ms
- Traceroute path changes
```

**Pattern C: Suboptimal Routing**
```
Symptoms:
- Packets take longer path than necessary
- Austrian target routed via Germany

Example:
  1  Gateway (Vienna, AT)            1ms
  2  ISP (Vienna, AT)                5ms
  3  ISP Core (Frankfurt, DE)       25ms  <- Should not go here!
  4  Target (Vienna, AT)            55ms  <- Should be ~10ms
```

### Root Causes
- Peering point congestion
- BGP routing errors
- ISP traffic engineering decisions
- Upstream provider issues
- International transit problems
- DDoS mitigation causing path changes

---

## Scenario 6: Evening/Peak Hour Degradation

### Description
Connection quality degrades predictably during evening hours (18:00-23:00) when neighborhood usage is high. Classic cable internet issue due to shared bandwidth.

### Symptoms
- Consistent daily pattern
- All metrics degrade during peak hours
- Weekday vs weekend differences
- Quality returns after 23:00

### Detection Method
```
Measurement Strategy:
1. Track all metrics across 24-hour periods
2. Create baseline for off-peak (03:00-06:00)
3. Compare peak hours to baseline
4. Build time-of-day performance profile

Time-of-Day Profile:
00:00-06:00  Excellent (lowest usage)
06:00-09:00  Good
09:00-17:00  Good
17:00-20:00  Degraded  <- Peak
20:00-23:00  Poor      <- Worst
23:00-24:00  Improving
```

### Data Signature
```
Example: Typical Day Performance

Hour   Avg_Latency  Avg_Upload  Avg_Loss%  Quality
00:00      12ms       5.2 Mbps      0.0%    Excellent
03:00      11ms       5.3 Mbps      0.0%    Excellent  <- Baseline
06:00      13ms       5.1 Mbps      0.1%    Good
09:00      14ms       5.0 Mbps      0.2%    Good
12:00      15ms       4.9 Mbps      0.3%    Good
15:00      16ms       4.8 Mbps      0.4%    Good
18:00      25ms       3.8 Mbps      1.2%    Degraded   <- Starts
19:00      38ms       2.5 Mbps      2.5%    Poor
20:00      45ms       2.1 Mbps      3.8%    Poor
21:00      42ms       2.3 Mbps      3.2%    Poor
22:00      35ms       3.2 Mbps      2.0%    Degraded
23:00      22ms       4.2 Mbps      0.8%    Improving
24:00      15ms       4.9 Mbps      0.2%    Good
```

### Root Cause
- Shared cable segment in neighborhood
- Insufficient upstream capacity allocation
- CMTS oversubscription
- Many neighbors streaming/gaming simultaneously

---

## Scenario 7: Micro-Disconnections

### Description
Very brief complete disconnections (1-3 seconds) that may not be noticed for simple browsing but break persistent connections.

### Symptoms
- SSH sessions disconnect
- VPN reconnects frequently
- Video calls drop and reconnect
- Online games kick player
- Long downloads restart
- TCP connections reset

### Detection Method
```
Primary Indicators:
- All packets lost for 1-3 seconds
- No ICMP replies
- TCP connections reset
- DNS queries timeout
- Pattern: Complete loss then complete recovery

Measurement Strategy:
1. Continuous ICMP pings at 1-second intervals
2. Persistent TCP connections to test servers
3. Monitor connection reset events
4. Track consecutive failures

Alert Threshold:
- Any period of >2 seconds complete packet loss
- Any TCP connection reset not initiated by client
```

### Data Signature
```
Time     ICMP_Reply  TCP_Status  DNS_Status
10:30:00    OK       Connected   OK
10:30:01    OK       Connected   OK
10:30:02    TIMEOUT  Connected   OK
10:30:03    TIMEOUT  RESET       TIMEOUT  <- Disconnect
10:30:04    TIMEOUT  CLOSED      TIMEOUT
10:30:05    OK       Reconnect   OK       <- Back online
10:30:06    OK       Connected   OK

Event:
- Type: Micro-disconnection
- Duration: ~3 seconds
- All services affected
- Complete recovery after
```

### Root Causes
- Modem re-ranging (negotiating with CMTS)
- Brief signal loss
- Modem reboot/firmware update
- Cable line fault
- Weather-related interference

---

## Detection Priority Matrix

| Scenario | Impact | Frequency | Detection Difficulty | Priority |
|----------|--------|-----------|---------------------|----------|
| Upload Degradation | High | Common | Easy | **Critical** |
| Bufferbloat | High | Common | Medium | **Critical** |
| Packet Loss Bursts | High | Common | Easy | **Critical** |
| Asymmetric Issues | High | Moderate | Easy | **High** |
| Routing Issues | Medium | Low | Medium | **Medium** |
| Peak Hour Issues | Medium | Predictable | Easy | **Medium** |
| Micro-Disconnects | High | Low | Easy | **High** |

## Summary

Bufferbane must be capable of detecting all these scenarios simultaneously, as they can occur independently or in combination. The key is maintaining 1-second granularity with multiple concurrent tests to capture the transient nature of these issues.

