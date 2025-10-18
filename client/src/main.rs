//! Bufferbane - Network Quality Monitoring Client
//!
//! Phase 1: Standalone mode with ICMP latency monitoring and basic chart export

mod config;
mod testing;
mod storage;
mod analysis;
mod output;
mod charts;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, error, warn};

#[derive(Parser, Debug)]
#[command(name = "bufferbane")]
#[command(author = "Florian Schüller <schuellerf@gmail.com>")]
#[command(version = "0.1.0")]
#[command(about = "Network quality monitoring for cable internet", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "client.conf")]
    config: PathBuf,
    
    /// Export data for time range
    #[arg(long)]
    export: bool,
    
    /// Generate chart
    #[arg(long)]
    chart: bool,
    
    /// Output file for export/chart
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Time range: --last 24h, 7d, etc.
    #[arg(long)]
    last: Option<String>,
    
    /// Start time for range: YYYY-MM-DD HH:MM
    #[arg(long)]
    start: Option<String>,
    
    /// End time for range: YYYY-MM-DD HH:MM
    #[arg(long)]
    end: Option<String>,
    
    /// Generate interactive HTML chart instead of static PNG
    #[arg(long)]
    interactive: bool,
    
    /// Number of time segments for chart aggregation (default: 100)
    #[arg(long, default_value = "100")]
    segments: usize,
    
    /// Quiet mode: Log hourly statistics instead of every ping (for systemd service)
    #[arg(short, long)]
    quiet: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();
    
    let args = Args::parse();
    
    info!("Bufferbane v0.1.0 - Network Quality Monitoring");
    info!("Phase 1: Standalone ICMP monitoring with chart export");
    
    // Load configuration
    let config = config::Config::load(&args.config)?;
    info!("Loaded configuration from {:?}", args.config);
    
    // Handle different modes
    if args.export {
        // Export mode
        info!("Export mode");
        run_export(&config, &args).await?;
    } else if args.chart {
        // Chart generation mode
        info!("Chart generation mode");
        run_chart(&config, &args).await?;
    } else {
        // Monitoring mode (default)
        info!("Starting monitoring mode");
        run_monitoring(&config, args.quiet).await?;
    }
    
    Ok(())
}

