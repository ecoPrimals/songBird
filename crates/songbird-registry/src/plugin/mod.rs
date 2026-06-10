// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Dynamic plugin registry and composition
//!
//! Provides runtime plugin discovery and composition capabilities.
//! Plugins are metadata-only [`RegisteredPlugin`] structs (no trait-object dispatch).

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json;
use uuid::Uuid;
use songbird_types::errors::{SongbirdResult, SongbirdError};
use tracing;

// NOTE: Plugin types defined locally until architecture is finalized
// FUTURE WORK: Move to songbird-discovery::traits once plugin system is fully implemented
// This is a deferred architectural decision pending plugin ecosystem maturity

/// Registered plugin metadata (replaces `dyn ComposablePlugin` trait-object map).
///
/// All plugin state is metadata-only — capabilities and health are tracked by the
/// registry without requiring a live trait object.
#[derive(Debug, Clone)]
pub struct RegisteredPlugin {
    /// Plugin identifier
    pub id: String,
    /// Capabilities this plugin provides
    pub capabilities: Vec<PluginCapability>,
    /// Whether this plugin is currently healthy
    pub healthy: bool,
}

/// A live composed system assembled from multiple plugins.
#[derive(Debug, Clone)]
pub struct ComposedSystem {
    /// Unique system identifier.
    pub system_id: String,
    /// Plugin IDs currently active in this composition.
    pub active_plugins: Vec<String>,
    /// Aggregate capabilities provided by all active plugins.
    pub system_capabilities: Vec<PluginCapability>,
    /// Aggregate health state across all plugins and integrations.
    pub system_health: SystemHealth,
}

/// A composition plan describing which plugins to activate and how.
#[derive(Debug, Clone)]
pub struct CompositionPlan {
    /// Plugin IDs to include in this composition.
    pub plugins: Vec<String>,
    /// Topologically-sorted activation order.
    pub integration_order: Vec<String>,
    /// Shared configuration passed to all plugins.
    pub shared_config: serde_json::Value,
    /// Estimated performance characteristics.
    pub estimated_performance: PerformanceEstimate,
}

/// Estimated performance characteristics of a composition.
#[derive(Debug, Clone)]
pub struct PerformanceEstimate {
    /// Estimated latency in milliseconds.
    pub latency_ms: f64,
    /// Estimated throughput in requests per second.
    pub throughput_rps: f64,
    /// Estimated memory usage in megabytes.
    pub memory_usage_mb: f64,
    /// Estimated CPU utilization percentage.
    pub cpu_utilization_percent: f64,
}

/// A capability provided by a plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginCapability {
    /// Encryption capability with supported algorithms.
    Encryption {
        /// Algorithm identifiers (e.g. `"chacha20-poly1305"`).
        algorithms: Vec<String>,
    },
    /// Service discovery capability with supported protocols.
    ServiceDiscovery {
        /// Protocol identifiers (e.g. `"birdsong"`, `"mdns"`).
        protocols: Vec<String>,
    },
    /// Compute capacity.
    Compute {
        /// Available CPU cores.
        cpu_cores: u32,
        /// Available memory in gigabytes.
        memory_gb: u32,
    },
    /// Network capacity.
    Network {
        /// Available bandwidth in Mbps.
        bandwidth_mbps: u64,
        /// Network latency in milliseconds.
        latency_ms: u64,
    },
    /// Extensible capability.
    Custom {
        /// Capability name.
        name: String,
        /// Arbitrary key-value attributes.
        attributes: HashMap<String, String>,
    },
}

/// A requirement that a plugin declares it needs fulfilled.
#[derive(Debug, Clone)]
pub enum PluginRequirement {
    /// Requires an encryption provider.
    RequiresEncryption {
        /// Minimum key size in bits (optional).
        min_key_size: Option<u32>,
    },
    /// Requires a service discovery provider.
    RequiresServiceDiscovery,
    /// Requires a compute provider meeting minimums.
    RequiresCompute {
        /// Minimum CPU cores.
        min_cpu_cores: u32,
        /// Minimum memory in gigabytes.
        min_memory_gb: u32,
    },
    /// Requires a network link meeting minimums.
    RequiresNetwork {
        /// Minimum bandwidth in Mbps.
        min_bandwidth_mbps: u64,
        /// Maximum tolerable latency in milliseconds.
        max_latency_ms: u64,
    },
    /// Extensible requirement.
    Custom {
        /// Requirement name.
        name: String,
        /// Arbitrary key-value requirements.
        requirements: HashMap<String, String>,
    },
}

/// Aggregate health state for a composed system.
#[derive(Debug, Clone)]
pub struct SystemHealth {
    /// Whether the overall system is considered healthy.
    pub overall_healthy: bool,
    /// Per-plugin health status.
    pub plugin_health: HashMap<String, bool>,
    /// Per-integration-link health status.
    pub integration_health: HashMap<String, bool>,
}

