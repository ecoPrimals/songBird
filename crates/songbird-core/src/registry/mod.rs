//! Service Registry Module
//!
//! Service registration and management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use async_trait::async_trait;
use songbird_discovery::traits::service::{ServiceInfo, UniversalService};
use songbird_discovery::traits::{
    ComposablePlugin, ComposedSystem, CompositionPlan, PerformanceEstimate, PluginCapability,
    PluginHealth, PluginRegistry, PluginRequirement, SystemHealth,
};
use songbird_errors::{Result, SongbirdError};
use tokio::sync::broadcast;

/// Service handle for managing a registered service
pub struct ServiceHandle<S: UniversalService> {
    pub service: Arc<RwLock<S>>,
    pub info: ServiceInfo,
}

impl<S: UniversalService> ServiceHandle<S> {
    pub fn new(service: S, info: ServiceInfo) -> Self {
        Self {
            service: Arc::new(RwLock::new(service)),
            info,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut service = self.service.write().await;
        service
            .start()
            .await
            .map_err(|e| SongbirdError::service_error(&self.info.service_id, e.to_string()))?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut service = self.service.write().await;
        service
            .stop()
            .await
            .map_err(|e| SongbirdError::service_error(&self.info.service_id, e.to_string()))?;
        Ok(())
    }

    pub async fn health_check(&self) -> Result<serde_json::Value> {
        let service = self.service.read().await;
        let health = service.health_check().await.map_err(|e| {
            SongbirdError::health_check_failed(&self.info.service_id, e.to_string())
        })?;
        serde_json::to_value(health)
            .map_err(|e| SongbirdError::service_error(&self.info.service_id, e.to_string()))
    }
}

/// Central service registry
pub struct ServiceRegistry {
    // Type alias to simplify the complex trait bound
    services: Arc<RwLock<HashMap<String, Box<dyn UniversalService<Error = SongbirdError>>>>>,
    service_info: Arc<RwLock<HashMap<String, ServiceInfo>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            service_info: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_service<S>(
        &self,
        service: S,
        info: ServiceInfo,
    ) -> Result<ServiceHandle<S>>
    where
        S: UniversalService<Error = SongbirdError> + 'static,
    {
        let service_id = info.service_id.clone();
        let handle = ServiceHandle::new(service, info.clone());

        // We need to box the service to store it in the registry
        // This is a design limitation we might need to refactor
        self.service_info
            .write()
            .await
            .insert(service_id, info);

        Ok(handle)
    }

    pub async fn deregister_service(&self, service_id: &str) -> Result<()> {
        let mut services = self.services.write().await;
        let mut service_info = self.service_info.write().await;

        services.remove(service_id);
        service_info.remove(service_id);

        Ok(())
    }

    pub async fn get_service_info(&self, service_id: &str) -> Option<ServiceInfo> {
        self.service_info.read().await.get(service_id).cloned()
    }

    pub async fn list_services(&self) -> Vec<ServiceInfo> {
        self.service_info.read().await.values().cloned().collect()
    }

    pub async fn get_service_count(&self) -> usize {
        self.services.read().await.len()
    }

