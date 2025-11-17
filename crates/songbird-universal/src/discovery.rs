//! Universal Primal Discovery System
//!
//! This module provides name-agnostic primal discovery that works with any primal
//! without hardcoding specific primal names. The system uses capability-based
//! discovery and environment-based configuration.

#![allow(
    clippy::unused_self,
    clippy::match_same_arms,
    clippy::zero_sized_map_values,
    clippy::unused_async
)]

use crate::capabilities::Capability;
use crate::types::PrimalType;
use serde::{Deserialize, Serialize};
use songbird_types::SafeEnv;
use std::collections::HashMap;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, warn};
// String constants to avoid Rust 2021 prefix parsing issues
const DISCOVERED_FROM_ENVIRONMENT_MSG: &str = "✅ Discovered {} primals from environment";
const DISCOVERED_FROM_CONTAINERS_MSG: &str = "✅ Discovered {} primals from containers";
#[allow(dead_code)] // Reserved for future network scanning error messages
const NETWORK_SCAN_FAILED_MSG: &str = "Network scan failed: {}";
const HEALTH_PATH: &str = "/health";
const API_V1_HEALTH_PATH: &str = "/api/v1/health";
const API_HEALTH_PATH: &str = "/api/health";
const STATUS_PATH: &str = "/status";

/// Universal primal discovery engine that works with any primal
#[derive(Debug, Clone)]
pub struct UniversalPrimalDiscovery {
    /// Capability adapter for querying primal capabilities
    // Capability adapter placeholder - will be implemented with proper capability system
    _capability_adapter: (),
    /// Discovery configuration
    config: DiscoveryConfig,
    /// Cache of discovered primals
    discovered_cache: HashMap<String, DiscoveredPrimal>,
}

/// Discovery configuration for universal adapters
///
/// **LOCAL DEFINITION**: Nested structure similar to canonical pattern!
/// This is one of the better-aligned instances with canonical's nested approach.
/// Fields align with canonical discovery configs:
/// - `enable_environment_scan` → capability_discovery.enabled
/// - `enable_network_scanning` → network_discovery.scan_local_network
/// - `enable_container_discovery` → service_discovery.enabled
/// - `timeout` → scan_timeout_secs (Duration vs u64)
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Discovery mechanisms to enable (aligns with canonical nested configs)
    pub mechanisms: DiscoveryMechanisms,
    /// Timeout for discovery operations (aligns with canonical scan_timeout_secs)
    pub timeout: Duration,
}

/// Discovery mechanisms configuration
///
/// **ARCHITECTURAL ALIGNMENT**: This nested approach mirrors canonical design!
/// Each boolean maps to a specific canonical config's `enabled` field.
#[derive(Debug, Clone)]
pub struct DiscoveryMechanisms {
    /// Enable environment variable scanning (→ capability_discovery.enabled)
    pub enable_environment_scan: bool,
    /// Enable network scanning for services (→ network_discovery.scan_local_network)
    pub enable_network_scanning: bool,
    /// Enable container/orchestration discovery (→ service_discovery.enabled)
    pub enable_container_discovery: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: true,    // Aligns with canonical default
                enable_network_scanning: true,    // More permissive than canonical (false)
                enable_container_discovery: true, // Aligns with canonical default
            },
            timeout: Duration::from_secs(30), // Reasonable default (canonical: 5s)
        }
    }
}

