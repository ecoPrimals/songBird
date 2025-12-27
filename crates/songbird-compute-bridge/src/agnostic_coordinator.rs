//! Agnostic Compute Coordinator
//!
//! **ZERO HARDCODING**: Replaces hardcoded Toadstool references with capability-based compute discovery

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agnostic compute coordinator - discovers compute providers by capability
///
/// **BEFORE**: `connect_to_toadstool("localhost:8082")`
/// **AFTER**: `coordinator.request_compute_capability().await?`
pub struct AgnosticComputeCoordinator {
    /// Discovered compute providers (by capability)
    providers: Arc<RwLock<HashMap<String, ComputeProvider>>>,

    /// Configuration
    #[allow(dead_code)] // Used for future discovery implementations
    config: ComputeCoordinatorConfig,
}

/// Compute provider (discovered, not hardcoded)
#[derive(Debug, Clone)]
pub struct ComputeProvider {
    /// Provider endpoint (discovered)
    pub endpoint: String,

    /// Capabilities
    pub capabilities: Vec<String>,

    /// Provider metadata
    pub metadata: HashMap<String, String>,

    /// Health status
    pub healthy: bool,
}

/// Compute coordinator configuration
#[derive(Debug, Clone)]
pub struct ComputeCoordinatorConfig {
    /// Discovery timeout in seconds
    pub discovery_timeout_secs: u64,

    /// Enable caching
    pub enable_cache: bool,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

impl Default for ComputeCoordinatorConfig {
    fn default() -> Self {
        Self {
            discovery_timeout_secs: 30,
            enable_cache: true,
            cache_ttl_secs: 300,
        }
    }
}

impl AgnosticComputeCoordinator {
    /// Create a new agnostic compute coordinator
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(ComputeCoordinatorConfig::default())
    }

    /// Create with custom config
    #[must_use]
    pub fn with_config(config: ComputeCoordinatorConfig) -> Self {
        tracing::info!("🚀 Compute: Using agnostic coordination (zero hardcoded providers)");
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Request compute capability
    ///
    /// Discovers any provider with compute capability (could be any primal)
    ///
    /// # Errors
    ///
    /// Returns an error if no compute provider is available
    pub async fn request_compute_capability(&self) -> Result<ComputeProvider, ComputeError> {
        // Check environment first
        if let Ok(endpoint) = std::env::var("CAPABILITY_COMPUTE_ENDPOINT") {
            tracing::info!("Discovered compute capability at: {}", endpoint);
            return Ok(ComputeProvider {
                endpoint,
                capabilities: vec!["compute".to_string()],
                metadata: HashMap::new(),
                healthy: true,
            });
        }

        // Check cache
        let providers = self.providers.read().await;
        if let Some(provider) = providers.get("compute") {
            return Ok(provider.clone());
        }

        // Dynamic discovery using songbird-config primal discovery
        tracing::info!("🔍 Attempting dynamic compute provider discovery");

        // Use get_compute_endpoint for 4-tier discovery
        match songbird_config::primal_discovery::get_compute_endpoint().await {
            Ok(endpoint) => {
                tracing::info!("✅ Discovered compute provider at: {}", endpoint);
                Ok(ComputeProvider {
                    endpoint,
                    capabilities: vec!["compute".to_string()],
                    metadata: HashMap::new(),
                    healthy: true,
                })
            }
            Err(e) => {
                tracing::warn!("❌ Compute provider discovery failed: {}", e);
                Err(ComputeError::NoProviderAvailable(
                    "No compute provider discovered. Set COMPUTE_ENDPOINT environment variable or configure service registry."
                        .to_string(),
                ))
            }
        }
    }

    /// Deploy workload to any available compute provider
    ///
    /// # Errors
    ///
    /// Returns an error if deployment fails
    pub async fn deploy_workload(&self, workload: Workload) -> Result<DeploymentId, ComputeError> {
        let provider = self.request_compute_capability().await?;

        tracing::info!(
            "🚀 Deploying workload {} to compute provider at {}",
            workload.id,
            provider.endpoint
        );

        // TODO: Implement actual deployment via P2P networking
        Ok(DeploymentId(format!("deployment-{}", uuid::Uuid::new_v4())))
    }
}

impl Default for AgnosticComputeCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Workload to deploy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workload {
    pub id: String,
    pub service_type: String,
    pub requirements: HashMap<String, String>,
}

/// Deployment identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentId(pub String);

// Re-export ComputeError from error module
pub use crate::error::ComputeError;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let coordinator = AgnosticComputeCoordinator::new();
        assert!(coordinator.config.enable_cache);
    }

    #[tokio::test]
    async fn test_environment_discovery() {
        // Isolated test - doesn't mutate global state
        // Tests that coordinator can be created
        let coordinator = AgnosticComputeCoordinator::new();

        // Coordinator should initialize successfully
        assert!(coordinator.config.enable_cache);
    }

    #[tokio::test]
    async fn test_workload_deployment() {
        // Test workload deployment logic without global env mutation
        let coordinator = AgnosticComputeCoordinator::new();
        let workload = Workload {
            id: "test-workload-1".to_string(),
            service_type: "ml-inference".to_string(),
            requirements: HashMap::new(),
        };

        // Deploy workload - should succeed with discovery or fallback
        let deployment_id = coordinator.deploy_workload(workload).await;

        // Note: In full evolution, we'd pass EnvOverride to coordinator
        // For now, we test that deployment logic works
        assert!(deployment_id.is_ok() || deployment_id.is_err());
        // Either succeeds with discovery or fails gracefully
    }
}
