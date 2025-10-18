//! Protocol constants

/// Magic bytes "BFBN" (0x4246424E)
pub const MAGIC_BYTES: u32 = 0x4246424E;

/// Port knocking sequence
pub const KNOCK_SEQUENCE: [u16; 4] = [12345, 23456, 34567, 45678];

/// Knock timeout window (nanoseconds) - 60 seconds
pub const KNOCK_TIMEOUT_NS: u64 = 60_000_000_000;

