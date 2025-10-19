//! SQLite database storage

use crate::testing::Measurement;
use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::Path;
use tracing::info;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)
            .context("Failed to open database")?;
        
        // Enable WAL mode for better concurrent read/write performance
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("Failed to enable WAL mode")?;
        
        // Set busy timeout to 5 seconds (handles brief lock conflicts)
        conn.pragma_update(None, "busy_timeout", "5000")
            .context("Failed to set busy timeout")?;
        
        Ok(Self { conn })
    }
    
    pub fn initialize(&self) -> Result<()> {
        info!("Initializing database schema");
        
        // Create measurements table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS measurements (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                monotonic_ns INTEGER NOT NULL,
                interface TEXT NOT NULL,
                connection_type TEXT NOT NULL,
                test_type TEXT NOT NULL,
                target TEXT NOT NULL,
                server_name TEXT,
                rtt_ms REAL,
                jitter_ms REAL,
                packet_loss_pct REAL,
                throughput_kbps REAL,
                dns_time_ms REAL,
                status TEXT NOT NULL,
                error_detail TEXT,
                upload_latency_ms REAL,
                download_latency_ms REAL,
                server_processing_us INTEGER
            )",
            [],
        )?;
        
        // Migrate existing databases: add new columns if they don't exist
        // SQLite doesn't have ALTER TABLE IF NOT EXISTS, so we need to check
        let _ = self.conn.execute(
            "ALTER TABLE measurements ADD COLUMN upload_latency_ms REAL",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE measurements ADD COLUMN download_latency_ms REAL",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE measurements ADD COLUMN server_processing_us INTEGER",
            [],
        );
        
        // Create indices for common queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON measurements(timestamp)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_interface ON measurements(interface)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_connection_type ON measurements(connection_type)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_test_type ON measurements(test_type)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_target ON measurements(target)",
            [],
        )?;
        
        // Create events table for alerts
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                target TEXT NOT NULL,
                severity TEXT NOT NULL,
                message TEXT NOT NULL,
                value REAL,
                threshold REAL
            )",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_event_timestamp ON events(timestamp)",
            [],
        )?;
        
        // Create hourly aggregations table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS aggregations_hourly (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                hour_timestamp INTEGER NOT NULL,
                interface TEXT NOT NULL,
                connection_type TEXT NOT NULL,
                test_type TEXT NOT NULL,
                target TEXT NOT NULL,
                server_name TEXT,
                count INTEGER NOT NULL,
                min_rtt_ms REAL,
                max_rtt_ms REAL,
                avg_rtt_ms REAL,
                p50_rtt_ms REAL,
                p95_rtt_ms REAL,
                p99_rtt_ms REAL,
                min_jitter_ms REAL,
                max_jitter_ms REAL,
                avg_jitter_ms REAL,
                packet_loss_pct REAL,
                avg_throughput_kbps REAL,
                avg_dns_time_ms REAL,
                UNIQUE(hour_timestamp, interface, test_type, target, server_name)
            )",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_hourly_timestamp ON aggregations_hourly(hour_timestamp)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_hourly_interface ON aggregations_hourly(interface)",
            [],
        )?;
        
        info!("Database schema initialized");
        
        Ok(())
    }
    
    pub fn store_measurement(&self, m: &Measurement) -> Result<()> {
        self.conn.execute(
            "INSERT INTO measurements (
                timestamp, monotonic_ns, interface, connection_type, test_type, target,
                server_name, rtt_ms, jitter_ms, packet_loss_pct, throughput_kbps,
                dns_time_ms, status, error_detail, upload_latency_ms, download_latency_ms,
                server_processing_us
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                m.timestamp,
                m.monotonic_ns as i64,
                &m.interface,
                &m.connection_type,
                &m.test_type,
                &m.target,
                &m.server_name,
                m.rtt_ms,
                m.jitter_ms,
                m.packet_loss_pct,
                m.throughput_kbps,
                m.dns_time_ms,
                &m.status,
                &m.error_detail,
                m.upload_latency_ms,
                m.download_latency_ms,
                m.server_processing_us,
            ],
        )?;
        
        Ok(())
    }
    
    pub fn query_range(&self, start: i64, end: i64) -> Result<Vec<Measurement>> {
        let mut stmt = self.conn.prepare(
            "SELECT 
                timestamp, monotonic_ns, interface, connection_type, test_type, target,
                server_name, rtt_ms, jitter_ms, packet_loss_pct, throughput_kbps,
                dns_time_ms, status, error_detail, upload_latency_ms, download_latency_ms,
                server_processing_us
            FROM measurements
            WHERE timestamp >= ?1 AND timestamp <= ?2
            ORDER BY timestamp ASC"
        )?;
        
        let measurements = stmt.query_map(params![start, end], |row| {
            Ok(Measurement {
                timestamp: row.get(0)?,
                monotonic_ns: row.get::<_, i64>(1)? as u128,
                interface: row.get(2)?,
                connection_type: row.get(3)?,
                test_type: row.get(4)?,
                target: row.get(5)?,
                server_name: row.get(6)?,
                rtt_ms: row.get(7)?,
                jitter_ms: row.get(8)?,
                packet_loss_pct: row.get(9)?,
                throughput_kbps: row.get(10)?,
                dns_time_ms: row.get(11)?,
                status: row.get(12)?,
                error_detail: row.get(13)?,
                upload_latency_ms: row.get(14)?,
                download_latency_ms: row.get(15)?,
                server_processing_us: row.get(16)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(measurements)
    }
    
    pub fn store_event(&self, 
        event_type: &str, 
        target: &str, 
        severity: &str, 
        message: &str,
        value: Option<f64>,
        threshold: Option<f64>
    ) -> Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        self.conn.execute(
            "INSERT INTO events (
                timestamp, event_type, target, severity, message, value, threshold
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                timestamp,
                event_type,
                target,
                severity,
                message,
                value,
                threshold,
            ],
        )?;
        
        Ok(())
    }
    
    pub fn query_events(&self, start: i64, end: i64) -> Result<Vec<Event>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, event_type, target, severity, message, value, threshold
            FROM events
            WHERE timestamp >= ?1 AND timestamp <= ?2
            ORDER BY timestamp ASC"
        )?;
        
        let events = stmt.query_map(params![start, end], |row| {
            Ok(Event {
                timestamp: row.get(0)?,
                event_type: row.get(1)?,
                target: row.get(2)?,
                severity: row.get(3)?,
                message: row.get(4)?,
                value: row.get(5)?,
                threshold: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(events)
    }
    
    /// Aggregate raw measurements to hourly statistics for a time range
    pub fn aggregate_to_hourly(&self, start: i64, end: i64) -> Result<usize> {
        info!("Aggregating measurements from {} to {}", start, end);
        
        // Query all measurements in the time range grouped by hour
        let mut stmt = self.conn.prepare(
            "SELECT 
                (timestamp / 3600) * 3600 as hour_ts,
                interface,
                connection_type,
                test_type,
                target,
                server_name,
                COUNT(*) as count,
                GROUP_CONCAT(rtt_ms) as rtt_values,
                GROUP_CONCAT(jitter_ms) as jitter_values,
                GROUP_CONCAT(CASE WHEN status = 'timeout' THEN 1 ELSE 0 END) as loss_flags,
                AVG(throughput_kbps) as avg_throughput,
                AVG(dns_time_ms) as avg_dns_time
            FROM measurements
            WHERE timestamp >= ?1 AND timestamp < ?2
            GROUP BY hour_ts, interface, connection_type, test_type, target, server_name"
        )?;
        
        let rows = stmt.query_map(params![start, end], |row| {
            Ok((
                row.get::<_, i64>(0)?,           // hour_ts
                row.get::<_, String>(1)?,        // interface
                row.get::<_, String>(2)?,        // connection_type
                row.get::<_, String>(3)?,        // test_type
                row.get::<_, String>(4)?,        // target
                row.get::<_, Option<String>>(5)?, // server_name
                row.get::<_, i64>(6)?,           // count
                row.get::<_, Option<String>>(7)?, // rtt_values
                row.get::<_, Option<String>>(8)?, // jitter_values
                row.get::<_, Option<String>>(9)?, // loss_flags
                row.get::<_, Option<f64>>(10)?,  // avg_throughput
                row.get::<_, Option<f64>>(11)?,  // avg_dns_time
            ))
        })?;
        
        let mut aggregated_count = 0;
        
        for row in rows {
            let (hour_ts, interface, conn_type, test_type, target, server_name, 
                 count, rtt_str, jitter_str, loss_str, avg_throughput, avg_dns_time) = row?;
            
            // Parse and calculate RTT statistics
            let (min_rtt, max_rtt, avg_rtt, p50_rtt, p95_rtt, p99_rtt) = 
                if let Some(rtt_str) = rtt_str {
                    Self::calculate_statistics(&rtt_str)
                } else {
                    (None, None, None, None, None, None)
                };
            
            // Parse and calculate jitter statistics
            let (min_jitter, max_jitter, avg_jitter, _, _, _) = 
                if let Some(jitter_str) = jitter_str {
                    Self::calculate_statistics(&jitter_str)
                } else {
                    (None, None, None, None, None, None)
                };
            
            // Calculate packet loss percentage
            let packet_loss_pct = if let Some(loss_str) = loss_str {
                let losses: Vec<i32> = loss_str
                    .split(',')
                    .filter_map(|s| s.parse().ok())
                    .collect();
                let total_loss: i32 = losses.iter().sum();
                Some((total_loss as f64 / losses.len() as f64) * 100.0)
            } else {
                None
            };
            
            // Insert or replace the aggregation
            self.conn.execute(
                "INSERT OR REPLACE INTO aggregations_hourly (
                    hour_timestamp, interface, connection_type, test_type, target, server_name,
                    count, min_rtt_ms, max_rtt_ms, avg_rtt_ms, p50_rtt_ms, p95_rtt_ms, p99_rtt_ms,
                    min_jitter_ms, max_jitter_ms, avg_jitter_ms, packet_loss_pct,
                    avg_throughput_kbps, avg_dns_time_ms
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
                params![
                    hour_ts, &interface, &conn_type, &test_type, &target, &server_name,
                    count, min_rtt, max_rtt, avg_rtt, p50_rtt, p95_rtt, p99_rtt,
                    min_jitter, max_jitter, avg_jitter, packet_loss_pct,
                    avg_throughput, avg_dns_time
                ],
            )?;
            
            aggregated_count += 1;
        }
        
        info!("Created {} hourly aggregations", aggregated_count);
        Ok(aggregated_count)
    }
    
    /// Calculate min, max, avg, P50, P95, P99 from comma-separated values
    fn calculate_statistics(values_str: &str) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
        let mut values: Vec<f64> = values_str
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
        
        if values.is_empty() {
            return (None, None, None, None, None, None);
        }
        
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min = values.first().copied();
        let max = values.last().copied();
        let avg = Some(values.iter().sum::<f64>() / values.len() as f64);
        
        let p50_idx = (values.len() as f64 * 0.50) as usize;
        let p95_idx = (values.len() as f64 * 0.95) as usize;
        let p99_idx = (values.len() as f64 * 0.99) as usize;
        
        let p50 = values.get(p50_idx.min(values.len() - 1)).copied();
        let p95 = values.get(p95_idx.min(values.len() - 1)).copied();
        let p99 = values.get(p99_idx.min(values.len() - 1)).copied();
        
        (min, max, avg, p50, p95, p99)
    }
    
    /// Delete raw measurements before a given timestamp
    pub fn delete_measurements_before(&self, timestamp: i64) -> Result<usize> {
        let deleted = self.conn.execute(
            "DELETE FROM measurements WHERE timestamp < ?1",
            params![timestamp],
        )?;
        info!("Deleted {} raw measurements", deleted);
        Ok(deleted)
    }
    
    /// Delete both raw measurements and aggregations before a given timestamp
    pub fn delete_all_before(&self, timestamp: i64) -> Result<(usize, usize)> {
        let measurements_deleted = self.conn.execute(
            "DELETE FROM measurements WHERE timestamp < ?1",
            params![timestamp],
        )?;
        
        let aggregations_deleted = self.conn.execute(
            "DELETE FROM aggregations_hourly WHERE hour_timestamp < ?1",
            params![timestamp],
        )?;
        
        info!("Deleted {} raw measurements and {} hourly aggregations", 
              measurements_deleted, aggregations_deleted);
        
        Ok((measurements_deleted, aggregations_deleted))
    }
    
    /// Count records that would be deleted before a given timestamp
    pub fn count_records_before(&self, timestamp: i64) -> Result<(usize, usize)> {
        let measurements_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM measurements WHERE timestamp < ?1",
            params![timestamp],
            |row| row.get(0),
        )?;
        
        let aggregations_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM aggregations_hourly WHERE hour_timestamp < ?1",
            params![timestamp],
            |row| row.get(0),
        )?;
        
        Ok((measurements_count, aggregations_count))
    }
    
    /// Query hourly aggregations for a time range
    #[allow(dead_code)]
    pub fn query_hourly_range(&self, start: i64, end: i64) -> Result<Vec<HourlyAggregation>> {
        let mut stmt = self.conn.prepare(
            "SELECT 
                hour_timestamp, interface, connection_type, test_type, target, server_name,
                count, min_rtt_ms, max_rtt_ms, avg_rtt_ms, p50_rtt_ms, p95_rtt_ms, p99_rtt_ms,
                min_jitter_ms, max_jitter_ms, avg_jitter_ms, packet_loss_pct,
                avg_throughput_kbps, avg_dns_time_ms
            FROM aggregations_hourly
            WHERE hour_timestamp >= ?1 AND hour_timestamp <= ?2
            ORDER BY hour_timestamp ASC"
        )?;
        
        let aggregations = stmt.query_map(params![start, end], |row| {
            Ok(HourlyAggregation {
                hour_timestamp: row.get(0)?,
                interface: row.get(1)?,
                connection_type: row.get(2)?,
                test_type: row.get(3)?,
                target: row.get(4)?,
                server_name: row.get(5)?,
                count: row.get(6)?,
                min_rtt_ms: row.get(7)?,
                max_rtt_ms: row.get(8)?,
                avg_rtt_ms: row.get(9)?,
                p50_rtt_ms: row.get(10)?,
                p95_rtt_ms: row.get(11)?,
                p99_rtt_ms: row.get(12)?,
                min_jitter_ms: row.get(13)?,
                max_jitter_ms: row.get(14)?,
                avg_jitter_ms: row.get(15)?,
                packet_loss_pct: row.get(16)?,
                avg_throughput_kbps: row.get(17)?,
                avg_dns_time_ms: row.get(18)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(aggregations)
    }
    
    /// Get the oldest timestamp for data that needs aggregation
    pub fn get_oldest_unaggregated_timestamp(&self, cutoff_days: u32) -> Result<Option<i64>> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let cutoff_timestamp = now - (cutoff_days as i64 * 86400);
        
        // Find the oldest measurement that's past the cutoff
        let oldest: Option<i64> = self.conn.query_row(
            "SELECT MIN(timestamp) FROM measurements WHERE timestamp < ?1",
            params![cutoff_timestamp],
            |row| row.get(0),
        ).ok().flatten();
        
        Ok(oldest)
    }
    
    /// Optimize database by reclaiming space after deletions
    pub fn vacuum(&self) -> Result<()> {
        info!("Running VACUUM to optimize database");
        self.conn.execute("VACUUM", [])?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Event {
    pub timestamp: i64,
    pub event_type: String,
    pub target: String,
    pub severity: String,
    pub message: String,
    pub value: Option<f64>,
    pub threshold: Option<f64>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HourlyAggregation {
    pub hour_timestamp: i64,
    pub interface: String,
    pub connection_type: String,
    pub test_type: String,
    pub target: String,
    pub server_name: Option<String>,
    pub count: i64,
    pub min_rtt_ms: Option<f64>,
    pub max_rtt_ms: Option<f64>,
    pub avg_rtt_ms: Option<f64>,
    pub p50_rtt_ms: Option<f64>,
    pub p95_rtt_ms: Option<f64>,
    pub p99_rtt_ms: Option<f64>,
    pub min_jitter_ms: Option<f64>,
    pub max_jitter_ms: Option<f64>,
    pub avg_jitter_ms: Option<f64>,
    pub packet_loss_pct: Option<f64>,
    pub avg_throughput_kbps: Option<f64>,
    pub avg_dns_time_ms: Option<f64>,
}

