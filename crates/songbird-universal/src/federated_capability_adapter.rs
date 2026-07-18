// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Federated Capability Adapter
//!
//! Extends capability routing to support federated service discovery.
//! This adapter queries both local and remote (federated) services when routing requests.

use crate::{
    UniversalAdapterError,
    types::{DiscoveredCapability, HealthStatus, PrimalType, QosMetrics, ServiceInfo},
};
use std::sync::Arc;
use tracing::{debug, info};

#[cfg(test)]
use songbird_types::SongbirdResult;

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
    pub const fn new() -> Self {
        Self {
            federation_client: None,
        }
    }

    /// Enable federation by providing a client
    #[must_use]
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
            debug!("🌐 Querying federation for capability '{}'", capability_type);

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
                    debug!("⚠️  Federation query failed (using local only): {}", e);
                    // Continue with local services
                }
            }
        }

        Ok(all_providers)
    }

    /// Check if federation is enabled
    #[must_use]
    pub const fn is_federation_enabled(&self) -> bool {
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
    /// Base URL of the federation API (e.g., "<http://localhost:8080>")
    base_url: String,
    // Note: HTTP client created on-demand to support async initialization
}

impl FederationClient {
    /// Create a new federation client
    #[must_use]
    pub const fn new(base_url: String) -> Self {
        Self {
            base_url,
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

        // Create client on-demand
        let client = songbird_http_client::IpcHttpClient::new().await.map_err(|e| {
            UniversalAdapterError::NetworkError(format!("Failed to create HTTP client: {e}"))
        })?;

        let response = client
            .get(&url)
            .await
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string()))?;

