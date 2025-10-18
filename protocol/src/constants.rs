//! Protocol constants and packet type definitions

/// Packet types for Bufferbane protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    /// Port knocking packet (Phase 2)
    Knock = 0x01,
    
    /// Echo request (ping-like)
    EchoRequest = 0x02,
    
    /// Echo reply
    EchoReply = 0x03,
    
    /// Start throughput upload test (Phase 2)
    ThroughputStart = 0x04,
    
    /// Throughput data packet (Phase 2)
    ThroughputData = 0x05,
    
    /// End throughput test (Phase 2)
    ThroughputEnd = 0x06,
    
    /// Throughput statistics from server (Phase 2)
    ThroughputStats = 0x07,
    
    /// Request download test (Phase 2)
    DownloadRequest = 0x08,
    
    /// Download data (Phase 2)
    DownloadData = 0x09,
    
    /// Download complete (Phase 2)
    DownloadEnd = 0x0A,
    
    /// Start bufferbloat test (Phase 2)
    BufferbloatStart = 0x0B,
    
    /// End bufferbloat test (Phase 2)
    BufferbloatEnd = 0x0C,
    
    /// Error packet
    Error = 0xFF,
}

impl PacketType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(PacketType::Knock),
            0x02 => Some(PacketType::EchoRequest),
            0x03 => Some(PacketType::EchoReply),
            0x04 => Some(PacketType::ThroughputStart),
            0x05 => Some(PacketType::ThroughputData),
            0x06 => Some(PacketType::ThroughputEnd),
            0x07 => Some(PacketType::ThroughputStats),
            0x08 => Some(PacketType::DownloadRequest),
            0x09 => Some(PacketType::DownloadData),
            0x0A => Some(PacketType::DownloadEnd),
            0x0B => Some(PacketType::BufferbloatStart),
            0x0C => Some(PacketType::BufferbloatEnd),
            0xFF => Some(PacketType::Error),
            _ => None,
        }
    }
    
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Port knocking sequence
pub const KNOCK_SEQUENCE: [u16; 4] = [12345, 23456, 34567, 45678];

/// Knock timeout window (nanoseconds) - 60 seconds
pub const KNOCK_TIMEOUT_NS: u64 = 60_000_000_000;

