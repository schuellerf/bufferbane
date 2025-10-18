//! Output and display management

use crate::config::Config;
use crate::testing::Measurement;
use anyhow::Result;
use std::path::Path;

pub struct OutputManager {
    #[allow(dead_code)]
    config: Config,
}

impl OutputManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    pub fn update(&self, measurements: &[Measurement]) -> Result<()> {
        // Simple console output for Phase 1
        for m in measurements {
            match &m.status[..] {
                "success" => {
                    if let Some(rtt) = m.rtt_ms {
                        println!("[{}] {} -> {:.2}ms", 
                            chrono::Local::now().format("%H:%M:%S"),
                            m.target, 
                            rtt
                        );
                    }
                }
                "timeout" => {
                    println!("[{}] {} -> TIMEOUT", 
                        chrono::Local::now().format("%H:%M:%S"),
                        m.target
                    );
                }
                "error" => {
                    println!("[{}] {} -> ERROR: {:?}", 
                        chrono::Local::now().format("%H:%M:%S"),
                        m.target,
                        m.error_detail
                    );
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}

/// Export measurements as CSV
pub fn export_csv(measurements: &[Measurement], output_path: &Path) -> Result<()> {
    let mut writer = csv::Writer::from_path(output_path)?;
    
    // Write header
    writer.write_record(&[
        "timestamp",
        "interface",
        "connection_type",
        "test_type",
        "target",
        "rtt_ms",
        "jitter_ms",
        "packet_loss_pct",
        "status",
        "error",
    ])?;
    
    // Write measurements
    for m in measurements {
        writer.write_record(&[
            m.timestamp.to_string(),
            m.interface.clone(),
            m.connection_type.clone(),
            m.test_type.clone(),
            m.target.clone(),
            m.rtt_ms.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            m.jitter_ms.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            m.packet_loss_pct.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            m.status.clone(),
            m.error_detail.clone().unwrap_or_default(),
        ])?;
    }
    
    writer.flush()?;
    
    Ok(())
}

