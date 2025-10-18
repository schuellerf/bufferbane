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
use tracing::{info, error};

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
        run_monitoring(&config).await?;
    }
    
    Ok(())
}

async fn run_monitoring(config: &config::Config) -> Result<()> {
    info!("Initializing monitoring...");
    info!("Test interval: {}ms", config.general.test_interval_ms);
    info!("Database: {:?}", config.general.database_path);
    
    // Initialize database
    let db = storage::Database::new(&config.general.database_path)?;
    db.initialize()?;
    info!("Database initialized");
    
    // Initialize ICMP tester
    let config_arc = std::sync::Arc::new(config.clone());
    let tester = testing::IcmpTester::new(config_arc)?;
    info!("ICMP tester initialized");
    
    // Initialize output
    let output_handle = output::OutputManager::new(config.clone());
    
    // Initialize alert system
    let alert_manager = analysis::AlertManager::new(config.clone());
    
    // Start monitoring loop
    info!("Starting monitoring loop (Press Ctrl+C to stop)");
    
    let mut interval = tokio::time::interval(
        tokio::time::Duration::from_millis(config.general.test_interval_ms)
    );
    
    loop {
        interval.tick().await;
        
        // Run tests
        match tester.run_tests().await {
            Ok(measurements) => {
                // Store measurements
                for measurement in &measurements {
                    if let Err(e) = db.store_measurement(measurement) {
                        error!("Failed to store measurement: {}", e);
                    }
                }
                
                // Check for alerts
                if let Err(e) = alert_manager.check_measurements(&measurements) {
                    error!("Alert check failed: {}", e);
                }
                
                // Update output
                if let Err(e) = output_handle.update(&measurements) {
                    error!("Output update failed: {}", e);
                }
            }
            Err(e) => {
                error!("Test failed: {}", e);
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
        PathBuf::from("latency.png")
    });
    
    // Generate chart with min/max/avg/percentile lines
    charts::generate_latency_chart(&measurements, &output_path, config)?;
    
    info!("Chart saved to {:?}", output_path);
    
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
