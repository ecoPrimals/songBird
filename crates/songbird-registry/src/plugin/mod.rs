//! Dynamic plugin registry and composition
//!
//! Provides runtime plugin discovery and composition capabilities

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use async_trait::async_trait;
use songbird_discovery::traits::{
    ComposablePlugin, ComposedSystem, CompositionPlan, PerformanceEstimate, PluginCapability,
    PluginRegistry, PluginRequirement, SystemHealth,
};
use songbird_errors::Result;

/// Dynamic Plugin Registry Implementation
///
/// This registry allows services to be discovered and composed at runtime
/// without requiring pre-configured TOML files for every possible combination.
pub struct DynamicPluginRegistry {
    plugins: Arc<RwLock<HashMap<String, Box<dyn ComposablePlugin>>>>,
    capabilities: Arc<RwLock<HashMap<String, PluginCapability>>>,
    requirements: Arc<RwLock<HashMap<String, Vec<PluginRequirement>>>>,
    #[allow(dead_code)]
    requirement_graph: Arc<RwLock<HashMap<String, Vec<String>>>>,
    system_health: Arc<RwLock<SystemHealth>>,
}

impl DynamicPluginRegistry {
    /// Create a new dynamic plugin registry
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            capabilities: Arc::new(RwLock::new(HashMap::new())),
            requirements: Arc::new(RwLock::new(HashMap::new())),
            requirement_graph: Arc::new(RwLock::new(HashMap::new())),
            system_health: Arc::new(RwLock::new(SystemHealth {
                overall_healthy: true,
                plugin_health: HashMap::new(),
                integration_health: HashMap::new(),
            })),
        }
    }

    /// List all registered plugins
    pub async fn list_plugins(&self) -> Vec<String> {
        self.plugins.read().await.keys().cloned().collect()
    }

    /// Get plugin capabilities
    pub async fn get_plugin_capabilities(&self, plugin_id: &str) -> Result<Vec<PluginCapability>> {
        let plugins = self.plugins.read().await;
        if let Some(plugin) = plugins.get(plugin_id) {
            Ok(plugin.capabilities())
        } else {
            Ok(vec![])
        }
    }

    /// Discover optimal composition for given requirements
    pub async fn discover_optimal_composition(
        &self,
        task_description: &str,
        required_capabilities: Vec<PluginCapability>,
        constraints: CompositionConstraints,
    ) -> Result<Vec<CompositionPlan>> {
        tracing::info!("Discovering optimal composition for: {}", task_description);

        // Find plugins that provide required capabilities
        let available_plugins = self
            .find_plugins_by_capabilities(&required_capabilities)
            .await?;

        if available_plugins.is_empty() {
            tracing::warn!("No plugins found with required capabilities");
            return Ok(vec![]);
        }

        // Generate combinations
        let combinations = self
            .generate_combinations(&available_plugins, &required_capabilities)
            .await?;

        // Create composition plans
        let mut plans = Vec::new();
        for combination in combinations {
            if let Ok(plan) = self
                .create_composition_plan(combination, &constraints)
                .await
            {
                plans.push(plan);
            }
        }

        // Sort by estimated performance (using cpu_utilization_percent as primary metric)
        plans.sort_by(|a, b| {
            b.estimated_performance
                .cpu_utilization_percent
                .partial_cmp(&a.estimated_performance.cpu_utilization_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(plans)
    }

    async fn find_plugins_by_capabilities(
        &self,
        capabilities: &[PluginCapability],
    ) -> Result<Vec<String>> {
        let mut plugins = Vec::new();
        let caps = self.capabilities.read().await;

        for capability in capabilities {
            // Find plugins that have this capability
            for (plugin_id, cap) in caps.iter() {
                if cap == capability {
                    plugins.push(plugin_id.clone());
                }
            }
        }

        plugins.sort();
        plugins.dedup();
        Ok(plugins)
    }

    async fn create_composition_plan(
        &self,
        plugin_combination: Vec<String>,
        _constraints: &CompositionConstraints,
    ) -> Result<CompositionPlan> {
        // Calculate estimated performance
        let estimated_performance = PerformanceEstimate {
            latency_ms: 50.0,
            throughput_rps: 1000.0,
            memory_usage_mb: 256.0,
            cpu_utilization_percent: 25.0,
        };

        Ok(CompositionPlan {
            plugins: plugin_combination,
            integration_order: vec![],
            shared_config: serde_json::Value::Null,
            estimated_performance,
        })
    }

    async fn generate_combinations(
        &self,
        available_plugins: &[String],
        _required_capabilities: &[PluginCapability],
    ) -> Result<Vec<Vec<String>>> {
        let mut combinations = Vec::new();

        // Generate single plugin "combinations"
        for plugin in available_plugins {
            combinations.push(vec![plugin.clone()]);
        }

        // Generate pairs if we have enough plugins
        if available_plugins.len() >= 2 {
            for i in 0..available_plugins.len() {
                for j in i + 1..available_plugins.len() {
                    combinations.push(vec![
                        available_plugins[i].clone(),
                        available_plugins[j].clone(),
                    ]);
                }
            }
        }

        Ok(combinations)
    }

    /// Integrate two plugins
    #[allow(dead_code)]
    async fn integrate_plugins(&self, plugin_a: &str, plugin_b: &str) -> Result<String> {
        let integration_id = format!("{plugin_a}_{plugin_b}");

        // Event broadcasting removed - would need to be implemented differently
        tracing::info!("Integrated plugins {} and {}", plugin_a, plugin_b);

        Ok(integration_id)
    }

    /// Check system health for given plugins
    #[allow(dead_code)]
    async fn check_system_health(&self, plugin_ids: &[String]) -> Result<SystemHealth> {
        let mut healthy_plugins = 0;
        let plugins = self.plugins.read().await;
        let mut plugin_health = HashMap::new();

        for plugin_id in plugin_ids {
            if let Some(plugin) = plugins.get(plugin_id) {
                let health = plugin.health_check().await;
                if health.healthy {
                    healthy_plugins += 1;
                }
                plugin_health.insert(plugin_id.clone(), health);
            }
        }

        let overall_healthy = if plugin_ids.is_empty() {
            false
        } else {
            healthy_plugins == plugin_ids.len()
        };

        Ok(SystemHealth {
            overall_healthy,
            plugin_health,
            integration_health: HashMap::new(),
        })
    }
}

impl Default for DynamicPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PluginRegistry for DynamicPluginRegistry {
    async fn register_plugin(
        &self,
        plugin_id: String,
        capabilities: Vec<PluginCapability>,
        _requirements: Vec<PluginRequirement>,
    ) -> Result<String> {
        // Store capabilities directly
        let mut caps = self.capabilities.write().await;
        let current_len = caps.len();
        for (i, capability) in capabilities.iter().enumerate() {
            caps.insert(
                format!("{}_{}", plugin_id, current_len + i),
                capability.clone(),
            );
        }

        // Event broadcasting removed - would need to be implemented differently
        tracing::info!("Registered plugin {} with capabilities", plugin_id);

        Ok(plugin_id)
    }

    async fn discover_plugins(&self, requirements: Vec<PluginRequirement>) -> Result<Vec<String>> {
        // Convert requirements to capabilities for discovery
        let capabilities: Vec<PluginCapability> = requirements
            .iter()
            .map(|req| self.requirement_to_capability(req))
            .collect();

        self.find_plugins_by_capabilities(&capabilities).await
    }

    async fn auto_compose(
        &self,
        target_capabilities: Vec<PluginCapability>,
    ) -> Result<CompositionPlan> {
        let constraints = CompositionConstraints::default();
        let plans = self
            .discover_optimal_composition("auto-compose", target_capabilities, constraints)
            .await?;

        plans.into_iter().next().ok_or_else(|| {
            songbird_errors::SongbirdError::service_error(
                "plugin-registry",
                "No viable composition found".to_string(),
            )
        })
    }

    async fn execute_composition(&self, plan: CompositionPlan) -> Result<ComposedSystem> {
        let system_id = Uuid::new_v4().to_string();

        let system = ComposedSystem {
            system_id: system_id.clone(),
            active_plugins: plan.plugins.clone(),
            system_capabilities: vec![], // Would be calculated from plugins
            system_health: SystemHealth {
                overall_healthy: true,
                plugin_health: HashMap::new(),
                integration_health: HashMap::new(),
            },
        };

        // Event broadcasting removed - would need to be implemented differently
        tracing::info!("Executed composition for system {}", system_id);

        Ok(system)
    }
}

impl DynamicPluginRegistry {
    fn requirement_to_capability(&self, requirement: &PluginRequirement) -> PluginCapability {
        // Convert plugin requirement to capability
        // This is a simplified mapping - in practice, this would be more sophisticated
        match requirement {
            PluginRequirement::RequiresEncryption { .. } => PluginCapability::Encryption {
                algorithms: vec!["aes".to_string()],
            },
            PluginRequirement::RequiresServiceDiscovery => PluginCapability::ServiceDiscovery {
                protocols: vec!["mdns".to_string()],
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
            PluginRequirement::Custom { name, .. } => PluginCapability::Custom {
                name: name.clone(),
                attributes: HashMap::new(),
            },
        }
    }
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

/// Composition constraints for optimization
#[derive(Debug, Clone)]
pub struct CompositionConstraints {
    pub max_latency_ms: Option<f64>,
    pub max_memory_mb: Option<f64>,
    pub max_plugins: Option<usize>,
    pub required_performance: Option<PerformanceRequirements>,
    pub security_level: Option<String>,
}

impl Default for CompositionConstraints {
    fn default() -> Self {
        Self {
            max_latency_ms: Some(1000.0),
            max_memory_mb: Some(1024.0),
            max_plugins: Some(10),
            required_performance: None,
            security_level: Some("standard".to_string()),
        }
    }
}

/// Performance requirements for composition
#[derive(Debug, Clone)]
pub struct PerformanceRequirements {
    pub min_throughput_rps: f64,
    pub max_latency_ms: f64,
    pub max_cpu_percent: f64,
    pub max_memory_mb: f64,
}
