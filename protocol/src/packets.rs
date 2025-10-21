//! Bufferbane protocol packet structures

use std::time::SystemTime;
use thiserror::Error;

/// Protocol version
pub const PROTOCOL_VERSION: u8 = 1;

/// Packet types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    /// Port knocking authentication
    Knock = 0x01,
    KnockAck = 0x02,
    
    /// Latency testing
    EchoRequest = 0x10,
    EchoReply = 0x11,
    
    /// Throughput testing (upload)
    ThroughputStart = 0x20,
    ThroughputData = 0x21,
    ThroughputEnd = 0x22,
    ThroughputStats = 0x23,
    
    /// Download testing
    DownloadRequest = 0x30,
    DownloadData = 0x31,
    DownloadEnd = 0x32,
    
    /// Bufferbloat testing
    BufferbloatStart = 0x40,
    BufferbloatEnd = 0x41,
    
    /// Error response
    Error = 0xFF,
}

impl PacketType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Knock),
            0x02 => Some(Self::KnockAck),
            0x10 => Some(Self::EchoRequest),
            0x11 => Some(Self::EchoReply),
            0x20 => Some(Self::ThroughputStart),
            0x21 => Some(Self::ThroughputData),
            0x22 => Some(Self::ThroughputEnd),
            0x23 => Some(Self::ThroughputStats),
            0x30 => Some(Self::DownloadRequest),
            0x31 => Some(Self::DownloadData),
            0x32 => Some(Self::DownloadEnd),
            0x40 => Some(Self::BufferbloatStart),
            0x41 => Some(Self::BufferbloatEnd),
            0xFF => Some(Self::Error),
            _ => None,
        }
    }
}

/// Cleartext packet header (24 bytes)
#[derive(Debug, Clone)]
pub struct PacketHeader {
    /// Magic bytes "BFBN" (4 bytes)
    pub magic: u32,
    /// Protocol version (1 byte)
    pub version: u8,
    /// Packet type (1 byte)
    pub packet_type: PacketType,
    /// Payload length (2 bytes)
    pub payload_len: u16,
    /// Client ID (8 bytes)
    pub client_id: u64,
    /// Nonce timestamp in nanoseconds (8 bytes)
    pub nonce_timestamp: u64,
}

impl PacketHeader {
    pub const SIZE: usize = 24;
    
    pub fn new(packet_type: PacketType, payload_len: u16, client_id: u64) -> Self {
        let nonce_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        Self {
            magic: crate::constants::MAGIC_BYTES,
            version: PROTOCOL_VERSION,
            packet_type,
            payload_len,
            client_id,
            nonce_timestamp,
        }
    }
    
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.magic.to_be_bytes());
        bytes[4] = self.version;
        bytes[5] = self.packet_type as u8;
        bytes[6..8].copy_from_slice(&self.payload_len.to_be_bytes());
        bytes[8..16].copy_from_slice(&self.client_id.to_be_bytes());
        bytes[16..24].copy_from_slice(&self.nonce_timestamp.to_be_bytes());
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < Self::SIZE {
            return Err(PacketError::TooShort);
        }
        
        let magic = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic != crate::constants::MAGIC_BYTES {
            return Err(PacketError::InvalidMagic);
        }
        
        let version = bytes[4];
        if version != PROTOCOL_VERSION {
            return Err(PacketError::UnsupportedVersion(version));
        }
        
        let packet_type = PacketType::from_u8(bytes[5])
            .ok_or(PacketError::UnknownPacketType(bytes[5]))?;
        
        let payload_len = u16::from_be_bytes([bytes[6], bytes[7]]);
        let client_id = u64::from_be_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let nonce_timestamp = u64::from_be_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19],
            bytes[20], bytes[21], bytes[22], bytes[23],
        ]);
        
        Ok(Self {
            magic,
            version,
            packet_type,
            payload_len,
            client_id,
            nonce_timestamp,
        })
    }
    
    /// Generate 12-byte nonce from client_id and nonce_timestamp
    pub fn nonce(&self) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        nonce[0..4].copy_from_slice(&self.client_id.to_be_bytes()[0..4]);
        nonce[4..12].copy_from_slice(&self.nonce_timestamp.to_be_bytes());
        nonce
    }
}

/// Packet errors
#[derive(Debug, Error)]
pub enum PacketError {
    #[error("Packet too short")]
    TooShort,
    