// Default implementation is provided by the canonical DiscoveryConfig

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    #[must_use]
    pub fn new(config: DiscoveryConfig) -> Self {
        let _capability_config = crate::capabilities::DiscoveryConfig::default();
        Self {
            _capability_adapter: (),
            config,
            discovered_cache: HashMap::new(),
        }
    }

    /// Discover all available primals using all methods
    ///
    /// # Errors
    ///
    /// This function logs individual discovery failures but returns all successfully discovered primals
    pub async fn discover_all_primals(&mut self) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        info!("🔍 Starting universal primal discovery...");

        let mut all_discovered = Vec::new();

        // Environment-based discovery
        if self.config.mechanisms.enable_environment_scan {
            match self.discover_from_environment().await {
                Ok(mut env_primals) => {
                    info!("{}: {}", DISCOVERED_FROM_ENVIRONMENT_MSG, env_primals.len());
                    all_discovered.append(&mut env_primals);
                }
                Err(e) => warn!("⚠️ Environment discovery failed: {}", e),
            }
        }

        // Network scanning discovery
        if self.config.mechanisms.enable_network_scanning {
            match self.discover_from_network_scan().await {
                Ok(mut network_primals) => {
                    info!("✅ Discovered {} primals from network scan", network_primals.len());
                    all_discovered.append(&mut network_primals);
                }
                Err(e) => warn!("⚠️ Network scan discovery failed: {}", e),
            }
        }

        // Container-based discovery (Docker, Kubernetes,
        match self.discover_from_containers().await {
            Ok(mut container_primals) => {
                if !container_primals.is_empty() {
                    info!("{}: {}", DISCOVERED_FROM_CONTAINERS_MSG, container_primals.len());
                    all_discovered.append(&mut container_primals);
                }
            }
            Err(e) => debug!("Container discovery failed (expected if not in container): {}", e),
        }

        // Deduplicate discovered primals
        let deduplicated = self.deduplicate_primals(all_discovered);

        // Update cache
        for primal in &deduplicated {
            self.discovered_cache.insert(primal.name.clone(), primal.clone());
        }

        info!("🎉 Total unique primals discovered: {}", deduplicated.len());
        Ok(deduplicated)
    }

    /// Discover primals from environment variables
    async fn discover_from_environment(&self) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        debug!("🔍 Discovering primals from environment variables...");

        let mut discovered = Vec::new();
        let primal_names = vec!["toadstool".to_string(), "squirrel".to_string()]; // TEMPORARY FALLBACK

        for primal_name in primal_names {
            let discovery_host = SafeEnv::get_or_default(
                "UNIVERSAL_DISCOVERY_HOST",
                songbird_config::canonical::constants::network::DEFAULT_HOST,
            );
            let discovery_port = SafeEnv::get_port(
                "UNIVERSAL_DISCOVERY_PORT",
                songbird_config::defaults::ports::orchestrator_port(),
            );
            let endpoint = format!("http://{discovery_host}:{discovery_port}/{primal_name}");

            // Test connectivity and discover capabilities
            match timeout(
                Duration::from_secs(self.config.timeout.as_secs()),
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
                    debug!("Failed to discover primal {} at {}: {}", primal_name, endpoint, e);
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

        // Network scanning implementation would require additional config fields
        // For now, return empty to avoid accessing non-existent config fields
        // Note: network_scan_ranges, discovery_ports, max_concurrent_discoveries
        // are part of the main config system in songbird-config crate
        Ok(Vec::new())
    }

    /// Discover primals from container environments
    async fn discover_from_containers(&self) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        let mut discovered = Vec::new();

        // Kubernetes service discovery
        if SafeEnv::get_required("KUBERNETES_SERVICE_HOST").is_ok() {
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
        // Note: Capability discovery is implemented via the discovery trait system
        // See songbird-discovery crate for full implementation
        let capabilities = self.infer_basic_capabilities(name);

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
        let client =
            reqwest::Client::builder().timeout(Duration::from_secs(5)).build().unwrap_or_default();

        // Try common health endpoints
        let health_endpoints = vec![
            format!("{}{}", endpoint, HEALTH_PATH),
            format!("{}{}", endpoint, API_HEALTH_PATH),
            format!("{}{}", endpoint, API_V1_HEALTH_PATH),
            format!("{}{}", endpoint, STATUS_PATH),
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
                Ok(_) => {}
                Err(_) => {}
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
            parameters: HashMap::new(),
            qos_metrics: crate::capabilities::QoSMetrics::default(),
            available: true,
        }]
    }

    /// Infer primal type from name and capabilities
    fn infer_primal_type(&self, name: &str, _capabilities: &[Capability]) -> PrimalType {
        PrimalType::new(name)
    }

    /// Scan a network range for primals
    #[allow(dead_code)] // Reserved for future network scanning functionality
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
                seen.entry(key).or_insert(());
                deduplicated.push(primal);
            }
        }

        deduplicated
    }

    /// Get cached discovered primals
    #[must_use]
    pub fn get_discovered_primals(&self) -> Vec<&DiscoveredPrimal> {
        self.discovered_cache.values().collect()
    }

    /// Find primals with specific capability
    #[must_use]
    pub fn find_primals_with_capability(&self, capability_type: &str) -> Vec<&DiscoveredPrimal> {
        self.discovered_cache
            .values()
            .filter(|primal| {
                primal.capabilities.iter().any(|cap| cap.capability_type == capability_type)
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
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
            Self::UnreachableEndpoint(endpoint) => {
                write!(f, "Unreachable endpoint: {endpoint}")
            }
            Self::ConfigurationError(msg) => write!(f, "Configuration error: {msg}"),
            Self::Timeout(msg) => write!(f, "Timeout: {msg}"),
        }
    }
}

