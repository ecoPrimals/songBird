//! Universal Capability Adapter implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::connection::{ConnectionHealth, PrimalConnection};
use super::error::CapabilityError;
use super::registry::CapabilityRegistry;
use super::types::{Capability, CapabilityResponse, DiscoveryConfig, PrimalType, QoSMetrics};
use super::HEALTH_PATH;

/// Universal capability adapter that works with any primal
#[derive(Debug, Clone)]
pub struct UniversalCapabilityAdapter {
    /// Registry of discovered primals and their capabilities
    capability_registry: Arc<RwLock<CapabilityRegistry>>,
    /// Active primal connections
    primal_connections: Arc<RwLock<HashMap<String, PrimalConnection>>>,
    /// Discovery configuration
    discovery_config: DiscoveryConfig,
}

impl UniversalCapabilityAdapter {
    /// Create a new universal capability adapter
    #[must_use]
    pub fn new(config: DiscoveryConfig) -> Self {
        Self {
            capability_registry: Arc::new(RwLock::new(CapabilityRegistry::default())),
            primal_connections: Arc::new(RwLock::new(HashMap::new())),
            discovery_config: config,
        }
    }

    /// Discover capabilities for a primal by name
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is unreachable or does not respond with valid capabilities
    pub async fn discover_primal_capabilities(
        &self,
        primal_name: &str,
    ) -> Result<Vec<Capability>, CapabilityError> {
        info!("🔍 Discovering capabilities for primal: {}", primal_name);

        // Get primal endpoint
        let capability_host =
            std::env::var("UNIVERSAL_CAPABILITY_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let capability_port = std::env::var("UNIVERSAL_CAPABILITY_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);
        let endpoint = format!("http://{capability_host}:{capability_port}/{primal_name}");

        // Attempt capability discovery via HTTP
        match self.query_primal_capabilities(&endpoint).await {
            Ok(capabilities) => {
                // Update registry
                let mut registry = self.capability_registry.write().await;
                registry.primal_capabilities.insert(primal_name.to_string(), capabilities.clone());
                registry.last_updated.insert(primal_name.to_string(), chrono::Utc::now());

                // Update capability providers index
                for capability in &capabilities {
                    registry
                        .capability_providers
                        .entry(capability.capability_type.clone())
                        .or_insert_with(Vec::new)
                        .push(primal_name.to_string());
                }

                info!("✅ Discovered {} capabilities for {}", capabilities.len(), primal_name);
                Ok(capabilities)
            }
            Err(e) => {
                warn!("❌ Failed to discover capabilities for {}: {}", primal_name, e);
                Err(e)
            }
        }
    }

    /// Find all primals that provide a specific capability
    pub async fn find_capability_providers(&self, capability_type: &str) -> Vec<String> {
        debug!("🔍 Finding providers for capability: {}", capability_type);

        let mut providers = Vec::new();

        // Check environment variables for capability-based discovery
        let capability_providers =
            self.discover_capability_providers_from_env(capability_type).await;
        providers.extend(capability_providers);

        // Network-based discovery (if enabled)
        if self.discovery_config.enable_network_discovery {
            let network_providers =
                self.discover_capability_providers_from_network(capability_type).await;
            providers.extend(network_providers);
        }

        // Capability inference from known patterns
        let inferred_providers = self.infer_capability_providers(capability_type).await;
        providers.extend(inferred_providers);

        // Remove duplicates and return
        providers.sort();
        providers.dedup();

        debug!(
            "✅ Found {} providers for capability {}: {:?}",
            providers.len(),
            capability_type,
            providers
        );

        providers
    }

    /// Discover capability providers from environment variables
    async fn discover_capability_providers_from_env(&self, capability_type: &str) -> Vec<String> {
        let mut providers = Vec::new();

        // Check for generic capability environment variables
        let capability_env = format!("{}_PROVIDERS ", capability_type.to_uppercase());
        if let Ok(provider_list) = std::env::var(&capability_env) {
            providers.extend(provider_list.split(',').map(|s| s.trim().to_string()));
        }

        // Check for specific primal environment variables
        let known_primals = ["beardog", "toadstool", "nestgate", "squirrel"];
        for primal in &known_primals {
            let endpoint_env = format!("{}{}", primal.to_uppercase(), "_ENDPOINT");
            if std::env::var(&endpoint_env).is_ok() {
                // Check if this primal provides the requested capability
                if self.primal_provides_capability(primal, capability_type) {
                    providers.push((*primal).to_string());
                }
            }
        }

        providers
    }