    #[error("Invalid magic bytes")]
    InvalidMagic,
    
    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(u8),
    
    #[error("Unknown packet type: {0:#x}")]
    UnknownPacketType(u8),
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Decryption error: {0}")]
    DecryptionError(String),
}

/// KNOCK packet payload
#[derive(Debug, Clone)]
pub struct KnockPayload {
    /// Random challenge (32 bytes)
    pub challenge: [u8; 32],
}

impl KnockPayload {
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut challenge = [0u8; 32];
        rng.fill(&mut challenge);
        Self { challenge }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        self.challenge.to_vec()
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < 32 {
            return Err(PacketError::TooShort);
        }
        let mut challenge = [0u8; 32];
        challenge.copy_from_slice(&bytes[0..32]);
        Ok(Self { challenge })
    }
}

/// KNOCK_ACK packet payload
#[derive(Debug, Clone)]
pub struct KnockAckPayload {
    /// Session ID assigned by server (8 bytes)
    pub session_id: u64,
    /// Challenge response (32 bytes - hash of client challenge)
    pub challenge_response: [u8; 32],
}

impl KnockAckPayload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(40);
        bytes.extend_from_slice(&self.session_id.to_be_bytes());
        bytes.extend_from_slice(&self.challenge_response);
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < 40 {
            return Err(PacketError::TooShort);
        }
        let session_id = u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let mut challenge_response = [0u8; 32];
        challenge_response.copy_from_slice(&bytes[8..40]);
        Ok(Self { session_id, challenge_response })
    }
}

/// ECHO_REQUEST packet payload
#[derive(Debug, Clone)]
pub struct EchoRequestPayload {
    /// Sequence number
    pub sequence: u32,
    /// Client send timestamp (nanoseconds)
    pub client_timestamp: u64,
}

impl EchoRequestPayload {
    /// Create new echo request with monotonic timestamp
    /// The timestamp is nanoseconds and should be from Instant (not SystemTime)
    pub fn new(sequence: u32) -> Self {
        // Caller should provide monotonic timestamp
        // This is a placeholder - should be set by caller
        Self { sequence, client_timestamp: 0 }
    }
    
    /// Create with explicit timestamp (from monotonic clock)
    pub fn with_timestamp(sequence: u32, timestamp_ns: u64) -> Self {
        Self { sequence, client_timestamp: timestamp_ns }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(12);
        bytes.extend_from_slice(&self.sequence.to_be_bytes());
        bytes.extend_from_slice(&self.client_timestamp.to_be_bytes());
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < 12 {
            return Err(PacketError::TooShort);
        }
        let sequence = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let client_timestamp = u64::from_be_bytes([
            bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11],
        ]);
        Ok(Self { sequence, client_timestamp })
    }
}

/// ECHO_REPLY packet payload
#[derive(Debug, Clone)]
pub struct EchoReplyPayload {
    /// Sequence number (echoed from request)
    pub sequence: u32,
    /// Client send timestamp (echoed from request, nanoseconds since UNIX_EPOCH)
    pub client_send_timestamp: u64,
    /// Server receive timestamp (nanoseconds since UNIX_EPOCH)
    pub server_recv_timestamp: u64,
    /// Server send timestamp (nanoseconds since UNIX_EPOCH)
    pub server_send_timestamp: u64,
}

impl EchoReplyPayload {
    /// Create reply with monotonic timestamp for T2 (server recv)
    /// Timestamp should be nanoseconds from server's monotonic clock (Instant)
    pub fn new(request: &EchoRequestPayload) -> Self {
        // Placeholder - caller should use with_timestamps instead
        Self {
            sequence: request.sequence,
            client_send_timestamp: request.client_timestamp,
            server_recv_timestamp: 0,
            server_send_timestamp: 0,
        }
    }
    
    /// Create reply with explicit monotonic timestamps
    pub fn with_timestamps(request: &EchoRequestPayload, recv_ns: u64, send_ns: u64) -> Self {
        Self {
            sequence: request.sequence,
            client_send_timestamp: request.client_timestamp,
            server_recv_timestamp: recv_ns,
            server_send_timestamp: send_ns,
        }
    }
    
    /// Update server send timestamp (monotonic)
    pub fn set_send_timestamp_monotonic(&mut self, send_ns: u64) {
        self.server_send_timestamp = send_ns;
    }
    
