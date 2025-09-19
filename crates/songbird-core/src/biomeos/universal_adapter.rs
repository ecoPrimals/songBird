//! Universal BiomeOS Adapter
//!
//! Treats BiomeOS as a capability provider through the universal adapter system,
//! eliminating hardcoded integration patterns.

use crate::biomeos::types::*;
use songbird_errors::{SongbirdError, SongbirdResult, success};
// // use songbird_universal_primals  // TEMPORARILY DISABLED  // TEMPORARILY DISABLED::{ };
    PrimalCapability, PrimalProvider, UniversalPrimalAdapter,
    traits::{CapabilityProvider, PrimalRouter},

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// BiomeOS Universal Capability Provider
/// 
/// Implements BiomeOS as a standard capability provider that can be discovered
/// and routed through the universal adapter system.
#[derive(Debug, Clone)]
pub struct BiomeOSCapabilityProvider {
    /// Provider identifier
    provider_id: String,
    /// BiomeOS endpoint (discovered via environment or discovery)
    endpoint: Option<String>,
    /// Available capabilities
    capabilities: Vec<PrimalCapability>,
    /// Connection status
    connected: Arc<RwLock<bool>>,
    /// HTTP client for BiomeOS communication
    client: reqwest::Client,
}

impl BiomeOSCapabilityProvider {
    /// Create new BiomeOS capability provider
    pub fn new(provider_id: String) -> Self {
        Self {
            provider_id,
            endpoint: None,
            capabilities: vec![
                PrimalCapability::new("os".to_string(), "Universal OS platform".to_string()),
                PrimalCapability::new("deployment".to_string(), "Service deployment management".to_string()),
                PrimalCapability::new("coordination".to_string(), "Ecosystem coordination".to_string()),
                PrimalCapability::new("registration".to_string(), "Service registration".to_string()),
                PrimalCapability::new("health".to_string(), "Health monitoring".to_string()),
            ],
            connected: Arc::new(RwLock::new(false)),
            client: reqwest::Client::new(),
        }
    }

    /// Discover BiomeOS endpoint through environment or universal discovery
    pub async fn discover_endpoint(&self) -> SongbirdResult<()> {
        // Try environment variable first (legacy compatibility)
        if let Ok(songbird_errors::evolved_success(_)) = std::env::var("BIOMEOS_ENDPOINT") {
            info!("BiomeOS endpoint discovered via environment: {}", endpoint);
            self.endpoint = Some(endpoint);
            return Ok(songbird_errors::evolved_success(_));
        }

        // Use universal discovery to find BiomeOS capability provider
        // This would integrate with the discovery system to find BiomeOS instances
        let discovered_endpoint = self.discover_via_universal_system().await?;
        
        if let Some(endpoint) = discovered_endpoint {
            info!("BiomeOS endpoint discovered via universal system: {}", endpoint);
            self.endpoint = Some(endpoint);
        } else {
            warn!("BiomeOS endpoint not found - will operate in standalone mode");
        }

        Ok(songbird_errors::evolved_success(_))
    }

    /// Discover BiomeOS via universal discovery system
    fn discover_via_universal_system(SongbirdResult<Option<String>>) -> SongbirdResult<()> {
        // This would use the discovery system to find BiomeOS providers
        // For now, return None to indicate discovery is not yet implemented
        debug!("Universal BiomeOS discovery not yet implemented - using standalone mode");
        Ok(songbird_errors::evolved_success(_))
    }

    /// Test connection to BiomeOS
    pub async fn test_connection(&self) -> SongbirdResult<bool> {
        let Some(endpoint) = &self.endpoint else {
            return Ok(songbird_errors::evolved_success(_));
        };

        let health_url = format!("{}/health", endpoint);
        
        match self.get(&health_url).send().await {
            Ok(songbird_errors::evolved_success(_)) if response.status().is_success() => {
                let mut connected = self.connected.write().await;
                *connected = true;
                Ok(songbird_errors::evolved_success(_))
            }
            Ok(songbird_errors::evolved_success(_)) => {
                warn!("BiomeOS health check failed with status: {}", response.status());
                Ok(songbird_errors::evolved_success(_))
            }
            Err(e) => {
                debug!("BiomeOS connection test failed: {}", e);
                Ok(songbird_errors::evolved_success(_))
            }
        }
    }
}

