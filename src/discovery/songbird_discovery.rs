//! Main SongbirdDiscovery service implementation
//!
//! This module provides the core discovery service that handles:
//! - Network scanning for available services
//! - Service endpoint discovery
//! - Primal discovery and coordination
//! - Health monitoring and status updates

use super::*;
use crate::errors::Result;

use crate::traits::service::ServiceInfo;
use async_trait::async_trait;
use futures_util::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Main SongbirdDiscovery service implementation
pub struct SongbirdDiscovery {
    config: SongbirdDiscoveryConfig,
    discovered_services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    primal_endpoints: Arc<RwLock<HashMap<String, String>>>,
}

impl SongbirdDiscovery {
    /// Create a new SongbirdDiscovery instance
    pub fn new(config: SongbirdDiscoveryConfig) -> Self {
        Self {
            config,
            discovered_services: Arc::new(RwLock::new(HashMap::new())),
            primal_endpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &SongbirdDiscoveryConfig {
        &self.config
    }

    /// Start the discovery service
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting SongbirdDiscovery service");

        // Start continuous network scanning
        self.start_continuous_scanning().await?;

        // Start Primal discovery
        self.start_primal_discovery().await?;

        info!("SongbirdDiscovery service started successfully");
        Ok(())
    }

    /// Start continuous network scanning for services
    async fn start_continuous_scanning(&self) -> Result<()> {
        info!("Starting continuous network scanning");

        let discovered_services = Arc::clone(&self.discovered_services);
        let scan_interval = Duration::from_secs(60); // 1 minute interval

        tokio::spawn(async move {
            loop {
                debug!("Starting network scan cycle");

                // Perform network discovery
                match Self::discover_network_services().await {
                    Ok(services) => {
                        let mut discovered = discovered_services.write().await;
                        for service in services {
                            discovered.insert(service.service_id.clone(), service);
                        }
                        debug!("Network scan completed successfully");
                    }
                    Err(e) => {
                        warn!("Network scan failed: {}", e);
                    }
                }

                tokio::time::sleep(scan_interval).await;
            }
        });

        Ok(())
    }

    /// Start Primal discovery and coordination
    async fn start_primal_discovery(&self) -> Result<()> {
        info!("Starting Primal discovery");

        let primal_endpoints = Arc::clone(&self.primal_endpoints);
        let primal_discovery_interval = Duration::from_secs(30);

        tokio::spawn(async move {
            loop {
                debug!("Starting Primal discovery cycle");

                // Discover known Primals
                let primals = vec!["toadstool", "nestgate", "beardog", "squirrel"];

                for primal_name in primals {
                    if let Some(endpoint) = Self::discover_primal_endpoint(primal_name).await {
                        primal_endpoints
                            .write()
                            .await
                            .insert(primal_name.to_string(), endpoint.clone());
                        info!("Discovered {} at: {}", primal_name, endpoint);
                    }
                }

                tokio::time::sleep(primal_discovery_interval).await;
            }
        });

        Ok(())
    }

    /// Discover services on the network
    async fn discover_network_services() -> Result<Vec<ServiceInfo>> {
        let mut discovered_services = Vec::new();

        // Scan common service ports
        let common_ports = vec![8080, 8081, 8082, 8083, 8084, 8085, 3000, 5000, 9000];
        let local_networks = vec![
            crate::config::constants::default_bind_address(),
            crate::config::constants::DEFAULT_LOCALHOST
        ];

        for host in local_networks {
            for port in &common_ports {
                match Self::probe_service_endpoint(host, *port).await {
                    Ok(Some(service)) => {
                        discovered_services.push(service);
                    }
                    Ok(None) => {
                        // No service found at this endpoint
                    }
                    Err(e) => {
                        debug!("Failed to probe {}:{}: {}", host, port, e);
                    }
                }
            }
        }

        Ok(discovered_services)
    }

    /// Probe a specific service endpoint
    async fn probe_service_endpoint(host: &str, port: u16) -> Result<Option<ServiceInfo>> {
        let address = format!("{host}:{port}");

        // Try to connect to the service
        match timeout(
            Duration::from_millis(500),
            tokio::net::TcpStream::connect(&address),
        )
        .await
        {
            Ok(Ok(_)) => {
                // Connection successful, create service info
                let service_info = ServiceInfo {
                    service_id: format!("service-{port}"),
                    name: format!("Service on port {port}"),
                    version: "unknown".to_string(),
                    service_type: "unknown".to_string(),
                    description: Some(format!("Discovered service at {address}")),
                    endpoints: vec![],
                    health_check_endpoint: None,
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "discovered_at".to_string(),
                            chrono::Utc::now().to_rfc3339().into(),
                        );
                        metadata.insert("address".to_string(), address.clone().into());
                        metadata
                    },
                    tags: vec!["discovered".to_string()],
                    dependencies: vec![],
                    status: crate::traits::service::ServiceStatus::Running,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    instance_id: format!("discovered-{port}"),
                    host: host.to_string(),
                    port,
                };

                info!("Discovered service: {} at {}", service_info.name, address);
                Ok(Some(service_info))
            }
            Ok(Err(_)) | Err(_) => {
                // Connection failed or timeout
                Ok(None)
            }
        }
    }

    /// Discover a specific Primal endpoint
    async fn discover_primal_endpoint(primal_name: &str) -> Option<String> {
        let common_ports = vec![8080, 8081, 8082, 8083, 8084, 8085];
        let hosts = vec![
            crate::config::constants::default_bind_address(),
            crate::config::constants::DEFAULT_LOCALHOST
        ];

        for host in hosts {
            for port in &common_ports {
                let endpoint = format!("http://{host}:{port}");

                if Self::test_primal_endpoint(&endpoint, primal_name).await {
                    return Some(endpoint);
                }
            }
        }

        None
    }

    /// Test if an endpoint is a specific Primal
    async fn test_primal_endpoint(endpoint: &str, primal_name: &str) -> bool {
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .build()
        {
            Ok(client) => client,
            Err(e) => {
                warn!("Failed to create HTTP client: {}", e);
                return false;
            }
        };

        // Test common Primal endpoints
        let test_endpoints = vec![
            format!("{}/health", endpoint),
            format!("{}/info", endpoint),
            format!("{}/api/v1/health", endpoint),
            format!("{}/status", endpoint),
        ];

        for test_endpoint in test_endpoints {
            match client.get(&test_endpoint).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(text) = response.text().await {
                            // Check if response contains Primal name
                            if text.to_lowercase().contains(primal_name) {
                                return true;
                            }
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        false
    }

    /// Get discovered Primal endpoints
    pub async fn get_primal_endpoints(&self) -> HashMap<String, String> {
        self.primal_endpoints.read().await.clone()
    }
}

#[async_trait]
impl ServiceDiscovery for SongbirdDiscovery {
    async fn register(&self, service: ServiceInfo) -> Result<()> {
        let mut services = self.discovered_services.write().await;
        services.insert(service.service_id.clone(), service);
        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> Result<()> {
        let mut services = self.discovered_services.write().await;
        services.remove(service_id);
        Ok(())
    }

    async fn discover(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        let services = self.discovered_services.read().await;
        let mut results = Vec::new();

        for service in services.values() {
            if self.service_matches_query(service, &query) {
                results.push(service.clone());
            }
        }

        Ok(results)
    }

    async fn watch(
        &self,
        _query: ServiceQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        // In a real implementation, this would return a stream of service events
        Ok(Box::pin(futures_util::stream::empty()))
    }

    async fn update_health(&self, service_id: &str, _health: ServiceHealthStatus) -> Result<()> {
        let mut services = self.discovered_services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.updated_at = chrono::Utc::now();
        }
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<ServiceInfo>> {
        let services = self.discovered_services.read().await;
        Ok(services.values().cloned().collect())
    }

    async fn exists(&self, service_id: &str) -> Result<bool> {
        let services = self.discovered_services.read().await;
        Ok(services.contains_key(service_id))
    }

    async fn is_registered(&self, service_id: &str) -> Result<bool> {
        self.exists(service_id).await
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        let mut services = self.discovered_services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            for (key, value) in metadata {
                service.metadata.insert(key, value.into());
            }
            service.updated_at = chrono::Utc::now();
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SongbirdDiscovery {
    /// Check if a service matches the query criteria
    fn service_matches_query(&self, service: &ServiceInfo, query: &ServiceQuery) -> bool {
        // Check service ID filter
        if let Some(ref service_id) = query.service_id {
            if service.service_id != *service_id {
                return false;
            }
        }

        // Check service type filter
        if let Some(ref service_type) = query.service_type {
            if service.service_type != *service_type {
                return false;
            }
        }

        // Check name filter
        if let Some(ref name_filter) = query.name {
            if !service
                .name
                .to_lowercase()
                .contains(&name_filter.to_lowercase())
            {
                return false;
            }
        }

        // Check tag filters
        for required_tag in &query.tags {
            if !service.tags.contains(required_tag) {
                return false;
            }
        }

        true
    }
}
