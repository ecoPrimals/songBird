//! Federated Capability Adapter
//!
//! Extends capability routing to support federated service discovery.
//! This adapter queries both local and remote (federated) services when routing requests.

use crate::{types::{DiscoveredCapability, HealthStatus, PrimalType, QosMetrics, ServiceInfo}, UniversalAdapterError};
use std::sync::Arc;
use tracing::{debug, info};

/// Federated capability adapter - extends local discovery with federation
#[derive(Clone)]
pub struct FederatedCapabilityAdapter {
    /// Optional federation service registry client
    /// If None, only local discovery is used
    federation_client: Option<Arc<FederationClient>>,
}

impl FederatedCapabilityAdapter {
    /// Create a new federated capability adapter
    #[must_use]
    pub fn new() -> Self {
        Self {
            federation_client: None,
        }
    }
    
    /// Enable federation by providing a client
    pub fn with_federation(mut self, client: Arc<FederationClient>) -> Self {
        self.federation_client = Some(client);
        self
    }
    
    /// Find capability providers from both local and federated sources
    ///
    /// This method extends local capability discovery by also checking
    /// the federation service registry for remote services.
    ///
    /// # Errors
    ///
    /// Returns an error if federation query fails (local services still returned)
    pub async fn find_capability_providers(
        &self,
        capability_type: &str,
        local_providers: Vec<ServiceInfo>,
    ) -> Result<Vec<ServiceInfo>, UniversalAdapterError> {
        let mut all_providers = local_providers;
        
        // Query federation if enabled
        if let Some(ref client) = self.federation_client {
            debug!(
                "🌐 Querying federation for capability '{}'",
                capability_type
            );
            
            match client.find_services_by_capability(capability_type).await {
                Ok(federated_services) => {
                    let count = federated_services.len();
                    all_providers.extend(federated_services);
                    info!(
                        "✅ Found {} federated services for capability '{}'",
                        count, capability_type
                    );
                }
                Err(e) => {
                    debug!(
                        "⚠️  Federation query failed (using local only): {}",
                        e
                    );
                    // Continue with local services
                }
            }
        }
        
        Ok(all_providers)
    }
    
    /// Check if federation is enabled
    #[must_use]
    pub fn is_federation_enabled(&self) -> bool {
        self.federation_client.is_some()
    }
}

impl Default for FederatedCapabilityAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Client for querying federation service registry
///
/// This is a lightweight HTTP client that queries the federation API
/// for service discovery.
pub struct FederationClient {
    /// Base URL of the federation API (e.g., "http://localhost:8080")
    base_url: String,
    
    /// HTTP client
    client: reqwest::Client,
}

impl FederationClient {
    /// Create a new federation client
    #[must_use]
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
    
    /// Find services by capability
    ///
    /// Queries GET /api/federation/services and filters by capability
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or response is invalid
    pub async fn find_services_by_capability(
        &self,
        capability: &str,
    ) -> Result<Vec<ServiceInfo>, UniversalAdapterError> {
        let url = format!("{}/api/federation/services", self.base_url);
        
        debug!("📡 Querying federation services: {}", url);
        
        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(UniversalAdapterError::NetworkError(format!(
                "Federation API returned status: {}",
                response.status()
            )));
        }
        
        // Parse response
        let services: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string()))?;
        
        // Filter services that have the requested capability
        let matching_services: Vec<ServiceInfo> = services
            .into_iter()
            .filter(|svc| {
                svc.get("capabilities")
                    .and_then(|caps| caps.as_array())
                    .map(|arr| {
                        arr.iter()
                            .any(|c| c.as_str() == Some(capability))
                    })
                    .unwrap_or(false)
            })
            .filter_map(|svc| {
                // Convert federation service to ServiceInfo
                let name = svc.get("service_name")?.as_str()?.to_string();
                let endpoint = svc.get("endpoint")?.as_str()?.to_string();
                
                // Convert capabilities from strings to DiscoveredCapability
                let capabilities: Vec<DiscoveredCapability> = svc
                    .get("capabilities")?
                    .as_array()?
                    .iter()
                    .filter_map(|c| {
                        let cap_str = c.as_str()?;
                        Some(DiscoveredCapability {
                            name: cap_str.to_string(),
                            version: "1.0".to_string(),
                            description: format!("{} capability", cap_str),
                            provider: "federated".to_string(),
                            endpoint: String::new(), // Will use service endpoint
                            qos_metrics: QosMetrics::default(),
                            health_status: HealthStatus::Healthy,
                        })
                    })
                    .collect();
                
                Some(ServiceInfo {
                    name,
                    primal_type: PrimalType {
                        category: "generic".to_string(),
                        subcategory: None,
                        version: "1.0".to_string(),
                    },
                    endpoint,
                    capabilities,
                    health: HealthStatus::Healthy, // Assume healthy if in registry
                    metadata: serde_json::from_value(
                        svc.get("metadata")?.clone()
                    )
                    .ok()?,
                })
            })
            .collect();
        
        Ok(matching_services)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_federated_adapter_without_federation() {
        let adapter = FederatedCapabilityAdapter::new();
        
        assert!(!adapter.is_federation_enabled());
        
        let local_services = vec![
            ServiceInfo {
                name: "Local Service".to_string(),
                primal_type: PrimalType {
                    category: "test".to_string(),
                    subcategory: None,
                    version: "1.0".to_string(),
                },
                endpoint: "http://localhost:8080".to_string(),
                capabilities: vec![DiscoveredCapability {
                    name: "test-capability".to_string(),
                    version: "1.0".to_string(),
                    description: "Test capability".to_string(),
                    provider: "local".to_string(),
                    endpoint: String::new(),
                    qos_metrics: QosMetrics::default(),
                    health_status: HealthStatus::Healthy,
                }],
                health: HealthStatus::Healthy,
                metadata: std::collections::HashMap::new(),
            },
        ];
        
        let result = adapter
            .find_capability_providers("test-capability", local_services.clone())
            .await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
    
    #[test]
    fn test_federation_client_construction() {
        let client = FederationClient::new("http://localhost:8080".to_string());
        assert_eq!(client.base_url, "http://localhost:8080");
    }
}

