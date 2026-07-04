// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Dynamic plugin registry and composition
//!
//! Provides runtime plugin discovery and composition capabilities.
//! Plugins are metadata-only `RegisteredPlugin` structs (no trait-object dispatch).

mod types;

#[cfg(test)]
mod tests;

pub use types::{
    ComposedSystem, CompositionConstraints, CompositionPlan, PerformanceEstimate,
    PerformanceRequirements, PluginCapability, PluginEvent, PluginRequirement, RegisteredPlugin,
    SystemHealth,
};

use songbird_types::errors::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing;
use uuid::Uuid;

/// Dynamic Plugin Registry Implementation
///
/// This registry allows services to be discovered and composed at runtime
/// without requiring pre-configured TOML files for every possible combination.
pub struct DynamicPluginRegistry {
    plugins: Arc<RwLock<HashMap<String, RegisteredPlugin>>>,
    capabilities: Arc<RwLock<HashMap<String, PluginCapability>>>,
    requirements: Arc<RwLock<HashMap<String, Vec<PluginRequirement>>>>,
    #[allow(
        dead_code,
        reason = "populated by register(); topological sort used by validate_composition()"
    )]
    requirement_graph: Arc<RwLock<HashMap<String, Vec<String>>>>,
    #[allow(dead_code, reason = "tracks global health; consumed by future lifecycle.health API")]
    system_health: Arc<RwLock<SystemHealth>>,
}

impl DynamicPluginRegistry {
    const DEFAULT_ESTIMATED_LATENCY_MS: f64 = 50.0;
    const DEFAULT_ESTIMATED_THROUGHPUT_RPS: f64 = 1000.0;
    const DEFAULT_ESTIMATED_MEMORY_MB: f64 = 256.0;
    const DEFAULT_ESTIMATED_CPU_PERCENT: f64 = 25.0;

    const WELL_KNOWN_ENCRYPTION_ALGORITHM: &str = "chacha20-poly1305";
    const WELL_KNOWN_DISCOVERY_PROTOCOL: &str = "birdsong";

    /// Create a new dynamic plugin registry
    #[must_use]
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
    pub async fn get_plugin_capabilities(
        &self,
        plugin_id: &str,
    ) -> SongbirdResult<Vec<PluginCapability>> {
        let plugins = self.plugins.read().await;
        Ok(plugins.get(plugin_id).map_or_else(Vec::new, |p| p.capabilities.clone()))
    }

    /// Discover optimal composition for given requirements
    pub async fn discover_optimal_composition(
        &self,
        task_description: &str,
        required_capabilities: Vec<PluginCapability>,
        constraints: CompositionConstraints,
    ) -> SongbirdResult<Vec<CompositionPlan>> {
        tracing::info!("Discovering optimal composition for: {}", task_description);

        let available_plugins = self.find_plugins_by_capabilities(&required_capabilities).await?;

        if available_plugins.is_empty() {
            tracing::warn!("No plugins found with required capabilities ");
            return Ok(vec![]);
        }

        let combinations =
            self.generate_combinations(&available_plugins, &required_capabilities).await?;

        let mut plans = Vec::new();
        for combination in combinations {
            if let Ok(plan) = self.create_composition_plan(combination, &constraints).await {
                plans.push(plan);
            }
        }

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
    ) -> SongbirdResult<Vec<String>> {
        let mut plugins = Vec::new();
        let caps = self.capabilities.read().await;

        for capability in capabilities {
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
        constraints: &CompositionConstraints,
    ) -> SongbirdResult<CompositionPlan> {
        let plugin_count = plugin_combination.len();

        let base_latency =
            constraints.max_latency_ms.map_or(Self::DEFAULT_ESTIMATED_LATENCY_MS, |max| max * 0.5);

        let estimated_performance = PerformanceEstimate {
            latency_ms: base_latency * plugin_count as f64,
            throughput_rps: Self::DEFAULT_ESTIMATED_THROUGHPUT_RPS / plugin_count as f64,
            memory_usage_mb: Self::DEFAULT_ESTIMATED_MEMORY_MB * plugin_count as f64,
            cpu_utilization_percent: (Self::DEFAULT_ESTIMATED_CPU_PERCENT * plugin_count as f64)
                .min(100.0),
        };

        Ok(CompositionPlan {
            plugins: plugin_combination,
            integration_order: vec![],
            shared_config: serde_json::Value::Null,
            estimated_performance,
        })
    }

    /// Generate plugin combinations (visible to tests)
    pub(crate) async fn generate_combinations(
        &self,
        available_plugins: &[String],
        _required_capabilities: &[PluginCapability],
    ) -> SongbirdResult<Vec<Vec<String>>> {
        let mut combinations = Vec::new();

        for plugin in available_plugins {
            combinations.push(vec![plugin.clone()]);
        }

        if available_plugins.len() >= 2 {
            for i in 0..available_plugins.len() {
                for j in i + 1..available_plugins.len() {
                    combinations
                        .push(vec![available_plugins[i].clone(), available_plugins[j].clone()]);
                }
            }
        }

        Ok(combinations)
    }
}

impl Default for DynamicPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicPluginRegistry {
    /// Register a plugin with given capabilities and requirements
    pub async fn register_plugin(
        &self,
        plugin_id: String,
        capabilities: Vec<PluginCapability>,
        requirements: Vec<PluginRequirement>,
    ) -> SongbirdResult<String> {
        let mut caps = self.capabilities.write().await;
        let current_len = caps.len();
        for (i, capability) in capabilities.iter().enumerate() {
            caps.insert(format!("{}_{}", plugin_id, current_len + i), capability.clone());
        }
        drop(caps);

        let mut reqs = self.requirements.write().await;
        reqs.insert(plugin_id.clone(), requirements);
        drop(reqs);

        let mut graph = self.requirement_graph.write().await;
        graph.entry(plugin_id.clone()).or_default();
        drop(graph);

        tracing::info!("Registered plugin {} with capabilities", plugin_id);
        Ok(plugin_id)
    }