async fn run_monitoring(config: &config::Config, quiet: bool) -> Result<()> {
    info!("Initializing monitoring...");
    info!("Test interval: {}ms", config.general.test_interval_ms);
    info!("Database: {:?}", config.general.database_path);
    if quiet {
        info!("Quiet mode: Logging hourly statistics only");
    }
    
    // Initialize database
    let db = storage::Database::new(&config.general.database_path)?;
    db.initialize()?;
    info!("Database initialized");
    
    // Initialize ICMP tester
    let config_arc = std::sync::Arc::new(config.clone());
    let tester = testing::IcmpTester::new(config_arc.clone())?;
    info!("ICMP tester initialized");
    
    // Initialize server tester (Phase 2) if enabled
    let server_tester = if let Some(ref server_config) = config.server {
        if server_config.enabled {
            let server_config_arc = std::sync::Arc::new(server_config.clone());
            match testing::ServerTester::new(
                server_config_arc,
                config.general.interfaces.first().cloned().unwrap_or_else(|| "default".to_string()),
                config.general.connection_type.clone(),
            ) {
                Ok(mut st) => {
                    // Authenticate with server
                    match st.authenticate() {
                        Ok(_) => {
                            info!("Server tester initialized and authenticated");
                            Some(st)
                        }
                        Err(e) => {
                            error!("Failed to authenticate with server: {}", e);
                            warn!("Continuing with ICMP-only mode");
                            None
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to initialize server tester: {}", e);
                    warn!("Continuing with ICMP-only mode");
                    None
                }
            }
        } else {
            info!("Server testing disabled in configuration");
            None
        }
    } else {
        None
    };
    
    // Initialize output
    let output_handle = output::OutputManager::new(config.clone());
    
    // Initialize alert system
    let alert_manager = analysis::AlertManager::new(config.clone());
    
    // Start monitoring loop
    info!("Starting monitoring loop (Press Ctrl+C to stop)");
    
    let mut interval = tokio::time::interval(
        tokio::time::Duration::from_millis(config.general.test_interval_ms)
    );
    
    // For hourly statistics in quiet mode
    let mut hourly_stats = HourlyStats::new();
    let mut last_stats_log = chrono::Local::now();
    
    // Make server_tester mutable for running tests
    let mut server_tester = server_tester;
    
    loop {
        interval.tick().await;
        
        let mut all_measurements = Vec::new();
        
        // Run ICMP tests
        match tester.run_tests().await {
            Ok(measurements) => {
                all_measurements.extend(measurements);
            }
            Err(e) => {
                error!("ICMP test failed: {}", e);
            }
        }
        
        // Run server tests (Phase 2) if available
        if let Some(ref mut st) = server_tester {
            match st.run_test() {
                Ok(measurements) => {
                    all_measurements.extend(measurements);
                }
                Err(e) => {
                    error!("Server test failed: {}", e);
                    // Try to re-authenticate on next iteration
                    if let Err(auth_err) = st.authenticate() {
                        error!("Re-authentication failed: {}", auth_err);
                        // Disable server testing for this session
                        server_tester = None;
                        warn!("Server testing disabled due to authentication failure");
                    }
                }
            }
        }
        
        if !all_measurements.is_empty() {
            // Store measurements
            for measurement in &all_measurements {
                if let Err(e) = db.store_measurement(measurement) {
                    error!("Failed to store measurement: {}", e);
                }
            }
            
            // Check for alerts
            if let Err(e) = alert_manager.check_measurements(&all_measurements) {
                error!("Alert check failed: {}", e);
            }
            
            // Update output or statistics
            if quiet {
                // Quiet mode: accumulate stats
                hourly_stats.add_measurements(&all_measurements);
                
                // Log hourly statistics
                let now = chrono::Local::now();
                if now.signed_duration_since(last_stats_log).num_minutes() >= 60 {
                    hourly_stats.log_and_reset();
                    last_stats_log = now;
                }
            } else {
                // Normal mode: show every measurement
                if let Err(e) = output_handle.update(&all_measurements) {
                    error!("Output update failed: {}", e);
                }
            }
        }
    }
}

async fn run_export(config: &config::Config, args: &Args) -> Result<()> {
    info!("Running export...");
    
    // Determine time range
    let (start, end) = parse_time_range(args)?;
    
    // Initialize database
    let db = storage::Database::new(&config.general.database_path)?;
    
    // Query measurements
    let measurements = db.query_range(start, end)?;
    
    info!("Found {} measurements", measurements.len());
    
    // Determine output file
    let output_path = args.output.clone().unwrap_or_else(|| {
        PathBuf::from(format!("bufferbane_export_{}.csv", chrono::Local::now().format("%Y%m%d_%H%M%S")))
    });
    
    // Export as CSV
    output::export_csv(&measurements, &output_path)?;
    
    info!("Exported to {:?}", output_path);
    
    Ok(())
}

async fn run_chart(config: &config::Config, args: &Args) -> Result<()> {
    info!("Generating chart...");
    
    // Determine time range
    let (start, end) = parse_time_range(args)?;
    
    // Initialize database
    let db = storage::Database::new(&config.general.database_path)?;
    
    // Query measurements
    let measurements = db.query_range(start, end)?;
    
    info!("Found {} measurements", measurements.len());
    
    if measurements.is_empty() {
        anyhow::bail!("No measurements found for the specified time range");
    }
    
    // Determine output file
    let output_path = args.output.clone().unwrap_or_else(|| {
        if args.interactive {
            PathBuf::from(format!("latency_{}.html", chrono::Local::now().format("%Y%m%d_%H%M%S")))
        } else {
            PathBuf::from(format!("latency_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S")))
        }
    });
    
    // Generate chart with min/max/avg/percentile lines
    info!("Using {} time segments for aggregation", args.segments);
    if args.interactive {
        charts::generate_interactive_chart(&measurements, &output_path, config, args.segments)?;
        info!("Interactive chart saved to {:?}", output_path);
        info!("Open the file in your web browser to view the interactive chart");
    } else {
        charts::generate_latency_chart(&measurements, &output_path, config, args.segments)?;
        info!("Chart saved to {:?}", output_path);
    }
    
    Ok(())
}

fn parse_time_range(args: &Args) -> Result<(i64, i64)> {
    if let Some(last) = &args.last {
        // Parse --last 24h, 7d, etc.
        let duration = parse_duration(last)?;
        let end = chrono::Utc::now().timestamp();
        let start = end - duration.num_seconds();
        Ok((start, end))
    } else if let (Some(start_str), Some(end_str)) = (&args.start, &args.end) {
        // Parse --start and --end
        let start = chrono::NaiveDateTime::parse_from_str(start_str, "%Y-%m-%d %H:%M")?
            .and_utc()
            .timestamp();
        let end = chrono::NaiveDateTime::parse_from_str(end_str, "%Y-%m-%d %H:%M")?
            .and_utc()
            .timestamp();
        Ok((start, end))
    } else {
        // Default: last 24 hours
        let end = chrono::Utc::now().timestamp();
        let start = end - 24 * 3600;
        Ok((start, end))
    }
}

fn parse_duration(s: &str) -> Result<chrono::Duration> {
    let s = s.trim();
    if s.ends_with('h') {
        let hours: i64 = s[..s.len()-1].parse()?;
        Ok(chrono::Duration::hours(hours))
    } else if s.ends_with('d') {
        let days: i64 = s[..s.len()-1].parse()?;
        Ok(chrono::Duration::days(days))
    } else if s.ends_with('m') {
        let minutes: i64 = s[..s.len()-1].parse()?;
        Ok(chrono::Duration::minutes(minutes))
    } else {
        anyhow::bail!("Invalid duration format. Use: 24h, 7d, 30m, etc.")
    }
}

/// Accumulates measurements for hourly statistics in quiet mode
struct HourlyStats {
    measurements_per_target: std::collections::HashMap<String, TargetStats>,
    total_measurements: usize,
    failed_measurements: usize,
}

struct TargetStats {
    rtts: Vec<f64>,
    jitters: Vec<f64>,
    packet_loss_count: usize,
    success_count: usize,
}

impl HourlyStats {
    fn new() -> Self {
        Self {
            measurements_per_target: std::collections::HashMap::new(),
            total_measurements: 0,
            failed_measurements: 0,
        }
    }
    
    fn add_measurements(&mut self, measurements: &[testing::Measurement]) {
        for m in measurements {
            self.total_measurements += 1;
            
            let target_stats = self.measurements_per_target
                .entry(m.target.clone())
                .or_insert_with(|| TargetStats {
                    rtts: Vec::new(),
                    jitters: Vec::new(),
                    packet_loss_count: 0,
                    success_count: 0,
                });
            
            if m.status == "success" {
                target_stats.success_count += 1;
                if let Some(rtt) = m.rtt_ms {
                    target_stats.rtts.push(rtt);
                }
                if let Some(jitter) = m.jitter_ms {
                    target_stats.jitters.push(jitter);
                }
            } else {
                target_stats.packet_loss_count += 1;
                self.failed_measurements += 1;
            }
        }
    }
    
    fn log_and_reset(&mut self) {
        if self.total_measurements == 0 {
            info!("Hourly stats: No measurements");
            return;
        }
        
        info!("═══ Hourly Statistics ═══");
        info!("Total measurements: {} (failed: {})", 
              self.total_measurements, self.failed_measurements);
        
        for (target, stats) in &self.measurements_per_target {
            if stats.success_count == 0 {
                info!("  {}: NO SUCCESSFUL MEASUREMENTS ({}% loss)",
                      target,
                      stats.packet_loss_count * 100 / (stats.success_count + stats.packet_loss_count));
                continue;
            }
            
            let total_tests = stats.success_count + stats.packet_loss_count;
            let loss_pct = if total_tests > 0 {
                (stats.packet_loss_count as f64 / total_tests as f64) * 100.0
            } else {
                0.0
            };
            
            // Calculate RTT statistics
            let (min_rtt, max_rtt, avg_rtt, p95_rtt) = if !stats.rtts.is_empty() {
                let mut sorted_rtts = stats.rtts.clone();
                sorted_rtts.sort_by(|a, b| a.partial_cmp(b).unwrap());
                
                let min = sorted_rtts[0];
                let max = sorted_rtts[sorted_rtts.len() - 1];
                let avg = sorted_rtts.iter().sum::<f64>() / sorted_rtts.len() as f64;
                let p95_idx = (sorted_rtts.len() as f64 * 0.95) as usize;
                let p95 = sorted_rtts.get(p95_idx).copied().unwrap_or(max);
                
                (min, max, avg, p95)
            } else {
                (0.0, 0.0, 0.0, 0.0)
            };
            
            // Calculate jitter statistics
            let avg_jitter = if !stats.jitters.is_empty() {
                stats.jitters.iter().sum::<f64>() / stats.jitters.len() as f64
            } else {
                0.0
            };
            
            info!("  {}: {} tests, {:.1}% loss", target, total_tests, loss_pct);
            info!("    RTT: min={:.2}ms avg={:.2}ms max={:.2}ms p95={:.2}ms",
                  min_rtt, avg_rtt, max_rtt, p95_rtt);
            info!("    Jitter: avg={:.2}ms", avg_jitter);
        }
        
        info!("═══════════════════════");
        
        // Reset
        self.measurements_per_target.clear();
        self.total_measurements = 0;
        self.failed_measurements = 0;
    }
}
