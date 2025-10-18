//! Bufferbane Server - Network quality monitoring server

mod config;
mod handlers;
mod session;

use anyhow::{Context, Result};
use clap::Parser;
use protocol::{
    crypto,
    packets::{PacketHeader, PacketType},
};
use session::SessionManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{debug, error, info, warn};

#[derive(Parser, Debug)]
#[command(author = "Florian Schüller <schuellerf@gmail.com>")]
#[command(version)]
#[command(about = "Bufferbane server - network quality monitoring", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "server.conf")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();
    
    info!("Starting Bufferbane server v{}", env!("CARGO_PKG_VERSION"));
    
    // Load configuration
    let config = config::Config::load(&args.config)
        .context("Failed to load configuration")?;
    
    // Parse shared secret
    let shared_secret = crypto::parse_shared_secret(&config.security.shared_secret)
        .map_err(|e| anyhow::anyhow!("Invalid shared secret in configuration: {}", e))?;
    
    info!(
        "Loaded configuration from: {}",
        args.config
    );
    
    // Create session manager
    let session_manager = Arc::new(SessionManager::new(
        config.security.session_timeout_sec,
    ));
    
    // Bind UDP socket
    let bind_addr = format!("{}:{}", config.general.bind_address, config.general.bind_port);
    let socket = Arc::new(
        UdpSocket::bind(&bind_addr)
            .await
            .context(format!("Failed to bind to {}", bind_addr))?
    );
    
    info!("Server listening on {}", bind_addr);
    info!("Max concurrent clients: {}", config.general.max_concurrent_clients);
    info!("Session timeout: {} seconds", config.security.session_timeout_sec);
    
    // Spawn cleanup task
    let cleanup_session_manager = session_manager.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            cleanup_session_manager.cleanup_expired().await;
            let active = cleanup_session_manager.active_sessions().await;
            if active > 0 {
                debug!("Active sessions: {}", active);
            }
        }
    });
    
    // Main server loop
    let mut buf = vec![0u8; 65535]; // Max UDP packet size
    
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, client_addr)) => {
                let data = buf[..len].to_vec();
                let socket_clone = socket.clone();
                let session_manager_clone = session_manager.clone();
                let shared_secret_clone = shared_secret;
                
                // Spawn task to handle packet
                tokio::spawn(async move {
                    if let Some(response) = handle_packet(
                        &data,
                        client_addr,
                        shared_secret_clone,
                        session_manager_clone,
                    )
                    .await
                    {
                        if let Err(e) = socket_clone.send_to(&response, client_addr).await {
                            error!("Failed to send response to {}: {}", client_addr, e);
                        }
                    }
                });
            }
            Err(e) => {
                error!("Error receiving packet: {}", e);
            }
        }
    }
}

/// Handle a received packet
async fn handle_packet(
    data: &[u8],
    client_addr: SocketAddr,
    shared_secret: [u8; 32],
    session_manager: Arc<SessionManager>,
) -> Option<Vec<u8>> {
    // Parse packet header
    let header = match PacketHeader::from_bytes(data) {
        Ok(h) => h,
        Err(e) => {
            debug!("Invalid packet header from {}: {}", client_addr, e);
            return None; // Silent drop
        }
    };
    
    // Check payload length
    if data.len() < PacketHeader::SIZE + header.payload_len as usize {
        debug!("Incomplete packet from {}", client_addr);
        return None; // Silent drop
    }
    
    let payload = &data[PacketHeader::SIZE..PacketHeader::SIZE + header.payload_len as usize];
    
    // Dispatch based on packet type
    match header.packet_type {
        PacketType::Knock => {
            // KNOCK always allowed (authentication)
            match handlers::handle_knock(
                payload,
                &header,
                client_addr,
                &shared_secret,
                session_manager,
            )
            .await
            {
                Ok(response) => Some(response),
                Err(e) => {
                    warn!("KNOCK failed from {}: {}", client_addr, e);
                    None // Silent drop on authentication failure
                }
            }
        }
        
        PacketType::EchoRequest => {
            // ECHO_REQUEST requires valid session
            // For MVP, we'll allow it without strict session validation
            match handlers::handle_echo_request(
                payload,
                &header,
                client_addr,
                &shared_secret,
                session_manager,
            )
            .await
            {
                Ok(response) => Some(response),
                Err(e) => {
                    debug!("ECHO_REQUEST failed from {}: {}", client_addr, e);
                    None
                }
            }
        }
        
        PacketType::ThroughputStart => {
            // Throughput test
            match handlers::handle_throughput(
                payload,
                &header,
                client_addr,
                &shared_secret,
                session_manager,
            )
            .await
            {
                Ok(response) => response,
                Err(e) => {
                    debug!("THROUGHPUT_START failed from {}: {}", client_addr, e);
                    None
                }
            }
        }
        
        _ => {
            debug!("Unsupported packet type: {:?}", header.packet_type);
            None // Silent drop
        }
    }
}

