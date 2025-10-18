//! Throughput testing handler (upload/download)

use crate::session::SessionManager;
use protocol::{
    crypto,
    packets::{PacketHeader, ThroughputStartPayload},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info};

/// Handle throughput-related packets
///
/// This is a simplified handler for Phase 2
pub async fn handle_throughput(
    payload: &[u8],
    header: &PacketHeader,
    _client_addr: SocketAddr,
    shared_secret: &[u8; 32],
    session_manager: Arc<SessionManager>,
) -> Result<Option<Vec<u8>>, String> {
    // Decrypt payload
    let nonce = header.nonce();
    let header_bytes = header.to_bytes();
    
    let decrypted = crypto::decrypt(payload, shared_secret, &nonce, &header_bytes)
        .map_err(|e| format!("Throughput decryption failed: {}", e))?;
    
    // For now, just parse THROUGHPUT_START and log it
    let start = ThroughputStartPayload::from_bytes(&decrypted)
        .map_err(|e| format!("Invalid throughput start: {}", e))?;
    
    info!(
        "Throughput test started: test_id={}, total_size={} bytes",
        start.test_id, start.total_size
    );
    
    // Update session
    session_manager.update_last_seen(header.client_id).await;
    
    // TODO: Implement full throughput testing in later implementation
    // For Phase 2 MVP, we're focusing on knock + echo first
    
    debug!("Throughput testing not fully implemented yet");
    
    Ok(None) // No response for now
}

