//! ICMP ping testing

use super::Measurement;
use crate::config::Config;
use anyhow::{Context, Result};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use surge_ping::{Client, Config as PingConfig, PingIdentifier, PingSequence};
use tracing::{debug, warn};

pub struct IcmpTester {
    #[allow(dead_code)]
    config: Arc<Config>,
    client: Client,
    targets: Vec<IpAddr>,
    interface: String,
    connection_type: String,
}

impl IcmpTester {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        Self::new_with_additional_targets(config, Vec::new())
    }
    
    pub fn new_with_additional_targets(config: Arc<Config>, additional_targets: Vec<IpAddr>) -> Result<Self> {
        // Create ICMP client
        let ping_config = PingConfig::default();
        let client = Client::new(&ping_config)
            .context("Failed to create ICMP client (CAP_NET_RAW required)")?;
        
        // Resolve target IPs
        let mut targets = Vec::new();
        
        // Add additional targets first (e.g., auto-detected gateway)
        targets.extend(additional_targets);
        
        // Add public DNS servers
        for dns in &config.targets.public_dns {
            match dns.parse::<IpAddr>() {
                Ok(ip) => targets.push(ip),
                Err(e) => warn!("Failed to parse DNS address {}: {}", dns, e),
            }
        }
        
        // Add custom targets
        for custom in &config.targets.custom {
            match custom.parse::<IpAddr>() {
                Ok(ip) => targets.push(ip),
                Err(e) => {
                    // Try to resolve as hostname
                    match resolve_hostname(custom) {
                        Ok(ip) => targets.push(ip),
                        Err(e2) => warn!("Failed to resolve {}: {} (parse: {})", custom, e2, e),
                    }
                }
            }
        }
        
        if targets.is_empty() {
            anyhow::bail!("No valid targets configured");
        }
        
        debug!("Initialized ICMP tester with {} targets", targets.len());
        
        // Determine interface and connection type
        let interface = if config.general.interfaces.is_empty() {
            "default".to_string()
        } else {
            config.general.interfaces[0].clone()
        };
        
        let connection_type = config.general.connection_type.clone();
        
        Ok(Self {
            config,
            client,
            targets,
            interface,
            connection_type,
        })
    }
    
    pub async fn run_tests(&self) -> Result<Vec<Measurement>> {
        let mut measurements = Vec::new();
        
        for target_ip in &self.targets {
            let mut measurement = Measurement::new_icmp(
                target_ip.to_string(),
                self.interface.clone(),
                self.connection_type.clone(),
            );
            
            // Ping with 5 second timeout
            match self.ping(*target_ip).await {
                Ok(rtt_ms) => {
                    measurement.set_success(rtt_ms);
                    debug!("ICMP {} -> {:.2}ms", target_ip, rtt_ms);
                }
                Err(e) => {
                    if e.to_string().contains("timeout") {
                        measurement.set_timeout();
                        debug!("ICMP {} -> timeout", target_ip);
                    } else {
                        measurement.set_error(e.to_string());
                        debug!("ICMP {} -> error: {}", target_ip, e);
                    }
                }
            }
            
            measurements.push(measurement);
        }
        
        Ok(measurements)
    }
    
    async fn ping(&self, target: IpAddr) -> Result<f64> {
        let payload = [0u8; 56]; // Standard ping payload size
        let timeout = Duration::from_secs(5);
        
        let mut pinger = self.client.pinger(target, PingIdentifier(rand::random())).await;
        
        match tokio::time::timeout(
            timeout,
            pinger.ping(PingSequence(0), &payload)
        ).await {
            Ok(Ok((_packet, duration))) => {
                Ok(duration.as_secs_f64() * 1000.0) // Convert to milliseconds
            }
            Ok(Err(e)) => {
                anyhow::bail!("Ping failed: {}", e)
            }
            Err(_) => {
                anyhow::bail!("Ping timeout after {:?}", timeout)
            }
        }
    }
}

fn resolve_hostname(hostname: &str) -> Result<IpAddr> {
    use std::net::ToSocketAddrs;
    
    let addr = format!("{}:0", hostname)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("No addresses found"))?;
    
    Ok(addr.ip())
}