    /// Discover plugins matching the given requirements
    pub async fn discover_plugins(
        &self,
        requirements: Vec<PluginRequirement>,
    ) -> SongbirdResult<Vec<String>> {
        let capabilities: Vec<PluginCapability> =
            requirements.iter().map(Self::requirement_to_capability).collect();
        self.find_plugins_by_capabilities(&capabilities).await
    }

    /// Auto-compose an optimal system from target capabilities
    pub async fn auto_compose(
        &self,
        target_capabilities: Vec<PluginCapability>,
    ) -> SongbirdResult<CompositionPlan> {
        let constraints = CompositionConstraints::default();
        let plans = self
            .discover_optimal_composition("auto-compose", target_capabilities, constraints)
            .await?;

        plans.into_iter().next().ok_or_else(|| {
            SongbirdError::service("plugin-registry", String::from("No viable composition found"))
        })
    }

    /// Execute a composition plan, producing a live composed system
    pub async fn execute_composition(
        &self,
        plan: CompositionPlan,
    ) -> SongbirdResult<ComposedSystem> {
        let system_id = Uuid::new_v4().to_string();

        let mut plugin_health = HashMap::new();
        let plugins_read = self.plugins.read().await;
        for plugin_id in &plan.plugins {
            let healthy = plugins_read.get(plugin_id).is_some_and(|p| p.healthy);
            plugin_health.insert(plugin_id.clone(), healthy);
        }
        let all_healthy = !plan.plugins.is_empty() && plugin_health.values().all(|&h| h);
        drop(plugins_read);

        let system_capabilities = self.collect_capabilities_for(&plan.plugins).await;

        let system = ComposedSystem {
            system_id: system_id.clone(),
            active_plugins: plan.plugins,
            system_capabilities,
            system_health: SystemHealth {
                overall_healthy: all_healthy,
                plugin_health,
                integration_health: HashMap::new(),
            },
        };

        tracing::info!("Executed composition for system {}", system_id);
        Ok(system)
    }

    async fn collect_capabilities_for(&self, plugin_ids: &[String]) -> Vec<PluginCapability> {
        let caps = self.capabilities.read().await;
        let mut result = Vec::new();
        for plugin_id in plugin_ids {
            for (key, cap) in caps.iter() {
                if key.starts_with(plugin_id) {
                    result.push(cap.clone());
                }
            }
        }
        result
    }
}

impl DynamicPluginRegistry {
    pub(crate) fn requirement_to_capability(requirement: &PluginRequirement) -> PluginCapability {
        match requirement {
            PluginRequirement::RequiresEncryption {
                ..
            } => PluginCapability::Encryption {
                algorithms: vec![String::from(Self::WELL_KNOWN_ENCRYPTION_ALGORITHM)],
            },
            PluginRequirement::RequiresServiceDiscovery => PluginCapability::ServiceDiscovery {
                protocols: vec![String::from(Self::WELL_KNOWN_DISCOVERY_PROTOCOL)],
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
            PluginRequirement::Custom {
                name,
                ..
            } => PluginCapability::Custom {
                name: name.clone(),
                attributes: HashMap::new(),
            },
        }
    }
}
