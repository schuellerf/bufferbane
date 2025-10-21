//! Server-based testing (Phase 2)
//!
//! Handles encrypted communication with Bufferbane server for:
//! - Enhanced latency testing (ECHO requests)
//! - Authentication via port knocking
//! - Future: Throughput and bufferbloat testing

use crate::config::ServerConfig;
use crate::testing::{Measurement, SyncEvent};
use anyhow::{Context, Result};
use protocol::{
    crypto,
    packets::{
        EchoReplyPayload, EchoRequestPayload, KnockAckPayload, KnockPayload,
        PacketHeader, PacketType,
    },
};
use std::collections::VecDeque;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, info, warn};

/// Sample of clock offset measurement
#[derive(Clone)]
struct OffsetSample {
    /// Measured offset (ns)
    offset_ns: f64,
    /// RTT for this sample (ns)
    rtt_ns: f64,
}

/// Time synchronization state for a server
struct TimeSyncState {
    /// Monotonic reference point for this session
    session_start: Instant,
    /// System time at session start (for storage)
    session_start_system: SystemTime,
    /// Ring buffer of recent offset samples (last 16)
    offset_samples: VecDeque<OffsetSample>,
    /// Current best offset estimate (ns)
    best_offset_ns: f64,
    /// Sync quality score (0-100)
    quality: u8,
    /// Is time sync good enough for reporting?
    is_synced: bool,
    /// Was synced in previous measurement (for event detection)
    was_synced: bool,
}

impl TimeSyncState {
    fn new() -> Self {
        Self {
            session_start: Instant::now(),
            session_start_system: SystemTime::now(),
            offset_samples: VecDeque::new(),
            best_offset_ns: 0.0,
            quality: 0,
            is_synced: false,
            was_synced: false,
        }
    }
}

/// Server tester for Phase 2 features
pub struct ServerTester {
    config: Arc<ServerConfig>,
    socket: UdpSocket,
    server_addr: SocketAddr,
    shared_secret: [u8; 32],
    client_id: u64,
    session_id: Option<u64>,
    interface: String,
    connection_type: String,
    sequence: u32,
    /// Time synchronization state
    time_sync: TimeSyncState,
}

impl ServerTester {
    /// Create a new server tester
    pub fn new(
        config: Arc<ServerConfig>,
        interface: String,
        connection_type: String,
    ) -> Result<Self> {
        // Parse shared secret
        let shared_secret = crypto::parse_shared_secret(&config.shared_secret)
            .map_err(|e| anyhow::anyhow!("Invalid shared secret: {}", e))?;
        
        // Resolve server address (supports both IP and hostname)
        let server_addr_str = format!("{}:{}", config.host, config.port);
        let server_addr: SocketAddr = server_addr_str
            .to_socket_addrs()
            .with_context(|| format!("Failed to resolve server address: {}", server_addr_str))?
            .next()
            .with_context(|| format!("No IP address found for: {}", server_addr_str))?;
        
        // Create UDP socket
        let socket = UdpSocket::bind("0.0.0.0:0")
            .context("Failed to bind UDP socket")?;
        
        // Set timeouts
        socket
            .set_read_timeout(Some(Duration::from_millis(config.knock_timeout_ms)))
            .context("Failed to set socket read timeout")?;
        
        socket
            .set_write_timeout(Some(Duration::from_millis(1000)))
            .context("Failed to set socket write timeout")?;
        
        let client_id = config.client_id;
        let host = config.host.clone();
        let port = config.port;
        
        info!(
            "Server tester initialized for {}:{} (interface: {})",
            host, port, interface
        );
        
        Ok(Self {
            config,
            socket,
            server_addr,
            shared_secret,
            client_id,
            session_id: None,
            interface,
            connection_type,
            sequence: 0,
            time_sync: TimeSyncState::new(),
        })
    }
    
