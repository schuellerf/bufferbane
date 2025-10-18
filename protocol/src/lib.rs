//! Bufferbane Protocol Library
//!
//! Shared protocol definitions for Bufferbane client-server communication.
//! This includes packet types, constants, and serialization/deserialization logic.

pub mod constants;
pub mod error;
pub mod packets;
pub mod crypto;

pub use constants::*;
pub use error::ProtocolError;

/// Protocol version
pub const PROTOCOL_VERSION: u8 = 1;

/// Magic bytes: "BFBN" (0x4246424E)
pub const MAGIC_BYTES: [u8; 4] = [0x42, 0x46, 0x42, 0x4E];

/// Maximum packet size (64 KB)
pub const MAX_PACKET_SIZE: usize = 65536;

/// Minimum packet size (header only)
pub const MIN_PACKET_SIZE: usize = 24;
