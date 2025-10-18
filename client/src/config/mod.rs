//! Configuration management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub targets: TargetsConfig,
    #[serde(default)]
    pub server: Option<ServerConfig>,
    pub alerts: AlertsConfig,
    pub retention: RetentionConfig,
    pub output: OutputConfig,
    pub export: ExportConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub shared_secret: String,
    #[serde(default)]
    pub client_id: u64,
    #[serde(default = "default_knock_retry_attempts")]
    pub knock_retry_attempts: u32,
    #[serde(default = "default_knock_timeout_ms")]
    pub knock_timeout_ms: u64,
    #[serde(default = "default_true")]
    pub enable_echo_test: bool,
    #[serde(default)]
    pub enable_throughput_test: bool,
    #[serde(default)]
    pub enable_download_test: bool,
    #[serde(default)]
    pub enable_bufferbloat_test: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneralConfig {
    pub test_interval_ms: u64,
    pub database_path: String,
    pub client_id: String,
    #[serde(default)]
    pub interfaces: Vec<String>,
    #[serde(default = "default_connection_type")]
    pub connection_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TargetsConfig {
    pub isp_gateway: String,
    pub public_dns: Vec<String>,
    pub custom: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlertsConfig {
    pub enabled: bool,
    pub log_path: String,
    pub latency_threshold_ms: f64,
    pub jitter_threshold_ms: f64,
    pub packet_loss_threshold_pct: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetentionConfig {
    pub measurements_days: u32,
    pub aggregations_days: u32,
    pub events_days: u32,
    pub cleanup_time: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputConfig {
    pub refresh_interval_ms: u64,
    pub stats_windows_s: Vec<u64>,
    pub percentiles: Vec<u8>,
    pub use_colors: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportConfig {
    pub enable_csv: bool,
    pub enable_json: bool,
    pub enable_charts: bool,
    pub chart_width: u32,
    pub chart_height: u32,
    pub chart_dpi: u32,
    pub chart_style: String,
    pub export_directory: String,
    pub default_charts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub path: String,
    pub max_size_mb: u32,
    pub max_files: u32,
}

fn default_connection_type() -> String {
    "auto".to_string()
}

fn default_knock_retry_attempts() -> u32 {
    3
}

fn default_knock_timeout_ms() -> u64 {
    2000
}

fn default_true() -> bool {
    true
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
        
        let mut config: Config = toml::from_str(&contents)
            .with_context(|| "Failed to parse config file")?;
        
        // Auto-generate client ID if needed
        if config.general.client_id == "auto" {
            config.general.client_id = generate_client_id();
        }
        
        // Auto-detect connection type if needed
        if config.general.connection_type == "auto" && config.general.interfaces.is_empty() {
            config.general.connection_type = detect_connection_type();
        }
        
        Ok(config)
    }
}

fn generate_client_id() -> String {
    use std::time::SystemTime;
    
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    
    format!("{:016x}", now.as_nanos() & 0xFFFFFFFFFFFFFFFF)
}

fn detect_connection_type() -> String {
    // Try to detect interface type based on default route
    // For Phase 1, we'll just return "unknown"
    // This will be enhanced in Phase 4
    "unknown".to_string()
}

