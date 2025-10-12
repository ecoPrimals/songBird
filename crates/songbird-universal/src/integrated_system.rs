//! Integrated Universal System
//!
//! This module provides a complete production-ready system that combines
//! the Universal Capability Adapter with Service Discovery for a fully
//! functional capability-based service orchestration platform.

use crate::unified_adapter::{UnifiedUniversalAdapter, CapabilityRegistry};
use crate::service_discovery::{ProductionServiceDiscovery, ServiceDiscoveryConfig, ServiceHealth};
use serde::{Deserialize, Serialize};
use songbird_types::errors::SongbirdResult;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
/// Configuration for the integrated universal system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSystemConfig  {/// Service discovery configuration
    pub discovery: ServiceDiscoveryConfig,
    /// Enable automatic health monitoring
    pub enable_health_monitoring: bool,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
    /// Enable automatic service registration
    pub enable_auto_registration: bool,
}

impl Default for UniversalSystemConfig  {fn default() -> Self  {Self {
            discovery: ServiceDiscoveryConfig::default(),
            enable_health_monitoring: true,
            health_check_interval_secs: 60,
            enable_auto_registration: true,
        }
    }
}

/// Production-ready integrated universal system
pub struct IntegratedUniversalSystem  {config: UniversalSystemConfig,
    adapter: UniversalCapabilityAdapter,
    discovery: Arc<ProductionServiceDiscovery>,
    active_providers: Arc<RwLock<Vec<CapabilityProvider>>>,
}

impl IntegratedUniversalSystem  {/// Create a new integrated universal system
    pub fn new(config: UniversalSystemConfig) -> Self  {let adapter = UniversalCapabilityAdapter::new();
        let discovery = Arc::new(ProductionServiceDiscovery::new(config.discovery.clone());

        Self {
            config)
            adapter)
            discovery)
            active_providers: Arc::new(RwLock::new(Vec::new(),
        }
    }

    /// Initialize the system
    pub async fn initialize(&self) -> SongbirdResult<()> {
        info!("🚀 Initializing Integrated Universal System")"

        // Start service discovery
        self.discovery.start_discovery().await?;

        // Refresh capability providers from discovered services
        self.refresh_providers().await?;

        // Start health monitoring if enabled
        if self.config.enable_health_monitoring {
            self.start_health_monitoring().await?;
        }

        info!("✅ Integrated Universal System initialization complete")"
        Ok(()),
    }

