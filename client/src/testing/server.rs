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
use std::net::{SocketAddr, UdpSocket};
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
        
        // Resolve server address
        let server_addr = format!("{}:{}", config.host, config.port);
        let server_addr: SocketAddr = server_addr
            .parse()
            .with_context(|| format!("Failed to parse server address: {}", server_addr))?;
        
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
        
        let _reply = self.send_echo_request(&echo_request)?;
        
        let end_time = SystemTime::now();
        let rtt = end_time
            .duration_since(start_time)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0; // Convert to milliseconds
        
        // Calculate one-way delays (if we trust clock sync)
        // For now, just use RTT
        
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
        };
        
        debug!(
            "Server ECHO test completed: target={}, rtt={:.2}ms, seq={}",
            self.config.host, rtt, self.sequence
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

