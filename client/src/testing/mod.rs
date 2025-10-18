//! Network testing implementation

mod icmp;
mod measurement;
pub mod server;

pub use icmp::IcmpTester;
pub use measurement::Measurement;
pub use server::ServerTester;

use crate::config::Config;
use anyhow::Result;
use std::sync::Arc;

/// Test runner that coordinates all network tests
#[allow(dead_code)]
pub struct TestRunner {
    config: Arc<Config>,
    icmp_tester: IcmpTester,
}

#[allow(dead_code)]
impl TestRunner {
    pub fn new(config: Config) -> Result<Self> {
        let config = Arc::new(config);
        let icmp_tester = IcmpTester::new(config.clone())?;
        
        Ok(Self {
            config,
            icmp_tester,
        })
    }
    
    pub async fn run_all_tests(&self) -> Result<Vec<Measurement>> {
        self.icmp_tester.run_tests().await
    }
}