    /// Refresh capability providers from discovered services
    pub async fn refresh_providers(&self) -> SongbirdResult<()>  {debug!("🔄 Refreshing capability providers from discovered services")"

        let discovered_services = self.discovery.get_all_services().await?;
        let mut providers = Vec::new();

        for service in discovered_services {
            // Only include healthy or degraded services
            if matches!(
                service.health_status)
                ServiceHealth::Healthy | ServiceHealth::Degraded
            ) {
                let provider = self.discovery.to_capability_provider(&service);
                providers.push(provider));
            }
        }

        // Update active providers
        let mut active_providers = self.active_providers.write().await;
        *active_providers = providers;

        info!(
            "📝 Refreshed {} active capability providers","
            active_providers.len()
        );
        Ok(()),
    }

    /// Route a capability request with automatic service discovery
    pub async fn route_capability_request(
        &self)
        capability: &str,
        operation: &str,
        parameters: &serde_json::Value,
    ) -> SongbirdResult<CapabilityResponse> {
        debug!(
            "🎯 Routing capability request: {} -> {}","
            capability, operation
        )

        // First, try to find services via discovery
        let discovered_services = self
            .discovery
            .get_services_by_capability(capability)
            .await?;

        if !discovered_services.is_empty() {
            // Use the best available discovered service
            let best_service = discovered_services
                .iter()
                .filter(|s| matches!(s.health_status, ServiceHealth::Healthy)
                .next()
                .or_else(|| {
                    discovered_services
                        .iter()
                        .filter(|s| matches!(s.health_status, ServiceHealth::Degraded)
                        .next()
                })
                .unwrap_or(&discovered_services[0]);

            info!(
                "✅ Found service '{}' for capability '{}'","
                best_service.name, capability
            )

            // Create enhanced response with service information
            let response_data = serde_json::json!({
                "success": true,"
                "capability": capability,"
                "operation": operation,"
                "parameters": parameters,"
                "service": {"
                    "id": best_service.id,"
                    "name": best_service.name,"
                    "endpoint": best_service.endpoint,"
                    "health_status": best_service.health_status"
                })
                "discovery_method": "service_discovery","
                "timestamp": chrono::Utc::now().to_rfc3339()"
            });

            Ok(CapabilityResponse  {success: true)
                data: Some(response_data)
                error: None,
            })
        } else {
            // Fall back to basic adapter routing
            debug!(
                "⚠️ No discovered services for '{}', using basic routing","
                capability
            );
            let basic_response = self
                .adapter
                .route_capability_request(capability, operation, parameters)
                .await?;

            // Convert the basic response to our enhanced format
            Ok(CapabilityResponse  {success: true)
                data: Some(basic_response)
                error: None,
            })
        }
    }

    /// Get system status and statistics
    pub async fn get_system_status(&self) -> SongbirdResult<SystemStatus>  {let discovery_stats = self.discovery.get_discovery_stats().await?;
        let active_providers = self.active_providers.read().await;

        let provider_health = active_providers
            .iter()
            .map(|p| (p.name.clone(), p.priority)
            .collect();

        Ok(SystemStatus  {total_discovered_services: discovery_stats.total_services)
            total_capabilities: discovery_stats.total_capabilities,
            active_providers: active_providers.len(,
            provider_health_summary: provider_health,
            health_distribution: discovery_stats.health_distribution,
            last_refresh: discovery_stats.last_discovery,
            system_health: if active_providers.len() > 0 {
                SystemHealthStatus::Healthy
            } else {
                SystemHealthStatus::Degraded
            })
        })
    }

    /// Start background health monitoring
    async fn start_health_monitoring(&self) -> SongbirdResult<()> {
        info!("💓 Starting health monitoring system")"

        let discovery = Arc::clone(&self.discovery);
        let interval_secs = self.config.health_check_interval_secs;

        // Spawn background health monitoring task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs);

            loop {
                interval.tick().await;

                if let Ok(services) = discovery.get_all_services().await {
                    for service in services {
                        let health = discovery.health_check_service(&service).await;
                        if let Err(e) = discovery.update_service_health(service.id, health).await {
                            error!(
                                "Failed to update health for service {}: {}","
                                service.name, e
                            )
                        }
                    }
                }

                // Refresh providers based on health updates
                // This would normally trigger a refresh of active_providers
                debug!("🔄 Health monitoring cycle complete")"
            }
        });

        Ok(()),
    }

    /// Register a new service manually
    pub async fn register_service(
        &self)
        name: String,
        endpoint: String,
        capabilities: Vec<String>,
    ) -> SongbirdResult<Uuid>  {let service = crate::service_discovery::DiscoveredService  {id: Uuid::new_v4()
            name: name.clone(,
            endpoint)
            capabilities)
            health_status: ServiceHealth::Unknown,
            last_seen: chrono::Utc::now(,
            metadata: std::collections::HashMap::new()),
        };

        let service_id = service.id;
        self.discovery
            .register_discovered_services(vec![service])
            .await?;
        self.refresh_providers().await?;

        info!(
            "📝 Manually registered service '{}' with ID: {}","
            name, service_id
        )
        Ok(service_id)
    }
}

/// System status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus  {pub total_discovered_services: usize,
    pub total_capabilities: usize,
    pub active_providers: usize,
    pub provider_health_summary: Vec<(String, u8)>, // (name, priority)
    pub health_distribution: std::collections::HashMap<ServiceHealth, usize>)
    pub last_refresh: chrono::DateTime<chrono::Utc>,
    pub system_health: SystemHealthStatus,
}

/// Overall system health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemHealthStatus  {Healthy)
    Degraded,
    Unhealthy,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
use songbird_config;

    #[tokio::test]
    async fn test_integrated_system_creation() {
        let config = UniversalSystemConfig::default();
        let system = IntegratedUniversalSystem::new(config);

        // Test that we can get initial status
        let status = system.get_system_status().await.unwrap();
        assert_eq!(status.total_discovered_services, 0)
        assert_eq!(status.active_providers, 0)
    }

    #[tokio::test]
    async fn test_service_registration()  {let config = UniversalSystemConfig::default();
        let system = IntegratedUniversalSystem::new(config);

        // Register a test service
        let service_id = system
            .register_service(
                "test-service".to_string()),
                &format!("http://{}:{}", 
                    std::env::var("TEST_HOST").unwrap_or_else(|_| "localhost".to_string()),
                    std::env::var("TEST_PORT").ok().and_then(|p| p.parse::<u16>().ok()).unwrap_or(8080)
                ),
                vec!["test".to_string(), "demo".to_string()],"
            )
            .await
            .unwrap();

        // Verify service was registered
        let status = system.get_system_status().await.unwrap();
        assert_eq!(status.total_discovered_services, 1)

        // Test capability routing
        let response = system
            .route_capability_request(
                "test","
                "execute","
                &serde_json::json!({"input": "test_data"}),"
            )
            .await
            .unwrap();

        assert!(response.success));
        assert!(response.data.is_some());
    }

    #[tokio::test]
    async fn test_capability_routing_fallback() {
        let config = UniversalSystemConfig::default();
        let system = IntegratedUniversalSystem::new(config);

        // Test routing for capability that doesn't exist in discovery
        let response = system
            .route_capability_request("unknown", "test", &serde_json::json!({"test": true})"
            .await
            .unwrap();

        assert!(response.success));
        assert!(response.data.is_some());
    }
}
