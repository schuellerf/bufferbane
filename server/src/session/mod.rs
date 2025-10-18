//! Session management for authenticated clients

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Client session information
#[derive(Debug, Clone)]
pub struct Session {
    #[allow(dead_code)]
    pub session_id: u64,
    #[allow(dead_code)]
    pub client_id: u64,
    #[allow(dead_code)]
    pub client_addr: SocketAddr,
    #[allow(dead_code)]
    pub authenticated_at: Instant,
    pub last_seen: Instant,
    pub packets_received: u64,
    #[allow(dead_code)]
    pub bytes_received: u64,
    #[allow(dead_code)]
    pub bytes_sent: u64,
}

/// Session manager
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<u64, Session>>>,
    session_timeout: Duration,
}

impl SessionManager {
    pub fn new(session_timeout_sec: u64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_timeout: Duration::from_secs(session_timeout_sec),
        }
    }
    
    /// Create a new session
    pub async fn create_session(
        &self,
        client_id: u64,
        client_addr: SocketAddr,
    ) -> u64 {
        // Generate random session ID (Send-safe)
        let session_id: u64 = rand::random();
        
        let session = Session {
            session_id,
            client_id,
            client_addr,
            authenticated_at: Instant::now(),
            last_seen: Instant::now(),
            packets_received: 0,
            bytes_received: 0,
            bytes_sent: 0,
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session);
        
        session_id
    }
    
    /// Get a session by session_id
    #[allow(dead_code)]
    pub async fn get_session(&self, session_id: u64) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(&session_id).cloned()
    }
    
    /// Update last_seen timestamp
    pub async fn update_last_seen(&self, session_id: u64) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.last_seen = Instant::now();
            session.packets_received += 1;
        }
    }
    
    /// Update statistics
    #[allow(dead_code)]
    pub async fn update_stats(&self, session_id: u64, bytes_received: u64, bytes_sent: u64) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.bytes_received += bytes_received;
            session.bytes_sent += bytes_sent;
        }
    }
    
    /// Remove expired sessions
    pub async fn cleanup_expired(&self) {
        let mut sessions = self.sessions.write().await;
        let now = Instant::now();
        
        sessions.retain(|_, session| {
            now.duration_since(session.last_seen) < self.session_timeout
        });
    }
    
    /// Get number of active sessions
    pub async fn active_sessions(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
    
    /// Check if a session exists and is valid
    #[allow(dead_code)]
    pub async fn is_valid(&self, session_id: u64) -> bool {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(&session_id) {
            let now = Instant::now();
            now.duration_since(session.last_seen) < self.session_timeout
        } else {
            false
        }
    }
}

