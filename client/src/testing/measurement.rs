//! Measurement data structures

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    /// Unix timestamp in seconds
    pub timestamp: i64,
    
    /// Monotonic timestamp in nanoseconds (for precise timing)
    pub monotonic_ns: u128,
    
    /// Network interface used (e.g., "wlan0", "eth0")
    pub interface: String,
    
    /// Connection type tag (e.g., "wifi", "wired")
    pub connection_type: String,
    
    /// Test type: "icmp", "udp_echo", "throughput_up", etc.
    pub test_type: String,
    
    /// Target address
    pub target: String,
    
    /// Server name (for server-based tests, None for ICMP)
    pub server_name: Option<String>,
    
    /// Round-trip time in milliseconds (None if packet lost)
    pub rtt_ms: Option<f64>,
    
    /// Jitter in milliseconds (calculated from previous measurements)
    pub jitter_ms: Option<f64>,
    
    /// Packet loss percentage (for batch tests)
    pub packet_loss_pct: Option<f64>,
    
    /// Throughput in Kbps
    pub throughput_kbps: Option<f64>,
    
    /// DNS resolution time in milliseconds
    pub dns_time_ms: Option<f64>,
    
    /// Status: "success", "timeout", "error"
    pub status: String,
    
    /// Error detail if status is "error"
    pub error_detail: Option<String>,
    
    /// Upload latency in milliseconds (client → server, for server tests only)
    pub upload_latency_ms: Option<f64>,
    
    /// Download latency in milliseconds (server → client, for server tests only)
    pub download_latency_ms: Option<f64>,
    
    /// Server processing time in microseconds (for server tests only)
    pub server_processing_us: Option<i64>,
    
    /// Sync event information (if a sync state change occurred)
    pub sync_event: Option<SyncEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    pub event_type: String,  // "sync_established", "sync_lost", "sync_invalid"
    pub message: String,
    pub quality: Option<u8>,
}

impl Measurement {
    pub fn new_icmp(
        target: String,
        interface: String,
        connection_type: String,
    ) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let monotonic_ns = std::time::Instant::now().elapsed().as_nanos();
        
        Self {
            timestamp,
            monotonic_ns,
            interface,
            connection_type,
            test_type: "icmp".to_string(),
            target,
            server_name: None,
            rtt_ms: None,
            jitter_ms: None,
            packet_loss_pct: None,
            throughput_kbps: None,
            dns_time_ms: None,
            status: "pending".to_string(),
            error_detail: None,
            upload_latency_ms: None,
            download_latency_ms: None,
            server_processing_us: None,
            sync_event: None,
        }
    }
    
    pub fn new_server_echo(
        target: String,
        interface: String,
        connection_type: String,
    ) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let monotonic_ns = std::time::Instant::now().elapsed().as_nanos();
        
        Self {
            timestamp,
            monotonic_ns,
            interface,
            connection_type,
            test_type: "server_echo".to_string(),
            target,
            server_name: None,
            rtt_ms: None,
            jitter_ms: None,
            packet_loss_pct: None,
            throughput_kbps: None,
            dns_time_ms: None,
            status: "pending".to_string(),
            error_detail: None,
            upload_latency_ms: None,
            download_latency_ms: None,
            server_processing_us: None,
            sync_event: None,
        }
    }
    
    pub fn set_success(&mut self, rtt_ms: f64) {
        self.rtt_ms = Some(rtt_ms);
        self.status = "success".to_string();
    }
    
    pub fn set_timeout(&mut self) {
        self.status = "timeout".to_string();
    }
    
    pub fn set_error(&mut self, error: String) {
        self.status = "error".to_string();
        self.error_detail = Some(error);
    }
}

