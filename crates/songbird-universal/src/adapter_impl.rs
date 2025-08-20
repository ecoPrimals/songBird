// Universal Capability Adapter Implementation
//
// This module contains the main implementation of the UniversalCapabilityAdapter,
// providing sophisticated primal discovery and capability management.

use songbird_errors::EvolvedResult;
use crate::capabilities::{Capability, CapabilityError, CapabilityRegistry, QoSMetrics, UniversalCapabilityAdapter};
// Removed unused import: SongbirdResponse
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use url::Url;

impl UniversalCapabilityAdapter {
    /// Creates a new adapter with default configuration
    pub fn new() -> Self {
        Self {
            config: songbird_config::SongbirdConfig::default().discovery,
            http_client: reqwest::Client::new(),
            capability_registry: Arc::new(RwLock::new(CapabilityRegistry::default())),
        }
    }

    /// Discover capabilities for a primal by name
    pub async fn discover_primal_capabilities(&self) -> SongbirdResult<()> {info!("🔍 Discovering capabilities for primal: {}", primal_name);

        // Get primal endpoint
        let endpoint = songbird_config::config::constants::get_primal_endpoint(primal_name);

        // Attempt capability discovery via HTTP
        match self.query_primal_capabilities(&endpoint).await {
            Ok(songbird_errors::evolved_success(capabilities)) => {
                // Update registry
                let mut registry = self.capability_registry.write().await;
                registry
                    .primal_capabilities
                    .insert(primal_name.to_string(), capabilities.clone());
                registry
                    .last_updated
                    .insert(primal_name.to_string(), chrono::Utc::now());

                // Update capability providers index
                for capability in &capabilities {
                    registry
                        .capability_providers
                        .entry(capability.capability_type.clone())
                        .or_insert_with(Vec::new)
                        .push(primal_name.to_string());
                }

                info!(
                    "✅ Discovered {} capabilities for {}",
                    capabilities.len(),
                    primal_name
                );
                Ok(songbird_errors::evolved_success(capabilities))
            }
            Err(e) => {
                warn!(
                    "❌ Failed to discover capabilities for {}: {}",
                    primal_name, e
                );
                Err(e)
            }
        }
    }

    /// Find all primals that provide a specific capability
    pub async fn find_capability_providers(&self) -> Vec<String> {
        debug!("🔍 Finding providers for capability: {}", capability_type);

        let mut providers = Vec::new();

        // First check the capability registry (for cached/registered capabilities)
        {
            let registry = self.capability_registry.read().await;
            if let Some(registered_providers) = registry.capability_providers.get(capability_type) {
                providers.extend(registered_providers.clone());
                debug!(
                    "Found {} providers in registry for '{}'",
                    registered_providers.len(),
                    capability_type
                );
            }
        }

        // Check environment variables for capability-based discovery
        let capability_providers = self
            .discover_capability_providers_from_env(capability_type)
            .await;
        providers.extend(capability_providers);

        // Capability inference from known patterns
        let inferred_providers = self.infer_capability_providers(capability_type).await;
        providers.extend(inferred_providers);

        // Remove duplicates and return
        providers.sort();
        providers.dedup();

        debug!(
            "✅ Found {} providers for capability '{}': {:?}",
            providers.len(),
            capability_type,
            providers
        );

        providers
    }

    /// Discover capability providers from environment variables
    pub async fn discover_capability_providers_from_env(&self) -> SongbirdResult<()> {let mut providers = Vec::new();

        // Check for generic capability environment variables
        let capability_env = format!("{}_PROVIDERS", capability_type.to_uppercase());
        if let Ok(songbird_errors::evolved_success(provider_list)) = std::env::var(&capability_env) {
            providers.extend(provider_list.split(',').map(|s| s.trim().to_string()));
        }

        // Dynamically discover primal environment variables using the PRIMAL_*_ENDPOINT pattern
        for (key, _value) in std::env::vars() {
            if key.starts_with("PRIMAL_") && key.ends_with("_ENDPOINT") {
                let primal_id = key
                    .strip_prefix("PRIMAL_")
                    .and_then(|s| s.strip_suffix("_ENDPOINT"))
                    .unwrap_or("unknown")
                    .to_lowercase();

                // Check if this primal provides the requested capability
                if self.service_provides_capability(&primal_id, capability_type) {
                    providers.push(primal_id);
                }
            }
        }

        providers
    }

    /// Infer capability providers based on known patterns
    pub async fn infer_capability_providers(&self) -> SongbirdResult<()> {let mut providers = Vec::new();

        // ✅ NEW: Pure capability-based discovery (no hardcoded primal names)
        // Check for manually registered services with matching capabilities
        for i in 1..=20 {
            // Increased range for more service discovery
            let service_name_env = format!("SERVICE_{i}_NAME");
            let service_endpoint_env = format!("SERVICE_{i}_ENDPOINT");
            let service_capabilities_env = format!("SERVICE_{i}_CAPABILITIES");

            if let (Ok(songbird_errors::evolved_success(name)), Ok(songbird_errors::evolved_success(_endpoint)), Ok(capabilities)) = (
                std::env::var(&service_name_env),
                std::env::var(&service_endpoint_env),
                std::env::var(&service_capabilities_env),
            ) {
                // Parse capabilities (comma-separated)
                let service_capabilities: Vec<&str> =
                    capabilities.split(',').map(|s| s.trim()).collect();

                // Check if this service provides the requested capability
                if service_capabilities
                    .iter()
                    .any(|cap| capability_type.contains(cap) || cap.contains(capability_type))
                {
                    providers.push(name);
                }
            }
        }

        providers
    }