/// Dynamic Plugin Registry Implementation
///
/// This registry allows services to be discovered and composed at runtime
/// without requiring pre-configured TOML files for every possible combination.
pub struct DynamicPluginRegistry {
    plugins: Arc<RwLock<HashMap<String, RegisteredPlugin>>>,
    capabilities: Arc<RwLock<HashMap<String, PluginCapability>>>,
    requirements: Arc<RwLock<HashMap<String, Vec<PluginRequirement>>>>,
    #[allow(dead_code, reason = "populated by register(); topological sort used by validate_composition()")]
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
    pub async fn get_plugin_capabilities(&self, plugin_id: &str) -> SongbirdResult<Vec<PluginCapability>> {
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

        // Find plugins that provide required capabilities
        let available_plugins = self.find_plugins_by_capabilities(&required_capabilities).await?;

        if available_plugins.is_empty() {
            tracing::warn!("No plugins found with required capabilities ");
            return Ok(vec![]);
        }

        // Generate combinations
        let combinations =
            self.generate_combinations(&available_plugins, &required_capabilities).await?;

        // Create composition plans
        let mut plans = Vec::new();
        for combination in combinations {
            if let Ok(plan) = self.create_composition_plan(combination, &constraints).await {
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
    ) -> SongbirdResult<Vec<String>> {
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
        constraints: &CompositionConstraints,
    ) -> SongbirdResult<CompositionPlan> {
        let plugin_count = plugin_combination.len();

        let base_latency = constraints
            .max_latency_ms
            .map_or(Self::DEFAULT_ESTIMATED_LATENCY_MS, |max| max * 0.5);

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

    async fn generate_combinations(
        &self,
        available_plugins: &[String],
        _required_capabilities: &[PluginCapability],
    ) -> SongbirdResult<Vec<Vec<String>>> {
        let mut combinations = Vec::new();

        // Generate single plugin combinations
        for plugin in available_plugins {
            combinations.push(vec![plugin.clone()]);
        }

        // Generate pairs if we have enough plugins
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
            SongbirdError::service(
                "plugin-registry",
                "No viable composition found".to_string(),
            )
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
    fn requirement_to_capability(requirement: &PluginRequirement) -> PluginCapability {
        match requirement {
            PluginRequirement::RequiresEncryption { .. } => PluginCapability::Encryption {
                algorithms: vec![String::from(Self::WELL_KNOWN_ENCRYPTION_ALGORITHM)],
            },
            PluginRequirement::RequiresServiceDiscovery => PluginCapability::ServiceDiscovery {
                protocols: vec![String::from(Self::WELL_KNOWN_DISCOVERY_PROTOCOL)],
            },
            PluginRequirement::RequiresCompute { min_cpu_cores, min_memory_gb } => PluginCapability::Compute {
                cpu_cores: *min_cpu_cores,
                memory_gb: *min_memory_gb,
            },
            PluginRequirement::RequiresNetwork { min_bandwidth_mbps, max_latency_ms } => PluginCapability::Network {
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

/// Plugin events for monitoring composition lifecycle.
#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// A new plugin was registered with capabilities.
    PluginRegistered {
        /// The registered plugin's identifier.
        plugin_id: String,
        /// Capabilities the plugin provides.
        capabilities: Vec<PluginCapability>,
    },
    /// Two plugins were integrated into a shared workflow.
    PluginIntegrated {
        /// First plugin in the integration.
        plugin_a: String,
        /// Second plugin in the integration.
        plugin_b: String,
        /// Unique integration link identifier.
        integration_id: String,
    },
    /// A new composed system was created.
    CompositionCreated {
        /// Unique system identifier.
        system_id: String,
        /// Plugin IDs in the composition.
        plugins: Vec<String>,
    },
    /// A composition attempt failed.
    CompositionFailed {
        /// Error description.
        error: String,
        /// Plugins that were attempted.
        attempted_plugins: Vec<String>,
    },
    /// A plugin's health state changed.
    PluginHealthChanged {
        /// The affected plugin's identifier.
        plugin_id: String,
        /// New health state.
        healthy: bool,
    },
}

/// Composition constraints for optimization.
#[derive(Debug, Clone)]
pub struct CompositionConstraints {
    /// Maximum acceptable latency in milliseconds.
    pub max_latency_ms: Option<f64>,
    /// Maximum acceptable memory usage in megabytes.
    pub max_memory_mb: Option<f64>,
    /// Maximum number of plugins in a composition.
    pub max_plugins: Option<usize>,
    /// Minimum performance requirements.
    pub required_performance: Option<PerformanceRequirements>,
    /// Required security level (e.g. `"standard"`, `"high"`).
    pub security_level: Option<String>,
}

impl Default for CompositionConstraints {
    fn default() -> Self {
        Self {
            max_latency_ms: Some(1000.0),
            max_memory_mb: Some(1024.0),
            max_plugins: Some(10),
            required_performance: None,
            security_level: None,
        }
    }
}

/// Performance requirements for composition
/// Minimum performance requirements for composition selection.
#[derive(Debug, Clone)]
pub struct PerformanceRequirements {
    /// Minimum throughput in requests per second.
    pub min_throughput_rps: f64,
    /// Maximum acceptable latency in milliseconds.
    pub max_latency_ms: f64,
    /// Maximum acceptable CPU utilization percentage.
    pub max_cpu_percent: f64,
    /// Maximum acceptable memory usage in megabytes.
    pub max_memory_mb: f64,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    fn encryption_cap() -> PluginCapability {
        PluginCapability::Encryption {
            algorithms: vec!["chacha20-poly1305".into()],
        }
    }

    fn discovery_cap() -> PluginCapability {
        PluginCapability::ServiceDiscovery {
            protocols: vec!["birdsong".into()],
        }
    }

    fn compute_cap(cores: u32, mem: u32) -> PluginCapability {
        PluginCapability::Compute {
            cpu_cores: cores,
            memory_gb: mem,
        }
    }

    #[tokio::test]
    async fn new_registry_is_empty() {
        let reg = DynamicPluginRegistry::new();
        assert!(reg.list_plugins().await.is_empty());
    }

    #[tokio::test]
    async fn register_plugin_returns_id() {
        let reg = DynamicPluginRegistry::new();
        let id = reg
            .register_plugin("crypto-provider".into(), vec![encryption_cap()], vec![])
            .await
            .unwrap();
        assert_eq!(id, "crypto-provider");
    }

    #[tokio::test]
    async fn get_plugin_capabilities_empty_for_unknown() {
        let reg = DynamicPluginRegistry::new();
        let caps = reg.get_plugin_capabilities("nonexistent").await.unwrap();
        assert!(caps.is_empty());
    }

    #[tokio::test]
    async fn discover_plugins_finds_matching() {
        let reg = DynamicPluginRegistry::new();
        reg.register_plugin("enc-1".into(), vec![encryption_cap()], vec![])
            .await
            .unwrap();
        reg.register_plugin("disc-1".into(), vec![discovery_cap()], vec![])
            .await
            .unwrap();

        let found = reg
            .discover_plugins(vec![PluginRequirement::RequiresEncryption {
                min_key_size: None,
            }])
            .await
            .unwrap();

        assert!(found.iter().any(|id| id.starts_with("enc-1")));
    }

    #[tokio::test]
    async fn discover_plugins_returns_empty_when_none_match() {
        let reg = DynamicPluginRegistry::new();
        reg.register_plugin("disc-1".into(), vec![discovery_cap()], vec![])
            .await
            .unwrap();

        let found = reg
            .discover_plugins(vec![PluginRequirement::RequiresCompute {
                min_cpu_cores: 8,
                min_memory_gb: 16,
            }])
            .await
            .unwrap();

        assert!(found.is_empty());
    }

    #[tokio::test]
    async fn discover_optimal_composition_empty_registry() {
        let reg = DynamicPluginRegistry::new();
        let plans = reg
            .discover_optimal_composition(
                "test task",
                vec![encryption_cap()],
                CompositionConstraints::default(),
            )
            .await
            .unwrap();
        assert!(plans.is_empty());
    }

    #[tokio::test]
    async fn discover_optimal_composition_returns_plans() {
        let reg = DynamicPluginRegistry::new();
        reg.register_plugin("enc-a".into(), vec![encryption_cap()], vec![])
            .await
            .unwrap();
        reg.register_plugin("enc-b".into(), vec![encryption_cap()], vec![])
            .await
            .unwrap();

        let plans = reg
            .discover_optimal_composition(
                "encrypt data",
                vec![encryption_cap()],
                CompositionConstraints::default(),
            )
            .await
            .unwrap();

        assert!(!plans.is_empty());
        assert!(plans.iter().any(|p| p.plugins.len() == 1));
        assert!(plans.iter().any(|p| p.plugins.len() == 2));
    }

    #[tokio::test]
    async fn auto_compose_succeeds_with_matching_plugin() {
        let reg = DynamicPluginRegistry::new();
        reg.register_plugin("net-1".into(), vec![compute_cap(4, 8)], vec![])
            .await
            .unwrap();

        let plan = reg.auto_compose(vec![compute_cap(4, 8)]).await.unwrap();
        assert!(!plan.plugins.is_empty());
    }

    #[tokio::test]
    async fn auto_compose_fails_with_no_matching_plugins() {
        let reg = DynamicPluginRegistry::new();
        let result = reg.auto_compose(vec![encryption_cap()]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn execute_composition_produces_system() {
        let reg = DynamicPluginRegistry::new();
        reg.register_plugin("p1".into(), vec![encryption_cap()], vec![])
            .await
            .unwrap();

        let plan = CompositionPlan {
            plugins: vec!["p1".into()],
            integration_order: vec![],
            shared_config: serde_json::Value::Null,
            estimated_performance: PerformanceEstimate {
                latency_ms: 10.0,
                throughput_rps: 500.0,
                memory_usage_mb: 128.0,
                cpu_utilization_percent: 10.0,
            },
        };

        let system = reg.execute_composition(plan).await.unwrap();
        assert_eq!(system.active_plugins, vec!["p1"]);
        assert!(!system.system_id.is_empty());
    }

    #[tokio::test]
    async fn execute_composition_unhealthy_when_plugin_not_in_plugins_map() {
        let reg = DynamicPluginRegistry::new();
        reg.register_plugin("registered".into(), vec![], vec![])
            .await
            .unwrap();

        let plan = CompositionPlan {
            plugins: vec!["registered".into(), "ghost".into()],
            integration_order: vec![],
            shared_config: serde_json::Value::Null,
            estimated_performance: PerformanceEstimate {
                latency_ms: 10.0,
                throughput_rps: 500.0,
                memory_usage_mb: 128.0,
                cpu_utilization_percent: 10.0,
            },
        };

        let system = reg.execute_composition(plan).await.unwrap();
        assert!(!system.system_health.overall_healthy);
    }

    #[test]
    fn requirement_to_capability_maps_correctly() {
        let enc = DynamicPluginRegistry::requirement_to_capability(
            &PluginRequirement::RequiresEncryption { min_key_size: Some(256) },
        );
        assert!(matches!(enc, PluginCapability::Encryption { .. }));

        let disc = DynamicPluginRegistry::requirement_to_capability(
            &PluginRequirement::RequiresServiceDiscovery,
        );
        assert!(matches!(disc, PluginCapability::ServiceDiscovery { .. }));

        let compute = DynamicPluginRegistry::requirement_to_capability(
            &PluginRequirement::RequiresCompute { min_cpu_cores: 2, min_memory_gb: 4 },
        );
        assert!(matches!(compute, PluginCapability::Compute { cpu_cores: 2, memory_gb: 4 }));

        let net = DynamicPluginRegistry::requirement_to_capability(
            &PluginRequirement::RequiresNetwork { min_bandwidth_mbps: 100, max_latency_ms: 5 },
        );
        assert!(matches!(net, PluginCapability::Network { bandwidth_mbps: 100, latency_ms: 5 }));
    }

    #[tokio::test]
    async fn composition_constraints_default_has_limits() {
        let c = CompositionConstraints::default();
        assert_eq!(c.max_latency_ms, Some(1000.0));
        assert_eq!(c.max_memory_mb, Some(1024.0));
        assert_eq!(c.max_plugins, Some(10));
    }

    #[tokio::test]
    async fn generate_combinations_single_and_pairs() {
        let reg = DynamicPluginRegistry::new();
        let plugins = vec!["a".into(), "b".into(), "c".into()];
        let combos = reg.generate_combinations(&plugins, &[]).await.unwrap();
        assert_eq!(combos.len(), 6); // 3 singles + 3 pairs
    }

    #[tokio::test]
    async fn generate_combinations_single_plugin() {
        let reg = DynamicPluginRegistry::new();
        let plugins = vec!["only".into()];
        let combos = reg.generate_combinations(&plugins, &[]).await.unwrap();
        assert_eq!(combos.len(), 1);
        assert_eq!(combos[0], vec!["only".to_string()]);
    }

    #[tokio::test]
    async fn performance_estimate_scales_with_plugins() {
        let reg = DynamicPluginRegistry::new();
        reg.register_plugin("p1".into(), vec![encryption_cap()], vec![])
            .await
            .unwrap();
        reg.register_plugin("p2".into(), vec![encryption_cap()], vec![])
            .await
            .unwrap();

        let plans = reg
            .discover_optimal_composition(
                "test",
                vec![encryption_cap()],
                CompositionConstraints::default(),
            )
            .await
            .unwrap();

        let single = plans.iter().find(|p| p.plugins.len() == 1).unwrap();
        let pair = plans.iter().find(|p| p.plugins.len() == 2).unwrap();

        assert!(pair.estimated_performance.latency_ms > single.estimated_performance.latency_ms);
        assert!(pair.estimated_performance.throughput_rps < single.estimated_performance.throughput_rps);
    }
}
