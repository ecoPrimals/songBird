//! Service Registry Module
//!
//! Service registration and management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::errors::{Result, SongbirdError};
use crate::traits::service::{ServiceInfo, UniversalService};
use crate::traits::{
    ComposablePlugin, PluginRegistry, PluginCapability, PluginRequirement,
    CompositionPlan, ComposedSystem, PerformanceEstimate, SystemHealth, PluginHealth
};

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
            .map_err(|e| SongbirdError::service_error(&self.info.id, e.to_string()))?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut service = self.service.write().await;
        service
            .stop()
            .await
            .map_err(|e| SongbirdError::service_error(&self.info.id, e.to_string()))?;
        Ok(())
    }

    pub async fn health_check(&self) -> Result<serde_json::Value> {
        let service = self.service.read().await;
        let health = service
            .health_check()
            .await
            .map_err(|e| SongbirdError::health_check_failed(&self.info.id, e.to_string()))?;
        serde_json::to_value(health)
            .map_err(|e| SongbirdError::service_error(&self.info.id, e.to_string()))
    }
}

/// Central service registry
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, Box<dyn UniversalService>>>>,
    service_info: Arc<RwLock<HashMap<String, ServiceInfo>>>,
}

impl ServiceRegistry {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            service_info: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn register(&self, service: Box<dyn UniversalService>) -> Result<()> {
        let service_info = service.service_info();
        let service_id = service_info.id.clone();
        
        tracing::info!(service_id = %service_id, "Registering service");
        
        // Store the service and its info
        self.services.write().await.insert(service_id.clone(), service);
        self.service_info.write().await.insert(service_id.clone(), service_info);
        
        tracing::info!(service_id = %service_id, "Service registered successfully");
        Ok(())
    }

    pub async fn unregister(&self, service_id: &str) -> Result<()> {
        tracing::info!(service_id = %service_id, "Unregistering service");
        
        self.services.write().await.remove(service_id);
        self.service_info.write().await.remove(service_id);
        
        tracing::info!(service_id = %service_id, "Service unregistered successfully");
        Ok(())
    }

    pub async fn list_services(&self) -> Result<Vec<ServiceInfo>> {
        Ok(self.service_info.read().await.values().cloned().collect())
    }

    pub async fn get_service(&self, service_id: &str) -> Result<Option<ServiceInfo>> {
        Ok(self.service_info.read().await.get(service_id).cloned())
    }

    pub async fn service_count(&self) -> usize {
        self.service_info.read().await.len()
    }

    pub async fn get_service_handle(&self, service_id: &str) -> Result<Option<Arc<RwLock<dyn UniversalService>>>> {
        if let Some(service) = self.services.read().await.get(service_id) {
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
    PluginRegistered { plugin_id: String, capabilities: Vec<PluginCapability> },
    PluginIntegrated { plugin_a: String, plugin_b: String, integration_id: String },
    CompositionCreated { system_id: String, plugins: Vec<String> },
    CompositionFailed { error: String, attempted_plugins: Vec<String> },
    PluginHealthChanged { plugin_id: String, healthy: bool },
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
        self.plugins.read().keys().cloned().collect()
    }
    
    /// Get plugin capabilities
    pub async fn get_plugin_capabilities(&self, plugin_id: &str) -> Result<Vec<PluginCapability>> {
        let plugins = self.plugins.read();
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
        
        let plugins = self.plugins.read();
        let mut candidate_plans = Vec::new();
        
        // Find all possible combinations that satisfy requirements
        let available_plugins: Vec<_> = plugins.keys().cloned().collect();
        
        for combination in self.generate_combinations(&available_plugins, &required_capabilities).await? {
            if let Ok(plan) = self.create_composition_plan(combination, &constraints).await {
                candidate_plans.push(plan);
            }
        }
        
        // Sort by performance estimate (best first)
        candidate_plans.sort_by(|a, b| {
            a.estimated_performance.latency_ms
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
        let plugins = self.plugins.read();
        let mut combinations = Vec::new();
        
        // Start with single plugins
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
                if self.capabilities_satisfy_requirements(&combined_capabilities, required_capabilities) {
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
    async fn get_combined_capabilities(&self, plugin_ids: &[String]) -> Result<Vec<PluginCapability>> {
        let plugins = self.plugins.read();
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
    
    /// Generate combinations of a specific size
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
    pub min_throughput_rps: f64,
    pub max_latency_ms: f64,
    pub max_cpu_percent: f64,
    pub max_memory_mb: f64,
}

#[async_trait]
impl PluginRegistry for DynamicPluginRegistry {
    async fn register_plugin(&self, mut plugin: Box<dyn ComposablePlugin>) -> Result<String> {
        let plugin_id = plugin.plugin_id().to_string();
        let capabilities = plugin.capabilities();
        
        tracing::info!(
            plugin_id = %plugin_id,
            capabilities = ?capabilities,
            "Registering plugin in dynamic registry"
        );
        
        // Update capability index
        {
            let mut index = self.capability_index.write();
            for capability in &capabilities {
                index.entry(capability.clone())
                    .or_insert_with(Vec::new)
                    .push(plugin_id.clone());
            }
        }
        
        // Store plugin
        self.plugins.write().insert(plugin_id.clone(), plugin);
        
        // Send event
        let _ = self.event_sender.send(PluginEvent::PluginRegistered {
            plugin_id: plugin_id.clone(),
            capabilities,
        });
        
        Ok(plugin_id)
    }
    
    async fn discover_plugins(&self, requirements: Vec<PluginRequirement>) -> Result<Vec<String>> {
        let index = self.capability_index.read();
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
    
    async fn auto_compose(&self, target_capabilities: Vec<PluginCapability>) -> Result<CompositionPlan> {
        let constraints = CompositionConstraints {
            max_latency_ms: Some(100.0), // Default: 100ms max latency
            max_memory_mb: Some(1024.0), // Default: 1GB max memory
            max_plugins: Some(10),       // Default: max 10 plugins
            required_performance: None,
            security_level: None,
        };
        
        let plans = self.discover_optimal_composition(
            "Auto-composition request",
            target_capabilities,
            constraints,
        ).await?;
        
        plans.into_iter().next()
            .ok_or_else(|| SongbirdError::CompositionFailed("No valid composition found".to_string()))
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
        self.active_compositions.write().insert(system_id.clone(), composed_system);
        
        // Send event
        let _ = self.event_sender.send(PluginEvent::CompositionCreated {
            system_id: system_id.clone(),
            plugins: plan.plugins,
        });
        
        // Return the composed system (we need to recreate it since we moved it)
        let composed_systems = self.active_compositions.read();
        composed_systems.get(&system_id).cloned().ok_or_else(|| SongbirdError::CompositionFailed(format!("System {} not found after creation", system_id)))
    }
} 