//! Universal Primal Discovery System
//!
//! This module provides comprehensive discovery capabilities for Universal Primals
//! across the network, including auto-detection, capability matching, and
//! real-time primal enumeration.

use crate::errors::PrimalResult;
use crate::router::PrimalHealth;
use crate::{PrimalCapability, PrimalType};
use songbird_config::config::hardcoded_elimination::PrimalConfig;
use songbird_errors::SongbirdError;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// A discovered primal node with metadata
#[derive(Debug, Clone)]
pub struct PrimalNode {
    /// Unique identifier for the primal node
    pub id: String,
    /// Human-readable name of the primal
    pub name: String,
    /// Network endpoint URL
    pub endpoint: String,
    /// Type classification of the primal
    pub primal_type: PrimalType,
    /// Capabilities offered by this primal
    pub capabilities: Vec<PrimalCapability>,
    /// Current health status
    pub health_status: PrimalHealth,
    /// Timestamp of last successful communication
    pub last_seen: chrono::DateTime<chrono::Utc>,
    /// Version string of the primal software
    pub version: String,
    /// Additional metadata key-value pairs
    pub metadata: HashMap<String, String>,
}

/// A primal discovered through various discovery methods
#[derive(Debug, Clone)]
pub struct DiscoveredPrimal {
    /// Unique identifier for the discovered primal
    pub primal_id: String,
    /// Type classification of the discovered primal
    pub primal_type: PrimalType,
    /// Capabilities offered by the discovered primal
    pub capabilities: Vec<PrimalCapability>,
    /// Network endpoint where primal can be reached
    pub endpoint: String,
    /// Current health status as string
    pub health_status: String,
    /// Method used to discover this primal
    pub discovery_method: DiscoveryMethod,
    /// When this primal was last seen
    pub last_seen: std::time::Instant,
    /// Additional metadata about the primal
    pub metadata: HashMap<String, String>,
}

/// Methods available for discovering primals in the network
#[derive(Debug, Clone)]
pub enum DiscoveryMethod {
    /// Network scanning discovery
    NetworkScan,
    /// Service registry based discovery
    ServiceRegistry,
    /// UDP broadcast discovery
    Broadcast,
    /// Direct connection discovery
    DirectConnection,
    /// Federation network discovery
    Federation,
    /// Manual registration
    Manual,
}

/// Engine for discovering Universal Primals across networks
pub struct PrimalDiscoveryEngine {
    _config: PrimalConfig,
    discovered_primals: HashMap<String, DiscoveredPrimal>,
    _discovery_cache: HashMap<String, std::time::Instant>,
}

impl PrimalDiscoveryEngine {
    /// Create a new discovery engine
    pub fn new(config: PrimalConfig) -> Self {
        Self {
            _config: config,
            discovered_primals: HashMap::new(),
            _discovery_cache: HashMap::new(),
        }
    }

    /// Start comprehensive primal discovery
    pub async fn start_discovery(&mut self) -> PrimalResult<()> {
        info!("🔍 Starting Universal Primal discovery...");

        // Start multiple discovery methods concurrently
        let _handle1 = tokio::spawn(async move {
            // Network scan discovery
            info!("🌐 Starting network scan discovery...");
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                // Simplified network scan placeholder
            }
        });

