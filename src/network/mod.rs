use crate::errors::SongbirdError;
use crate::config::{constants::network, environment};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
    pub ssl_config: Option<SslConfig>,
    pub domains: HashMap<String, DomainConfig>,
    pub load_balancer_strategy: LoadBalancerStrategy,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: environment::get_container_bind_address(),
            port: network::DEFAULT_PORT,
            max_connections: 1000,
            connection_timeout: network::DEFAULT_CONNECTION_TIMEOUT,
            read_timeout: network::DEFAULT_READ_TIMEOUT,
            write_timeout: network::DEFAULT_WRITE_TIMEOUT,
            ssl_config: None,
            domains: HashMap::new(),
            load_balancer_strategy: LoadBalancerStrategy::RoundRobin,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
    pub ca_cert_path: Option<String>,
    pub verify_client: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub domain: String,
    pub upstream_servers: Vec<String>,
    pub health_check_path: String,
    pub health_check_interval: Duration,
    pub proxy_routes: Vec<ProxyRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRoute {
    pub path: String,
    pub upstream: String,
    pub proxy_type: ProxyType,
    pub health_check: Option<ProxyHealthCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyType {
    Http,
    Tcp,
    WebSocket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyHealthCheck {
    pub enabled: bool,
    pub path: String,
    pub interval: Duration,
    pub timeout: Duration,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerStrategy {
    RoundRobin,
    LeastConnections,
    IpHash,
    WeightedRoundRobin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub connection: Duration,
    pub read: Duration,
    pub write: Duration,
    pub idle: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connection: Duration::from_secs(30),
            read: Duration::from_secs(30),
            write: Duration::from_secs(30),
            idle: Duration::from_secs(300),
        }
    }
}

pub struct NetworkManager {
    config: NetworkConfig,
    active_connections: HashMap<String, u32>,
}

impl NetworkManager {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            active_connections: HashMap::new(),
        }
    }

    pub async fn start(&mut self) -> Result<(), SongbirdError> {
        // Network manager startup logic
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), SongbirdError> {
        // Network manager shutdown logic
        self.active_connections.clear();
        Ok(())
    }

    pub fn get_config(&self) -> &NetworkConfig {
        &self.config
    }

    pub async fn add_connection(&mut self, connection_id: String) -> Result<(), SongbirdError> {
        let count = self.active_connections.entry(connection_id).or_insert(0);
        *count += 1;
        Ok(())
    }

    pub async fn remove_connection(&mut self, connection_id: &str) -> Result<(), SongbirdError> {
        if let Some(count) = self.active_connections.get_mut(connection_id) {
            if *count > 0 {
                *count -= 1;
            }
            if *count == 0 {
                self.active_connections.remove(connection_id);
            }
        }
        Ok(())
    }

    pub fn get_connection_count(&self) -> usize {
        self.active_connections.values().sum::<u32>() as usize
    }
}