    /// Update server send timestamp to current time (DEPRECATED - use monotonic)
    #[deprecated(note = "Use monotonic clock instead of SystemTime")]
    pub fn set_send_timestamp(&mut self) {
        self.server_send_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(28);
        bytes.extend_from_slice(&self.sequence.to_be_bytes());
        bytes.extend_from_slice(&self.client_send_timestamp.to_be_bytes());
        bytes.extend_from_slice(&self.server_recv_timestamp.to_be_bytes());
        bytes.extend_from_slice(&self.server_send_timestamp.to_be_bytes());
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < 28 {
            return Err(PacketError::TooShort);
        }
        let sequence = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let client_send_timestamp = u64::from_be_bytes([
            bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11],
        ]);
        let server_recv_timestamp = u64::from_be_bytes([
            bytes[12], bytes[13], bytes[14], bytes[15],
            bytes[16], bytes[17], bytes[18], bytes[19],
        ]);
        let server_send_timestamp = u64::from_be_bytes([
            bytes[20], bytes[21], bytes[22], bytes[23],
            bytes[24], bytes[25], bytes[26], bytes[27],
        ]);
        Ok(Self { 
            sequence, 
            client_send_timestamp, 
            server_recv_timestamp,
            server_send_timestamp,
        })
    }
}

/// THROUGHPUT_START packet payload (for upload testing)
#[derive(Debug, Clone)]
pub struct ThroughputStartPayload {
    /// Test ID
    pub test_id: u32,
    /// Expected total size in bytes
    pub total_size: u64,
}

impl ThroughputStartPayload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(12);
        bytes.extend_from_slice(&self.test_id.to_be_bytes());
        bytes.extend_from_slice(&self.total_size.to_be_bytes());
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < 12 {
            return Err(PacketError::TooShort);
        }
        let test_id = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let total_size = u64::from_be_bytes([
            bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11],
        ]);
        Ok(Self { test_id, total_size })
    }
}

/// THROUGHPUT_DATA packet payload
#[derive(Debug, Clone)]
pub struct ThroughputDataPayload {
    /// Test ID
    pub test_id: u32,
    /// Sequence number
    pub sequence: u32,
    /// Data chunk (variable size)
    pub data: Vec<u8>,
}

impl ThroughputDataPayload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.data.len());
        bytes.extend_from_slice(&self.test_id.to_be_bytes());
        bytes.extend_from_slice(&self.sequence.to_be_bytes());
        bytes.extend_from_slice(&self.data);
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < 8 {
            return Err(PacketError::TooShort);
        }
        let test_id = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let sequence = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let data = bytes[8..].to_vec();
        Ok(Self { test_id, sequence, data })
    }
}

/// THROUGHPUT_END packet payload
#[derive(Debug, Clone)]
pub struct ThroughputEndPayload {
    /// Test ID
    pub test_id: u32,
    /// Total bytes sent
    pub total_bytes: u64,
}

impl ThroughputEndPayload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(12);
        bytes.extend_from_slice(&self.test_id.to_be_bytes());
        bytes.extend_from_slice(&self.total_bytes.to_be_bytes());
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < 12 {
            return Err(PacketError::TooShort);
        }
        let test_id = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let total_bytes = u64::from_be_bytes([
            bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11],
        ]);
        Ok(Self { test_id, total_bytes })
    }
}

/// THROUGHPUT_STATS packet payload (server response)
#[derive(Debug, Clone)]
pub struct ThroughputStatsPayload {
    /// Test ID
    pub test_id: u32,
    /// Total bytes received
    pub total_bytes: u64,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Throughput in kbps
    pub throughput_kbps: u32,
    /// Packet loss percentage
    pub packet_loss_pct: f32,
}

impl ThroughputStatsPayload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(24);
        bytes.extend_from_slice(&self.test_id.to_be_bytes());
        bytes.extend_from_slice(&self.total_bytes.to_be_bytes());
        bytes.extend_from_slice(&self.duration_ms.to_be_bytes());
        bytes.extend_from_slice(&self.throughput_kbps.to_be_bytes());
        bytes.extend_from_slice(&self.packet_loss_pct.to_be_bytes());
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < 24 {
            return Err(PacketError::TooShort);
        }
        let test_id = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let total_bytes = u64::from_be_bytes([
            bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11],
        ]);
        let duration_ms = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let throughput_kbps = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let packet_loss_pct = f32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        Ok(Self {
            test_id,
            total_bytes,
            duration_ms,
            throughput_kbps,
            packet_loss_pct,
        })
    }
}


