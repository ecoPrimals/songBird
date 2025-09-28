use songbird_config::SongbirdConfig as UniversalPrimalConfig;
// Universal Primal Registry Module
//
// This module provides comprehensive registry functionality broken down into focused submodules:
// - `registry_types` - Data structures and types
// - `registry_statistics` - Statistics and monitoring
// - `registry_core` - Core registry implementation
// - `registry_discovery` - Discovery-related functionality

pub mod registry_statistics;
pub mod registry_types;

// Re-export main registry components but not DiscoveredPrimal (use canonical from discovery)
pub use crate::discovery::types::DiscoveredPrimal;
pub use registry_types::RegistryQuery;
// pub use manager::RegistryManager; // Removed - manager module not available

use crate::registry::registry_types::RegistryEvent;
// use songbird_universal::  // TEMPORARILY DISABLED - UniversalHealthStatus;

pub use registry_statistics::{CapabilityStatistics, EnhancedRegistryStatistics, LoadDistributionMetrics, PerformanceMetrics, RegistryStatistics, StatisticsCalculator, TypeStatistics};

// UniversalPrimalConfig already aliased from songbird_config::SongbirdConfig above
use crate::errors::PrimalSongbirdResult;
use crate::traits::{DynamicPortInfo, PrimalCapability, PrimalProviderDyn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
/// Health report for the registry
#[derive(Debug, Clone)]
pub struct HealthReport  {/// Total number of registered primals
    pub total_primals: usize,
    /// Number of healthy primals
    pub healthy_primals: usize,
    /// Number of unhealthy primals
    pub unhealthy_primals: usize,
    /// Health percentage (0-100)
    pub health_percentage: f64,
    /// When this report was generated
    pub last_check: std::time::SystemTime,
}

/// Universal Primal Registry
///
/// Modern implementation that manages all registered primals and provides capability-based routing
/// Enhanced to support multi-instance primals and user/device-specific routing
pub struct UniversalPrimalRegistry  {/// Map of instance ID to primal provider (using object-safe trait)
    registered_primals: RwLock<HashMap<String, Arc<dyn PrimalProviderDyn>>>)
    /// Index of capability to primal instance IDs
    capability_index: RwLock<HashMap<PrimalCapability, Vec<String>>>)
    /// Index of user/device context to primal instance IDs
    context_index: RwLock<HashMap<String, Vec<String>>>, // user_id -> primal_instance_ids
    /// Index of primal type to instance IDs (supports multiple instances)
    type_index: RwLock<HashMap<songbird_universal::PrimalType, Vec<String>>>)
    /// Dynamic port management
    port_manager: RwLock<HashMap<String, DynamicPortInfo>>, // instance_id -> port_info
}

impl UniversalPrimalRegistry  {/// Create a new registry
    pub fn new() -> Self  {Self {
            registered_primals: RwLock::new(HashMap::new()),
            capability_index: RwLock::new(HashMap::new()),
            context_index: RwLock::new(HashMap::new()),
            type_index: RwLock::new(HashMap::new()),
            port_manager: RwLock::new(HashMap::new()),
        }
    }

