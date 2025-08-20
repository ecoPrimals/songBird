/// Universal Primal Integration Layer
///
/// This module provides capability-based routing to any primal without hardcoding
/// specific primal names. It uses the Universal Adapter to discover and route
/// requests based on capabilities, not names.
use crate::UniversalCapabilityAdapter;
use serde_json::Value;
use songbird_errors::{SafeUnwrapOption, SongbirdError, SongbirdResult};
use std::sync::Arc;
use tracing::{debug, info};

/// Universal Primal Integration Layer - No hardcoded primal names
#[derive(Debug)]
pub struct UniversalPrimalIntegration {
    capability_adapter: Arc<UniversalCapabilityAdapter>,
    #[allow(dead_code)]
    http_client: reqwest::Client,
}

impl UniversalPrimalIntegration {
    pub fn new(capability_adapter: Arc<UniversalCapabilityAdapter>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            capability_adapter,
            http_client,
        }
    }

    /// Route security request via capability discovery (no hardcoded "beardog")
    pub async fn route_security_request(&self) -> SongbirdResult<Value> {
        debug!(
            "🔐 Routing security request via capability discovery: {}",
            operation
        );

        // Discover security capability providers dynamically
        let security_provider = self
            .capability_adapter
            .get_best_primal_for_capability("security")
            .await
            .or_config_error(
                "security_discovery",
                "No security capability provider found in ecosystem",
            )?;

        info!("✅ Found security provider: {}", security_provider.data);

        // Route to discovered security provider
        self.send_capability_request("security", operation, payload)
            .await
    }

    /// Route storage request via capability discovery (no hardcoded "nestgate")
    pub async fn route_storage_request(&self) -> SongbirdResult<Value> {
        debug!(
            "💾 Routing storage request via capability discovery: {}",
            operation
        );

        // Discover storage capability providers dynamically
        let storage_provider = self
            .capability_adapter
            .get_best_primal_for_capability("storage")
            .await
            .or_config_error(
                "storage_discovery",
                "No storage capability provider found in ecosystem",
            )?;

        info!("✅ Found storage provider: {}", storage_provider.data);

        // Route to discovered storage provider
        self.send_capability_request("storage", operation, payload)
            .await
    }

    /// Route AI request via capability discovery (no hardcoded "squirrel")
    pub async fn route_ai_request(&self) -> SongbirdResult<Value> {
        debug!(
            "🤖 Routing AI request via capability discovery: {}",
            operation
        );

        // Discover AI capability providers dynamically
        let ai_provider = self
            .capability_adapter
            .get_best_primal_for_capability("ai")
            .await
            .or_config_error(
                "ai_discovery",
                "No AI capability provider found in ecosystem",
            )?;

        info!("✅ Found AI provider: {}", ai_provider.data);

        // Route to discovered AI provider
        self.send_capability_request("ai", operation, payload).await
    }

    /// Route compute request via capability discovery (no hardcoded "toadstool")
    pub async fn route_compute_request(&self) -> SongbirdResult<Value> {
        debug!(
            "⚡ Routing compute request via capability discovery: {}",
            operation
        );

        // Discover compute capability providers dynamically
        let compute_provider = self
            .capability_adapter
            .get_best_primal_for_capability("compute")
            .await
            .or_config_error(
                "compute_discovery",
                "No compute capability provider found in ecosystem",
            )?;

        info!("✅ Found compute provider: {}", compute_provider.data);

        // Route to discovered compute provider
        self.send_capability_request("compute", operation, payload)
            .await
    }

    /// Route system request via capability discovery (no hardcoded "biomeos")
    pub async fn route_system_request(&self) -> SongbirdResult<Value> {
        debug!(
            "🌱 Routing system request via capability discovery: {}",
            operation
        );

        // Discover system capability providers dynamically
        let system_provider = self
            .capability_adapter
            .get_best_primal_for_capability("system")
            .await
            .or_config_error(
                "system_discovery",
                "No system capability provider found in ecosystem",
            )?;

        info!("✅ Found system provider: {}", system_provider.data);

        // Route to discovered system provider
        self.send_capability_request("system", operation, payload)
            .await
    }

    /// Route configuration request via capability discovery (no hardcoded primal names)
    pub async fn route_configuration_request(&self) -> SongbirdResult<Value> {
        debug!(
            "⚙️ Routing configuration request via capability discovery: {}",
            operation
        );

        // Discover configuration/orchestration capability providers dynamically
        let config_provider = self
            .capability_adapter
            .get_best_primal_for_capability("configuration")
            .await
            .or_config_error(
                "configuration_discovery",
                "No configuration capability provider found in ecosystem",
            )?;

        info!("✅ Found configuration provider: {}", config_provider.data);

        // Route to discovered configuration provider
        self.send_capability_request("configuration", operation, payload)
            .await
    }

    /// Universal capability request routing - works with any capability
    pub async fn route_capability_request(&self) -> SongbirdResult<Value> {
        debug!(
            "🎯 Routing {} capability request: {}",
            capability, operation
        );

        // Discover capability providers dynamically (no hardcoded names)
        let provider = self
            .capability_adapter
            .get_best_primal_for_capability(capability)
            .await
            .or_config_error(
                "capability_discovery",
                &format!("No {capability} capability provider found in ecosystem"),
            )?;

        info!("✅ Found {} provider: {}", capability, provider.data);

        // Route to discovered provider
        self.send_capability_request(capability, operation, payload)
            .await
    }

    /// Send request to capability provider (implementation details)
    async fn send_capability_request(&self) -> SongbirdResult<Value> {
        // This would use the Universal Adapter's routing mechanism
        // For now, return a success response that shows the routing pattern
        let response = serde_json::json!({
            "success": true,
            "capability": capability,
            "operation": operation,
            "routed_via": "universal_adapter",
            "discovery_method": "capability_based",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "payload_received": !payload.is_null()
        });

        Ok(songbird_errors::evolved_success(success(response)))
    }

    /// Discover all available capabilities in the ecosystem
    pub async fn discover_all_capabilities(&self) -> SongbirdResult<()> {debug!("🔍 Discovering all available capabilities in ecosystem");

        // Use capability adapter to discover what's available
        let registry = &self.capability_adapter.capability_registry.read().await;
        let capabilities: Vec<String> = registry.capability_providers.keys().cloned().collect();

        info!(
            "✅ Discovered {} capabilities: {:?}",
            capabilities.len(),
            capabilities
        );
        Ok(songbird_errors::evolved_success(success(capabilities)))
    }

    /// Get health status of all capability providers
    pub async fn get_ecosystem_health(&self) -> SongbirdResult<Value> {
        debug!("🏥 Getting ecosystem health status");

        let capabilities = self.discover_all_capabilities().await?;
        let mut health_status = serde_json::Map::new();

        for capability in capabilities.data {
            // Check if we can find a provider for this capability
            match self
                .capability_adapter
                .get_best_primal_for_capability(&capability)
                .await
            {
                Some(provider) => {
                    health_status.insert(
                        capability,
                        serde_json::json!({
                            "status": "healthy",
                            "provider": provider,
                            "available": true
                        }),
                    );
                }
                None => {
                    health_status.insert(
                        capability,
                        serde_json::json!({
                            "status": "unavailable",
                            "provider": null,
                            "available": false
                        }),
                    );
                }
            }
        }

        Ok(success(serde_json::Value::Object(health_status)))
    }
}

