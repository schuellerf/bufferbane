//! Server-based testing (Phase 2)
//!
//! Handles encrypted communication with Bufferbane server for:
//! - Enhanced latency testing (ECHO requests)
//! - Authentication via port knocking
//! - Future: Throughput and bufferbloat testing

use crate::config::ServerConfig;
use crate::testing::Measurement;
use anyhow::{Context, Result};
use protocol::{
    crypto,
    packets::{
        EchoReplyPayload, EchoRequestPayload, KnockAckPayload, KnockPayload,
        PacketHeader, PacketType,
    },
};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

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
    /// Estimated clock offset (server_clock - client_clock) in nanoseconds
    /// Calculated using: offset = ((T2-T1) + (T3-T4)) / 2
    /// Updated with exponential moving average for stability
    clock_offset_ns: f64,
    /// Weight factor for EMA (0.1 = 10% new, 90% old)
    offset_ema_alpha: f64,
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
            clock_offset_ns: 0.0,  // Will be calculated from measurements
            offset_ema_alpha: 0.1,  // 10% new sample, 90% history
        })
    }
    
    /// Authenticate with server (port knocking)
    pub fn authenticate(&mut self) -> Result<()> {
        for attempt in 1..=self.config.knock_retry_attempts {
            debug!("Authentication attempt {}/{}", attempt, self.config.knock_retry_attempts);
            
            match self.send_knock() {
                Ok(session_id) => {
                    self.session_id = Some(session_id);
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
    
    /// Run echo test (send ECHO_REQUEST, wait for ECHO_REPLY)
    pub fn run_test(&mut self) -> Result<Vec<Measurement>> {
        if !self.config.enable_echo_test {
            return Ok(Vec::new());
        }
        
        // Ensure we're authenticated
        if self.session_id.is_none() {
            self.authenticate()?;
        }
        
        // Increment sequence number
        self.sequence += 1;
        
        // Send echo request
        let start_time = SystemTime::now();
        let echo_request = EchoRequestPayload::new(self.sequence);
        let request_timestamp = echo_request.client_timestamp;
        
        let reply = self.send_echo_request(&echo_request)?;
        
        let end_time = SystemTime::now();
        let client_recv_ns = end_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        let rtt = end_time
            .duration_since(start_time)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0; // Convert to milliseconds
        
        // Calculate clock offset using NTP-style algorithm
        // Given timestamps: T1 (client send), T2 (server recv), T3 (server send), T4 (client recv)
        // Clock offset θ (server - client) = ((T2 - T1) + (T3 - T4)) / 2
        // This works because:
        //   T2 - T1 = upload_time + θ
        //   T4 - T3 = download_time - θ
        //   Adding: (T2-T1) + (T3-T4) = upload_time - download_time + 2θ
        //   If path is symmetric (upload ≈ download): 2θ ≈ (T2-T1) + (T3-T4)
        
        let t1 = reply.client_send_timestamp as f64;
        let t2 = reply.server_recv_timestamp as f64;
        let t3 = reply.server_send_timestamp as f64;
        let t4 = client_recv_ns as f64;
        
        // Calculate raw offset for this measurement
        let measured_offset_ns = ((t2 - t1) + (t3 - t4)) / 2.0;
        
        // Update moving average (Exponential Moving Average)
        if self.sequence == 1 {
            // First measurement - use it directly
            self.clock_offset_ns = measured_offset_ns;
        } else {
            // Smooth with EMA: new_avg = alpha * new_sample + (1-alpha) * old_avg
            self.clock_offset_ns = self.offset_ema_alpha * measured_offset_ns 
                                    + (1.0 - self.offset_ema_alpha) * self.clock_offset_ns;
        }
        
        // Apply offset correction to get true one-way latencies
        let upload_latency_ns = (t2 - t1) - self.clock_offset_ns;
        let download_latency_ns = (t4 - t3) + self.clock_offset_ns;
        let upload_latency_ms = upload_latency_ns / 1_000_000.0;
        let download_latency_ms = download_latency_ns / 1_000_000.0;
        
        // Server processing time (uses only server clock, no offset needed)
        let server_processing_ns = t3 - t2;
        let server_processing_us = (server_processing_ns / 1_000.0) as i64;
        let server_processing_ms = server_processing_us as f64 / 1000.0;
        
        // Sanity check: corrected times should sum to RTT
        let calculated_rtt = upload_latency_ms + download_latency_ms + server_processing_ms;
        let rtt_diff = (rtt - calculated_rtt).abs();
        
        // If offset correction worked, difference should be < 5ms
        // If not, something is wrong (e.g., asymmetric path, packet reordering)
        let correction_valid = rtt_diff < 5.0;
        
        if !correction_valid {
            debug!(
                "Offset correction validation failed for {}: RTT={:.2}ms but corrected_sum={:.2}ms (diff={:.2}ms, offset={:.2}ms)",
                self.config.host, rtt, calculated_rtt, rtt_diff, self.clock_offset_ns / 1_000_000.0
            );
        }
        
        // Log offset for first few measurements and periodically
        if self.sequence <= 5 || self.sequence % 100 == 0 {
            debug!(
                "Clock offset for {}: {:.2}ms (measured: {:.2}ms, EMA smoothed)",
                self.config.host,
                self.clock_offset_ns / 1_000_000.0,
                measured_offset_ns / 1_000_000.0
            );
        }
        
        // Create measurement
        let measurement = Measurement {
            timestamp: start_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            monotonic_ns: request_timestamp as u128,
            interface: self.interface.clone(),
            connection_type: self.connection_type.clone(),
            test_type: "server_echo".to_string(),
            target: self.config.host.clone(),
            server_name: Some(self.config.host.clone()),
            rtt_ms: Some(rtt),
            jitter_ms: None, // TODO: Calculate jitter
            packet_loss_pct: Some(0.0), // Successful = 0% loss
            throughput_kbps: None,
            dns_time_ms: None,
            status: "success".to_string(),
            error_detail: None,
            upload_latency_ms: Some(upload_latency_ms),
            download_latency_ms: Some(download_latency_ms),
            server_processing_us: Some(server_processing_us),
        };
        
        debug!(
            "Server ECHO test completed: target={}, rtt={:.2}ms, upload={:.2}ms, download={:.2}ms, processing={}μs, offset={:.2}ms, seq={}",
            self.config.host, rtt, upload_latency_ms, download_latency_ms, server_processing_us,
            self.clock_offset_ns / 1_000_000.0, self.sequence
        );
        
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

