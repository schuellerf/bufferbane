//! Server configuration

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub security: SecurityConfig,
    pub rate_limiting: RateLimitingConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeneralConfig {
    pub bind_address: String,
    pub bind_port: u16,
    pub max_concurrent_clients: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub shared_secret: String,  // Hex-encoded 32-byte secret
    pub knock_timeout_sec: u64,
    pub session_timeout_sec: u64,
    pub enable_rate_limiting: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitingConfig {
    pub max_packets_per_second: usize,
    pub max_bandwidth_mbps: usize,
    pub burst_size: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub log_successful_knocks: bool,
    pub log_failed_knocks: bool,
    pub log_echo_requests: bool,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read config file")?;
        
        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;
        
        // Validate shared secret format
        if config.security.shared_secret.len() != 64 {
            anyhow::bail!("shared_secret must be exactly 64 hex characters (32 bytes)");
        }
        
        Ok(config)
    }
}