    /// Authenticate with server (port knocking)
    pub fn authenticate(&mut self) -> Result<()> {
        for attempt in 1..=self.config.knock_retry_attempts {
            debug!("Authentication attempt {}/{}", attempt, self.config.knock_retry_attempts);
            
            match self.send_knock() {
                Ok(session_id) => {
                    self.session_id = Some(session_id);
                    
                    // Reset time sync on new session
                    self.time_sync = TimeSyncState::new();
                    
                    info!("Authenticated with server {} (session_id: {})", self.server_addr, session_id);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Authentication attempt {} failed: {}", attempt, e);
                    if attempt < self.config.knock_retry_attempts {
                        std::thread::sleep(Duration::from_millis(500));
                    }
                }
            }
        }
        
        anyhow::bail!("Failed to authenticate after {} attempts", self.config.knock_retry_attempts)
    }
    
    /// Send KNOCK packet and wait for KNOCK_ACK
    fn send_knock(&mut self) -> Result<u64> {
        // Create knock payload
        let knock = KnockPayload::new();
        let knock_bytes = knock.to_bytes();
        
        // Create packet header
        let header = PacketHeader::new(
            PacketType::Knock,
            (knock_bytes.len() + crypto::TAG_SIZE) as u16,
            self.client_id,
        );
        
        // Encrypt payload
        let nonce = header.nonce();
        let header_bytes = header.to_bytes();
        let encrypted = crypto::encrypt(&knock_bytes, &self.shared_secret, &nonce, &header_bytes)
            .context("Failed to encrypt KNOCK packet")?;
        
        // Build packet
        let mut packet = Vec::with_capacity(PacketHeader::SIZE + encrypted.len());
        packet.extend_from_slice(&header_bytes);
        packet.extend_from_slice(&encrypted);
        
        // Send packet
        self.socket
            .send_to(&packet, self.server_addr)
            .context("Failed to send KNOCK packet")?;
        
        debug!("Sent KNOCK packet to {}", self.server_addr);
        
        // Wait for KNOCK_ACK
        let mut buf = vec![0u8; 4096];
        let (len, _) = self
            .socket
            .recv_from(&mut buf)
            .context("Failed to receive KNOCK_ACK")?;
        
        // Parse response header
        let response_header = PacketHeader::from_bytes(&buf[..len])
            .context("Invalid KNOCK_ACK header")?;
        
        if response_header.packet_type != PacketType::KnockAck {
            anyhow::bail!("Expected KNOCK_ACK, got {:?}", response_header.packet_type);
        }
        
        // Decrypt response
        let response_nonce = response_header.nonce();
        let response_header_bytes = response_header.to_bytes();
        let encrypted_payload = &buf[PacketHeader::SIZE..len];
        
        let decrypted = crypto::decrypt(
            encrypted_payload,
            &self.shared_secret,
            &response_nonce,
            &response_header_bytes,
        )
        .context("Failed to decrypt KNOCK_ACK")?;
        
        // Parse KNOCK_ACK payload
        let ack = KnockAckPayload::from_bytes(&decrypted)
            .context("Invalid KNOCK_ACK payload")?;
        
        debug!("Received KNOCK_ACK: session_id={}", ack.session_id);
        
        Ok(ack.session_id)
    }
    
