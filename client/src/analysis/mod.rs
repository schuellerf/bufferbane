//! Analysis and alert detection

use crate::config::Config;
use crate::storage::Database;
use crate::testing::Measurement;
use anyhow::Result;
use std::sync::Arc;
use tracing::warn;

pub struct AlertManager {
    config: Arc<Config>,
    db: Arc<Database>,
}

impl AlertManager {
    pub fn new(config: Config, db: Arc<Database>) -> Self {
        Self {
            config: Arc::new(config),
            db,
        }
    }
    
    pub fn check_measurements(&self, measurements: &[Measurement]) -> Result<()> {
        if !self.config.alerts.enabled {
            return Ok(());
        }
        
        for m in measurements {
            // Check latency threshold
            if let Some(rtt) = m.rtt_ms {
                if rtt > self.config.alerts.latency_threshold_ms {
                    warn!(
                        "HIGH LATENCY ALERT: {} -> {:.2}ms (threshold: {:.2}ms)",
                        m.target, rtt, self.config.alerts.latency_threshold_ms
                    );
                    
                    // Store event in database
                    let _ = self.db.store_event(
                        "high_latency",
                        &m.target,
                        "warning",
                        &format!("Latency {:.2}ms exceeds threshold {:.2}ms", 
                                rtt, self.config.alerts.latency_threshold_ms),
                        Some(rtt),
                        Some(self.config.alerts.latency_threshold_ms),
                    );
                }
            }
            
            // Check for timeouts (packet loss)
            if m.status == "timeout" {
                warn!("PACKET LOSS: {} -> timeout", m.target);
                
                // Store event in database
                let _ = self.db.store_event(
                    "packet_loss",
                    &m.target,
                    "warning",
                    "Packet loss detected (timeout)",
                    None,
                    None,
                );
            }
            
            // Check for errors
            if m.status == "error" {
                warn!("ERROR: {} -> {:?}", m.target, m.error_detail);
                
                // Store event in database
                let _ = self.db.store_event(
                    "error",
                    &m.target,
                    "error",
                    &format!("Test error: {:?}", m.error_detail),
                    None,
                    None,
                );
            }
        }
        
        Ok(())
    }
}

