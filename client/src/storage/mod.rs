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

