//! Analysis and alert detection

use crate::config::Config;
use crate::testing::Measurement;
use anyhow::Result;
use std::sync::Arc;
use tracing::warn;

pub struct AlertManager {
    config: Arc<Config>,
}

impl AlertManager {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
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
                    // TODO: Log to alerts file
                }
            }
            
            // Check for timeouts (packet loss)
            if m.status == "timeout" {
                warn!("PACKET LOSS: {} -> timeout", m.target);
            }
            
            // Check for errors
            if m.status == "error" {
                warn!("ERROR: {} -> {:?}", m.target, m.error_detail);
            }
        }
        
        Ok(())
    }
}

