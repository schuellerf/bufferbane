//! Echo request handler (latency testing)

use crate::session::SessionManager;
use protocol::{
    crypto,
    packets::{
        EchoReplyPayload, EchoRequestPayload, PacketHeader, PacketType,
    },
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

// Lazy static for server start time (monotonic reference)
use std::sync::OnceLock;
static SERVER_START: OnceLock<Instant> = OnceLock::new();

/// Get nanoseconds since server start (monotonic)
fn monotonic_ns() -> u64 {
    let start = SERVER_START.get_or_init(|| Instant::now());
    Instant::now().duration_since(*start).as_nanos() as u64
}

/// Handle ECHO_REQUEST packet
///
/// This echoes back the request with server timestamp for RTT calculation
pub async fn handle_echo_request(
    payload: &[u8],
    header: &PacketHeader,
    _client_addr: SocketAddr,
    shared_secret: &[u8; 32],
    session_manager: Arc<SessionManager>,
) -> Result<Vec<u8>, String> {
    // Decrypt echo request payload
    let nonce = header.nonce();
    let header_bytes = header.to_bytes();
    
    let decrypted = crypto::decrypt(payload, shared_secret, &nonce, &header_bytes)
        .map_err(|e| format!("Echo decryption failed: {}", e))?;
    
    // Parse echo request
    let request = EchoRequestPayload::from_bytes(&decrypted)
        .map_err(|e| format!("Invalid echo request: {}", e))?;
    
    debug!(
        "Received ECHO_REQUEST seq={} from client_id={}",
        request.sequence, header.client_id
    );
    
    // Update session stats
    session_manager.update_last_seen(header.client_id).await;
    
    // T2: Server receive time (monotonic, nanoseconds since server start)
    let t2_ns = monotonic_ns();
    
    // Create echo reply with T2 and T3 (T3 will be set just before sending)
    let mut reply = EchoReplyPayload::new(&request);
    reply.server_recv_timestamp = t2_ns;
    
    // T3: Server send timestamp (monotonic, set just before encrypting)
    let t3_ns = monotonic_ns();
    reply.server_send_timestamp = t3_ns;
    
    let reply_bytes = reply.to_bytes();
    
    // Build response packet
    let response_header = PacketHeader::new(
        PacketType::EchoReply,
        (reply_bytes.len() + crypto::TAG_SIZE) as u16,
        header.client_id,
    );
    
    // Encrypt response
    let response_nonce = response_header.nonce();
    let response_header_bytes = response_header.to_bytes();
    
    let encrypted = crypto::encrypt(&reply_bytes, shared_secret, &response_nonce, &response_header_bytes)
        .map_err(|e| format!("Failed to encrypt response: {}", e))?;
    
    // Combine header + encrypted payload
    let mut response = Vec::with_capacity(PacketHeader::SIZE + encrypted.len());
    response.extend_from_slice(&response_header_bytes);
    response.extend_from_slice(&encrypted);
    
    Ok(response)
}