    /// Update time synchronization state with a new measurement
    fn update_time_sync(&mut self, t1: u64, t2: u64, t3: u64, t4: u64, rtt_ns: f64) {
        // Calculate raw offset using NTP algorithm
        // offset = ((T2 - T1) + (T3 - T4)) / 2
        let offset_ns = ((t2 as f64 - t1 as f64) + (t3 as f64 - t4 as f64)) / 2.0;
        
        // Validate by checking if this offset produces reasonable upload/download times
        // They should both be positive and less than RTT
        let test_upload = (t2 as f64 - t1 as f64) - offset_ns;
        let test_download = (t4 as f64 - t3 as f64) + offset_ns;
        
        if test_upload <= 0.0 || test_download <= 0.0 || test_upload >= rtt_ns || test_download >= rtt_ns {
            debug!(
                "Rejecting offset sample for {}: offset={:.2}ms would produce invalid latencies (up={:.2}ms, down={:.2}ms, rtt={:.2}ms)",
                self.config.host,
                offset_ns / 1_000_000.0,
                test_upload / 1_000_000.0,
                test_download / 1_000_000.0,
                rtt_ns / 1_000_000.0
            );
            return;
        }
        
        // Add to ring buffer
        self.time_sync.offset_samples.push_back(OffsetSample {
            offset_ns,
            rtt_ns,
        });
        
        // Keep last 16 samples
        if self.time_sync.offset_samples.len() > 16 {
            self.time_sync.offset_samples.pop_front();
        }
        
        // Need at least 8 samples for good sync
        if self.time_sync.offset_samples.len() < 8 {
            self.time_sync.is_synced = false;
            self.time_sync.quality = (self.time_sync.offset_samples.len() * 12) as u8; // 0-96
            return;
        }
        
        // Use best quartile (lowest RTT = most reliable)
        let mut sorted: Vec<_> = self.time_sync.offset_samples.iter().collect();
        sorted.sort_by(|a, b| a.rtt_ns.partial_cmp(&b.rtt_ns).unwrap());
        
        let best_count = sorted.len() / 2; // Top 50%
        let best_samples: Vec<_> = sorted.iter().take(best_count).collect();
        
        // Calculate median offset from best samples
        let mut best_offsets: Vec<f64> = best_samples.iter().map(|s| s.offset_ns).collect();
        best_offsets.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.time_sync.best_offset_ns = best_offsets[best_offsets.len() / 2];
        
        // Calculate standard deviation for quality
        let mean = best_offsets.iter().sum::<f64>() / best_offsets.len() as f64;
        let variance = best_offsets.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / best_offsets.len() as f64;
        let std_dev_ms = (variance.sqrt()) / 1_000_000.0;
        
        // Quality score: 100 if std_dev < 1ms, decreasing to 0 at 10ms
        self.time_sync.quality = ((1.0 - (std_dev_ms / 10.0).min(1.0)) * 100.0) as u8;
        self.time_sync.is_synced = self.time_sync.quality >= 80;
        
        if !self.time_sync.is_synced {
            debug!(
                "Time sync quality low for {}: {}% (std_dev={:.2}ms, samples={})",
                self.config.host, self.time_sync.quality, std_dev_ms, self.time_sync.offset_samples.len()
            );
        } else if self.sequence <= 10 || self.sequence % 100 == 0 {
            debug!(
                "Time sync for {}: offset={:.2}ms, quality={}%, samples={}",
                self.config.host,
                self.time_sync.best_offset_ns / 1_000_000.0,
                self.time_sync.quality,
                self.time_sync.offset_samples.len()
            );
        }
    }
    
