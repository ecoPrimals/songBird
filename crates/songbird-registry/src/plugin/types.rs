// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Plugin type definitions: capabilities, requirements, composition plans,
//! health, and lifecycle events.

use std::collections::HashMap;

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
