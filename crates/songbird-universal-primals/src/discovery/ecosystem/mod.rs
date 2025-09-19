//! Ecosystem Primal Discovery
//!
//! Discovers and connects to actual primals in the ecoPrimals ecosystem
//! This module is refactored into focused sub-modules for maintainability.

pub mod api_discovery;
pub mod capability_inference;
pub mod config;
pub mod filesystem;
pub mod network;

// Re-export main types and functions
use super::types::DiscoveredPrimal;
use crate::errors::PrimalResult;
use crate::traits::PrimalContext;
pub use config::EcosystemDiscoveryConfig;
use std::collections::HashMap;
use tracing::info;

/// Ecosystem primal discoverer - main coordination logic
#[derive(Clone)]
pub struct EcosystemDiscovery {
    config: EcosystemDiscoveryConfig,
    http_client: reqwest::Client,
}

impl EcosystemDiscovery {
    /// Create new ecosystem discovery instance
    pub fn new(config: EcosystemDiscoveryConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(
                config.health_check_timeout_ms,
            ))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            config,
            http_client,
        }
    }

    /// Discover all primals in the ecosystem
    pub async fn discover_ecosystem_primals(&self) -> PrimalResult<Vec<DiscoveredPrimal>> {
        info!(
            "🌌 Discovering ecosystem primals at {}",
            self.config.ecosystem_base_path
        );

        let mut discovered_primals = Vec::new();

        // 1. Filesystem-based discovery
        if self.config.enable_filesystem_discovery {
            match filesystem::discover_via_filesystem(&self.config, &self.http_client).await {
                Ok(mut primals) => {
                    info!("🗂️ Filesystem discovery found {} primals", primals.len());
                    discovered_primals.append(&mut primals);
                }
                Err(e) => {
                    tracing::warn!("Filesystem discovery failed: {}", e);
                }
            }
        }

        // 2. Network-based discovery
        if self.config.enable_network_discovery {
            match network::network_capability_discovery().await {
                Ok(mut primals) => {
                    info!("🌐 Network discovery found {} primals", primals.len());
                    discovered_primals.append(&mut primals);
                }
                Err(e) => {
                    tracing::warn!("Network discovery failed: {}", e);
                }
            }
        }

        // Remove duplicates based on endpoint
        discovered_primals.sort_by(|a, b| a.endpoint.cmp(&b.endpoint));
        discovered_primals.dedup_by(|a, b| a.endpoint == b.endpoint);

        info!(
            "✅ Total ecosystem primals discovered: {}",
            discovered_primals.len()
        );
        Ok(discovered_primals)
    }

    /// Get default capabilities for a primal name (for backward compatibility)
    pub fn get_default_capabilities_for_primal(
        &self,
        primal_name: &str,
    ) -> (
        songbird_universal::PrimalType,
        Vec<crate::traits::PrimalCapability>,
    ) {
        capability_inference::get_default_capabilities_for_primal(primal_name)
    }
}

/// Create universal primal context for routing
pub fn create_universal_context(
    user_id: String,
    device_id: String,
    session_id: String,
) -> PrimalContext {
    PrimalContext {
        org_id: Some("ecosystem_discovery".to_string()),
        user_id: Some(user_id),
        device_id: Some(device_id),
        session_id: Some(session_id),
        request_id: Some(uuid::Uuid::new_v4().to_string()),
        metadata: HashMap::new(),
    }
}
