//! Universal Primal Discovery System
//!
//! This module provides name-agnostic primal discovery that works with any primal
//! without hardcoding specific primal names. The system uses capability-based
//! discovery and environment-based configuration.

use crate::capabilities::{Capability, CapabilityError, UniversalCapabilityAdapter};
use crate::types::PrimalType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, warn};

/// Universal primal discovery engine that works with any primal
#[derive(Debug, Clone)]
pub struct UniversalPrimalDiscovery {
    /// Capability adapter for querying primal capabilities
    capability_adapter: UniversalCapabilityAdapter,
    /// Discovery configuration
    config: DiscoveryConfig,
    /// Cache of discovered primals
    discovered_cache: HashMap<String, DiscoveredPrimal>,
}

/// Configuration for primal discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Timeout for discovery operations
    pub discovery_timeout: Duration,
    /// Whether to scan the network for primals
    pub enable_network_scan: bool,
    /// Port ranges to scan
    pub scan_ports: Vec<u16>,
    /// Network ranges to scan (CIDR notation)
    pub scan_networks: Vec<String>,
    /// Whether to use environment variables for discovery
    pub use_environment: bool,
    /// Maximum concurrent discovery operations
    pub max_concurrent_discoveries: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_timeout: Duration::from_secs(10),
            enable_network_scan: true,
            scan_ports: vec![8080, 8081, 8082, 8083, 8084, 8085, 8443, 3000, 5000],
            scan_networks: vec![
                "127.0.0.0/8".to_string(),
                "10.0.0.0/8".to_string(),
                "192.168.0.0/16".to_string(),
            ],
            use_environment: true,
            max_concurrent_discoveries: 20,
        }
    }
}

/// A discovered primal with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Name of the primal
    pub name: String,
    /// Primal type
    pub primal_type: PrimalType,
    /// Endpoint URL
    pub endpoint: String,
    /// Discovered capabilities
    pub capabilities: Vec<Capability>,
    /// Health status
    pub health: PrimalHealth,
    /// Discovery method used
    pub discovery_method: DiscoveryMethod,
    /// When this primal was discovered
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    /// Metadata about the primal
    pub metadata: HashMap<String, String>,
}

/// Methods used to discover primals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryMethod {
    /// Discovered via environment variables
    Environment,
    /// Discovered via network scanning
    NetworkScan,
    /// Discovered via mDNS/Bonjour
    Mdns,
    /// Discovered via configuration file
    Configuration,
    /// Discovered via Kubernetes service discovery
    Kubernetes,
    /// Discovered via Docker container discovery
    Docker,
}

/// Health status of discovered primals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrimalHealth {
    /// Healthy and responding
    Healthy,
    /// Responding but with issues
    Degraded,
    /// Not responding
    Unhealthy,
    /// Health status unknown
    Unknown,
}

impl UniversalPrimalDiscovery {
    /// Create a new universal primal discovery engine
    pub fn new(config: DiscoveryConfig) -> Self {
        let capability_config = crate::capabilities::DiscoveryConfig::default();
        Self {
            capability_adapter: UniversalCapabilityAdapter::new(capability_config),
            config,
            discovered_cache: HashMap::new(),
        }
    }

    /// Discover all available primals using all methods
    pub async fn discover_all_primals(&mut self) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        info!("🔍 Starting universal primal discovery...");

        let mut all_discovered = Vec::new();

        // Environment-based discovery
        if self.config.use_environment {
            match self.discover_from_environment().await {
                Ok(mut env_primals) => {
                    info!(
                        "✅ Discovered {} primals from environment",
                        env_primals.len()
                    );
                    all_discovered.append(&mut env_primals);
                }
                Err(e) => warn!("⚠️ Environment discovery failed: {}", e),
            }
        }

        // Network scanning discovery
        if self.config.enable_network_scan {
            match self.discover_from_network_scan().await {
                Ok(mut network_primals) => {
                    info!(
                        "✅ Discovered {} primals from network scan",
                        network_primals.len()
                    );
                    all_discovered.append(&mut network_primals);
                }
                Err(e) => warn!("⚠️ Network scan discovery failed: {}", e),
            }
        }

