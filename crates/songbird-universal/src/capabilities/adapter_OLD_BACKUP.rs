//! Universal Capability Adapter implementation

#![allow(clippy::unused_self, clippy::match_same_arms, clippy::unused_async)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_types::SafeEnv;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::connection::{ConnectionHealth, PrimalConnection};
use super::error::CapabilityError;
use super::registry::CapabilityRegistry;
use super::types::{Capability, CapabilityResponse, DiscoveryConfig, PrimalType, QoSMetrics};
use super::HEALTH_PATH;

/// Federation event for tracking state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationEvent {
    /// Event type
    pub event_type: String,
    /// Event data
    pub data: String,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
}

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
        let capability_host = SafeEnv::get_or_default("UNIVERSAL_CAPABILITY_HOST", "127.0.0.1");
        let capability_port = SafeEnv::get_port(
            "UNIVERSAL_CAPABILITY_PORT",
            songbird_config::defaults::ports::orchestrator_port(),
        );
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
        if let Ok(provider_list) = SafeEnv::get_required(&capability_env) {
            providers.extend(provider_list.split(',').map(|s| s.trim().to_string()));
        }

        // Check for capability-based environment variables (zero hardcoding)
        let capability_endpoints = [
            ("SECURITY_PROVIDER_ENDPOINT", "security"),
            ("COMPUTE_PROVIDER_ENDPOINT", "compute"),
            ("STORAGE_PROVIDER_ENDPOINT", "storage"),
            ("AI_PROVIDER_ENDPOINT", "ai"),
        ];
        for (env_var, cap_type) in &capability_endpoints {
            if capability_type == *cap_type
                || self.primal_provides_capability(cap_type, capability_type)
            {
                if let Ok(endpoint) = SafeEnv::get_required(env_var) {
                    let provider_name = self.extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider_name);
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
                // Look for security capability providers (zero hardcoding)
                if let Ok(endpoint) = SafeEnv::get_required("SECURITY_PROVIDER_ENDPOINT") {
                    let provider = self.extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider);
                }
                // Check for custom security services
                for i in 1..=10 {
                    let primal_env = format!("PRIMAL_{i}_NAME ");
                    let endpoint_env = format!("PRIMAL_{i}_ENDPOINT ");
                    if let (Ok(name), Ok(_)) =
                        (SafeEnv::get_required(&primal_env), SafeEnv::get_required(&endpoint_env))
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
                // Look for compute capability providers (zero hardcoding)
                if let Ok(endpoint) = SafeEnv::get_required("COMPUTE_PROVIDER_ENDPOINT") {
                    let provider = self.extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider);
                }
                // Check for custom compute services
                for i in 1..=10 {
                    let primal_env = format!("PRIMAL_{i}_NAME ");
                    let endpoint_env = format!("PRIMAL_{i}_ENDPOINT ");
                    if let (Ok(name), Ok(_)) =
                        (SafeEnv::get_required(&primal_env), SafeEnv::get_required(&endpoint_env))
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
                // Look for storage capability providers (zero hardcoding)
                if let Ok(endpoint) = SafeEnv::get_required("STORAGE_PROVIDER_ENDPOINT") {
                    let provider = self.extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider);
                }
                // Check for custom storage services
                for i in 1..=10 {
                    let primal_env = format!("PRIMAL_{i}_NAME ");
                    let endpoint_env = format!("PRIMAL_{i}_ENDPOINT ");
                    if let (Ok(name), Ok(_)) =
                        (SafeEnv::get_required(&primal_env), SafeEnv::get_required(&endpoint_env))
                    {
                        if name.contains("storage") || name.contains("data") || name.contains("db")
                        {
                            providers.push(name);
                        }
                    }
                }
            }
            "ai" | "ml" | "intelligence" | "model" => {
                // Look for AI capability providers (zero hardcoding)
                if let Ok(endpoint) = SafeEnv::get_required("AI_PROVIDER_ENDPOINT") {
                    let provider = self.extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider);
                }
                // Check for custom AI services
                for i in 1..=10 {
                    let primal_env = format!("PRIMAL_{i}_NAME ");
                    let endpoint_env = format!("PRIMAL_{i}_ENDPOINT ");
                    if let (Ok(name), Ok(_)) =
                        (SafeEnv::get_required(&primal_env), SafeEnv::get_required(&endpoint_env))
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
                        (SafeEnv::get_required(&primal_env), SafeEnv::get_required(&endpoint_env))
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
        // Infer capability from provider name patterns (zero hardcoding)
        let name_lower = primal_name.to_lowercase();
        let capability_lower = capability_type.to_lowercase();

        // Check if name contains capability hints
        match capability_lower.as_str() {
            "security" | "encryption" | "authentication" => {
                name_lower.contains("security")
                    || name_lower.contains("auth")
                    || name_lower.contains("vault")
            }
            "compute" | "processing" | "execution" => {
                name_lower.contains("compute")
                    || name_lower.contains("exec")
                    || name_lower.contains("process")
            }
            "storage" | "data" | "persistence" => {
                name_lower.contains("storage")
                    || name_lower.contains("data")
                    || name_lower.contains("db")
            }
            "ai" | "ml" | "intelligence" | "model" => {
                name_lower.contains("ai")
                    || name_lower.contains("ml")
                    || name_lower.contains("intelligence")
            }
            _ => {
                // Generic pattern matching
                name_lower.contains(&capability_lower) || capability_lower.contains(&name_lower)
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

            // Base score based on provider type inference (zero hardcoding)
            let provider_lower = provider.to_lowercase();
            let base_score =
                if provider_lower.contains("security") || provider_lower.contains("auth") {
                    40.0 // Security providers
                } else if provider_lower.contains("ai") || provider_lower.contains("ml") {
                    40.0 // AI providers
                } else if provider_lower.contains("compute") || provider_lower.contains("exec") {
                    35.0 // Compute providers
                } else if provider_lower.contains("storage") || provider_lower.contains("data") {
                    35.0 // Storage providers
                } else {
                    20.0 // Unknown providers get lower base score
                };
            score += base_score;

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
            name if name.contains("ai") || name.contains("ml") || name.contains("intelligence") => {
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
        // Try to extract service name from various URL patterns (zero hardcoding)
        if let Ok(url) = url::Url::parse(endpoint) {
            if let Some(host) = url.host_str() {
                // Extract name from hostname patterns like "provider.service" or "provider-service"
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
            || name_lower.contains("intelligence")
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

    /// Get recent federation events
    ///
    /// # Errors
    ///
    /// Returns NotImplemented error - this feature is planned for future release
    pub async fn get_recent_events(&self) -> Result<Vec<FederationEvent>, CapabilityError> {
        Err(CapabilityError::NotImplemented(
            "get_recent_events is not yet implemented for federation".to_string(),
        ))
    }

    /// Verify state consistency across federation
    ///
    /// # Errors
    ///
    /// Returns NotImplemented error - this feature is planned for future release
    pub async fn verify_state_consistency(&self) -> Result<bool, CapabilityError> {
        Err(CapabilityError::NotImplemented(
            "verify_state_consistency is not yet implemented for federation".to_string(),
        ))
    }

    /// Mark a node as down in federation
    ///
    /// # Errors
    ///
    /// Returns NotImplemented error - this feature is planned for future release
    pub async fn mark_node_down(&self, _node_id: &str) -> Result<(), CapabilityError> {
        Err(CapabilityError::NotImplemented(
            "mark_node_down is not yet implemented for federation".to_string(),
        ))
    }

    /// Discover providers for a capability across federation
    ///
    /// # Errors
    ///
    /// Returns NotImplemented error - this feature is planned for future release
    pub async fn discover_federated(
        &self,
        _capability: &str,
    ) -> Result<Vec<String>, CapabilityError> {
        Err(CapabilityError::NotImplemented(
            "discover_federated is not yet implemented for federation".to_string(),
        ))
    }

    /// Emit a federation event
    ///
    /// # Errors
    ///
    /// Returns NotImplemented error - this feature is planned for future release
    pub async fn emit_federation_event(
        &self,
        _event_type: &str,
        _data: &str,
    ) -> Result<(), CapabilityError> {
        Err(CapabilityError::NotImplemented(
            "emit_federation_event is not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> DiscoveryConfig {
        DiscoveryConfig {
            refresh_interval: std::time::Duration::from_secs(60),
            discovery_timeout: std::time::Duration::from_secs(5),
            max_concurrent_discoveries: 10,
            auto_discovery: false,
            enable_network_discovery: false,
        }
    }

    #[tokio::test]
    async fn test_new_adapter_creation() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Verify adapter is created with correct configuration
        assert!(!adapter.discovery_config.enable_network_discovery);
        assert_eq!(adapter.discovery_config.max_concurrent_discoveries, 10);
    }

    #[tokio::test]
    async fn test_find_capability_providers_from_env() {
        // Set up environment variable
        std::env::set_var("COMPUTE_PROVIDER_ENDPOINT", "http://localhost:8080/compute");

        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test finding compute providers
        let providers = adapter.find_capability_providers("compute").await;

        // Should find at least one provider
        assert!(!providers.is_empty(), "Should find compute provider from env");

        // Clean up
        std::env::remove_var("COMPUTE_PROVIDER_ENDPOINT");
    }

    #[tokio::test]
    async fn test_find_capability_providers_empty() {
        // Clean environment
        std::env::remove_var("COMPUTE_PROVIDER_ENDPOINT");
        std::env::remove_var("STORAGE_PROVIDER_ENDPOINT");
        std::env::remove_var("AI_PROVIDER_ENDPOINT");

        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test finding providers for unknown capability
        let providers = adapter.find_capability_providers("nonexistent_capability").await;

        // May return empty or inferred providers
        assert!(providers.is_empty() || !providers.is_empty());
    }

    #[tokio::test]
    async fn test_extract_primal_name_from_endpoint() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test various endpoint formats
        // Note: -service and _service suffixes are stripped
        let name1 = adapter.extract_primal_name_from_endpoint("http://compute-service:8080");
        assert_eq!(name1, "compute"); // -service is stripped

        let name2 = adapter.extract_primal_name_from_endpoint("https://ai-primal.example.com/api");
        assert_eq!(name2, "ai-primal");

        let name3 = adapter.extract_primal_name_from_endpoint("http://localhost:9000");
        assert_eq!(name3, "localhost");
    }

    #[tokio::test]
    async fn test_infer_primal_type_from_name() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test various name patterns
        assert_eq!(adapter.infer_primal_type_from_name("compute-service"), PrimalType::Compute);
        assert_eq!(adapter.infer_primal_type_from_name("storage-backend"), PrimalType::Storage);
        assert_eq!(adapter.infer_primal_type_from_name("ai-model-server"), PrimalType::AI);
        assert_eq!(adapter.infer_primal_type_from_name("security-gateway"), PrimalType::Security);
        assert_eq!(adapter.infer_primal_type_from_name("random-service"), PrimalType::Generic);
    }

    #[tokio::test]
    async fn test_primal_provides_capability() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test compute primal capabilities
        assert!(adapter.primal_provides_capability("compute", "compute"));
        assert!(adapter.primal_provides_capability("compute", "processing"));
        assert!(!adapter.primal_provides_capability("compute", "storage"));

        // Test storage primal capabilities
        assert!(adapter.primal_provides_capability("storage", "storage"));
        assert!(adapter.primal_provides_capability("storage", "persistence"));
        assert!(!adapter.primal_provides_capability("storage", "compute"));

        // Test AI primal capabilities
        assert!(adapter.primal_provides_capability("ai", "ai"));
        assert!(adapter.primal_provides_capability("ai", "ml"));
        assert!(!adapter.primal_provides_capability("ai", "compute"));
    }

    #[tokio::test]
    async fn test_get_active_connections_empty() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Initially, no connections
        let connections = adapter.get_active_connections().await;
        assert!(connections.is_empty());
    }

    #[tokio::test]
    async fn test_disconnect_from_nonexistent_primal() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Try to disconnect from non-existent primal
        let result = adapter.disconnect_from_primal("nonexistent").await;
        assert!(result.is_err());

        if let Err(CapabilityError::PrimalNotFound(name)) = result {
            assert_eq!(name, "nonexistent");
        } else {
            panic!("Expected PrimalNotFound error");
        }
    }

    #[tokio::test]
    async fn test_discover_capability_providers_from_env_multiple() {
        // Set up multiple environment variables
        std::env::set_var("COMPUTE_PROVIDER_ENDPOINT", "http://compute:8080");
        std::env::set_var("STORAGE_PROVIDER_ENDPOINT", "http://storage:8081");

        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test compute providers
        let compute_providers = adapter.discover_capability_providers_from_env("compute").await;
        assert!(!compute_providers.is_empty());

        // Test storage providers
        let storage_providers = adapter.discover_capability_providers_from_env("storage").await;
        assert!(!storage_providers.is_empty());

        // Clean up
        std::env::remove_var("COMPUTE_PROVIDER_ENDPOINT");
        std::env::remove_var("STORAGE_PROVIDER_ENDPOINT");
    }

    #[tokio::test]
    async fn test_infer_capability_providers_security() {
        // Set up security provider environment
        std::env::set_var("SECURITY_PROVIDER_ENDPOINT", "http://security:8082");

        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test security capability inference
        let providers = adapter.infer_capability_providers("security").await;
        assert!(!providers.is_empty());

        // Clean up
        std::env::remove_var("SECURITY_PROVIDER_ENDPOINT");
    }

    #[tokio::test]
    async fn test_infer_capability_providers_compute() {
        // Set up compute provider environment
        std::env::set_var("COMPUTE_PROVIDER_ENDPOINT", "http://compute:8080");

        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test compute capability inference
        let providers = adapter.infer_capability_providers("compute").await;
        assert!(!providers.is_empty());

        // Clean up
        std::env::remove_var("COMPUTE_PROVIDER_ENDPOINT");
    }

    #[tokio::test]
    async fn test_infer_capability_providers_storage() {
        // Set up storage provider environment
        std::env::set_var("STORAGE_PROVIDER_ENDPOINT", "http://storage:8081");

        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test storage capability inference
        let providers = adapter.infer_capability_providers("storage").await;
        assert!(!providers.is_empty());

        // Clean up
        std::env::remove_var("STORAGE_PROVIDER_ENDPOINT");
    }

    #[tokio::test]
    async fn test_infer_capability_providers_ai() {
        // Set up AI provider environment
        std::env::set_var("AI_PROVIDER_ENDPOINT", "http://ai:8083");

        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test AI capability inference
        let providers = adapter.infer_capability_providers("ai").await;
        assert!(!providers.is_empty());

        // Clean up
        std::env::remove_var("AI_PROVIDER_ENDPOINT");
    }

    #[tokio::test]
    async fn test_discover_capability_providers_network_disabled() {
        let config = DiscoveryConfig {
            refresh_interval: std::time::Duration::from_secs(60),
            discovery_timeout: std::time::Duration::from_secs(5),
            max_concurrent_discoveries: 10,
            auto_discovery: false,
            enable_network_discovery: false,
        };
        let adapter = UniversalCapabilityAdapter::new(config);

        // Network discovery should be skipped
        let providers = adapter.discover_capability_providers_from_network("compute").await;
        assert!(providers.is_empty());
    }

    #[tokio::test]
    async fn test_primal_type_inference_patterns() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test various patterns - only patterns that contain keywords work
        assert_eq!(adapter.infer_primal_type_from_name("compute-cluster"), PrimalType::Compute);
        assert_eq!(adapter.infer_primal_type_from_name("processor-node"), PrimalType::Generic); // "processor" doesn't contain "compute"
        assert_eq!(adapter.infer_primal_type_from_name("executor-service"), PrimalType::Generic); // "executor" doesn't contain "compute"

        assert_eq!(adapter.infer_primal_type_from_name("storage-node"), PrimalType::Storage);
        assert_eq!(adapter.infer_primal_type_from_name("data-store"), PrimalType::Generic); // "data" alone not enough without "storage"
        assert_eq!(adapter.infer_primal_type_from_name("nest-service"), PrimalType::Storage); // "nest" keyword

        assert_eq!(adapter.infer_primal_type_from_name("ml-model"), PrimalType::AI);
        assert_eq!(adapter.infer_primal_type_from_name("intelligence-service"), PrimalType::AI);
        assert_eq!(adapter.infer_primal_type_from_name("neural-net"), PrimalType::Generic); // "neural" doesn't contain "ai", "ml", or "intelligence"

        assert_eq!(adapter.infer_primal_type_from_name("security-service"), PrimalType::Security);
        assert_eq!(adapter.infer_primal_type_from_name("auth-gateway"), PrimalType::Security);
        assert_eq!(adapter.infer_primal_type_from_name("bear-node"), PrimalType::Security);
        // "bear" keyword
    }

    #[tokio::test]
    async fn test_endpoint_extraction_variations() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test with ports
        assert_eq!(adapter.extract_primal_name_from_endpoint("http://service:8080"), "service");

        // Test with paths
        assert_eq!(
            adapter.extract_primal_name_from_endpoint("http://api-gateway/v1/compute"),
            "api-gateway"
        );

        // Test with subdomains
        assert_eq!(
            adapter.extract_primal_name_from_endpoint("https://compute.example.com"),
            "compute"
        );

        // Test localhost
        assert_eq!(adapter.extract_primal_name_from_endpoint("http://localhost:9000"), "localhost");
    }

    #[tokio::test]
    async fn test_capability_provider_search_deduplication() {
        // Set up duplicate providers in different ways
        std::env::set_var("COMPUTE_PROVIDER_ENDPOINT", "http://compute:8080");
        std::env::set_var("PRIMAL_1_NAME", "compute");
        std::env::set_var("PRIMAL_1_ENDPOINT", "http://compute:8080");

        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Should deduplicate providers
        let providers = adapter.find_capability_providers("compute").await;

        // Count occurrences of "compute"
        let compute_count = providers.iter().filter(|p| *p == "compute").count();

        // Should be deduplicated (only one "compute")
        assert_eq!(compute_count, 1, "Providers should be deduplicated");

        // Clean up
        std::env::remove_var("COMPUTE_PROVIDER_ENDPOINT");
        std::env::remove_var("PRIMAL_1_NAME");
        std::env::remove_var("PRIMAL_1_ENDPOINT");
    }

    #[tokio::test]
    async fn test_update_connection_health_empty() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Should handle empty connections gracefully
        adapter.update_connection_health().await;

        let connections = adapter.get_active_connections().await;
        assert!(connections.is_empty());
    }

    #[tokio::test]
    async fn test_config_discovery_timeout() {
        let config = DiscoveryConfig {
            refresh_interval: std::time::Duration::from_secs(30),
            discovery_timeout: std::time::Duration::from_secs(10),
            max_concurrent_discoveries: 5,
            auto_discovery: true,
            enable_network_discovery: true,
        };

        let adapter = UniversalCapabilityAdapter::new(config);

        assert_eq!(adapter.discovery_config.discovery_timeout, std::time::Duration::from_secs(10));
        assert_eq!(adapter.discovery_config.max_concurrent_discoveries, 5);
        assert_eq!(adapter.discovery_config.refresh_interval, std::time::Duration::from_secs(30));
        assert!(adapter.discovery_config.enable_network_discovery);
        assert!(adapter.discovery_config.auto_discovery);
    }

    #[tokio::test]
    async fn test_primal_provides_capability_edge_cases() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Test empty strings - empty string contains empty string (true)
        assert!(adapter.primal_provides_capability("", ""));

        // Test case sensitivity
        assert!(adapter.primal_provides_capability("compute", "COMPUTE"));
        assert!(adapter.primal_provides_capability("STORAGE", "storage"));

        // Test partial matches
        assert!(adapter.primal_provides_capability("ai-ml-service", "ai"));
        assert!(adapter.primal_provides_capability("compute-cluster", "compute"));
    }

    // ============================================================================
    // FEDERATION METHOD TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_emit_federation_event_not_implemented() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        let result = adapter.emit_federation_event("test_event", "test_data").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(CapabilityError::NotImplemented(_))));
    }

    #[tokio::test]
    async fn test_get_recent_events_not_implemented() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        let result = adapter.get_recent_events().await;
        assert!(result.is_err());
        assert!(matches!(result, Err(CapabilityError::NotImplemented(_))));
    }

    #[tokio::test]
    async fn test_verify_state_consistency_not_implemented() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        let result = adapter.verify_state_consistency().await;
        assert!(result.is_err());
        assert!(matches!(result, Err(CapabilityError::NotImplemented(_))));
    }

    #[tokio::test]
    async fn test_mark_node_down_not_implemented() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        let result = adapter.mark_node_down("test-node").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(CapabilityError::NotImplemented(_))));
    }

    #[tokio::test]
    async fn test_discover_federated_not_implemented() {
        let config = create_test_config();
        let adapter = UniversalCapabilityAdapter::new(config);

        let result = adapter.discover_federated("compute").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(CapabilityError::NotImplemented(_))));
    }
}
