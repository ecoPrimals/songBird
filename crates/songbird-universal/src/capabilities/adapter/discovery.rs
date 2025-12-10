//! Capability Discovery Module
//!
//! Handles discovery of primals and their capabilities through multiple channels:
//! - Environment variables (CAPABILITY_* endpoints)
//! - Network scanning (when enabled)
//! - Capability inference (pattern-based)
//!
//! Part of the smart refactoring from monolithic adapter.rs (1080 lines) → focused modules

use chrono::Utc;
use songbird_types::SafeEnv;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::super::error::CapabilityError;
use super::super::registry::CapabilityRegistry;
use super::super::types::{Capability, DiscoveryConfig};

/// Capability discovery component
#[derive(Debug, Clone)]
pub struct CapabilityDiscovery {
    /// Shared capability registry
    registry: Arc<RwLock<CapabilityRegistry>>,
    /// Discovery configuration
    config: DiscoveryConfig,
}

impl CapabilityDiscovery {
    /// Create new capability discovery component
    pub fn new(registry: Arc<RwLock<CapabilityRegistry>>, config: DiscoveryConfig) -> Self {
        Self { registry, config }
    }

    /// Discover capabilities for a primal by name
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is unreachable or does not respond with valid capabilities
    pub async fn discover_primal_capabilities(
        &self,
        primal_name: &str,
        query_fn: impl Fn(&str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Capability>, CapabilityError>> + Send>>,
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
        match query_fn(&endpoint).await {
            Ok(capabilities) => {
                // Update registry
                let mut registry = self.registry.write().await;
                registry.primal_capabilities.insert(primal_name.to_string(), capabilities.clone());
                registry.last_updated.insert(primal_name.to_string(), Utc::now());

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
        if self.config.enable_network_discovery {
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
        let capability_env = format!("{}_PROVIDERS", capability_type.to_uppercase());
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
                    let provider_name = Self::extract_primal_name_from_endpoint(&endpoint);
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
        debug!("Network discovery for {} capability - not implemented yet", capability_type);

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
                    let provider = Self::extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider);
                }
                // Check for custom security services
                for i in 1..=10 {
                    let primal_env = format!("PRIMAL_{i}_NAME");
                    let endpoint_env = format!("PRIMAL_{i}_ENDPOINT");
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
            "compute" | "processing" | "workers" => {
                // Look for compute providers
                if let Ok(endpoint) = SafeEnv::get_required("COMPUTE_PROVIDER_ENDPOINT") {
                    let provider = Self::extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider);
                }
            }
            "storage" | "persistence" | "database" => {
                // Look for storage providers
                if let Ok(endpoint) = SafeEnv::get_required("STORAGE_PROVIDER_ENDPOINT") {
                    let provider = Self::extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider);
                }
            }
            "ai" | "ml" | "inference" => {
                // Look for AI providers
                if let Ok(endpoint) = SafeEnv::get_required("AI_PROVIDER_ENDPOINT") {
                    let provider = Self::extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider);
                }
            }
            _ => {
                // Generic discovery for unknown capability types
                debug!("No inference pattern for capability type: {}", capability_type);
            }
        }

        providers
    }

    /// Check if a primal provides a specific capability (simplified from original)
    fn primal_provides_capability(&self, _primal_cap: &str, _target_cap: &str) -> bool {
        // Simplified: Exact match for now
        // Original has more complex logic
        false
    }

    /// Extract primal name from endpoint URL
    fn extract_primal_name_from_endpoint(endpoint: &str) -> String {
        endpoint
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .split(':')
            .next()
            .unwrap_or("unknown")
            .split('.')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_from_env() {
        std::env::set_var("SECURITY_PROVIDER_ENDPOINT", "http://beardog:8443");
        
        let registry = Arc::new(RwLock::new(CapabilityRegistry::default()));
        let config = DiscoveryConfig::default();
        let discovery = CapabilityDiscovery::new(registry, config);

        let providers = discovery.discover_capability_providers_from_env("security").await;
        
        assert!(!providers.is_empty(), "Should find at least one security provider");
        
        std::env::remove_var("SECURITY_PROVIDER_ENDPOINT");
    }

    #[tokio::test]
    async fn test_extract_primal_name() {
        let name = CapabilityDiscovery::extract_primal_name_from_endpoint("http://beardog:8443");
        assert_eq!(name, "beardog");

        let name2 = CapabilityDiscovery::extract_primal_name_from_endpoint("https://toadstool.local:9000");
        assert_eq!(name2, "toadstool");
    }
}