    /// Discover capability providers from network scanning
    async fn discover_capability_providers_from_network(
        &self,
        capability_type: &str,
    ) -> Vec<String> {
        let providers = Vec::new();

        // Network discovery implementation would go here
        // For now, return empty to avoid network dependencies in basic functionality
        debug!("Network discovery for {} capability - not implemented yet ", capability_type);

        providers
    }

    /// Infer capability providers based on known patterns
    async fn infer_capability_providers(&self, capability_type: &str) -> Vec<String> {
        let mut providers = Vec::new();

        // Infer providers based on capability type patterns
        match capability_type {
            "security" | "encryption" | "authentication" => {
                // Look for security-related primals
                if std::env::var("BEARDOG_ENDPOINT ").is_ok() {
                    providers.push("beardog".to_string());
                }
                // Check for custom security services
                for i in 1..=10 {
                    let primal_env = format!("PRIMAL_{i}_NAME ");
                    let endpoint_env = format!("PRIMAL_{i}_ENDPOINT ");
                    if let (Ok(name), Ok(_)) =
                        (std::env::var(&primal_env), std::env::var(&endpoint_env))
                    {
                        if name.contains("security")
                            || name.contains("auth")
                            || name.contains("crypto")
                        {
                            providers.push(name);
                        }
                    }
                }
            }
            "compute" | "processing" | "execution" => {
                // Look for compute-related primals
                if std::env::var("TOADSTOOL_ENDPOINT ").is_ok() {
                    providers.push("toadstool".to_string());
                }
                // Check for custom compute services
                for i in 1..=10 {
                    let primal_env = format!("PRIMAL_{i}_NAME ");
                    let endpoint_env = format!("PRIMAL_{i}_ENDPOINT ");
                    if let (Ok(name), Ok(_)) =
                        (std::env::var(&primal_env), std::env::var(&endpoint_env))
                    {
                        if name.contains("compute")
                            || name.contains("process")
                            || name.contains("exec")
                        {
                            providers.push(name);
                        }
                    }
                }
            }
            "storage" | "data" | "persistence" => {
                // Look for storage-related primals
                if std::env::var("NESTGATE_ENDPOINT ").is_ok() {
                    providers.push("nestgate".to_string());
                }
                // Check for custom storage services
                for i in 1..=10 {
                    let primal_env = format!("PRIMAL_{i}_NAME ");
                    let endpoint_env = format!("PRIMAL_{i}_ENDPOINT ");
                    if let (Ok(name), Ok(_)) =
                        (std::env::var(&primal_env), std::env::var(&endpoint_env))
                    {
                        if name.contains("storage") || name.contains("data") || name.contains("db")
                        {
                            providers.push(name);
                        }
                    }
                }
            }
            "ai" | "ml" | "intelligence" | "model" => {
                // Look for AI-related primals
                if std::env::var("SQUIRREL_ENDPOINT ").is_ok() {
                    providers.push("squirrel".to_string());
                }
                // Check for custom AI services
                for i in 1..=10 {
                    let primal_env = format!("PRIMAL_{i}_NAME ");
                    let endpoint_env = format!("PRIMAL_{i}_ENDPOINT ");
                    if let (Ok(name), Ok(_)) =
                        (std::env::var(&primal_env), std::env::var(&endpoint_env))
                    {
                        if name.contains("ai")
                            || name.contains("ml")
                            || name.contains("neural")
                            || name.contains("model")
                        {
                            providers.push(name);
                        }
                    }
                }
            }
            _ => {
                // Generic capability - check for any matching service names
                for i in 1..=10 {
                    let primal_env = format!("PRIMAL_{i}_NAME ");
                    let endpoint_env = format!("PRIMAL_{i}_ENDPOINT ");
                    if let (Ok(name), Ok(_)) =
                        (std::env::var(&primal_env), std::env::var(&endpoint_env))
                    {
                        if name.contains(capability_type) {
                            providers.push(name);
                        }
                    }
                }
            }
        }

        providers
    }

