//! BYOB Integration
//!
//! Handles integration with primal discovery.
//! Storage operations are handled by the universal primal adapter system.

use super::super::NestGateConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Integration manager
pub struct IntegrationManager {
    /// NestGate storage configuration (for discovery only)
    nestgate_config: Option<NestGateConfig>,
    /// Primal discovery endpoints
    primal_discovery: Arc<RwLock<HashMap<String, String>>>,
}

impl IntegrationManager {
    /// Create new integration manager
    pub fn new() -> Self {
        Self {
            nestgate_config: None,
            primal_discovery: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Configure with NestGate (for discovery only)
    pub fn with_nestgate(mut self, config: NestGateConfig) -> Self {
        self.nestgate_config = Some(config);
        self
    }

    /// Storage operations are handled by the universal primal adapter system
    /// This integration manager only handles primal discovery

    /// Add primal discovery endpoint
    pub async fn add_primal_discovery_endpoint(
        &self,
        primal_name: String,
        endpoint: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Adding primal discovery endpoint: {} -> {}",
            primal_name, endpoint
        );

        self.primal_discovery
            .write()
            .await
            .insert(primal_name, endpoint);

        Ok(())
    }

    /// Remove primal discovery endpoint
    pub async fn remove_primal_discovery_endpoint(
        &self,
        primal_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Removing primal discovery endpoint: {}", primal_name);

        self.primal_discovery.write().await.remove(primal_name);

        Ok(())
    }

    /// List all primal discovery endpoints
    pub async fn list_primal_discovery_endpoints(&self) -> HashMap<String, String> {
        self.primal_discovery.read().await.clone()
    }

    /// Discover available primals
    pub async fn discover_primals(
        &self,
    ) -> Result<Vec<PrimalInfo>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Discovering primals");

        let mut primals = Vec::new();
        let discovery = self.primal_discovery.read().await;

        for (name, endpoint) in discovery.iter() {
            // Primal discovery is handled by the core discovery system
            // This would involve HTTP calls to primal endpoints

            primals.push(PrimalInfo {
                name: name.clone(),
                endpoint: endpoint.clone(),
                capabilities: vec!["discovery".to_string()],
                health: "unknown".to_string(),
                last_seen: chrono::Utc::now(),
            });
        }

        Ok(primals)
    }
}

/// Primal information
#[derive(Debug, Clone)]
pub struct PrimalInfo {
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub health: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

impl Default for IntegrationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for IntegrationManager {
    fn clone(&self) -> Self {
        Self {
            nestgate_config: self.nestgate_config.clone(),
            primal_discovery: self.primal_discovery.clone(),
        }
    }
}
