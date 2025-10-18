//! Port knocking handler

use crate::session::SessionManager;
use protocol::{
    crypto,
    packets::{
        KnockAckPayload, KnockPayload, PacketHeader, PacketType,
    },
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info};

/// Handle KNOCK packet
///
/// This authenticates a client and creates a session
pub async fn handle_knock(
    payload: &[u8],
    header: &PacketHeader,
    client_addr: SocketAddr,
    shared_secret: &[u8; 32],
    session_manager: Arc<SessionManager>,
) -> Result<Vec<u8>, String> {
    // Decrypt knock payload
    let nonce = header.nonce();
    let header_bytes = header.to_bytes();
    
    let decrypted = crypto::decrypt(payload, shared_secret, &nonce, &header_bytes)
        .map_err(|e| format!("Knock decryption failed: {}", e))?;
    
    // Parse knock payload
    let knock = KnockPayload::from_bytes(&decrypted)
        .map_err(|e| format!("Invalid knock payload: {}", e))?;
    
    debug!(
        "Received valid KNOCK from client_id={}, addr={}",
        header.client_id, client_addr
    );
    
    // Create session
    let session_id = session_manager
        .create_session(header.client_id, client_addr)
        .await;
    
    info!(
        "Created session {} for client {} ({})",
        session_id, header.client_id, client_addr
    );
    
    // Prepare KNOCK_ACK response
    // Challenge response is SHA256 of client challenge
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&knock.challenge);
    let challenge_response: [u8; 32] = hasher.finalize().into();
    
    let ack_payload = KnockAckPayload {
        session_id,
        challenge_response,
    };
    
    // Build response packet
    let ack_bytes = ack_payload.to_bytes();
    let response_header = PacketHeader::new(
        PacketType::KnockAck,
        (ack_bytes.len() + crypto::TAG_SIZE) as u16,
        header.client_id,
    );
    
    // Encrypt response
    let response_nonce = response_header.nonce();
    let response_header_bytes = response_header.to_bytes();
    
    let encrypted = crypto::encrypt(&ack_bytes, shared_secret, &response_nonce, &response_header_bytes)
        .map_err(|e| format!("Failed to encrypt response: {}", e))?;
    
    // Combine header + encrypted payload
    let mut response = Vec::with_capacity(PacketHeader::SIZE + encrypted.len());
    response.extend_from_slice(&response_header_bytes);
    response.extend_from_slice(&encrypted);
    
    Ok(response)
}