        if !response.is_success() {
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
                    .is_some_and(|arr| arr.iter().any(|c| c.as_str() == Some(capability)))
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
                            version: String::from("1.0"),
                            description: format!("{cap_str} capability"),
                            provider: String::from("federated"),
                            endpoint: String::new(), // Will use service endpoint
                            qos_metrics: QosMetrics::default(),
                            health_status: HealthStatus::Healthy,
                        })
                    })
                    .collect();

                Some(ServiceInfo {
                    name,
                    primal_type: PrimalType {
                        category: String::from("generic"),
                        subcategory: None,
                        version: String::from("1.0"),
                    },
                    endpoint,
                    capabilities,
                    health: HealthStatus::Healthy, // Assume healthy if in registry
                    metadata: serde_json::from_value(svc.get("metadata")?.clone()).ok()?,
                })
            })
            .collect();

        Ok(matching_services)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::*;
    use songbird_types::SongbirdError;

    #[tokio::test]
    async fn test_federated_adapter_without_federation() -> SongbirdResult<()> {
        let adapter = FederatedCapabilityAdapter::new();

        assert!(!adapter.is_federation_enabled());

        let local_services = vec![ServiceInfo {
            name: String::from("Local Service"),
            primal_type: PrimalType {
                category: String::from("test"),
                subcategory: None,
                version: String::from("1.0"),
            },
            endpoint: String::from("http://localhost:8080"),
            capabilities: vec![DiscoveredCapability {
                name: String::from("test-capability"),
                version: String::from("1.0"),
                description: String::from("Test capability"),
                provider: String::from("local"),
                endpoint: String::new(),
                qos_metrics: QosMetrics::default(),
                health_status: HealthStatus::Healthy,
            }],
            health: HealthStatus::Healthy,
            metadata: std::collections::HashMap::new(),
        }];

        let providers = adapter
            .find_capability_providers("test-capability", local_services.clone())
            .await
            .map_err(|e| {
                SongbirdError::configuration(format!(
                    "Failed to find capability providers without federation: {e}"
                ))
            })?;
        assert_eq!(providers.len(), 1);
        Ok(())
    }

    #[test]
    fn test_federation_client_construction() {
        let client = FederationClient::new(String::from("http://localhost:8080"));
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_federated_adapter_default() {
        let adapter = FederatedCapabilityAdapter::default();
        assert!(!adapter.is_federation_enabled());
    }

    #[test]
    fn test_federated_adapter_with_federation() {
        let client = Arc::new(FederationClient::new(String::from("http://localhost:8080")));
        let adapter = FederatedCapabilityAdapter::new().with_federation(client);

        assert!(adapter.is_federation_enabled());
    }

    #[test]
    fn test_federated_adapter_clone() {
        let client = Arc::new(FederationClient::new(String::from("http://localhost:8080")));
        let adapter = FederatedCapabilityAdapter::new().with_federation(client);

        let cloned = adapter;
        assert!(cloned.is_federation_enabled());
    }

    #[tokio::test]
    async fn test_find_capability_providers_empty_local() -> Result<(), Box<dyn std::error::Error>>
    {
        let adapter = FederatedCapabilityAdapter::new();

        let result = adapter.find_capability_providers("test-capability", vec![]).await;

        assert!(result.is_ok());
        assert_eq!(
            result.map_err(|e| SongbirdError::configuration(format!("Error: {e}")))?.len(),
            0
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_find_capability_providers_multiple_local()
    -> Result<(), Box<dyn std::error::Error>> {
        let adapter = FederatedCapabilityAdapter::new();

        let local_services = vec![
            ServiceInfo {
                name: String::from("Service 1"),
                primal_type: PrimalType {
                    category: String::from("test"),
                    subcategory: None,
                    version: String::from("1.0"),
                },
                endpoint: String::from("http://localhost:8080"),
                capabilities: vec![DiscoveredCapability {
                    name: String::from("test-capability"),
                    version: String::from("1.0"),
                    description: String::from("Test capability"),
                    provider: String::from("local"),
                    endpoint: String::new(),
                    qos_metrics: QosMetrics::default(),
                    health_status: HealthStatus::Healthy,
                }],
                health: HealthStatus::Healthy,
                metadata: std::collections::HashMap::new(),
            },
            ServiceInfo {
                name: String::from("Service 2"),
                primal_type: PrimalType {
                    category: String::from("test"),
                    subcategory: None,
                    version: String::from("1.0"),
                },
                endpoint: String::from("http://localhost:8081"),
                capabilities: vec![DiscoveredCapability {
                    name: String::from("test-capability"),
                    version: String::from("1.0"),
                    description: String::from("Test capability"),
                    provider: String::from("local"),
                    endpoint: String::new(),
                    qos_metrics: QosMetrics::default(),
                    health_status: HealthStatus::Healthy,
                }],
                health: HealthStatus::Healthy,
                metadata: std::collections::HashMap::new(),
            },
        ];

        let result =
            adapter.find_capability_providers("test-capability", local_services.clone()).await;

        assert!(result.is_ok());
        assert_eq!(
            result.map_err(|e| SongbirdError::configuration(format!("Error: {e}")))?.len(),
            2
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_find_capability_providers_preserves_local_on_federation_failure()
    -> Result<(), Box<dyn std::error::Error>> {
        // Create client with invalid URL to force failure
        let client = Arc::new(FederationClient::new(String::from(
            "http://invalid-domain-that-does-not-exist-12345.com",
        )));
        let adapter = FederatedCapabilityAdapter::new().with_federation(client);

        let local_services = vec![ServiceInfo {
            name: String::from("Local Service"),
            primal_type: PrimalType {
                category: String::from("test"),
                subcategory: None,
                version: String::from("1.0"),
            },
            endpoint: String::from("http://localhost:8080"),
            capabilities: vec![DiscoveredCapability {
                name: String::from("test-capability"),
                version: String::from("1.0"),
                description: String::from("Test capability"),
                provider: String::from("local"),
                endpoint: String::new(),
                qos_metrics: QosMetrics::default(),
                health_status: HealthStatus::Healthy,
            }],
            health: HealthStatus::Healthy,
            metadata: std::collections::HashMap::new(),
        }];

        // Should return local services even if federation fails
        let result =
            adapter.find_capability_providers("test-capability", local_services.clone()).await;

        assert!(result.is_ok());
        let services = result.map_err(|e| {
            SongbirdError::configuration(format!("Missing performance configuration: {e}"))
        })?;
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "Local Service");
        Ok(())
    }

    #[test]
    fn test_federation_client_new() {
        let client = FederationClient::new(String::from("http://api.example.com"));
        assert_eq!(client.base_url, "http://api.example.com");
    }

    #[test]
    fn test_federation_client_with_trailing_slash() {
        let client = FederationClient::new(String::from("http://localhost:8080/"));
        assert_eq!(client.base_url, "http://localhost:8080/");
        // The implementation should handle this in the URL construction
    }

    #[test]
    fn test_federation_client_with_port() {
        let client = FederationClient::new(String::from("http://localhost:9000"));
        assert_eq!(client.base_url, "http://localhost:9000");
    }

    #[test]
    fn test_federation_client_https() {
        let client = FederationClient::new(String::from("https://secure.example.com"));
        assert_eq!(client.base_url, "https://secure.example.com");
    }

    #[test]
    fn test_adapter_new_has_no_federation() {
        let adapter = FederatedCapabilityAdapter::new();
        assert!(!adapter.is_federation_enabled());

        // federation_client should be None
        assert!(adapter.federation_client.is_none());
    }

    #[test]
    fn test_adapter_builder_pattern() {
        let client = Arc::new(FederationClient::new(String::from("http://localhost:8080")));

        // Test builder pattern fluent API
        let adapter = FederatedCapabilityAdapter::new().with_federation(client);

        assert!(adapter.is_federation_enabled());
    }

    #[test]
    fn test_adapter_multiple_with_federation_calls() -> SongbirdResult<()> {
        let client1 = Arc::new(FederationClient::new(String::from("http://localhost:8080")));
        let client2 = Arc::new(FederationClient::new(String::from("http://localhost:9000")));

        // Last call should win
        let adapter = FederatedCapabilityAdapter::new()
            .with_federation(client1)
            .with_federation(client2.clone());

        assert!(adapter.is_federation_enabled());
        // Verify it's using client2 (indirectly through base_url check)
        assert!(Arc::ptr_eq(
            &adapter.federation_client.ok_or_else(|| SongbirdError::configuration(
                String::from("Federation client should be present")
            ))?,
            &client2
        ));
        Ok(())
    }
}