    /// Check if a service provides a specific capability (capability-based, not name-based)
    fn service_provides_capability(&self, service_name: &str, capability_type: &str) -> bool {
        // ✅ NEW: Capability-based checking using environment variables
        for i in 1..=20 {
            let service_name_env = format!("SERVICE_{i}_NAME");
            let service_capabilities_env = format!("SERVICE_{i}_CAPABILITIES");

            if let (Ok(songbird_errors::evolved_success(name)), Ok(capabilities)) = (
                std::env::var(&service_name_env),
                std::env::var(&service_capabilities_env),
            ) {
                if name == service_name {
                    let service_capabilities: Vec<&str> =
                        capabilities.split(',').map(|s| s.trim()).collect();
                    return service_capabilities
                        .iter()
                        .any(|cap| capability_type.contains(cap) || cap.contains(capability_type));
                }
            }
        }

        false // Default: no capability match found
    }

    /// Get the best primal for a given capability type using canonical response patterns
    /// Returns the primal ID that best matches the capability requirements
    pub async fn get_best_primal_for_capability(&self) -> Option<String> {
        let registry = self.capability_registry.read().await;

        let providers = registry.capability_providers.get(capability_type)?;
        if providers.is_empty() {
            return None;
        }

        // Simple selection algorithm: choose the first available provider
        // Future enhancement: implement sophisticated scoring based on QoS metrics
        let selected_primal = providers[0].clone();
        Some(selected_primal)
    }

    /// Calculate QoS score for a provider based on multiple metrics
    #[allow(dead_code)]
    async fn calculate_qos_score(&self) -> f64 {
        let registry = self.capability_registry.read().await;

        // Get QoS metrics for this provider-capability combination
        let qos_metrics = registry
            .primal_capabilities
            .get(provider)
            .and_then(|capabilities| {
                capabilities
                    .iter()
                    .find(|cap| cap.capability_type == capability_type)
                    .map(|cap| &cap.qos_metrics)
            });

        if let Some(metrics) = qos_metrics {
            // Weighted scoring algorithm based on multiple QoS factors
            let mut score = 0.0;

            // Latency factor (lower is better, weight: 0.4)
            let latency_ms = metrics.latency_ms;
            if latency_ms > 0.0 {
                // Normalize latency: score decreases as latency increases
                let latency_score = 1000.0 / (latency_ms + 1.0); // +1 to avoid division by zero
                score += 0.4 * latency_score;
            }

            // Throughput factor (higher is better, weight: 0.3)
            let throughput = metrics.throughput_ops_sec;
            if throughput > 0.0 {
                score += 0.3 * throughput;
            }

            // Reliability factor (weight: 0.2)
            let reliability = metrics.reliability;
            if reliability > 0.0 {
                score += 0.2 * reliability * 100.0; // Convert percentage to points
            }

            // Availability factor (weight: 0.1)
            let availability = metrics.availability;
            if availability > 0.0 {
                score += 0.1 * availability * 100.0; // Convert percentage to points
            }

            score
        } else {
            // Default score for providers without QoS metrics
            // Still viable but lower priority
            10.0
        }
    }

    /// Query primal capabilities via HTTP
    pub async fn query_primal_capabilities(&self) -> SongbirdResult<()> {// Use the configured HTTP client instead of creating a new one

        // Try standard capability endpoints
        let capability_endpoints = vec![
            format!("{}/capabilities", endpoint),
            format!("{}/api/v1/capabilities", endpoint),
            format!("{}/primal/capabilities", endpoint),
            format!("{}/health", endpoint), // Fallback - infer capabilities from health
        ];

        for cap_endpoint in capability_endpoints {
            debug!("Trying capability endpoint: {}", cap_endpoint);

            match self.http_client.get(&cap_endpoint).send().await {
                Ok(songbird_errors::evolved_success(response)) if response.status().is_success() => {
                    match response.json::<CapabilityResponse>().await {
                        Ok(songbird_errors::evolved_success(capability_response)) => {
                            return Ok(songbird_errors::evolved_success(capability_response.capabilities));
                        }
                        Err(_) => {
                            // Try parsing as simple capability list
                            continue;
                        }
                    }
                }
                Ok(songbird_errors::evolved_success(_)) => continue,
                Err(_) => continue,
            }
        }

        // If no capability endpoint works, return inferred capabilities
        Ok(songbird_errors::evolved_success(self.infer_capabilities_from_name(endpoint)))
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
                    provider_name: Some(primal_name.to_string()),
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
                    provider_name: Some(primal_name.to_string()),
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
                    provider_name: Some(primal_name.to_string()),
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
                    provider_name: Some(primal_name.to_string()),
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
                    provider_name: Some(primal_name.to_string()),
                }]
            }
        };

        debug!(
            "Inferred {} capabilities for primal: {}",
            capabilities.len(),
            primal_name
        );
        capabilities
    }

    /// Extract primal name from endpoint URL
    fn extract_primal_name_from_endpoint(&self, endpoint: &str) -> String {
        // Try to extract primal name from various URL patterns
        if let Ok(songbird_errors::evolved_success(url)) = Url::parse(endpoint) {
            if let Some(host) = url.host_str() {
                // Extract name from hostname patterns like "security.service" or "security-service"
                let name_part = host.split('.').next().unwrap_or(host);
                let clean_name = name_part.replace("-service", "").replace("_service", "");
                return clean_name;
            }
        }

        // Fallback: extract from path
        endpoint.split('/').nth(2).unwrap_or("unknown").to_string()
    }
}

/// Response format for capability queries
#[derive(Debug, serde::Deserialize)]
struct CapabilityResponse {
    capabilities: Vec<Capability>,
}