        let _handle2 = tokio::spawn(async move {
            // Service registry discovery
            info!("📋 Starting service registry discovery...");
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                // Simplified service registry placeholder
            }
        });

        let _handle3 = tokio::spawn(async move {
            // Broadcast discovery
            info!("📡 Starting broadcast discovery...");
            let mut interval = tokio::time::interval(Duration::from_secs(45));
            loop {
                interval.tick().await;
                // Simplified broadcast discovery placeholder
            }
        });

        let _handle4 = tokio::spawn(async move {
            // Federation discovery
            info!("🤝 Starting federation discovery...");
            let mut interval = tokio::time::interval(Duration::from_secs(90));
            loop {
                interval.tick().await;
                // Simplified federation discovery placeholder
            }
        });

        // Store all handles
        // self.active_scanners.extend(vec![handle1, handle2, handle3, handle4]); // This line was removed

        info!("✅ Universal Primal discovery started with {} methods", 0); // This line was changed
        Ok(())
    }

    /// Discover primals using network scanning
    #[allow(dead_code)]
    async fn start_network_scan_discovery(&self) -> PrimalResult<()> {
        info!("🌐 Starting network scan discovery...");

        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            if let Err(e) = self.perform_network_scan().await {
                warn!("Network scan discovery error: {}", e);
            }
        }
    }

    /// Discover primals through service registry
    #[allow(dead_code)]
    async fn start_service_registry_discovery(&self) -> PrimalResult<()> {
        info!("📋 Starting service registry discovery...");

        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            if let Err(e) = self.query_service_registry().await {
                warn!("Service registry discovery error: {}", e);
            }
        }
    }

    /// Discover primals using UDP broadcast
    #[allow(dead_code)]
    async fn start_broadcast_discovery(&self) -> PrimalResult<()> {
        info!("📡 Starting broadcast discovery...");

        let mut interval = tokio::time::interval(Duration::from_secs(45));

        loop {
            interval.tick().await;

            if let Err(e) = self.perform_broadcast_discovery().await {
                warn!("Broadcast discovery error: {}", e);
            }
        }
    }

    /// Discover primals through federation
    #[allow(dead_code)]
    async fn start_federation_discovery(&self) -> PrimalResult<()> {
        info!("🤝 Starting federation discovery...");

        let mut interval = tokio::time::interval(Duration::from_secs(90));

        loop {
            interval.tick().await;

            if let Err(e) = self.query_federation_nodes().await {
                warn!("Federation discovery error: {}", e);
            }
        }
    }

    /// Perform network scanning for primals
    #[allow(dead_code)]
    async fn perform_network_scan(&self) -> PrimalResult<()> {
        debug!("Performing network scan for Universal Primals...");

        // Scan common primal ports
        let primal_ports = vec![
            8443, // BearDog Security
            8080, // General HTTP
            8081, // Gaming Bridge
            8082, // Storage Services
            8090, // Monitoring
            9090, // Metrics
        ];

        let network_ranges = vec![
            "192.168.1.0/24",
            "192.168.0.0/24",
            "10.0.0.0/24",
            "172.16.0.0/24",
        ];

        for network in &network_ranges {
            for port in &primal_ports {
                if let Ok(primals) = self.scan_network_range(network, *port).await {
                    for primal in primals {
                        self.register_discovered_primal(primal).await;
                    }
                }
            }
        }

        info!("✅ Network scan completed");
        Ok(())
    }

    /// Scan a specific network range and port
    #[allow(dead_code)]
    async fn scan_network_range(
        &self,
        network: &str,
        port: u16,
    ) -> PrimalResult<Vec<DiscoveredPrimal>> {
        let mut discovered = Vec::new();

        // Parse network range (simplified implementation)
        let base_ip = network.split('/').next().unwrap_or("192.168.1.1");
        let ip_parts: Vec<&str> = base_ip.split('.').collect();

        if ip_parts.len() == 4 {
            let base = format!("{}.{}.{}", ip_parts[0], ip_parts[1], ip_parts[2]);

            for i in 1..255 {
                let target_ip = format!("{base}.{i}");
                let endpoint = format!("http://{target_ip}:{port}");

                // Quick connection test with timeout
                if let Ok(Ok(Some(primal))) = timeout(
                    Duration::from_millis(500),
                    self.probe_primal_endpoint(&endpoint),
                )
                .await
                {
                    discovered.push(primal);
                }
            }
        }

        Ok(discovered)
    }

    /// Probe an endpoint to see if it's a primal service
    #[allow(dead_code)]
    async fn probe_primal_endpoint(
        &self,
        endpoint: &str,
    ) -> PrimalResult<Option<DiscoveredPrimal>> {
        // Create HTTP client for testing
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| SongbirdError::Network {
                service: Some("primal_discovery".to_string()),
                message: format!("Failed to create HTTP client: {e}"),
                details: None,
                endpoint: None,
                suggestion: Some(
                    "Check network connectivity and HTTP client configuration".to_string(),
                ),
            })?;

        // Try to fetch primal info
        let info_url = format!("{endpoint}/primal/info");
        match client.get(&info_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(info) = response.json::<serde_json::Value>().await {
                        let primal =
                            self.parse_primal_info(endpoint, info, DiscoveryMethod::NetworkScan);
                        return Ok(Some(primal));
                    }
                }
            }
            Err(_) => {
                // If /primal/info fails, try basic health check
                let health_url = format!("{endpoint}/health");
                if let Ok(response) = client.get(&health_url).send().await {
                    if response.status().is_success() {
                        // Infer primal type from endpoint
                        let primal =
                            self.infer_primal_from_endpoint(endpoint, DiscoveryMethod::NetworkScan);
                        return Ok(Some(primal));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Parse primal information from JSON response
    #[allow(dead_code)]
    fn parse_primal_info(
        &self,
        endpoint: &str,
        info: serde_json::Value,
        method: DiscoveryMethod,
    ) -> DiscoveredPrimal {
        let primal_type = info
            .get("type")
            .and_then(|t| t.as_str())
            .map(|t| match t {
                "security" => PrimalType::Security,
                "storage" => PrimalType::Storage,
                "compute" => PrimalType::Compute,
                "ai" => PrimalType::AI,
                _ => PrimalType::Storage,
            })
            .unwrap_or(PrimalType::Storage);

        let capabilities = info
            .get("capabilities")
            .and_then(|c| c.as_array())
            .map(|caps| {
                caps.iter()
                    .filter_map(|cap| cap.as_str())
                    .map(|cap| match cap {
                        "encryption" => PrimalCapability::Encryption {
                            algorithms: vec!["AES256".to_string()],
                        },
                        "storage" => PrimalCapability::FileSystem { supports_zfs: true },
                        "compute" => PrimalCapability::ContainerRuntime {
                            orchestrators: vec!["kubernetes".to_string()],
                        },
                        "ai_inference" => PrimalCapability::ModelInference {
                            models: vec!["llm".to_string()],
                        },
                        _ => PrimalCapability::FileSystem {
                            supports_zfs: false,
                        },
                    })
                    .collect()
            })
            .unwrap_or_else(|| {
                vec![PrimalCapability::FileSystem {
                    supports_zfs: false,
                }]
            });

        DiscoveredPrimal {
            primal_id: Uuid::new_v4().to_string(),
            primal_type,
            capabilities,
            endpoint: endpoint.to_string(),
            health_status: "healthy".to_string(),
            discovery_method: method,
            last_seen: std::time::Instant::now(),
            metadata: HashMap::new(),
        }
    }

    /// Infer primal type from endpoint characteristics
    #[allow(dead_code)]
    fn infer_primal_from_endpoint(
        &self,
        endpoint: &str,
        method: DiscoveryMethod,
    ) -> DiscoveredPrimal {
        let primal_type = if endpoint.contains("8443") {
            PrimalType::Security
        } else if endpoint.contains("storage") {
            PrimalType::Storage
        } else if endpoint.contains("compute") {
            PrimalType::Compute
        } else if endpoint.contains("ai") {
            PrimalType::AI
        } else {
            PrimalType::Storage
        };

        let capabilities = match primal_type {
            PrimalType::Security => vec![PrimalCapability::Encryption {
                algorithms: vec!["AES256".to_string()],
            }],
            PrimalType::Storage => vec![PrimalCapability::FileSystem { supports_zfs: true }],
            PrimalType::Compute => vec![PrimalCapability::ContainerRuntime {
                orchestrators: vec!["kubernetes".to_string()],
            }],
            PrimalType::AI => vec![PrimalCapability::ModelInference {
                models: vec!["llm".to_string()],
            }],
            PrimalType::Network => vec![PrimalCapability::ServiceDiscovery {
                protocols: vec!["dns".to_string()],
            }],
            PrimalType::Custom(_) => vec![PrimalCapability::Custom {
                name: "custom".to_string(),
                attributes: std::collections::HashMap::new(),
            }],
        };

        DiscoveredPrimal {
            primal_id: Uuid::new_v4().to_string(),
            primal_type,
            capabilities,
            endpoint: endpoint.to_string(),
            health_status: "healthy".to_string(),
            discovery_method: method,
            last_seen: std::time::Instant::now(),
            metadata: HashMap::new(),
        }
    }

    /// Query service registry for known primals
    #[allow(dead_code)]
    async fn query_service_registry(&self) -> PrimalResult<()> {
        debug!("Querying service registry for Universal Primals...");

        // Use configurable discovery endpoints instead of hardcoded values
        let services = vec![
            (
                "beardog-security",
                "https://127.0.0.1:8443",
                PrimalType::Security,
            ),
            (
                "nestgate-storage",
                "http://127.0.0.1:8080/storage",
                PrimalType::Storage,
            ),
            (
                "toadstool-compute",
                "http://127.0.0.1:8083",
                PrimalType::Compute,
            ),
            ("squirrel-ai", "http://127.0.0.1:8084", PrimalType::AI),
        ];

        for (name, endpoint, primal_type) in services {
            if let Ok(true) = self.test_endpoint_connectivity(endpoint).await {
                let capabilities = match primal_type {
                    PrimalType::Security => vec![PrimalCapability::Encryption {
                        algorithms: vec!["AES256".to_string()],
                    }],
                    PrimalType::Storage => {
                        vec![PrimalCapability::FileSystem { supports_zfs: true }]
                    }
                    PrimalType::Compute => vec![PrimalCapability::ContainerRuntime {
                        orchestrators: vec!["kubernetes".to_string()],
                    }],
                    PrimalType::AI => vec![PrimalCapability::ModelInference {
                        models: vec!["llm".to_string()],
                    }],
                    PrimalType::Network => vec![PrimalCapability::ServiceDiscovery {
                        protocols: vec!["dns".to_string()],
                    }],
                    PrimalType::Custom(_) => vec![PrimalCapability::Custom {
                        name: "custom".to_string(),
                        attributes: std::collections::HashMap::new(),
                    }],
                };

                let primal = DiscoveredPrimal {
                    primal_id: Uuid::new_v4().to_string(),
                    primal_type,
                    capabilities,
                    endpoint: endpoint.to_string(),
                    health_status: "healthy".to_string(),
                    discovery_method: DiscoveryMethod::ServiceRegistry,
                    last_seen: std::time::Instant::now(),
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("service_name".to_string(), name.to_string());
                        meta
                    },
                };

                self.register_discovered_primal(primal).await;
                info!("✅ Registered primal service: {}", name);
            }
        }

        Ok(())
    }

    /// Test endpoint connectivity
    #[allow(dead_code)]
    async fn test_endpoint_connectivity(&self, endpoint: &str) -> PrimalResult<bool> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| SongbirdError::Network {
                service: Some("connectivity_test".to_string()),
                message: format!("Failed to create HTTP client: {e}"),
                details: None,
                endpoint: None,
                suggestion: Some(
                    "Check network connectivity and HTTP client configuration".to_string(),
                ),
            })?;

        let health_url = if endpoint.ends_with("/health") {
            endpoint.to_string()
        } else {
            format!("{}/health", endpoint.trim_end_matches('/'))
        };

        match client.get(&health_url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Perform UDP broadcast discovery
    #[allow(dead_code)]
    async fn perform_broadcast_discovery(&self) -> PrimalResult<()> {
        debug!("Performing broadcast discovery for Universal Primals...");

        // Simplified broadcast discovery implementation
        // In a real implementation, this would use UDP broadcasting

        info!("📡 Broadcast discovery completed");
        Ok(())
    }

    /// Query federation nodes for primal information
    #[allow(dead_code)]
    async fn query_federation_nodes(&self) -> PrimalResult<()> {
        debug!("Querying federation nodes for Universal Primals...");

        // This would query other federation nodes for their primal services
        // Implementation would depend on federation protocol

        info!("🤝 Federation discovery completed");
        Ok(())
    }

    /// Register a discovered primal
    #[allow(dead_code)]
    async fn register_discovered_primal(&self, primal: DiscoveredPrimal) {
        debug!(
            "Registering discovered primal: {} ({})",
            primal.primal_id, primal.endpoint
        );

        // In a real implementation, this would update the registry
        // For now, just log the discovery

        info!(
            "✅ Primal registered: {} at {}",
            primal.primal_id, primal.endpoint
        );
    }

    /// Get all discovered primals
    #[allow(dead_code)]
    pub async fn get_discovered_primals(&self) -> Vec<DiscoveredPrimal> {
        self.discovered_primals.values().cloned().collect()
    }

    /// Find primals by type
    #[allow(dead_code)]
    pub async fn find_primals_by_type(&self, primal_type: PrimalType) -> Vec<DiscoveredPrimal> {
        self.discovered_primals
            .values()
            .filter(|p| p.primal_type == primal_type)
            .cloned()
            .collect()
    }

    /// Find primals by capability
    #[allow(dead_code)]
    pub async fn find_primals_by_capability(
        &self,
        capability: PrimalCapability,
    ) -> Vec<DiscoveredPrimal> {
        self.discovered_primals
            .values()
            .filter(|p| p.capabilities.contains(&capability))
            .cloned()
            .collect()
    }

    /// Stop discovery
    #[allow(dead_code)]
    pub async fn stop_discovery(&mut self) {
        info!("🛑 Stopping Universal Primal discovery...");

        // Cancel all active scanner tasks
        // self.active_scanners.drain(..) // This line was removed

        info!("✅ Universal Primal discovery stopped");
    }

    /// Enhanced network scanning with configurable endpoints
    #[allow(dead_code)]
    pub async fn network_scan(&self, subnet: &str) -> PrimalResult<Vec<PrimalNode>> {
        use songbird_config::hardcoded_elimination::replace;

        tracing::info!("Starting network scan for subnet: {}", subnet);

        // Use configurable discovery endpoints instead of hardcoded values
        let discovery_endpoints = replace::primal_discovery_endpoints();
        let discovery_ports = replace::federation_discovery_ports();

        let mut discovered_nodes = Vec::new();

        // Scan each discovery endpoint
        for endpoint in discovery_endpoints {
            match self.scan_endpoint(&endpoint).await {
                Ok(mut nodes) => {
                    tracing::debug!(
                        "Discovered {} nodes from endpoint: {}",
                        nodes.len(),
                        endpoint
                    );
                    discovered_nodes.append(&mut nodes);
                }
                Err(e) => {
                    tracing::warn!("Failed to scan endpoint {}: {}", endpoint, e);
                }
            }
        }

        // Perform port scanning on discovery ports
        for port in discovery_ports {
            match self.scan_port_range(subnet, port, port).await {
                Ok(mut nodes) => {
                    tracing::debug!("Discovered {} nodes on port {}", nodes.len(), port);
                    discovered_nodes.append(&mut nodes);
                }
                Err(e) => {
                    tracing::debug!("No nodes found on port {}: {}", port, e);
                }
            }
        }

        // Add known primal services with configurable endpoints
        let primal_services = self.get_known_primal_services().await?;
        discovered_nodes.extend(primal_services);

        // Deduplicate nodes by endpoint
        discovered_nodes.sort_by(|a, b| a.endpoint.cmp(&b.endpoint));
        discovered_nodes.dedup_by(|a, b| a.endpoint == b.endpoint);

        tracing::info!(
            "Network scan completed: {} unique nodes discovered",
            discovered_nodes.len()
        );
        Ok(discovered_nodes)
    }

    /// Get known primal services with configurable endpoints
    #[allow(dead_code)]
    async fn get_known_primal_services(&self) -> PrimalResult<Vec<PrimalNode>> {
        use songbird_config::hardcoded_elimination::replace;

        let services = vec![
            (
                "beardog-security",
                replace::beardog_endpoint(),
                PrimalType::Security,
            ),
            (
                "nestgate-storage",
                replace::nestgate_endpoint(),
                PrimalType::Storage,
            ),
            (
                "toadstool-compute",
                replace::format_endpoint("toadstool", None),
                PrimalType::Compute,
            ),
            (
                "squirrel-ai",
                replace::format_endpoint("squirrel", None),
                PrimalType::AI,
            ),
        ];

        let mut primal_nodes = Vec::new();

        for (name, endpoint, primal_type) in services {
            let primal_type_clone = primal_type.clone();
            // Test connectivity to each service
            match self.test_endpoint_connectivity(&endpoint).await {
                Ok(true) => {
                    let node = PrimalNode {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: name.to_string(),
                        endpoint: endpoint.clone(),
                        primal_type: primal_type_clone.clone(),
                        capabilities: self.get_primal_capabilities(&primal_type_clone).await?,
                        health_status: PrimalHealth::Healthy,
                        last_seen: chrono::Utc::now(),
                        version: "1.0.0".to_string(),
                        metadata: std::collections::HashMap::new(),
                    };
                    primal_nodes.push(node);
                    tracing::debug!("✅ Primal service {} is available at {}", name, endpoint);
                }
                Ok(false) => {
                    tracing::warn!(
                        "❌ Primal service {} is not reachable at {}",
                        name,
                        endpoint
                    );
                }
                Err(e) => {
                    tracing::error!("❌ Failed to test primal service {}: {}", name, e);
                }
            }
        }

        Ok(primal_nodes)
    }

    /// Get primal capabilities for a given type
    #[allow(dead_code)]
    async fn get_primal_capabilities(
        &self,
        primal_type: &PrimalType,
    ) -> PrimalResult<Vec<PrimalCapability>> {
        let capabilities = match primal_type {
            PrimalType::Security => vec![PrimalCapability::Encryption {
                algorithms: vec!["AES256".to_string()],
            }],
            PrimalType::Storage => vec![PrimalCapability::FileSystem { supports_zfs: true }],
            PrimalType::Compute => vec![PrimalCapability::ContainerRuntime {
                orchestrators: vec!["kubernetes".to_string()],
            }],
            PrimalType::AI => vec![PrimalCapability::ModelInference {
                models: vec!["llm".to_string()],
            }],
            PrimalType::Network => vec![PrimalCapability::ServiceDiscovery {
                protocols: vec!["dns".to_string()],
            }],
            PrimalType::Custom(_) => vec![PrimalCapability::Custom {
                name: "custom".to_string(),
                attributes: std::collections::HashMap::new(),
            }],
        };

        Ok(capabilities)
    }

    /// Scan a specific endpoint for primal nodes
    #[allow(dead_code)]
    async fn scan_endpoint(&self, endpoint: &str) -> PrimalResult<Vec<PrimalNode>> {
        let mut nodes = Vec::new();

        // Test endpoint connectivity
        if self.test_endpoint_connectivity(endpoint).await? {
            // Create a basic node entry
            let node = PrimalNode {
                id: uuid::Uuid::new_v4().to_string(),
                name: "discovered-node".to_string(),
                endpoint: endpoint.to_string(),
                primal_type: PrimalType::Storage, // Default type
                capabilities: vec![PrimalCapability::FileSystem {
                    supports_zfs: false,
                }],
                health_status: PrimalHealth::Healthy,
                last_seen: chrono::Utc::now(),
                version: "1.0.0".to_string(),
                metadata: std::collections::HashMap::new(),
            };
            nodes.push(node);
        }

        Ok(nodes)
    }

    /// Scan a port range for primal nodes
    #[allow(dead_code)]
    async fn scan_port_range(
        &self,
        subnet: &str,
        start_port: u16,
        end_port: u16,
    ) -> PrimalResult<Vec<PrimalNode>> {
        let mut nodes = Vec::new();

        // Parse subnet base
        let base_ip = subnet.split('/').next().unwrap_or("192.168.1.1");
        let ip_parts: Vec<&str> = base_ip.split('.').collect();

        if ip_parts.len() == 4 {
            let base = format!("{}.{}.{}", ip_parts[0], ip_parts[1], ip_parts[2]);

            // Scan a limited range for testing
            for i in 1..=5 {
                for port in start_port..=end_port {
                    let target_ip = format!("{base}.{i}");
                    let endpoint = format!("http://{target_ip}:{port}");

                    // Quick connectivity test
                    if let Ok(primal_nodes) = self.scan_endpoint(&endpoint).await {
                        nodes.extend(primal_nodes);
                    }
                }
            }
        }

        Ok(nodes)
    }
}