    /// Run echo test (send ECHO_REQUEST, wait for ECHO_REPLY)
    pub fn run_test(&mut self) -> Result<Vec<Measurement>> {
        if !self.config.enable_echo_test {
            return Ok(Vec::new());
        }
        
        // Ensure we're authenticated
        if self.session_id.is_none() {
            if let Err(e) = self.authenticate() {
                // Authentication failed - create error measurement
                let mut measurement = Measurement::new_server_echo(
                    self.config.host.clone(),
                    self.interface.clone(),
                    self.connection_type.clone(),
                );
                measurement.set_error(format!("Authentication failed: {}", e));
                return Ok(vec![measurement]);
            }
        }
        
        // Increment sequence number
        self.sequence += 1;
        
        // Create measurement (will be updated based on test result)
        let mut measurement = Measurement::new_server_echo(
            self.config.host.clone(),
            self.interface.clone(),
            self.connection_type.clone(),
        );
        
        // Use monotonic clock for ALL timing (T1, T4, and RTT)
        let start_instant = Instant::now();
        
        // T1: Client send time (monotonic nanoseconds since session start)
        let t1_ns = start_instant
            .duration_since(self.time_sync.session_start)
            .as_nanos() as u64;
        
        let echo_request = EchoRequestPayload::with_timestamp(self.sequence, t1_ns);
        
        let reply = match self.send_echo_request(&echo_request) {
            Ok(r) => r,
            Err(e) => {
                // Check if it's a timeout or other error
                let error_msg = e.to_string();
                if error_msg.contains("timeout") || error_msg.contains("timed out") {
                    measurement.set_timeout();
                    debug!("Server {} -> timeout", self.config.host);
                } else {
                    measurement.set_error(error_msg.clone());
                    debug!("Server {} -> error: {}", self.config.host, error_msg);
                }
                return Ok(vec![measurement]);
            }
        };
        
        let end_instant = Instant::now();
        
        // Calculate RTT using monotonic clock
        let rtt = end_instant
            .duration_since(start_instant)
            .as_secs_f64()
            * 1000.0; // Convert to milliseconds
        let rtt_ns = rtt * 1_000_000.0;
        
        // T4: Client recv time (monotonic nanoseconds since session start)
        let t4_ns = end_instant
            .duration_since(self.time_sync.session_start)
            .as_nanos() as u64;
        
        // Extract timestamps from reply (T2 and T3 are from server's monotonic clock)
        let t1 = reply.client_send_timestamp;  // Our T1, echoed back
        let t2 = reply.server_recv_timestamp;  // Server's monotonic time
        let t3 = reply.server_send_timestamp;  // Server's monotonic time
        let t4 = t4_ns;  // Our T4 (monotonic)
        
        // Update time sync with this measurement
        self.update_time_sync(t1, t2, t3, t4, rtt_ns);
        
        // Calculate measurement timestamp from session start + elapsed monotonic time
        let measurement_time = self.time_sync.session_start_system 
            + end_instant.duration_since(self.time_sync.session_start);
        
        // Update measurement with base data
        measurement.timestamp = measurement_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        measurement.monotonic_ns = t1_ns as u128;  // Store monotonic timestamp for reference
        measurement.server_name = Some(self.config.host.clone());
        measurement.rtt_ms = Some(rtt);
        measurement.packet_loss_pct = Some(0.0); // Successful = 0% loss
        measurement.status = "success".to_string();
        
        // Track previous sync state for event detection
        let prev_synced = self.time_sync.was_synced;
        
        // Only include timing data if synced
        if self.time_sync.is_synced {
            let upload_latency_ns = (t2 as f64 - t1 as f64) - self.time_sync.best_offset_ns;
            let download_latency_ns = (t4 as f64 - t3 as f64) + self.time_sync.best_offset_ns;
            let server_processing_ns = t3 as f64 - t2 as f64;
            
            // Final validation: ensure calculated values are reasonable
            // Both should be positive and less than RTT
            let values_valid = upload_latency_ns > 0.0 
                && download_latency_ns > 0.0 
                && upload_latency_ns < rtt_ns 
                && download_latency_ns < rtt_ns;
            
            if values_valid {
                measurement.upload_latency_ms = Some(upload_latency_ns / 1_000_000.0);
                measurement.download_latency_ms = Some(download_latency_ns / 1_000_000.0);
                measurement.server_processing_us = Some((server_processing_ns / 1_000.0) as i64);
                
                debug!(
                    "Server {} -> rtt={:.2}ms, upload={:.2}ms, download={:.2}ms, processing={:.0}μs, sync_quality={}%",
                    self.config.host,
                    rtt,
                    upload_latency_ns / 1_000_000.0,
                    download_latency_ns / 1_000_000.0,
                    server_processing_ns / 1_000.0,
                    self.time_sync.quality
                );
            } else {
                // Calculated values invalid - mark sync as lost and don't store them
                let message = format!(
                    "Invalid latencies detected: up={:.2}ms, down={:.2}ms (RTT={:.2}ms) - offset corrupted",
                    upload_latency_ns / 1_000_000.0,
                    download_latency_ns / 1_000_000.0,
                    rtt
                );
                warn!("Time sync for {} {}", self.config.host, message);
                
                self.time_sync.is_synced = false;
                self.time_sync.quality = 0;
                measurement.upload_latency_ms = None;
                measurement.download_latency_ms = None;
                measurement.server_processing_us = None;
                
                // Store sync event
                measurement.sync_event = Some(SyncEvent {
                    event_type: "sync_invalid".to_string(),
                    message,
                    quality: Some(0),
                });
            }
        } else {
            // Not synced - only store RTT
            measurement.upload_latency_ms = None;
            measurement.download_latency_ms = None;
            measurement.server_processing_us = None;
            
            if self.sequence % 10 == 0 {
                debug!(
                    "Server {} -> rtt={:.2}ms, time sync not ready ({}/8 samples, quality={}%)",
                    self.config.host,
                    rtt,
                    self.time_sync.offset_samples.len(),
                    self.time_sync.quality
                );
            }
        }
        
        // Update sync state tracking
        self.time_sync.was_synced = self.time_sync.is_synced;
        
        // Detect sync state changes
        if !prev_synced && self.time_sync.is_synced {
            let message = format!(
                "Time sync established (quality={}%, offset={:.2}ms)",
                self.time_sync.quality,
                self.time_sync.best_offset_ns / 1_000_000.0
            );
            info!("Time sync for {} {}", self.config.host, message);
            
            measurement.sync_event = Some(SyncEvent {
                event_type: "sync_established".to_string(),
                message,
                quality: Some(self.time_sync.quality),
            });
        } else if prev_synced && !self.time_sync.is_synced {
            let message = format!("Time sync lost (quality dropped to {}%)", self.time_sync.quality);
            warn!("Time sync for {} {}", self.config.host, message);
            
            measurement.sync_event = Some(SyncEvent {
                event_type: "sync_lost".to_string(),
                message,
                quality: Some(self.time_sync.quality),
            });
        }
        
        Ok(vec![measurement])
    }
    