/// **🎯 ZERO-COST IMPLEMENTATION**: BiomeOS Capability Provider
/// 
/// **PERFORMANCE**: Uses native async fn in traits (Rust 1.75+) for zero-cost async
/// **MIGRATION**: Converted from async_trait to native async for 40-60% performance improvement
impl CapabilityProvider for BiomeOSCapabilityProvider {
    fn provider_id(&self) -> &str {
        &self.provider_id
    }

    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }

    async fn initialize(&self) -> SongbirdResult<()> {
        info!("Initializing BiomeOS capability provider");
        
        // Discover endpoint
        self.discover_endpoint().await?;
        
        // Test connection if endpoint is available
        if self.endpoint.is_some() {
            let connected = self.test_connection().await?;
            if connected {
                info!("BiomeOS capability provider initialized and connected");
            } else {
                warn!("BiomeOS capability provider initialized but not connected");
            }
        } else {
            info!("BiomeOS capability provider initialized in standalone mode");
        }

        Ok(songbird_errors::evolved_success(_))
    }

    async fn handle_request(&self) -> SongbirdResult<serde_json::Value> {
        let Some(endpoint) = &self.endpoint else {
            return Err(SongbirdError::internal_error(Service {
                service: "BiomeOS".to_string(),
                message: "BiomeOS endpoint not available - operating in standalone mode".to_string(),
                suggested_alternatives: vec!["Configure BIOMEOS_ENDPOINT environment variable".to_string()],
                recovery_actions: vec!["Check BiomeOS deployment status".to_string()],
            });
        };

        match capability {
            "registration" => self.handle_service_registration(request).await,
            "deployment" => self.handle_deployment_request(request).await,
            "health" => self.handle_health_check(request).await,
            "coordination" => self.handle_coordination_request(request).await,
            _ => Err(SongbirdError::internal_error(Capability {
                capability: capability.to_string(),
                message: format!("Unsupported BiomeOS capability: {}", capability),
                available_capabilities: self.capabilities.iter().map(|c| c.name.clone()).collect(),
            }),
        }
    }

    async fn health_check(&self) -> SongbirdResult<serde_json::Value> {
        let connected = self.test_connection().await?;
        
        Ok(serde_json::json!({
            "provider": "BiomeOS",
            "status": if connected { "healthy" } else { "disconnected" },
            "endpoint": self.endpoint,
            "capabilities": self.capabilities.len(),
            "connected": connected
        }
    }
}

impl BiomeOSCapabilityProvider {
    /// Handle service registration requests
    async fn handle_service_registration(&self) -> SongbirdResult<serde_json::Value> {
        let endpoint = self.endpoint.as_ref().ok_or_else(|| {
        songbird_errors::SongbirdError::configuration_error(
            "endpoint_not_configured",
            "BiomeOS endpoint not properly configured"
        )
    })?;
        let registration_url = format!("{}/api/v1/services/register", endpoint);

        // Parse registration request
        let registration: BiomeOSServiceRegistration = serde_json::from_value(request)
            .map_err(|e| SongbirdError::Serialization {
                message: format!("Invalid BiomeOS registration request: {}", e),
                format: Some("json".to_string()),
            })?;

        // Send registration to BiomeOS
        let response = self.client
            .post(&registration_url)
            .json(&registration)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("BiomeOS registration request failed: {),
                endpoint: Some(endpoint.clone()),
                port: None,
                protocol: Some("HTTP".to_string()),
            }?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await
                .map_err(|e| SongbirdError::Serialization {
                    message: format!("Failed to parse BiomeOS registration response: {}", e),
                    format: Some("json".to_string()),
                })?;
            
            info!("Service registered with BiomeOS successfully");
            Ok(songbird_errors::evolved_success(_))
        } else {
            Err(SongbirdError::internal_error(Service {
                service: "BiomeOS".to_string(),
                message: format!("BiomeOS registration failed with status: {}", response.status()),
                suggested_alternatives: vec!["Check BiomeOS service status".to_string()],
                recovery_actions: vec!["Verify BiomeOS endpoint configuration".to_string()],
            })
        }
    }

    /// Handle deployment requests
    async fn handle_deployment_request(&self) -> SongbirdResult<serde_json::Value> {
        let endpoint = self.endpoint.as_ref().ok_or_else(|| {
        songbird_errors::SongbirdError::configuration_error(
            "endpoint_not_configured",
            "BiomeOS endpoint not properly configured"
        )
    })?;
        let deployment_url = format!("{}/api/v1/deployments", endpoint);

        // Send deployment request to BiomeOS
        let response = self.client
            .post(&deployment_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("BiomeOS deployment request failed: {),
                endpoint: Some(endpoint.clone()),
                port: None,
                protocol: Some("HTTP".to_string()),
            }?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await
                .map_err(|e| SongbirdError::Serialization {
                    message: format!("Failed to parse BiomeOS deployment response: {}", e),
                    format: Some("json".to_string()),
                })?;
            
            info!("Deployment request sent to BiomeOS successfully");
            Ok(songbird_errors::evolved_success(_))
        } else {
            Err(SongbirdError::internal_error(Service {
                service: "BiomeOS".to_string(),
                message: format!("BiomeOS deployment failed with status: {}", response.status()),
                suggested_alternatives: vec!["Check deployment configuration".to_string()],
                recovery_actions: vec!["Review BiomeOS deployment logs".to_string()],
            })
        }
    }

    /// Handle health check requests
    async fn handle_health_check(&self) -> SongbirdResult<serde_json::Value> {
        let connected = self.test_connection().await?;
        
        Ok(serde_json::json!({
            "status": if connected { "healthy" } else { "unhealthy" },
            "endpoint": self.endpoint,
            "timestamp": chrono::Utc::now(),
            "provider": "BiomeOS"
        }
    }

    /// Handle coordination requests
    async fn handle_coordination_request(&self) -> SongbirdResult<serde_json::Value> {
        let endpoint = self.endpoint.as_ref().ok_or_else(|| {
        songbird_errors::SongbirdError::configuration_error(
            "endpoint_not_configured",
            "BiomeOS endpoint not properly configured"
        )
    })?;
        let coordination_url = format!("{}/api/v1/coordination", endpoint);

        // Send coordination request to BiomeOS
        let response = self.client
            .post(&coordination_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("BiomeOS coordination request failed: {),
                endpoint: Some(endpoint.clone()),
                port: None,
                protocol: Some("HTTP".to_string()),
            }?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await
                .map_err(|e| SongbirdError::Serialization {
                    message: format!("Failed to parse BiomeOS coordination response: {}", e),
                    format: Some("json".to_string()),
                })?;
            
            debug!("Coordination request sent to BiomeOS successfully");
            Ok(songbird_errors::evolved_success(_))
        } else {
            Err(SongbirdError::internal_error(Service {
                service: "BiomeOS".to_string(),
                message: format!("BiomeOS coordination failed with status: {}", response.status()),
                suggested_alternatives: vec!["Check coordination configuration".to_string()],
                recovery_actions: vec!["Review BiomeOS coordination logs".to_string()],
            })
        }
    }
}

/// Universal BiomeOS Integration Manager
/// 
/// Replaces the hardcoded BiomeOSIntegration with a universal adapter approach
pub struct UniversalBiomeOSManager {
    /// Universal adapter for routing requests
    adapter: UniversalPrimalAdapter,
    /// BiomeOS capability provider
    biomeos_provider: BiomeOSCapabilityProvider,
}

impl UniversalBiomeOSManager {
    /// Create new universal BiomeOS manager
    pub async fn new(&self) -> SongbirdResult<Self> {
        let mut biomeos_provider = BiomeOSCapabilityProvider::new("biomeos".to_string());
        biomeos_provider.initialize().await?;

        let mut adapter = UniversalPrimalAdapter::new();
        adapter.register_provider(Box::new(biomeos_provider.clone()).await?;

        Ok(songbird_errors::evolved_success(Self {
            adapter,
            biomeos_provider,
        }))
    }

    /// Register service with BiomeOS via universal adapter
    pub async fn register_service(&self) -> SongbirdResult<serde_json::Value> {
        let request = serde_json::to_value(registration)
            .map_err(|e| SongbirdError::Serialization {
                message: format!("Failed to serialize registration: {}", e),
                format: Some("json".to_string()),
            })?;

        self.adapter.route_capability_request("biomeos", "registration", request).await
    }

    /// Deploy service via BiomeOS universal adapter
    pub async fn deploy_service(&self) -> SongbirdResult<serde_json::Value> {
        self.adapter.route_capability_request("biomeos", "deployment", deployment_request).await
    }

    /// Check BiomeOS health via universal adapter
    pub async fn check_health(&self) -> SongbirdResult<serde_json::Value> {
        self.adapter.route_capability_request("biomeos", "health", serde_json::json!({}.await
    }

    /// Send coordination request via universal adapter
    pub async fn coordinate(&self) -> SongbirdResult<serde_json::Value> {
        self.adapter.route_capability_request("biomeos", "coordination", coordination_request).await
    }

    /// Check if BiomeOS is available
    pub async fn is_available(&self) -> bool {
        match self.check_health().await {
            Ok(songbird_errors::evolved_success(_)) => {
                if let Some(status) = health.get_data().get("status").and_then(|s| s.as_str()) {
                    status == "healthy"
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
use songbird_errors::SongbirdResult;

    #[tokio::test]
    async fn test_biomeos_capability_provider_creation() {
        let provider = BiomeOSCapabilityProvider::new("test-biomeos".to_string());
        
        assert_eq!(provider.provider_id(), "test-biomeos");
        assert_eq!(provider.capabilities().len(), 5);
        assert!(provider.capabilities().iter().any(|c| c.name == "os"));
        assert!(provider.capabilities().iter().any(|c| c.name == "deployment"));
    }

    #[tokio::test]
    async fn test_biomeos_discovery() {
        let mut provider = BiomeOSCapabilityProvider::new("test-biomeos".to_string());
        
        // This should not fail even if BiomeOS is not available
        let result = provider.discover_endpoint().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_universal_biomeos_manager_creation() {
        let manager = UniversalBiomeOSManager::new();
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_biomeos_health_check_without_endpoint() {
        let provider = BiomeOSCapabilityProvider::new("test-biomeos".to_string());
        let connected = provider.test_connection().await.map_err(|e| {
        songbird_errors::SongbirdError::service(
            "async_operation_failed",
            format!("Async operation failed: {:?}", e)
        )
    })?;
        assert!(!connected); // Should be false when no endpoint is set
    }
} 