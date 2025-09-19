//! Universal Capability Adapters for Metrics Collection
//!
//! This module provides capability-based adapters that work with any primal
//! without hardcoding specific primal names. Adapters discover and use primals
//! based on their declared capabilities.

use songbird_errors::SongbirdResult;
use songbird_universal::capabilities::UniversalCapabilityAdapter;
use std::collections::HashMap;
use tracing::info;

/// Universal metrics capability adapter
#[derive(Debug, Clone)]
pub struct UniversalMetricsAdapter {
    /// Capability adapter for primal discovery
    capability_adapter: UniversalCapabilityAdapter,

    /// Discovered compute primals (replaces hardcoded toadstool)
    pub compute_endpoints: Vec<String>,

    /// Discovered security primals (replaces hardcoded beardog)  
    pub security_endpoints: Vec<String>,

    /// Discovered storage primals (replaces hardcoded nestgate)
    pub storage_endpoints: Vec<String>,

    /// Discovered AI primals (replaces hardcoded squirrel)
    pub ai_endpoints: Vec<String>,

    /// Custom capability endpoints
    pub custom_endpoints: HashMap<String, Vec<String>>,

    /// Last discovery update
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for UniversalMetricsAdapter {
    fn default() -> Self {
        let discovery_config = songbird_universal::capabilities::DiscoveryConfig::default();
        Self {
            capability_adapter: UniversalCapabilityAdapter::new(discovery_config),
            compute_endpoints: Vec::new(),
            security_endpoints: Vec::new(),
            storage_endpoints: Vec::new(),
            ai_endpoints: Vec::new(),
            custom_endpoints: HashMap::new(),
            last_updated: None,
        }
    }
}

impl UniversalMetricsAdapter {
    /// Create a new universal metrics adapter
    pub fn new() -> Self {
        Self::default()
    }

    /// Discover and update all primal endpoints based on capabilities
    pub async fn discover_and_update_endpoints(&mut self) -> SongbirdResult<()> {
        info!("🔍 Discovering primals for metrics collection...");

        // Discover compute primals (anything with "compute" capability)
        self.compute_endpoints = self.discover_primals_with_capability("compute").await?;
        info!("✅ Found {} compute primals", self.compute_endpoints.len());

        // Discover security primals (anything with "security" capability)
        self.security_endpoints = self.discover_primals_with_capability("security").await?;
        info!(
            "✅ Found {} security primals",
            self.security_endpoints.len()
        );

        // Discover storage primals (anything with "storage" capability)
        self.storage_endpoints = self.discover_primals_with_capability("storage").await?;
        info!("✅ Found {} storage primals", self.storage_endpoints.len());

        // Discover AI primals (anything with "ai" capability)
        self.ai_endpoints = self.discover_primals_with_capability("ai").await?;
        info!("✅ Found {} AI primals", self.ai_endpoints.len());

        self.last_updated = Some(chrono::Utc::now());
        Ok(())
    }

    /// Discover primals with a specific capability
    async fn discover_primals_with_capability(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<String>> {
        let providers = self
            .capability_adapter
            .find_capability_providers(capability)
            .await;

        let mut endpoints = Vec::new();
        for primal_name in providers {
            let endpoint = songbird_config::config::constants::get_primal_endpoint(&primal_name);
            endpoints.push(endpoint);
        }

        // If no primals discovered via capability, try environment fallback
        if endpoints.is_empty() {
            endpoints = self.discover_capability_fallback(capability).await;
        }

        Ok(endpoints)
    }

    /// Fallback discovery for capabilities when no primals found
    async fn discover_capability_fallback(&self, capability: &str) -> Vec<String> {
        let mut endpoints = Vec::new();

        // Try well-known environment variables for each capability type
        match capability {
            "compute" => {
                if let Ok(endpoint) = std::env::var("COMPUTE_ENDPOINT") {
                    endpoints.push(endpoint);
                }
                // Check for toadstool as legacy fallback
                if let Ok(endpoint) = std::env::var("TOADSTOOL_ENDPOINT") {
                    endpoints.push(endpoint);
                }
            }
            "security" => {
                if let Ok(endpoint) = std::env::var("SECURITY_ENDPOINT") {
                    endpoints.push(endpoint);
                }
                // Check for beardog as legacy fallback
                if let Ok(endpoint) = std::env::var("BEARDOG_ENDPOINT") {
                    endpoints.push(endpoint);
                }
            }
            "storage" => {
                if let Ok(endpoint) = std::env::var("STORAGE_ENDPOINT") {
                    endpoints.push(endpoint);
                }
                // Check for nestgate as legacy fallback
                if let Ok(endpoint) = std::env::var("NESTGATE_ENDPOINT") {
                    endpoints.push(endpoint);
                }
            }
            "ai" => {
                if let Ok(endpoint) = std::env::var("AI_ENDPOINT") {
                    endpoints.push(endpoint);
                }
                // Check for squirrel as legacy fallback
                if let Ok(endpoint) = std::env::var("SQUIRREL_ENDPOINT") {
                    endpoints.push(endpoint);
                }
            }
            _ => {
                // Custom capability - try generic pattern
                let env_var = format!("{}_ENDPOINT", capability.to_uppercase());
                if let Ok(endpoint) = std::env::var(&env_var) {
                    endpoints.push(endpoint);
                }
            }
        }

        endpoints
    }

    /// Get endpoints for a specific capability
    pub fn get_endpoints_for_capability(&self, capability: &str) -> &[String] {
        match capability {
            "compute" => &self.compute_endpoints,
            "security" => &self.security_endpoints,
            "storage" => &self.storage_endpoints,
            "ai" => &self.ai_endpoints,
            _ => self
                .custom_endpoints
                .get(capability)
                .map(|v| v.as_slice())
                .unwrap_or(&[]),
        }
    }

    /// Check if any endpoints are available for a capability
    pub fn has_capability(&self, capability: &str) -> bool {
        !self.get_endpoints_for_capability(capability).is_empty()
    }

    /// Get the first available endpoint for a capability
    pub fn get_primary_endpoint_for_capability(&self, capability: &str) -> Option<&String> {
        self.get_endpoints_for_capability(capability).first()
    }
}

/// Errors that can occur in metrics operations
#[derive(Debug)]
pub enum MetricsError {
    /// Discovery failed
    DiscoveryFailed(String),
    /// No endpoints found for capability
    NoEndpointsFound(String),
    /// Network error
    NetworkError(String),
}

impl std::fmt::Display for MetricsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricsError::DiscoveryFailed(msg) => write!(f, "Discovery failed: {}", msg),
            MetricsError::NoEndpointsFound(cap) => {
                write!(f, "No endpoints found for capability: {}", cap)
            }
            MetricsError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for MetricsError {}