    pub async fn get_service_handle(
        &self,
        service_id: &str,
    ) -> Result<Option<Arc<RwLock<dyn UniversalService<Error = songbird_errors::SongbirdError>>>>>
    {
        if let Some(_service) = self.services.read().await.get(service_id) {
            // This is a bit tricky due to the Box<dyn> - we'd need to refactor this
            // For now, return None as this requires more complex handling
            Ok(None)
        } else {
            Ok(None)
        }
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            service_info: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Dynamic Plugin Registry Implementation
///
/// This registry allows services to be discovered and composed at runtime
/// without requiring pre-configured TOML files for every possible combination.
///
/// Perfect for scenarios like:
/// - 8 different projects that need to work together
/// - Songbird + BearDog + Toadstool combinations
/// - Dynamic service chaining (toadstool on toadstool)
pub struct DynamicPluginRegistry {
    plugins: Arc<RwLock<HashMap<String, Box<dyn ComposablePlugin>>>>,
    active_compositions: Arc<RwLock<HashMap<String, ComposedSystem>>>,
    capability_index: Arc<RwLock<HashMap<PluginCapability, Vec<String>>>>,
    requirement_graph: Arc<RwLock<HashMap<String, Vec<String>>>>,
    event_sender: broadcast::Sender<PluginEvent>,
}

/// Plugin events for monitoring
#[derive(Debug, Clone)]
pub enum PluginEvent {
    PluginRegistered {
        plugin_id: String,
        capabilities: Vec<PluginCapability>,
    },
    PluginIntegrated {
        plugin_a: String,
        plugin_b: String,
        integration_id: String,
    },
    CompositionCreated {
        system_id: String,
        plugins: Vec<String>,
    },
    CompositionFailed {
        error: String,
        attempted_plugins: Vec<String>,
    },
    PluginHealthChanged {
        plugin_id: String,
        healthy: bool,
    },
}

impl Default for DynamicPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicPluginRegistry {
    /// Create a new dynamic plugin registry
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);

        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            active_compositions: Arc::new(RwLock::new(HashMap::new())),
            capability_index: Arc::new(RwLock::new(HashMap::new())),
            requirement_graph: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    /// Subscribe to plugin events
    pub fn subscribe_events(&self) -> broadcast::Receiver<PluginEvent> {
        self.event_sender.subscribe()
    }

    /// Get all registered plugins
    pub async fn list_plugins(&self) -> Vec<String> {
        self.plugins.read().await.keys().cloned().collect()
    }

    /// Get plugin capabilities
    pub async fn get_plugin_capabilities(&self, plugin_id: &str) -> Result<Vec<PluginCapability>> {
        let plugins = self.plugins.read().await;
        if let Some(plugin) = plugins.get(plugin_id) {
            Ok(plugin.capabilities())
        } else {
            Err(SongbirdError::PluginNotFound(plugin_id.to_string()))
        }
    }

    /// Auto-discover optimal plugin composition for a task
    pub async fn discover_optimal_composition(
        &self,
        task_description: &str,
        required_capabilities: Vec<PluginCapability>,
        constraints: CompositionConstraints,
    ) -> Result<Vec<CompositionPlan>> {
        tracing::info!(
            task = task_description,
            required_capabilities = ?required_capabilities,
            "Discovering optimal plugin composition"
        );

        let plugins = self.plugins.read().await;
        let mut candidate_plans = Vec::new();

        // Find all possible combinations that satisfy requirements
        let available_plugins: Vec<_> = plugins.keys().cloned().collect();
        drop(plugins); // Release the lock

        for combination in self
            .generate_combinations(&available_plugins, &required_capabilities)
            .await?
        {
            if let Ok(plan) = self
                .create_composition_plan(combination, &constraints)
                .await
            {
                candidate_plans.push(plan);
            }
        }

        // Sort by performance estimate (best first)
        candidate_plans.sort_by(|a, b| {
            a.estimated_performance
                .latency_ms
                .partial_cmp(&b.estimated_performance.latency_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(candidate_plans)
    }

    /// Generate plugin combinations that could satisfy requirements
    async fn generate_combinations(
        &self,
        available_plugins: &[String],
        required_capabilities: &[PluginCapability],
    ) -> Result<Vec<Vec<String>>> {
        let mut combinations = Vec::new();

        // Start with single plugins
        let plugins = self.plugins.read().await;
        for plugin_id in available_plugins {
            if let Some(plugin) = plugins.get(plugin_id) {
                let capabilities = plugin.capabilities();
                if self.capabilities_satisfy_requirements(&capabilities, required_capabilities) {
                    combinations.push(vec![plugin_id.clone()]);
                }
            }
        }

        // Try pairs, triples, etc. until we have enough combinations
        for size in 2..=std::cmp::min(available_plugins.len(), 5) {
            for combo in self.combinations(available_plugins, size) {
                let combined_capabilities = self.get_combined_capabilities(&combo).await?;
                if self.capabilities_satisfy_requirements(
                    &combined_capabilities,
                    required_capabilities,
                ) {
                    combinations.push(combo);
                }
            }

            // Limit search to prevent exponential explosion
            if combinations.len() > 50 {
                break;
            }
        }

        Ok(combinations)
    }

    /// Check if capabilities satisfy requirements
    fn capabilities_satisfy_requirements(
        &self,
        capabilities: &[PluginCapability],
        requirements: &[PluginCapability],
    ) -> bool {
        for requirement in requirements {
            if !capabilities.contains(requirement) {
                return false;
            }
        }
        true
    }

    /// Get combined capabilities for a set of plugins
    async fn get_combined_capabilities(
        &self,
        plugin_ids: &[String],
    ) -> Result<Vec<PluginCapability>> {
        let plugins = self.plugins.read().await;
        let mut combined = Vec::new();

        for plugin_id in plugin_ids {
            if let Some(plugin) = plugins.get(plugin_id) {
                combined.extend(plugin.capabilities());
            }
        }

        // Remove duplicates and merge compatible capabilities
        combined.sort();
        combined.dedup();

        Ok(combined)
    }

    /// Convert requirement to capability for matching
    fn requirement_to_capability(&self, requirement: &PluginRequirement) -> PluginCapability {
        match requirement {
            PluginRequirement::RequiresEncryption { min_key_size: _ } => {
                PluginCapability::Encryption {
                    algorithms: vec!["AES256".to_string()],
                }
            }
            PluginRequirement::RequiresServiceDiscovery => PluginCapability::ServiceDiscovery {
                protocols: vec!["HTTP".to_string()],
            },
            PluginRequirement::RequiresCompute {
                min_cpu_cores,
                min_memory_gb,
            } => PluginCapability::Compute {
                cpu_cores: *min_cpu_cores,
                memory_gb: *min_memory_gb,
            },
            PluginRequirement::RequiresNetwork {
                min_bandwidth_mbps,
                max_latency_ms,
            } => PluginCapability::Network {
                bandwidth_mbps: *min_bandwidth_mbps,
                latency_ms: *max_latency_ms,
            },
            PluginRequirement::Custom { name, constraints } => PluginCapability::Custom {
                name: name.clone(),
                attributes: constraints.clone(),
            },
        }
    }

    /// Integrate two plugins
    async fn integrate_plugins(&self, plugin_a: &str, plugin_b: &str) -> Result<()> {
        tracing::info!(
            plugin_a = %plugin_a,
            plugin_b = %plugin_b,
            "Integrating plugins"
        );

        // In a real implementation, this would perform actual plugin integration
        // For now, just send an event
        let integration_id = uuid::Uuid::new_v4().to_string();
        let _ = self.event_sender.send(PluginEvent::PluginIntegrated {
            plugin_a: plugin_a.to_string(),
            plugin_b: plugin_b.to_string(),
            integration_id,
        });

        Ok(())
    }

    /// Check system health for a composition
    async fn check_system_health(&self, plugin_ids: &[String]) -> Result<SystemHealth> {
        let mut plugin_health = std::collections::HashMap::new();
        let mut integration_health = std::collections::HashMap::new();
        let mut overall_healthy = true;

        for plugin_id in plugin_ids {
            // In a real implementation, this would check actual plugin health
            let health = PluginHealth {
                healthy: true,
                status_message: "OK".to_string(),
                last_check: chrono::Utc::now(),
                performance_metrics: std::collections::HashMap::new(),
            };

            if !health.healthy {
                overall_healthy = false;
            }

            plugin_health.insert(plugin_id.clone(), health);
        }

        // Check integration health between plugins
        for i in 0..plugin_ids.len() {
            for j in i + 1..plugin_ids.len() {
                let integration_key = format!("{}+{}", plugin_ids[i], plugin_ids[j]);
                integration_health.insert(integration_key, true);
            }
        }

        Ok(SystemHealth {
            overall_healthy,
            plugin_health,
            integration_health,
        })
    }

    /// Create composition plan for a combination of plugins
    async fn create_composition_plan(
        &self,
        combination: Vec<String>,
        _constraints: &CompositionConstraints,
    ) -> Result<CompositionPlan> {
        let mut integration_order = Vec::new();

        // Create integration pairs
        for i in 0..combination.len() {
            for j in i + 1..combination.len() {
                integration_order.push((combination[i].clone(), combination[j].clone()));
            }
        }

        let plan = CompositionPlan {
            plugins: combination,
            integration_order,
            shared_config: serde_json::json!({}),
            estimated_performance: PerformanceEstimate {
                latency_ms: 50.0,
                throughput_rps: 1000.0,
                memory_usage_mb: 256.0,
                cpu_utilization_percent: 30.0,
            },
        };

        Ok(plan)
    }

    /// Generate combinations of a specific size
    #[allow(clippy::only_used_in_recursion)]
    fn combinations(&self, items: &[String], size: usize) -> Vec<Vec<String>> {
        if size == 0 {
            return vec![vec![]];
        }
        if items.is_empty() {
            return vec![];
        }

        let mut result = Vec::new();
        let first = &items[0];
        let rest = &items[1..];

        // Combinations including the first item
        for mut combo in self.combinations(rest, size - 1) {
            combo.insert(0, first.clone());
            result.push(combo);
        }

        // Combinations not including the first item
        result.extend(self.combinations(rest, size));

        result
    }
}

/// Constraints for plugin composition
#[derive(Debug, Clone)]
pub struct CompositionConstraints {
    pub max_latency_ms: Option<f64>,
    pub max_memory_mb: Option<f64>,
    pub max_plugins: Option<usize>,
    pub required_performance: Option<PerformanceRequirements>,
    pub security_level: Option<String>,
}

/// Performance requirements
#[derive(Debug, Clone)]
pub struct PerformanceRequirements {
    pub min_throughput_rps: Option<f64>,
    pub max_response_time_ms: Option<f64>,
    pub min_availability_percent: Option<f64>,
}

impl Default for CompositionConstraints {
    fn default() -> Self {
        Self {
            max_latency_ms: Some(100.0),
            max_memory_mb: Some(1024.0),
            max_plugins: Some(10),
            required_performance: None,
            security_level: Some("standard".to_string()),
}
    }
}

/// Implementation of PluginRegistry trait for dynamic plugin management
#[async_trait]
impl PluginRegistry for DynamicPluginRegistry {
    async fn register_plugin(
        &self,
        plugin_id: String,
        capabilities: Vec<PluginCapability>,
        requirements: Vec<PluginRequirement>,
    ) -> Result<String> {
        // In a real implementation, this would register the plugin
        // For now, just return success
        Ok(plugin_id)
    }

    async fn discover_plugins(&self, requirements: Vec<PluginRequirement>) -> Result<Vec<String>> {
        let index = self.capability_index.read().await;
        let mut matching_plugins = std::collections::HashSet::new();

        // For each requirement, find plugins that can satisfy it
        for requirement in requirements {
            let compatible_capability = self.requirement_to_capability(&requirement);
            if let Some(plugin_ids) = index.get(&compatible_capability) {
                for plugin_id in plugin_ids {
                    matching_plugins.insert(plugin_id.clone());
                }
            }
        }

        Ok(matching_plugins.into_iter().collect())
    }

    async fn auto_compose(
        &self,
        target_capabilities: Vec<PluginCapability>,
    ) -> Result<CompositionPlan> {
        let constraints = CompositionConstraints {
            max_latency_ms: Some(100.0), // Default: 100ms max latency
            max_memory_mb: Some(1024.0), // Default: 1GB max memory
            max_plugins: Some(10),       // Default: max 10 plugins
            required_performance: None,
            security_level: None,
        };

        let plans = self
            .discover_optimal_composition(
                "Auto-composition request",
                target_capabilities,
                constraints,
            )
            .await?;

        plans.into_iter().next().ok_or_else(|| {
            SongbirdError::CompositionFailed("No valid composition found".to_string())
        })
    }

    async fn execute_composition(&self, plan: CompositionPlan) -> Result<ComposedSystem> {
        let system_id = uuid::Uuid::new_v4().to_string();

        tracing::info!(
            system_id = %system_id,
            plugins = ?plan.plugins,
            "Executing plugin composition"
        );

        // Execute integrations in order
        for (plugin_a, plugin_b) in &plan.integration_order {
            self.integrate_plugins(plugin_a, plugin_b).await?;
        }

        // Create composed system
        let system_capabilities = self.get_combined_capabilities(&plan.plugins).await?;
        let system_health = self.check_system_health(&plan.plugins).await?;

        let composed_system = ComposedSystem {
            system_id: system_id.clone(),
            active_plugins: plan.plugins.clone(),
            system_capabilities,
            system_health,
        };

        // Store active composition
        self.active_compositions
            .write()
            .await
            .insert(system_id.clone(), composed_system);

        // Send event
        let _ = self.event_sender.send(PluginEvent::CompositionCreated {
            system_id: system_id.clone(),
            plugins: plan.plugins,
        });

        // Return the composed system (we need to recreate it since we moved it)
        let composed_systems = self.active_compositions.read().await;
        composed_systems.get(&system_id).cloned().ok_or_else(|| {
            SongbirdError::CompositionFailed(format!(
                "System {} not found after creation",
                system_id
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_composition_constraints_default() {
        let constraints = CompositionConstraints::default();
        assert_eq!(constraints.max_latency_ms, Some(100.0));
        assert_eq!(constraints.max_memory_mb, Some(1024.0));
        assert_eq!(constraints.max_plugins, Some(10));
        assert_eq!(constraints.security_level, Some("standard".to_string()));
    }

    #[test]
    fn test_performance_requirements_creation() {
        let requirements = PerformanceRequirements {
            min_throughput_rps: Some(1000.0),
            max_response_time_ms: Some(100.0),
            min_availability_percent: Some(99.9),
        };

        assert_eq!(requirements.min_throughput_rps, Some(1000.0));
        assert_eq!(requirements.max_response_time_ms, Some(100.0));
        assert_eq!(requirements.min_availability_percent, Some(99.9));
    }

    #[tokio::test]
    async fn test_dynamic_plugin_registry_creation() {
        let registry = DynamicPluginRegistry::new();
        assert!(registry.list_plugins().await.is_empty());
    }

    #[tokio::test]
    async fn test_registry_default() {
        let registry = DynamicPluginRegistry::default();
        assert!(registry.list_plugins().await.is_empty());
    }

    #[tokio::test]
    async fn test_plugin_capabilities_not_found() {
        let registry = DynamicPluginRegistry::new();
        
        // Test getting capabilities for non-existent plugin
        let result = registry.get_plugin_capabilities("non-existent").await;
        assert!(result.is_err());
        if let Err(SongbirdError::PluginNotFound(plugin_id)) = result {
            assert_eq!(plugin_id, "non-existent");
        }
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let registry = DynamicPluginRegistry::new();
        let _receiver = registry.subscribe_events();
        // Test passes if we can create a receiver without panicking
    }

    #[tokio::test]
    async fn test_discover_optimal_composition_empty() {
        let registry = DynamicPluginRegistry::new();
        let constraints = CompositionConstraints::default();
        let required_capabilities = vec![PluginCapability::Encryption {
            algorithms: vec!["AES256".to_string()],
        }];

        let result = registry.discover_optimal_composition(
            "Test composition",
            required_capabilities,
            constraints,
        ).await;

        assert!(result.is_ok());
        let plans = result.unwrap();
        assert!(plans.is_empty()); // No plugins registered, so no plans
    }

    #[tokio::test]
    async fn test_capabilities_satisfy_requirements() {
        let registry = DynamicPluginRegistry::new();
        
        let capabilities = vec![
            PluginCapability::Encryption {
                algorithms: vec!["AES256".to_string()],
            },
            PluginCapability::ServiceDiscovery {
                protocols: vec!["HTTP".to_string()],
            },
        ];

        let requirements = vec![
            PluginCapability::Encryption {
                algorithms: vec!["AES256".to_string()],
            },
        ];

        assert!(registry.capabilities_satisfy_requirements(&capabilities, &requirements));

        let requirements = vec![
            PluginCapability::Compute {
                cpu_cores: 4,
                memory_gb: 8,
            },
        ];

        assert!(!registry.capabilities_satisfy_requirements(&capabilities, &requirements));
    }

    #[tokio::test]
    async fn test_requirement_to_capability_conversion() {
        let registry = DynamicPluginRegistry::new();

        let requirement = PluginRequirement::RequiresEncryption { min_key_size: Some(256) };
        let capability = registry.requirement_to_capability(&requirement);

        if let PluginCapability::Encryption { algorithms } = capability {
            assert_eq!(algorithms, vec!["AES256".to_string()]);
        } else {
            panic!("Expected Encryption capability");
        }
    }

    #[tokio::test]
    async fn test_plugin_integration() {
        let registry = DynamicPluginRegistry::new();

        // Test plugin integration
        let result = registry.integrate_plugins("plugin-a", "plugin-b").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_system_health_check() {
        let registry = DynamicPluginRegistry::new();
        let plugin_ids = vec!["plugin-1".to_string(), "plugin-2".to_string()];

        let result = registry.check_system_health(&plugin_ids).await;
        assert!(result.is_ok());

        let health = result.unwrap();
        assert!(health.overall_healthy);
        assert_eq!(health.plugin_health.len(), 2);
        assert!(health.plugin_health.contains_key("plugin-1"));
        assert!(health.plugin_health.contains_key("plugin-2"));
    }

    #[tokio::test]
    async fn test_composition_plan_creation() {
        let registry = DynamicPluginRegistry::new();
        let combination = vec!["plugin-1".to_string(), "plugin-2".to_string()];
        let constraints = CompositionConstraints::default();

        let result = registry.create_composition_plan(combination, &constraints).await;
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.plugins.len(), 2);
        assert_eq!(plan.integration_order.len(), 1); // One pair
        assert_eq!(plan.integration_order[0], ("plugin-1".to_string(), "plugin-2".to_string()));
    }

    #[tokio::test]
    async fn test_combinations_generation() {
        let registry = DynamicPluginRegistry::new();
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let combinations = registry.combinations(&items, 2);
        assert_eq!(combinations.len(), 3); // [a,b], [a,c], [b,c]

        let combinations = registry.combinations(&items, 1);
        assert_eq!(combinations.len(), 3); // [a], [b], [c]

        let combinations = registry.combinations(&items, 0);
        assert_eq!(combinations.len(), 1); // [[]]
        assert!(combinations[0].is_empty());
    }

    #[tokio::test]
    async fn test_auto_compose_no_plugins() {
        let registry = DynamicPluginRegistry::new();
        let capabilities = vec![PluginCapability::Encryption {
            algorithms: vec!["AES256".to_string()],
        }];

        let result = registry.auto_compose(capabilities).await;
        assert!(result.is_err());
        if let Err(SongbirdError::CompositionFailed(msg)) = result {
            assert_eq!(msg, "No valid composition found");
        }
    }

    #[tokio::test]
    async fn test_discover_plugins_empty() {
        let registry = DynamicPluginRegistry::new();
        let requirements = vec![PluginRequirement::RequiresEncryption { min_key_size: Some(256) }];

        let result = registry.discover_plugins(requirements).await;
        assert!(result.is_ok());
        let plugins = result.unwrap();
        assert!(plugins.is_empty()); // No plugins registered
    }

    #[tokio::test]
    async fn test_get_combined_capabilities_empty() {
        let registry = DynamicPluginRegistry::new();
        let plugin_ids = vec!["non-existent".to_string()];

        let result = registry.get_combined_capabilities(&plugin_ids).await;
        assert!(result.is_ok());
        let capabilities = result.unwrap();
        assert!(capabilities.is_empty());
    }

    #[tokio::test]
    async fn test_generate_combinations_empty() {
        let registry = DynamicPluginRegistry::new();
        let available_plugins = vec!["plugin-1".to_string()];
        let required_capabilities = vec![PluginCapability::Encryption {
            algorithms: vec!["AES256".to_string()],
        }];

        let result = registry.generate_combinations(&available_plugins, &required_capabilities).await;
        assert!(result.is_ok());
        let combinations = result.unwrap();
        assert!(combinations.is_empty()); // No plugins satisfy requirements
    }

    #[tokio::test]
    async fn test_service_registry_creation() {
        let registry = ServiceRegistry::new();
        assert_eq!(registry.get_service_count().await, 0);
        assert!(registry.list_services().await.is_empty());
    }

    #[tokio::test]
    async fn test_service_registry_default() {
        let registry = ServiceRegistry::default();
        assert_eq!(registry.get_service_count().await, 0);
    }

    #[tokio::test]
    async fn test_registry_plugin_trait_implementation() {
        let registry = DynamicPluginRegistry::new();
        
        // Test plugin registry trait methods
        let result = registry.register_plugin(
            "test-plugin".to_string(),
            vec![PluginCapability::Encryption { algorithms: vec!["AES256".to_string()] }],
            vec![],
        ).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-plugin");
    }

    #[tokio::test]
    async fn test_plugin_event_enum() {
        // Test that we can create all plugin event types
        let _events = vec![
            PluginEvent::PluginRegistered {
                plugin_id: "test".to_string(),
                capabilities: vec![],
            },
            PluginEvent::PluginIntegrated {
                plugin_a: "a".to_string(),
                plugin_b: "b".to_string(),
                integration_id: "int".to_string(),
            },
            PluginEvent::CompositionCreated {
                system_id: "sys".to_string(),
                plugins: vec!["p1".to_string()],
            },
            PluginEvent::CompositionFailed {
                error: "error".to_string(),
                attempted_plugins: vec!["p1".to_string()],
            },
            PluginEvent::PluginHealthChanged {
                plugin_id: "p1".to_string(),
                healthy: true,
            },
        ];
    }

    #[test]
    fn test_plugin_capability_comparison() {
        let cap1 = PluginCapability::Encryption {
            algorithms: vec!["AES256".to_string()],
        };
        let cap2 = PluginCapability::Encryption {
            algorithms: vec!["AES256".to_string()],
        };
        let cap3 = PluginCapability::ServiceDiscovery {
            protocols: vec!["HTTP".to_string()],
        };

        assert_eq!(cap1, cap2);
        assert_ne!(cap1, cap3);
    }

    #[test]
    fn test_plugin_requirement_types() {
        let _req1 = PluginRequirement::RequiresEncryption { min_key_size: Some(256) };
        let _req2 = PluginRequirement::RequiresServiceDiscovery;
        let _req3 = PluginRequirement::RequiresCompute { min_cpu_cores: 2, min_memory_gb: 4 };
        let _req4 = PluginRequirement::RequiresNetwork { min_bandwidth_mbps: 100, max_latency_ms: 50 };
        let _req5 = PluginRequirement::Custom { 
            name: "test".to_string(), 
            constraints: HashMap::new() 
        };
    }
}