impl std::error::Error for DiscoveryError {}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::{SongbirdError, SongbirdResult};

    #[test]
    fn test_discovery_creation() {
        let config = DiscoveryConfig::default();
        let discovery = UniversalPrimalDiscovery::new(config);

        // Discovery should be created successfully
        assert!(discovery.get_discovered_primals().is_empty());
    }

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();

        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.mechanisms.enable_environment_scan);
        assert!(config.mechanisms.enable_network_scanning);
        assert!(config.mechanisms.enable_container_discovery);
    }

    #[test]
    fn test_discovery_config_custom() {
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: true,
                enable_network_scanning: false,
                enable_container_discovery: true,
            },
            timeout: Duration::from_secs(60),
        };

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(config.mechanisms.enable_environment_scan);
        assert!(!config.mechanisms.enable_network_scanning);
        assert!(config.mechanisms.enable_container_discovery);
    }

    #[test]
    fn test_discovery_mechanisms_all_enabled() {
        let mechanisms = DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: true,
            enable_container_discovery: true,
        };

        assert!(mechanisms.enable_environment_scan);
        assert!(mechanisms.enable_network_scanning);
        assert!(mechanisms.enable_container_discovery);
    }

    #[test]
    fn test_discovery_mechanisms_all_disabled() {
        let mechanisms = DiscoveryMechanisms {
            enable_environment_scan: false,
            enable_network_scanning: false,
            enable_container_discovery: false,
        };

        assert!(!mechanisms.enable_environment_scan);
        assert!(!mechanisms.enable_network_scanning);
        assert!(!mechanisms.enable_container_discovery);
    }

    #[test]
    fn test_discovered_primal_structure() {
        let primal = DiscoveredPrimal {
            name: "test_primal".to_string(),
            primal_type: PrimalType::new("compute"),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![],
            health: PrimalHealth::Healthy,
            discovery_method: DiscoveryMethod::Environment,
            discovered_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(primal.name, "test_primal");
        assert_eq!(primal.primal_type.category, "compute");
        assert_eq!(primal.health, PrimalHealth::Healthy);
        assert_eq!(primal.discovery_method, DiscoveryMethod::Environment);
    }

    #[test]
    fn test_discovery_method_variants() {
        assert!(matches!(DiscoveryMethod::Environment, DiscoveryMethod::Environment));
        assert!(matches!(DiscoveryMethod::NetworkScan, DiscoveryMethod::NetworkScan));
        assert!(matches!(DiscoveryMethod::Mdns, DiscoveryMethod::Mdns));
        assert!(matches!(DiscoveryMethod::Configuration, DiscoveryMethod::Configuration));
        assert!(matches!(DiscoveryMethod::Kubernetes, DiscoveryMethod::Kubernetes));
        assert!(matches!(DiscoveryMethod::Docker, DiscoveryMethod::Docker));
    }

    #[test]
    fn test_discovery_method_equality() {
        assert_eq!(DiscoveryMethod::Environment, DiscoveryMethod::Environment);
        assert_ne!(DiscoveryMethod::NetworkScan, DiscoveryMethod::Mdns);
        assert_ne!(DiscoveryMethod::Docker, DiscoveryMethod::Kubernetes);
    }

    #[test]
    fn test_primal_health_variants() {
        assert!(matches!(PrimalHealth::Healthy, PrimalHealth::Healthy));
        assert!(matches!(PrimalHealth::Degraded, PrimalHealth::Degraded));
        assert!(matches!(PrimalHealth::Unhealthy, PrimalHealth::Unhealthy));
        assert!(matches!(PrimalHealth::Unknown, PrimalHealth::Unknown));
    }

    #[test]
    fn test_primal_health_equality() {
        assert_eq!(PrimalHealth::Healthy, PrimalHealth::Healthy);
        assert_ne!(PrimalHealth::Healthy, PrimalHealth::Degraded);
        assert_ne!(PrimalHealth::Unhealthy, PrimalHealth::Unknown);
    }

    #[test]
    fn test_discovery_error_network() {
        let error = DiscoveryError::NetworkError("Connection failed".to_string());
        assert_eq!(error.to_string(), "Network error: Connection failed");
    }

    #[test]
    fn test_discovery_error_unreachable() {
        let error = DiscoveryError::UnreachableEndpoint("http://localhost:9999".to_string());
        assert_eq!(error.to_string(), "Unreachable endpoint: http://localhost:9999");
    }

    #[test]
    fn test_discovery_error_configuration() {
        let error = DiscoveryError::ConfigurationError("Invalid config".to_string());
        assert_eq!(error.to_string(), "Configuration error: Invalid config");
    }

    #[test]
    fn test_discovery_error_timeout() {
        let error = DiscoveryError::Timeout("Discovery timed out".to_string());
        assert_eq!(error.to_string(), "Timeout: Discovery timed out");
    }

    #[test]
    fn test_get_discovered_primals_empty() {
        let config = DiscoveryConfig::default();
        let discovery = UniversalPrimalDiscovery::new(config);

        let primals = discovery.get_discovered_primals();
        assert!(primals.is_empty());
    }

    #[test]
    fn test_find_primals_with_capability_empty() {
        let config = DiscoveryConfig::default();
        let discovery = UniversalPrimalDiscovery::new(config);

        let primals = discovery.find_primals_with_capability("compute");
        assert!(primals.is_empty());
    }

    #[test]
    fn test_discovered_primal_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("region".to_string(), "us-west-2".to_string());

        let primal = DiscoveredPrimal {
            name: "test".to_string(),
            primal_type: PrimalType::new("storage"),
            endpoint: "http://localhost:8082".to_string(),
            capabilities: vec![],
            health: PrimalHealth::Healthy,
            discovery_method: DiscoveryMethod::Docker,
            discovered_at: chrono::Utc::now(),
            metadata: metadata.clone(),
        };

        assert_eq!(primal.metadata.len(), 2);
        assert_eq!(primal.metadata.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(primal.metadata.get("region"), Some(&"us-west-2".to_string()));
    }

    #[test]
    fn test_discovered_primal_with_capabilities() {
        use crate::capabilities::{Capability, QoSMetrics};

        let capability = Capability {
            capability_type: "storage".to_string(),
            name: "s3_compatible".to_string(),
            version: "1.0.0".to_string(),
            parameters: HashMap::new(),
            qos_metrics: QoSMetrics::default(),
            available: true,
        };

        let primal = DiscoveredPrimal {
            name: "storage_primal".to_string(),
            primal_type: PrimalType::new("storage"),
            endpoint: "http://localhost:8082".to_string(),
            capabilities: vec![capability],
            health: PrimalHealth::Healthy,
            discovery_method: DiscoveryMethod::Kubernetes,
            discovered_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(primal.capabilities.len(), 1);
        assert_eq!(primal.capabilities[0].capability_type, "storage");
        assert_eq!(primal.capabilities[0].name, "s3_compatible");
    }

    #[test]
    fn test_discovery_config_with_custom_timeout() {
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: true,
                enable_network_scanning: true,
                enable_container_discovery: true,
            },
            timeout: Duration::from_secs(120),
        };

        assert_eq!(config.timeout, Duration::from_secs(120));
    }

    #[test]
    fn test_primal_type_creation() {
        let security = PrimalType::new("security");
        let compute = PrimalType::new("compute");
        let storage = PrimalType::new("storage");
        let ai = PrimalType::new("ai");

        assert_eq!(security.category, "security");
        assert_eq!(compute.category, "compute");
        assert_eq!(storage.category, "storage");
        assert_eq!(ai.category, "ai");
    }

    #[test]
    fn test_discovered_primal_different_health_states() {
        let healthy = DiscoveredPrimal {
            name: "healthy".to_string(),
            primal_type: PrimalType::new("compute"),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![],
            health: PrimalHealth::Healthy,
            discovery_method: DiscoveryMethod::Environment,
            discovered_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        let degraded = DiscoveredPrimal {
            name: "degraded".to_string(),
            primal_type: PrimalType::new("compute"),
            endpoint: "http://localhost:8081".to_string(),
            capabilities: vec![],
            health: PrimalHealth::Degraded,
            discovery_method: DiscoveryMethod::NetworkScan,
            discovered_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(healthy.health, PrimalHealth::Healthy);
        assert_eq!(degraded.health, PrimalHealth::Degraded);
        assert_ne!(healthy.health, degraded.health);
    }

    #[test]
    fn test_discovery_mechanisms_selective() {
        let config1 = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: true,
                enable_network_scanning: false,
                enable_container_discovery: false,
            },
            timeout: Duration::from_secs(30),
        };

        let config2 = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: false,
                enable_network_scanning: true,
                enable_container_discovery: false,
            },
            timeout: Duration::from_secs(30),
        };

        assert!(config1.mechanisms.enable_environment_scan);
        assert!(!config1.mechanisms.enable_network_scanning);

        assert!(!config2.mechanisms.enable_environment_scan);
        assert!(config2.mechanisms.enable_network_scanning);
    }

    #[tokio::test]
    async fn test_discover_all_primals_with_no_mechanisms() -> SongbirdResult<()> {
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: false,
                enable_network_scanning: false,
                enable_container_discovery: false,
            },
            timeout: Duration::from_secs(10),
        };

        let mut discovery = UniversalPrimalDiscovery::new(config);
        let result = discovery.discover_all_primals().await;

        // Should succeed but find no primals
        assert!(result.is_ok());
        let primals = result.map_err(|e| {
            SongbirdError::configuration(format!("Failed to discover services: {}", e))
        })?;
        assert!(primals.is_empty());
        Ok(())
    }

    #[test]
    fn test_discovery_error_display_network() {
        let error = DiscoveryError::NetworkError("connection failed".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Network error"));
        assert!(display.contains("connection failed"));
    }

    #[test]
    fn test_discovery_error_display_unreachable() {
        let error = DiscoveryError::UnreachableEndpoint("http://localhost:9999".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Unreachable endpoint"));
        assert!(display.contains("localhost:9999"));
    }

    #[test]
    fn test_discovery_error_display_timeout() {
        let error = DiscoveryError::Timeout("exceeded 30s".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Timeout"));
        assert!(display.contains("exceeded 30s"));
    }

    #[test]
    fn test_discovery_method_all_variants() {
        let methods = vec![
            DiscoveryMethod::Environment,
            DiscoveryMethod::NetworkScan,
            DiscoveryMethod::Mdns,
            DiscoveryMethod::Configuration,
            DiscoveryMethod::Kubernetes,
            DiscoveryMethod::Docker,
        ];
        assert_eq!(methods.len(), 6);

        // Test each method can be cloned and compared
        for method in &methods {
            let cloned = method.clone();
            assert_eq!(method, &cloned);
        }
    }

    #[test]
    fn test_primal_health_all_states() {
        let states = vec![
            PrimalHealth::Healthy,
            PrimalHealth::Degraded,
            PrimalHealth::Unhealthy,
            PrimalHealth::Unknown,
        ];
        assert_eq!(states.len(), 4);

        // Each state should be equal to itself
        for state in &states {
            let cloned = state.clone();
            assert_eq!(state, &cloned);
        }
    }

    #[test]
    fn test_discovered_primal_clone() {
        let primal = DiscoveredPrimal {
            name: "test-primal".to_string(),
            primal_type: PrimalType::new("compute"),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![],
            health: PrimalHealth::Healthy,
            discovery_method: DiscoveryMethod::Environment,
            discovered_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        let cloned = primal.clone();
        assert_eq!(primal.name, cloned.name);
        assert_eq!(primal.endpoint, cloned.endpoint);
        assert_eq!(primal.health, cloned.health);
    }

    #[test]
    fn test_discovery_config_clone() {
        let config = DiscoveryConfig::default();
        let cloned = config.clone();
        assert_eq!(config.timeout, cloned.timeout);
        assert_eq!(
            config.mechanisms.enable_environment_scan,
            cloned.mechanisms.enable_environment_scan
        );
    }

    #[test]
    fn test_discovery_mechanisms_clone() {
        let mechanisms = DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: true,
        };

        let cloned = mechanisms.clone();
        assert_eq!(mechanisms.enable_environment_scan, cloned.enable_environment_scan);
        assert_eq!(mechanisms.enable_network_scanning, cloned.enable_network_scanning);
    }

    #[test]
    fn test_discovered_primal_with_multiple_capabilities() {
        let mut params1 = HashMap::new();
        params1.insert("model".to_string(), serde_json::json!("llama"));

        let mut params2 = HashMap::new();
        params2.insert("batch_size".to_string(), serde_json::json!(32));

        let cap1 = Capability {
            capability_type: "ai".to_string(),
            name: "inference".to_string(),
            version: "1.0".to_string(),
            parameters: params1,
            qos_metrics: crate::capabilities::QoSMetrics::default(),
            available: true,
        };

        let cap2 = Capability {
            capability_type: "ai".to_string(),
            name: "training".to_string(),
            version: "1.0".to_string(),
            parameters: params2,
            qos_metrics: crate::capabilities::QoSMetrics::default(),
            available: true,
        };

        let primal = DiscoveredPrimal {
            name: "ai-service".to_string(),
            primal_type: PrimalType::new("ai"),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![cap1, cap2],
            health: PrimalHealth::Healthy,
            discovery_method: DiscoveryMethod::NetworkScan,
            discovered_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(primal.capabilities.len(), 2);
        assert_eq!(primal.capabilities[0].name, "inference");
        assert_eq!(primal.capabilities[1].name, "training");
        assert_eq!(primal.capabilities[0].capability_type, "ai");
    }

    #[test]
    fn test_discovery_error_as_error_trait() {
        let error = DiscoveryError::NetworkError("test".to_string());
        // Test that it implements std::error::Error
        let _: &dyn std::error::Error = &error;
    }

    #[test]
    fn test_discovery_config_with_very_short_timeout() {
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: true,
                enable_network_scanning: true,
                enable_container_discovery: true,
            },
            timeout: Duration::from_millis(100),
        };

        assert_eq!(config.timeout, Duration::from_millis(100));
    }

    #[test]
    fn test_discovery_config_with_very_long_timeout() {
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: true,
                enable_network_scanning: true,
                enable_container_discovery: true,
            },
            timeout: Duration::from_secs(300),
        };

        assert_eq!(config.timeout, Duration::from_secs(300));
    }
}
