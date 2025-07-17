//! Internet Connection Module
//!
//! Basic internet connection management

use songbird_errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternetConnectionConfig {
    pub enabled: bool,
    pub tunnel_type: String,
    pub network_name: String,
    pub auto_discovery: bool,
}

impl Default for InternetConnectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tunnel_type: "wireguard".to_string(),
            network_name: "songbird-network".to_string(),
            auto_discovery: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SongbirdPorts {
    pub orchestrator_port: u16,
    pub federation_port: u16,
    pub metrics_port: u16,
    pub discovery_port: u16,
    pub additional_service_ports: HashMap<String, u16>,
}

impl SongbirdPorts {
    pub fn get_all_required_ports(&self) -> Vec<u16> {
        let mut ports = vec![
            self.orchestrator_port,
            self.federation_port,
            self.metrics_port,
            self.discovery_port,
        ];
        ports.extend(self.additional_service_ports.values());
        ports
    }
}

pub struct InternetConnectionWizard {
    #[allow(dead_code)]
    config: InternetConnectionConfig,
}

impl InternetConnectionWizard {
    pub fn new(config: InternetConnectionConfig) -> Self {
        Self { config }
    }

    pub async fn configure(&self) -> Result<()> {
        // Minimal implementation
        Ok(())
    }

    pub async fn discover_songbird_ports(&self) -> Result<SongbirdPorts> {
        // Return default port configuration
        Ok(SongbirdPorts {
            orchestrator_port: crate::config::constants::network::DEFAULT_PORT,
            federation_port: 8081,
            metrics_port: 9090,
            discovery_port: 8082,
            additional_service_ports: HashMap::new(),
        })
    }
}