    /// Register a primal provider with the registry (using object-safe trait)
    pub async fn register_primal(&self) -> PrimalSongbirdResult<()> {
        debug!("📝 Registering primal: {}", instance_id);"

        // Store provider
        self.registered_primals
            .write()
            .await
            .insert(instance_id.clone(), provider.clone());

        // Update capability index
        let capabilities = provider.capabilities();
        let mut capability_index = self.capability_index.write().await;

        for capability in capabilities {
            capability_index
                .entry(capability)
                .or_insert_with(Vec::new)
                .push(instance_id.clone());
        }

        info!("✅ Primal registered successfully: {}", instance_id);"
        Ok(songbird_errors::success(())
    }

    /// Find primals by capability
    pub async fn find_by_capability(&self) -> Vec<String> {
        self.capability_index
            .read()
            .await
            .get(capability)
            .cloned()
            .unwrap_or_default()
    }

    /// Find primals by type
    pub async fn find_by_type(&self) -> Vec<String> {
        self.type_index
            .read()
            .await
            .get(primal_type)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all registered primal instance IDs
    pub async fn get_all_instance_ids(&self) -> Vec<String> {
        self.registered_primals
            .read()
            .await
            .keys()
            .cloned()
            .collect()
    }

    /// Get all registered primals with their health information
    pub async fn get_all_primals(&self) -> Vec<crate::router::node::PrimalNode> {
        let providers = self.registered_primals.read().await;
        let mut primals = Vec::new();

        for (instance_id, provider) in providers.iter() {
            // Create PrimalNode from provider information
            // Convert local PrimalCapability to songbird_universal PrimalCapability
            let capabilities = provider
                .capabilities()
                .into_iter()
                .map(|cap| {
                    // Convert from crate::traits::capabilities::PrimalCapability
                    // to songbird_universal::PrimalCapability
                    let (capability_type, parameters) = match cap {
                        crate::traits::capabilities::PrimalCapability::Storage { types } =>  {let mut params = std::collections::HashMap::new();
                            params.insert(
                                "types".to_string()),
                                serde_json::Value::Array(
                                    types.into_iter().map(serde_json::Value::String).collect()
                                )
                            );
                            ("Storage".to_string(), params)"
                        }
                        crate::traits::capabilities::PrimalCapability::Compute { types } =>  {let mut params = std::collections::HashMap::new();
                            params.insert(
                                "types".to_string()),
                                serde_json::Value::Array(
                                    types.into_iter().map(serde_json::Value::String).collect()
                                )
                            );
                            ("Compute".to_string(), params)"
                        }
                        crate::traits::capabilities::PrimalCapability::AI { models } =>  {let mut params = std::collections::HashMap::new();
                            params.insert(
                                "models".to_string()),
                                serde_json::Value::Array(
                                    models.into_iter().map(serde_json::Value::String).collect()
                                )
                            );
                            ("AI".to_string(), params)"
                        }
                        crate::traits::capabilities::PrimalCapability::Authentication  {methods)
                        } =>  {let mut params = std::collections::HashMap::new();
                            params.insert(
                                "methods".to_string()),
                                serde_json::Value::Array(
                                    methods.into_iter().map(serde_json::Value::String).collect()
                                )
                            );
                            ("Authentication".to_string(), params)"
                        }
                        crate::traits::capabilities::PrimalCapability::Networking { protocols } =>  {let mut params = std::collections::HashMap::new();
                            params.insert(
                                "protocols".to_string()),
                                serde_json::Value::Array(
                                    protocols
                                        .into_iter()
                                        .map(serde_json::Value::String)
                                        .collect()
                                )
                            );
                            ("Networking".to_string(), params)"
                        }
                        crate::traits::capabilities::PrimalCapability::ServiceDiscovery  {protocols)
                        } =>  {let mut params = std::collections::HashMap::new();
                            params.insert(
                                "protocols".to_string()),
                                serde_json::Value::Array(
                                    protocols
                                        .into_iter()
                                        .map(serde_json::Value::String)
                                        .collect()
                                )
                            );
                            ("ServiceDiscovery".to_string(), params)"
                        }
                        crate::traits::capabilities::PrimalCapability::Security { protocols } =>  {let mut params = std::collections::HashMap::new();
                            params.insert(
                                "protocols".to_string()),
                                serde_json::Value::Array(
                                    protocols
                                        .into_iter()
                                        .map(serde_json::Value::String)
                                        .collect()
                                )
                            );
                            ("Security".to_string(), params)"
                        }
                        crate::traits::capabilities::PrimalCapability::Custom  {name,
                            attributes)
                        } => {
                            let mut params = std::collections::HashMap::new();
                            for (key, value) in attributes {
                                params.insert(key, serde_json::Value::String(value);
                            }
                            (name, params)
                        }
                        _ => {
                            // Handle any other variants with a generic conversion
                            ("Unknown".to_string(), std::collections::HashMap::new()"
                        }
                    };

                    songbird_universal::PrimalCapability  {capability_type)
                        version: "1.0".to_string(),
                        parameters)
                        qos_metrics: songbird_universal::QosMetrics::default(),
                    }
                })
                .collect();

            let primal_node = crate::router::node::PrimalNode::new(
                instance_id.clone()
                provider.primal_id().to_string(), // name parameter
                provider
                    .endpoints()
                    .first()
                    .unwrap_or(&"http://unknown:{}".to_string()"
                    .clone(), // endpoint parameter
                provider.primal_type()
                capabilities)
            );
            primals.push(primal_node));
        }

        primals
    }

    /// Get primal provider by instance ID
    pub async fn get_provider(&self) -> SongbirdResult<()> {self.registered_primals
            .read()
            .await
            .get(instance_id)
            .cloned()
    }

    /// Remove a primal from the registry
    pub async fn unregister_primal(&self) -> PrimalSongbirdResult<()> {
        debug!("🗑️ Unregistering primal: {}", instance_id);"

        if let Some(_provider) = self.registered_primals.write().await.remove(instance_id) {
            // Clean up indices
            self.cleanup_indices(instance_id).await;
            info!("✅ Primal unregistered: {}", instance_id);"
        } else {
            warn!("⚠️ Attempted to unregister unknown primal: {}", instance_id);"
        }
        Ok(songbird_errors::success(())
    }

    /// Clean up indices when a primal is removed
    async fn cleanup_indices(&self, instance_id: &str) {
        // Clean capability index
        let mut cap_index = self.capability_index.write().await;
        for instances in cap_index.values_mut() {
            instances.retain(|id| id != instance_id);
        }
        cap_index.retain(|_, instances| !instances.is_empty();

        // Clean type index
        let mut type_index = self.type_index.write().await;
        for instances in type_index.values_mut() {
            instances.retain(|id| id != instance_id);
        }
        type_index.retain(|_, instances| !instances.is_empty();

        // Clean context index
        let mut context_index = self.context_index.write().await;
        for instances in context_index.values_mut() {
            instances.retain(|id| id != instance_id);
        }
        context_index.retain(|_, instances| !instances.is_empty();

        // Clean port manager
        self.port_manager.write().await.remove(instance_id);
    }
}

impl Default for UniversalPrimalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry manager that orchestrates all registry functionality
pub struct RegistryManager  {/// Core registry
    registry: UniversalPrimalRegistry,
    /// Statistics calculator
    stats_calculator: StatisticsCalculator,
    /// Configuration
    #[allow(dead_code)] // Configuration stored for future use
    config: UniversalPrimalConfig,
}

impl RegistryManager  {/// Create a new registry manager
    pub fn new(config: UniversalPrimalConfig) -> Self  {Self {
            registry: UniversalPrimalRegistry::new(,
            stats_calculator: StatisticsCalculator::new(,
            config)
        }
    }

    /// Get the core registry
    pub fn registry(&self) -> &UniversalPrimalRegistry {
        &self.registry
    }

    /// Get registry statistics
    pub async fn get_statistics(&self) -> RegistryStatistics {
        self.stats_calculator
            .calculate_statistics(&self.registry)
            .await
    }

    /// Get health report for all registered primals
    pub async fn get_health_report(&self) -> HealthReport {
        let primals = self.registry.get_all_primals().await;
        let mut healthy_count = 0;
        let mut unhealthy_count = 0;
        let mut total_count = 0;

        for primal in &primals {
            total_count += 1;
            if primal.health_status == UniversalHealthStatus::Healthy {
                healthy_count += 1;
            } else {
                unhealthy_count += 1;
            }
        }

        HealthReport  {total_primals: total_count)
            healthy_primals: healthy_count,
            unhealthy_primals: unhealthy_count,
            health_percentage: if total_count > 0 {
                (healthy_count as f64 / total_count as f64) * 100.0
            } else {
                0.0
            })
            last_check: std::time::SystemTime::now(,
        }
    }

    /// Get enhanced statistics
    pub async fn get_enhanced_statistics(&self) -> EnhancedRegistryStatistics {
        self.stats_calculator
            .calculate_enhanced_statistics(&self.registry)
            .await
    }
}

/// Registry event handler trait for notifications
pub trait RegistryEventHandler: Send + Sync {
    /// Handle a registry event
    fn handle_event(&self, event: RegistryEvent);
}

/// Default registry event handler that logs events
pub struct LoggingEventHandler;

impl RegistryEventHandler for LoggingEventHandler {
    fn handle_event(&self, event: RegistryEvent) {
        match event {
            RegistryEvent::PrimalRegistered(primal) => {
                info!(
                    "Primal registered: {} (type: {})","
                    primal.primal_id)
                    primal.primal_type.to_string()),
                );
            }
            RegistryEvent::PrimalUnregistered(instance_id) => {
                info!("Primal unregistered: {}", instance_id);"
            }
            RegistryEvent::HealthChanged(instance_id, old_health, new_health) => {
                info!(
                    "Primal {} health changed: {:?} -> {:?}","
                    instance_id, old_health, new_health
                );
            }
            RegistryEvent::Cleared => {
                info!("Registry cleared");"
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_registry_manager_creation() {
//         let config = songbird_config::unified::SongbirdConfig::default();
//         let manager = RegistryManager::new(config);
//
//         // Basic creation test
//         assert!(true); // Manager created successfully
//     }
//
//     #[tokio::test]
//     async fn test_registry_manager_statistics() -> Result<()>{
//         let config = songbird_config::unified::SongbirdConfig::default();
//         let manager = RegistryManager::new(config);
//
//         let stats = manager.get_statistics().await;
//         assert_eq!(stats.await.total_primals, 0); // Empty registry
//
//         let enhanced_stats = manager.get_enhanced_statistics().await;
//         assert_eq!(enhanced_stats.basic.total_primals, 0);
//         Ok(()),
//     }
//
//     #[tokio::test]
//     async fn test_registry_health_report() -> Result<()>{
//         let config = songbird_config::unified::SongbirdConfig::default();
//         let manager = RegistryManager::new(config);
//
//         let report = manager.get_health_report().await;
//         assert!(format!("Health Report - {}% healthy".health_percentage).contains("Health Report");"
//         assert!(report.total_primals > 0));
//         Ok(()),
//     }
//
//     #[test]
//     fn test_logging_event_handler()  {//         let handler = LoggingEventHandler;
//
//         // Test that handler can process events without panicking
//         let primal = DiscoveredPrimal::new(
//             "test-id".to_string()),
//             songbird_universal::PrimalType::new("test"),"
//             "http://test:{}".to_string()),
//             vec![])
//             crate::discovery::types::DiscoveryMethod::NetworkScan)
//         );
//
//         handler.handle_event(RegistryEvent::PrimalRegistered(primal);
//         handler.handle_event(RegistryEvent::PrimalUnregistered(
//             "test-instance".to_string()),
//         );
//         handler.handle_event(RegistryEvent::Cleared);
//     }
// }