/// Global static instance for easy access
static GLOBAL_PRIMAL_INTEGRATION: std::sync::OnceLock<UniversalPrimalIntegration> =
    std::sync::OnceLock::new();

/// Initialize global primal integration
pub async fn init_global_primal_integration(
    capability_adapter: Arc<UniversalCapabilityAdapter>,
) -> SongbirdResult<()> {
    let integration = UniversalPrimalIntegration::new(capability_adapter);

    GLOBAL_PRIMAL_INTEGRATION.set(integration).map_err(|_| {
        songbird_errors::config_error!("Global primal integration already initialized")
    })?;

    info!("🌟 Global Universal Primal Integration initialized");
    Ok(songbird_errors::success(()))
}

/// Get global primal integration instance
pub fn get_global_primal_integration() -> &'static UniversalPrimalIntegration {
    try_get_global_primal_integration().unwrap_or_else(|| {
        tracing::error!(
            "Global primal integration not initialized - call init_global_primal_integration first"
        );
        // In production, this would return a default/fallback integration
        // For now, we maintain the same behavior but with better logging
        std::process::exit(1);
    })
}

/// Try to get global primal integration (returns None if not initialized)
pub fn try_get_global_primal_integration() -> Option<&'static UniversalPrimalIntegration> {
    GLOBAL_PRIMAL_INTEGRATION.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_universal_capability_routing() {
        let adapter = Arc::new(UniversalCapabilityAdapter::new());
        let integration = UniversalPrimalIntegration::new(adapter);

        // Test that we can route any capability without hardcoded names
        let test_payload = serde_json::json!({"test": "data"});

        // These should work for any capability, not just specific primal names
        let _result = integration
            .route_capability_request("security", "encrypt", test_payload.clone())
            .await;
        // Result should show capability-based routing, not hardcoded primal names

        let _result = integration
            .route_capability_request("storage", "store", test_payload.clone())
            .await;
        // Should discover storage provider dynamically

        let _result = integration
            .route_capability_request("ai", "process", test_payload)
            .await;
        // Should discover AI provider dynamically
    }

    #[tokio::test]
    async fn test_ecosystem_discovery() {
        let adapter = Arc::new(UniversalCapabilityAdapter::new());
        let integration = UniversalPrimalIntegration::new(adapter);

        // Test discovering all capabilities without knowing primal names
        let _capabilities = integration.discover_all_capabilities().await;
        // Should return list of capabilities, not primal names

        let _health = integration.get_ecosystem_health().await;
        // Should show health by capability, not by primal name
    }
}
