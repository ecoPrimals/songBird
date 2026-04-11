// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::error::UniversalAdapterError;
use super::types::{CapabilityRegistry, RegistryStats, UnifiedAdapterConfig};
use crate::types::{HealthStatus, ServiceInfo, UniversalRequest, UniversalResponse};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// **UNIFIED**: Universal capability adapter that consolidates all adapter patterns
///
/// This replaces:
/// - `songbird-universal::adapters::UniversalCapabilityAdapter`
/// - `songbird-universal::capabilities::UniversalCapabilityAdapter`
/// - `songbird-universal-primals::universal_adapter::*`
/// - Various other adapter fragments
#[derive(Debug, Clone)]
pub struct UnifiedUniversalAdapter {
    /// Registry of discovered capabilities
    capability_registry: Arc<RwLock<CapabilityRegistry>>,
    /// Active service connections (reserved for connection pooling implementation)
    #[allow(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
    service_connections: Arc<RwLock<HashMap<String, super::types::ServiceConnection>>>,
    /// Adapter configuration
    config: UnifiedAdapterConfig,
}

impl UnifiedUniversalAdapter {
    /// Create a new unified adapter with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(UnifiedAdapterConfig::default())
    }

    /// Create a new unified adapter with custom configuration
    #[must_use]
    pub fn with_config(config: UnifiedAdapterConfig) -> Self {
        Self {
            capability_registry: Arc::new(RwLock::new(CapabilityRegistry::default())),
            service_connections: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Discover services and their capabilities
    ///
    /// # Errors
    ///
    /// This function logs errors but does not fail - it returns all successfully discovered services
    pub async fn discover_services(&self) -> Result<Vec<ServiceInfo>, UniversalAdapterError> {
        info!("🔍 Starting universal service discovery");

        let mut discovered_services = Vec::new();

        for endpoint in &self.config.discovery_endpoints {
            match self.discover_from_endpoint(endpoint).await {
                Ok(mut services) => {
                    discovered_services.append(&mut services);
                }
                Err(e) => {
                    warn!("Failed to discover from endpoint {}: {}", endpoint, e);
                }
            }
        }

        // Merge services already in the local registry (prior discovery, tests, or manual seed).
        // Keeps sovereignty / routing useful when HTTP endpoints are down or empty.
        {
            let registry = self.capability_registry.read().await;
            for (name, info) in &registry.service_info {
                if !discovered_services.iter().any(|s| s.name == *name) {
                    discovered_services.push(info.clone());
                }
            }
        }

        // Update registry with discovered services
        {
            let mut registry = self.capability_registry.write().await;
            for service in &discovered_services {
                registry.service_info.insert(service.name.clone(), service.clone());
                registry.last_updated.insert(service.name.clone(), chrono::Utc::now());

                // Index capabilities
                for capability in &service.capabilities {
                    registry
                        .capability_providers
                        .entry(capability.name.clone())
                        .or_insert_with(Vec::new)
                        .push(service.name.clone());
                }
            }
        } // Drop registry lock here

        info!("✅ Discovered {} services", discovered_services.len());
        Ok(discovered_services)
    }

    /// Find services that provide specific capabilities
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn find_capability_providers(
        &self,
        capability_type: &str,
    ) -> Result<Vec<ServiceInfo>, UniversalAdapterError> {
        let registry = self.capability_registry.read().await;

        let providers =
            registry.capability_providers.get(capability_type).cloned().unwrap_or_default();

        let mut services = Vec::new();
        for provider in providers {
            if let Some(service) = registry.service_info.get(&provider) {
                services.push(service.clone());
            }
        }

        debug!("Found {} providers for capability '{}'", services.len(), capability_type);
        Ok(services)
    }

    /// Route a request to the best available service
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The required capability is not found
    /// - No services are available for the capability
    /// - All service requests fail
    pub async fn route_request(
        &self,
        request: UniversalRequest,
    ) -> Result<UniversalResponse, UniversalAdapterError> {
        // Extract required capability from request
        let capability_type = request
            .parameters
            .get("capability_type")
            .and_then(|v| v.as_str())
            .ok_or(UniversalAdapterError::MissingCapability)?;

        // Find providers
        let providers = self.find_capability_providers(capability_type).await?;
        if providers.is_empty() {
            return Err(UniversalAdapterError::NoProvidersAvailable(capability_type.to_string()));
        }

        // Select best provider (simple round-robin for now)
        let provider = &providers[0];

        // Route request to selected provider
        self.send_request_to_service(provider, request).await
    }

    /// Send request to a specific service
    async fn send_request_to_service(
        &self,
        service: &ServiceInfo,
        request: UniversalRequest,
    ) -> Result<UniversalResponse, UniversalAdapterError> {
        let url = format!("{}/api/v1/{}", service.endpoint, request.action);

        // Create HTTP client on-demand
        let client = songbird_http_client::IpcHttpClient::new().await.map_err(|e| {
            UniversalAdapterError::NetworkError(format!("Failed to create HTTP client: {e}"))
        })?;

        // IpcHttpClient::post() returns RequestBuilder directly
        let request_builder = client.post(&url).await;

        let response = request_builder
            .json(&request)
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string()))?
            .send()
            .await
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string()))?;

        if response.is_success() {
            let universal_response: UniversalResponse = response
                .json()
                .await
                .map_err(|e| UniversalAdapterError::ParseError(e.to_string()))?;
            Ok(universal_response)
        } else {
            Err(UniversalAdapterError::ServiceError(format!("HTTP {}", response.status())))
        }
    }

    /// Discover services from a specific endpoint
    async fn discover_from_endpoint(
        &self,
        endpoint: &str,
    ) -> Result<Vec<ServiceInfo>, UniversalAdapterError> {
        debug!("Discovering services from endpoint: {}", endpoint);

        // Create HTTP client on-demand
        let client = songbird_http_client::IpcHttpClient::new().await.map_err(|e| {
            UniversalAdapterError::NetworkError(format!("Failed to create HTTP client: {e}"))
        })?;

        let response = client
            .get(endpoint)
            .await
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string()))?;

        if response.is_success() {
            let services: Vec<ServiceInfo> = response
                .json()
                .await
                .map_err(|e| UniversalAdapterError::ParseError(e.to_string()))?;
            Ok(services)
        } else {
            Err(UniversalAdapterError::DiscoveryError(format!("HTTP {}", response.status())))
        }
    }

    /// Get current registry statistics
    pub async fn get_registry_stats(&self) -> RegistryStats {
        let registry = self.capability_registry.read().await;

        RegistryStats {
            total_services: registry.service_info.len(),
            total_capabilities: registry.capability_providers.len(),
            healthy_services: registry
                .service_info
                .values()
                .filter(|s| s.health == HealthStatus::Healthy)
                .count(),
        }
    }
}

impl Default for UnifiedUniversalAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