    /// Check if a primal provides a specific capability
    fn primal_provides_capability(&self, primal_name: &str, capability_type: &str) -> bool {
        // Basic heuristic mapping of known primals to capabilities
        match (primal_name, capability_type) {
            ("beardog", "security" | "encryption" | "authentication") => true,
            ("toadstool", "compute" | "processing" | "execution") => true,
            ("nestgate", "storage" | "data" | "persistence") => true,
            ("squirrel", "ai" | "ml" | "intelligence") => true,
            _ => {
                // For custom primals, infer from name patterns
                primal_name.contains(capability_type) || capability_type.contains(primal_name)
            }
        }
    }

    /// Get the best primal for a capability based on `QoS` metrics
    pub async fn get_best_primal_for_capability(&self, capability_type: &str) -> Option<String> {
        let providers = self.find_capability_providers(capability_type).await;

        if providers.is_empty() {
            return None;
        }

        // Sophisticated QoS-based selection
        let mut scored_providers: Vec<(String, f64)> = Vec::new();

        for provider in providers {
            let mut score = 0.0;

            // Base score for known high-quality providers (40% weight)
            match provider.as_str() {
                "beardog" => score += 40.0,   // Security specialist
                "toadstool" => score += 35.0, // Compute specialist
                "nestgate" => score += 35.0,  // Storage specialist
                "squirrel" => score += 40.0,  // AI specialist
                _ => score += 20.0,           // Unknown providers get lower base score
            }

            // Capability match score (30% weight)
            if self.primal_provides_capability(&provider, capability_type) {
                score += 30.0;
            } else {
                score += 5.0; // Partial match
            }

            // Name-based heuristic score (20% weight)
            if provider.contains(capability_type) || capability_type.contains(&provider) {
                score += 20.0;
            }

            // Availability heuristic (10% weight)
            // In a real implementation, this would check actual health status
            // For now, assume all providers are reasonably available
            score += 10.0;

            scored_providers.push((provider, score));
        }

        // Sort by score (highest first) and return the best provider
        scored_providers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored_providers.into_iter().next().map(|(provider, _)| provider)
    }

    /// Query primal capabilities via HTTP
    async fn query_primal_capabilities(
        &self,
        endpoint: &str,
    ) -> Result<Vec<Capability>, CapabilityError> {
        let client = reqwest::Client::builder()
            .timeout(self.discovery_config.discovery_timeout)
            .build()
            .map_err(|e| CapabilityError::NetworkError(format!("HTTP client error: {e}")))?;

        // Try standard capability endpoints
        let capability_endpoints = vec![
            format!("{}/capabilities", endpoint),
            format!("{}/api/v1/capabilities", endpoint),
            format!("{}/primal/capabilities", endpoint),
            format!("{}{}", endpoint, HEALTH_PATH), // Fallback - infer capabilities from health
        ];

        for cap_endpoint in capability_endpoints {
            debug!("Trying capability endpoint: {}", cap_endpoint);

            match client.get(&cap_endpoint).send().await {
                Ok(response) if response.status().is_success() => {
                    if let Ok(capability_response) = response.json::<CapabilityResponse>().await {
                        return Ok(capability_response.capabilities);
                    }
                    // Try parsing as simple capability list
                }
                Ok(_) => {}
                Err(_) => {}
            }
        }

        // If no capability endpoint works, return inferred capabilities
        Ok(self.infer_capabilities_from_name(endpoint))
    }