    /// Send ECHO_REQUEST and wait for ECHO_REPLY
    fn send_echo_request(&self, request: &EchoRequestPayload) -> Result<EchoReplyPayload> {
        let request_bytes = request.to_bytes();
        
        // Create packet header
        let header = PacketHeader::new(
            PacketType::EchoRequest,
            (request_bytes.len() + crypto::TAG_SIZE) as u16,
            self.client_id,
        );
        
        // Encrypt payload
        let nonce = header.nonce();
        let header_bytes = header.to_bytes();
        let encrypted = crypto::encrypt(&request_bytes, &self.shared_secret, &nonce, &header_bytes)
            .context("Failed to encrypt ECHO_REQUEST")?;
        
        // Build packet
        let mut packet = Vec::with_capacity(PacketHeader::SIZE + encrypted.len());
        packet.extend_from_slice(&header_bytes);
        packet.extend_from_slice(&encrypted);
        
        // Send packet
        self.socket
            .send_to(&packet, self.server_addr)
            .context("Failed to send ECHO_REQUEST")?;
        
        // Wait for ECHO_REPLY
        let mut buf = vec![0u8; 4096];
        let (len, _) = self
            .socket
            .recv_from(&mut buf)
            .context("Failed to receive ECHO_REPLY")?;
        
        // Parse response header
        let response_header = PacketHeader::from_bytes(&buf[..len])
            .context("Invalid ECHO_REPLY header")?;
        
        if response_header.packet_type != PacketType::EchoReply {
            anyhow::bail!("Expected ECHO_REPLY, got {:?}", response_header.packet_type);
        }
        
        // Decrypt response
        let response_nonce = response_header.nonce();
        let response_header_bytes = response_header.to_bytes();
        let encrypted_payload = &buf[PacketHeader::SIZE..len];
        
        let decrypted = crypto::decrypt(
            encrypted_payload,
            &self.shared_secret,
            &response_nonce,
            &response_header_bytes,
        )
        .context("Failed to decrypt ECHO_REPLY")?;
        
        // Parse ECHO_REPLY payload
        let reply = EchoReplyPayload::from_bytes(&decrypted)
            .context("Invalid ECHO_REPLY payload")?;
        
        Ok(reply)
    }
}