        // Container-based discovery (Docker, Kubernetes)
        match self.discover_from_containers().await {
            Ok(mut container_primals) => {
                if !container_primals.is_empty() {
                    info!(
                        "✅ Discovered {} primals from containers",
                        container_primals.len()
                    );
                    all_discovered.append(&mut container_primals);
                }
            }
            Err(e) => debug!(
                "Container discovery failed (expected if not in container): {}",
                e
            ),
        }

        // Deduplicate discovered primals
        let deduplicated = self.deduplicate_primals(all_discovered);

        // Update cache
        for primal in &deduplicated {
            self.discovered_cache
                .insert(primal.name.clone(), primal.clone());
        }

        info!("🎉 Total unique primals discovered: {}", deduplicated.len());
        Ok(deduplicated)
    }

    /// Discover primals from environment variables
    async fn discover_from_environment(&self) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        debug!("🔍 Discovering primals from environment variables...");

        let mut discovered = Vec::new();
        let primal_names = songbird_config::config::constants::get_configured_primal_names();

        for primal_name in primal_names {
            let endpoint = songbird_config::config::constants::get_primal_endpoint(&primal_name);

            // Test connectivity and discover capabilities
            match timeout(
                self.config.discovery_timeout,
                self.discover_primal_at_endpoint(&primal_name, &endpoint),
            )
            .await
            {
                Ok(Ok(primal)) => {
                    let mut env_primal = primal;
                    env_primal.discovery_method = DiscoveryMethod::Environment;
                    discovered.push(env_primal);
                }
                Ok(Err(e)) => {
                    debug!(
                        "Failed to discover primal {} at {}: {}",
                        primal_name, endpoint, e
                    );
                }
                Err(_) => {
                    debug!("Timeout discovering primal {} at {}", primal_name, endpoint);
                }
            }
        }

        Ok(discovered)
    }

    /// Discover primals via network scanning
    async fn discover_from_network_scan(&self) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        debug!("🔍 Scanning network for primals...");

        let mut discovered = Vec::new();
        let mut scan_tasks = Vec::new();

        // Create scan tasks for each network/port combination
        for network in &self.config.scan_networks {
            for &port in &self.config.scan_ports {
                let network = network.clone();
                let timeout_duration = self.config.discovery_timeout;

                let scan_task = async move {
                    self.scan_network_range(&network, port, timeout_duration)
                        .await
                };

                scan_tasks.push(scan_task);

                // Limit concurrent scans
                if scan_tasks.len() >= self.config.max_concurrent_discoveries {
                    break;
                }
            }
        }

        // Execute scans concurrently
        let scan_results = futures::future::join_all(scan_tasks).await;

        for result in scan_results {
            match result {
                Ok(mut scan_primals) => discovered.append(&mut scan_primals),
                Err(e) => debug!("Network scan failed: {}", e),
            }
        }

        Ok(discovered)
    }

    /// Discover primals from container environments
    async fn discover_from_containers(&self) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        let mut discovered = Vec::new();

        // Kubernetes service discovery
        if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
            match self.discover_kubernetes_services().await {
                Ok(mut k8s_primals) => discovered.append(&mut k8s_primals),
                Err(e) => debug!("Kubernetes discovery failed: {}", e),
            }
        }

        // Docker container discovery
        match self.discover_docker_containers().await {
            Ok(mut docker_primals) => discovered.append(&mut docker_primals),
            Err(e) => debug!("Docker discovery failed: {}", e),
        }

        Ok(discovered)
    }

    /// Discover a specific primal at an endpoint
    async fn discover_primal_at_endpoint(
        &self,
        name: &str,
        endpoint: &str,
    ) -> Result<DiscoveredPrimal, DiscoveryError> {
        debug!("🔍 Discovering primal {} at endpoint {}", name, endpoint);

        // Test basic connectivity
        let health = self.test_primal_health(endpoint).await;

        if health == PrimalHealth::Unhealthy {
            return Err(DiscoveryError::UnreachableEndpoint(endpoint.to_string()));
        }

        // Discover capabilities
        let capabilities = match self
            .capability_adapter
            .discover_primal_capabilities(name)
            .await
        {
            Ok(caps) => caps,
            Err(CapabilityError::NetworkError(_)) => {
                // If capability discovery fails, infer basic capabilities
                self.infer_basic_capabilities(name)
            }
            Err(e) => {
                warn!("Capability discovery failed for {}: {}", name, e);
                self.infer_basic_capabilities(name)
            }
        };

        // Infer primal type from capabilities or name
        let primal_type = self.infer_primal_type(name, &capabilities);

        Ok(DiscoveredPrimal {
            name: name.to_string(),
            primal_type,
            endpoint: endpoint.to_string(),
            capabilities,
            health,
            discovery_method: DiscoveryMethod::NetworkScan, // Will be updated by caller
            discovered_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        })
    }

    /// Test health of a primal endpoint
    async fn test_primal_health(&self, endpoint: &str) -> PrimalHealth {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        // Try common health endpoints
        let health_endpoints = vec![
            format!("{}/health", endpoint),
            format!("{}/api/health", endpoint),
            format!("{}/api/v1/health", endpoint),
            format!("{}/status", endpoint),
            endpoint.to_string(), // Root endpoint as fallback
        ];

        for health_endpoint in health_endpoints {
            match client.get(&health_endpoint).send().await {
                Ok(response) if response.status().is_success() => {
                    return PrimalHealth::Healthy;
                }
                Ok(response) if response.status().is_server_error() => {
                    return PrimalHealth::Degraded;
                }
                Ok(_) => continue,
                Err(_) => continue,
            }
        }

        PrimalHealth::Unhealthy
    }

    /// Infer basic capabilities from primal name patterns
    fn infer_basic_capabilities(&self, _name: &str) -> Vec<Capability> {
        // Basic capability inference based on common patterns
        vec![Capability {
            capability_type: "universal".to_string(),
            name: "generic".to_string(),
            version: "1.0".to_string(),
            parameters: std::collections::HashMap::new(),
            qos_metrics: crate::capabilities::QoSMetrics::default(),
            available: true,
        }]
    }

    /// Infer primal type from name and capabilities
    fn infer_primal_type(&self, name: &str, _capabilities: &[Capability]) -> PrimalType {
        PrimalType::new(name)
    }

    /// Scan a network range for primals
    async fn scan_network_range(
        &self,
        _network: &str,
        _port: u16,
        _timeout: Duration,
    ) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        // Network scanning implementation would go here
        // For now, return empty to avoid complexity
        Ok(Vec::new())
    }

    /// Discover Kubernetes services
    async fn discover_kubernetes_services(&self) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        // Kubernetes discovery implementation would go here
        Ok(Vec::new())
    }

    /// Discover Docker containers
    async fn discover_docker_containers(&self) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        // Docker discovery implementation would go here
        Ok(Vec::new())
    }

    /// Deduplicate discovered primals
    fn deduplicate_primals(&self, primals: Vec<DiscoveredPrimal>) -> Vec<DiscoveredPrimal> {
        let mut seen = HashMap::new();
        let mut deduplicated = Vec::new();

        for primal in primals {
            let key = format!("{}:{}", primal.name, primal.endpoint);
            if !seen.contains_key(&key) {
                seen.insert(key, ());
                deduplicated.push(primal);
            }
        }

        deduplicated
    }

    /// Get cached discovered primals
    pub fn get_discovered_primals(&self) -> Vec<&DiscoveredPrimal> {
        self.discovered_cache.values().collect()
    }

    /// Find primals with specific capability
    pub fn find_primals_with_capability(&self, capability_type: &str) -> Vec<&DiscoveredPrimal> {
        self.discovered_cache
            .values()
            .filter(|primal| {
                primal
                    .capabilities
                    .iter()
                    .any(|cap| cap.capability_type == capability_type)
            })
            .collect()
    }
}

/// Errors that can occur during discovery
#[derive(Debug)]
pub enum DiscoveryError {
    /// Network error during discovery
    NetworkError(String),
    /// Endpoint is unreachable
    UnreachableEndpoint(String),
    /// Invalid configuration
    ConfigurationError(String),
    /// Timeout during discovery
    Timeout(String),
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            DiscoveryError::UnreachableEndpoint(endpoint) => {
                write!(f, "Unreachable endpoint: {}", endpoint)
            }
            DiscoveryError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            DiscoveryError::Timeout(msg) => write!(f, "Timeout: {}", msg),
        }
    }
}

impl std::error::Error for DiscoveryError {}