    /// Infer basic capabilities from primal name and endpoint
    fn infer_capabilities_from_name(&self, endpoint: &str) -> Vec<Capability> {
        // Extract primal name from endpoint for inference
        let primal_name = self.extract_primal_name_from_endpoint(endpoint);

        // Basic capability inference based on common patterns
        let capabilities = match primal_name.to_lowercase().as_str() {
            name if name.contains("security") || name.contains("auth") || name.contains("bear") => {
                vec![Capability {
                    capability_type: "security".to_string(),
                    name: "authentication".to_string(),
                    version: "1.0".to_string(),
                    parameters: HashMap::new(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                }]
            }
            name if name.contains("compute") || name.contains("toad") => {
                vec![Capability {
                    capability_type: "compute".to_string(),
                    name: "container_runtime".to_string(),
                    version: "1.0".to_string(),
                    parameters: HashMap::new(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                }]
            }
            name if name.contains("storage") || name.contains("nest") => {
                vec![Capability {
                    capability_type: "storage".to_string(),
                    name: "file_system".to_string(),
                    version: "1.0".to_string(),
                    parameters: HashMap::new(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                }]
            }
            name if name.contains("ai") || name.contains("ml") || name.contains("squirrel") => {
                vec![Capability {
                    capability_type: "ai".to_string(),
                    name: "model_inference".to_string(),
                    version: "1.0".to_string(),
                    parameters: HashMap::new(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                }]
            }
            _ => {
                // Generic capability for unknown primals
                vec![Capability {
                    capability_type: "generic".to_string(),
                    name: "service".to_string(),
                    version: "1.0".to_string(),
                    parameters: HashMap::new(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                }]
            }
        };

        debug!("Inferred {} capabilities for primal: {}", capabilities.len(), primal_name);
        capabilities
    }

    /// Extract primal name from endpoint URL
    fn extract_primal_name_from_endpoint(&self, endpoint: &str) -> String {
        // Try to extract primal name from various URL patterns
        if let Ok(url) = url::Url::parse(endpoint) {
            if let Some(host) = url.host_str() {
                // Extract name from hostname patterns like "beardog.service" or "beardog-service"
                let name_part = host.split('.').next().unwrap_or(host);
                let clean_name = name_part.replace("-service", "").replace("_service", "");
                return clean_name;
            }
        }

        // Fallback: extract from path
        endpoint.split('/').nth(2).unwrap_or("unknown").to_string()
    }

    /// Establish connection to a primal
    ///
    /// # Errors
    ///
    /// Returns an error if the connection test fails or the primal health check fails
    pub async fn connect_to_primal(
        &self,
        name: &str,
        endpoint: &str,
    ) -> Result<(), CapabilityError> {
        let connection = PrimalConnection {
            name: name.to_string(),
            primal_type: self.infer_primal_type_from_name(name),
            endpoint: endpoint.to_string(),
            health: ConnectionHealth::Unknown,
            last_contact: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        // Test the connection
        let health = self.test_primal_health(endpoint).await;

        let mut connections = self.primal_connections.write().await;
        connections.insert(
            name.to_string(),
            PrimalConnection {
                health,
                last_contact: chrono::Utc::now(),
                ..connection
            },
        );

        info!("Established connection to primal: {} at {}", name, endpoint);
        Ok(())
    }

    /// Test primal health
    async fn test_primal_health(&self, endpoint: &str) -> ConnectionHealth {
        // Basic health check - try to connect
        match reqwest::Client::new()
            .get(format!("{endpoint}{HEALTH_PATH}"))
            .timeout(self.discovery_config.discovery_timeout)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => ConnectionHealth::Healthy,
            Ok(_) => ConnectionHealth::Degraded,
            Err(_) => ConnectionHealth::Unhealthy,
        }
    }

    /// Infer primal type from name
    fn infer_primal_type_from_name(&self, name: &str) -> PrimalType {
        let name_lower = name.to_lowercase();
        if name_lower.contains("security")
            || name_lower.contains("auth")
            || name_lower.contains("bear")
        {
            PrimalType::Security
        } else if name_lower.contains("compute") || name_lower.contains("toad") {
            PrimalType::Compute
        } else if name_lower.contains("storage") || name_lower.contains("nest") {
            PrimalType::Storage
        } else if name_lower.contains("ai")
            || name_lower.contains("ml")
            || name_lower.contains("squirrel")
        {
            PrimalType::AI
        } else {
            PrimalType::Generic
        }
    }

    /// Get all active connections
    pub async fn get_active_connections(&self) -> HashMap<String, PrimalConnection> {
        self.primal_connections.read().await.clone()
    }

    /// Disconnect from a primal
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is not currently connected
    pub async fn disconnect_from_primal(&self, name: &str) -> Result<(), CapabilityError> {
        let mut connections = self.primal_connections.write().await;
        if connections.remove(name).is_some() {
            info!("Disconnected from primal: {}", name);
            Ok(())
        } else {
            Err(CapabilityError::PrimalNotFound(name.to_string()))
        }
    }

    /// Update connection health for all primals
    pub async fn update_connection_health(&self) {
        let connections = self.primal_connections.read().await.clone();
        for (name, connection) in connections {
            let health = self.test_primal_health(&connection.endpoint).await;

            let mut connections_write = self.primal_connections.write().await;
            if let Some(conn) = connections_write.get_mut(&name) {
                conn.health = health;
                conn.last_contact = chrono::Utc::now();
            }
        }
    }
}
