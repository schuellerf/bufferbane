//! Network monitoring - gateway detection and public IP tracking

use anyhow::{Context, Result};
use std::net::IpAddr;
use std::process::Command;
use std::str::FromStr;
use tracing::{debug, info, warn};

/// Detect the default gateway using `ip route` command
pub fn detect_default_gateway() -> Result<IpAddr> {
    let output = Command::new("ip")
        .args(&["route", "show", "default"])
        .output()
        .context("Failed to execute 'ip route' command")?;
    
    if !output.status.success() {
        anyhow::bail!("Failed to get default route");
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!("Default route output: {}", stdout);
    
    // Parse output like: "default via 192.168.1.1 dev eth0 proto dhcp metric 100"
    for line in stdout.lines() {
        if line.starts_with("default") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(via_idx) = parts.iter().position(|&p| p == "via") {
                if let Some(&gateway_str) = parts.get(via_idx + 1) {
                    if let Ok(gateway) = IpAddr::from_str(gateway_str) {
                        debug!("Detected default gateway: {}", gateway);
                        return Ok(gateway);
                    }
                }
            }
        }
    }
    
    anyhow::bail!("Could not parse default gateway from 'ip route' output")
}

/// Get public IP address from external service
pub async fn get_public_ip(service_url: &str) -> Result<IpAddr> {
    debug!("Querying public IP from: {}", service_url);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    
    let response = client
        .get(service_url)
        .send()
        .await
        .context("Failed to query public IP service")?;
    
    if !response.status().is_success() {
        anyhow::bail!("Public IP service returned error: {}", response.status());
    }
    
    let ip_str = response
        .text()
        .await
        .context("Failed to read public IP response")?
        .trim()
        .to_string();
    
    let ip = IpAddr::from_str(&ip_str)
        .context("Failed to parse public IP address")?;
    
    debug!("Detected public IP: {}", ip);
    Ok(ip)
}

/// Gateway monitor that tracks changes
pub struct GatewayMonitor {
    current_gateway: Option<IpAddr>,
}

impl GatewayMonitor {
    pub fn new() -> Self {
        Self {
            current_gateway: None,
        }
    }
    
    /// Check gateway and detect changes
    /// Returns Some(old_gateway, new_gateway) if gateway changed, None if unchanged or error
    pub fn check(&mut self) -> Option<(Option<IpAddr>, IpAddr)> {
        match detect_default_gateway() {
            Ok(new_gateway) => {
                if self.current_gateway.as_ref() != Some(&new_gateway) {
                    let old_gateway = self.current_gateway;
                    self.current_gateway = Some(new_gateway);
                    
                    if let Some(old) = old_gateway {
                        info!("Gateway changed: {} -> {} (ISP failover?)", old, new_gateway);
                    } else {
                        info!("Initial gateway detected: {}", new_gateway);
                    }
                    
                    return Some((old_gateway, new_gateway));
                }
                None
            }
            Err(e) => {
                warn!("Failed to check gateway: {}", e);
                None
            }
        }
    }
    
    pub fn get_current_gateway(&self) -> Option<IpAddr> {
        self.current_gateway
    }
}

/// Public IP monitor that tracks changes
pub struct PublicIpMonitor {
    service_url: String,
    current_ip: Option<IpAddr>,
    check_interval_sec: u64,
}

impl PublicIpMonitor {
    pub fn new(service_url: String, check_interval_sec: u64) -> Self {
        Self {
            service_url,
            current_ip: None,
            check_interval_sec,
        }
    }
    
    /// Check public IP and detect changes
    /// Returns Some(old_ip, new_ip) if IP changed, None if unchanged or error
    pub async fn check(&mut self) -> Option<(Option<IpAddr>, IpAddr)> {
        match get_public_ip(&self.service_url).await {
            Ok(new_ip) => {
                if self.current_ip.as_ref() != Some(&new_ip) {
                    let old_ip = self.current_ip;
                    self.current_ip = Some(new_ip);
                    
                    if let Some(old) = old_ip {
                        info!("Public IP changed: {} -> {}", old, new_ip);
                    } else {
                        info!("Initial public IP detected: {}", new_ip);
                    }
                    
                    return Some((old_ip, new_ip));
                }
                None
            }
            Err(e) => {
                warn!("Failed to check public IP: {}", e);
                None
            }
        }
    }
    
    pub fn get_check_interval(&self) -> u64 {
        self.check_interval_sec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_gateway() {
        // This test only runs on systems with ip route
        if let Ok(gateway) = detect_default_gateway() {
            println!("Detected gateway: {}", gateway);
            assert!(gateway.is_ipv4() || gateway.is_ipv6());
        }
    }
    
    #[tokio::test]
    async fn test_public_ip() {
        // This test requires internet connection
        if let Ok(ip) = get_public_ip("https://api.ipify.org").await {
            println!("Public IP: {}", ip);
            assert!(ip.is_ipv4() || ip.is_ipv6());
        }
    }
}

