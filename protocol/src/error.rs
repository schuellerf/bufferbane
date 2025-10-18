//! Protocol error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid magic bytes")]
    InvalidMagicBytes,
    
    #[error("Invalid protocol version: {0}")]
    InvalidVersion(u8),
    
    #[error("Invalid packet type: {0}")]
    InvalidPacketType(u8),
    
    #[error("Packet too small: expected at least {expected}, got {actual}")]
    PacketTooSmall { expected: usize, actual: usize },
    
    #[error("Packet too large: maximum {max}, got {actual}")]
    PacketTooLarge { max: usize, actual: usize },
    
    #[error("Encryption failed")]
    EncryptionFailed,
    
    #[error("Decryption failed")]
    DecryptionFailed,
    
    #[error("Invalid nonce")]
    InvalidNonce,
    
    #[error("Replay attack detected")]
    ReplayDetected,
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

