// Module imports
//! Core traits for the Songbird Orchestrator
//!
//! This module defines the fundamental traits that enable universal service orchestration
//! across different project types and deployment environments.

pub mod communication;
pub mod config;
pub mod discovery;
pub mod feature_flags;
pub mod health;
pub mod hooks;
pub mod load_balancer;
pub mod observability;
pub mod resource_management;
pub mod service;
pub mod validation;
// Re-export all trait types
pub use communication::CommunicationLayer;
pub use config::ConfigProvider;
pub use discovery::{ServiceDiscovery, ServiceEvent, ServiceQuery};
// Re-export core traits for easy access
// pub use health::{HealthCheck, HealthMonitor, HealthStatus}; // Commented out due to conflicts
pub use load_balancer::LoadBalancer;
pub use service::{ServiceInfo, ServiceStatus};
// Re-export new trait modules
pub use feature_flags::*;
pub use hooks::*;
pub use observability::*;
pub use resource_management::*;
pub use validation::*;
// Re-export health state from observability
// pub use crate::health::HealthState; // Commented out - module doesn't exist
pub use songbird_observability::HealthStatus as ObservabilityHealthStatus;

// Health check trait definition
use async_trait::async_trait;
use songbird_errors::SongbirdResult;

/// Health check trait for services
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform health check
    async fn check_health(&self) -> SongbirdResult<HealthStatus>;

    /// Get health check name
    fn health_check_name(&self) -> &str;
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Health monitor trait
#[async_trait]
pub trait HealthMonitor: Send + Sync {
    /// Add health check
    async fn add_health_check(&mut self, check: Box<dyn HealthCheck>);

    /// Remove health check
    async fn remove_health_check(&mut self, name: &str);

    /// Run all health checks
    async fn run_all_checks(&self) -> SongbirdResult<Vec<(String, bool)>>;
}

/// Health state for backward compatibility
pub type HealthState = HealthStatus;

/// Composable Plugin System for Dynamic Service Integration
///
/// This trait enables services to act as "lego blocks" that can be
/// dynamically discovered and composed without requiring static configuration.
/// Perfect for scenarios where you might have 8+ projects that need to work
/// in any combination on the fly.
#[async_trait]
pub trait ComposablePlugin: Send + Sync {
    /// Unique plugin identifier (e.g., "beardog-encryption", "toadstool-compute")
    fn plugin_id(&self) -> &str;

    /// Plugin capabilities - what this plugin can provide
    fn capabilities(&self) -> Vec<PluginCapability>;

    /// Plugin requirements - what this plugin needs from other plugins
    fn requirements(&self) -> Vec<PluginRequirement>;

    /// Check if this plugin can integrate with another plugin
    fn can_integrate_with(&self, other_id: &str, other_capabilities: &[PluginCapability]) -> bool;

    /// Dynamically integrate with another plugin at runtime
    async fn integrate_with(
        &mut self,
        other_id: &str,
        other_capabilities: &[PluginCapability],
    ) -> SongbirdResult<IntegrationResult>;

    /// Get plugin configuration schema (for dynamic UI generation)
    fn config_schema(&self) -> serde_json::Value;

    /// Apply configuration dynamically
    fn apply_config(&mut self, config: serde_json::Value) -> SongbirdResult<()>;

    /// Health check for this plugin
    async fn health_check(&self) -> PluginHealth;
}

/// Plugin capability - what a plugin can provide
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginCapability {
    /// Encryption services
    Encryption { algorithms: Vec<String> },
    /// Service discovery
    ServiceDiscovery { protocols: Vec<String> },
    /// Load balancing
    LoadBalancing { strategies: Vec<String> },
    /// Gaming protocol bridging
    GamingBridge { protocols: Vec<String> },
    /// Compute resources
    Compute { cpu_cores: u32, memory_gb: u32 },
    /// Storage services
    Storage {
        capacity_gb: u64,
        storage_type: String,
    },
    /// Network services
    Network {
        bandwidth_mbps: u32,
        latency_ms: u32,
    },
    /// Custom capability
    Custom {
        name: String,
        attributes: std::collections::HashMap<String, String>,
    },
}

impl std::hash::Hash for PluginCapability {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            PluginCapability::Encryption { algorithms } => {
                "encryption".hash(state);
                algorithms.hash(state);
            }
            PluginCapability::ServiceDiscovery { protocols } => {
                "service_discovery".hash(state);
                protocols.hash(state);
            }
            PluginCapability::LoadBalancing { strategies } => {
                "load_balancing".hash(state);
                strategies.hash(state);
            }
            PluginCapability::GamingBridge { protocols } => {
                "gaming_bridge".hash(state);
                protocols.hash(state);
            }
            PluginCapability::Compute {
                cpu_cores,
                memory_gb,
            } => {
                "compute".hash(state);
                cpu_cores.hash(state);
                memory_gb.hash(state);
            }
            PluginCapability::Storage {
                capacity_gb,
                storage_type,
            } => {
                "storage".hash(state);
                capacity_gb.hash(state);
                storage_type.hash(state);
            }
            PluginCapability::Network {
                bandwidth_mbps,
                latency_ms,
            } => {
                "network".hash(state);
                bandwidth_mbps.hash(state);
                latency_ms.hash(state);
            }
            PluginCapability::Custom {
                name,
                attributes: _,
            } => {
                "custom".hash(state);
                name.hash(state);
                // We don't hash attributes since HashMap doesn't implement Hash
                // This means two Custom capabilities with the same name but different
                // attributes will hash to the same value, which is acceptable for our use case
            }
        }
    }
}

/// Plugin requirement - what a plugin needs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginRequirement {
    /// Requires encryption
    RequiresEncryption { min_key_size: Option<u32> },
    /// Requires service discovery
    RequiresServiceDiscovery,
    /// Requires compute resources
    RequiresCompute {
        min_cpu_cores: u32,
        min_memory_gb: u32,
    },
    /// Requires network connectivity
    RequiresNetwork {
        min_bandwidth_mbps: u32,
        max_latency_ms: u32,
    },
    /// Custom requirement
    Custom {
        name: String,
        constraints: std::collections::HashMap<String, String>,
    },
}

/// Result of plugin integration
#[derive(Debug, Clone)]
pub struct IntegrationResult {
    pub success: bool,
    pub integration_id: String,
    pub shared_capabilities: Vec<PluginCapability>,
    pub configuration_updates: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

/// Plugin health status
#[derive(Debug, Clone)]
pub struct PluginHealth {
    pub healthy: bool,
    pub status_message: String,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub performance_metrics: std::collections::HashMap<String, f64>,
}

/// Dynamic Plugin Registry for runtime composition
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Register a plugin dynamically
    async fn register_plugin(
        &self,
        plugin_id: String,
        capabilities: Vec<PluginCapability>,
        requirements: Vec<PluginRequirement>,
    ) -> SongbirdResult<String>;

    /// Discover plugins that can satisfy requirements
    async fn discover_plugins(
        &self,
        requirements: Vec<PluginRequirement>,
    ) -> SongbirdResult<Vec<String>>;

    /// Auto-compose plugins based on capabilities and requirements
    async fn auto_compose(
        &self,
        target_capabilities: Vec<PluginCapability>,
    ) -> SongbirdResult<CompositionPlan>;

    /// Execute a composition plan
    async fn execute_composition(&self, plan: CompositionPlan) -> SongbirdResult<ComposedSystem>;
}

/// Composition plan for combining plugins
#[derive(Debug, Clone)]
pub struct CompositionPlan {
    pub plugins: Vec<String>,
    pub integration_order: Vec<(String, String)>,
    pub shared_config: serde_json::Value,
    pub estimated_performance: PerformanceEstimate,
}

/// Performance estimate for a composition
#[derive(Debug, Clone)]
pub struct PerformanceEstimate {
    pub latency_ms: f64,
    pub throughput_rps: f64,
    pub memory_usage_mb: f64,
    pub cpu_utilization_percent: f64,
}

/// Composed system result
#[derive(Debug, Clone)]
pub struct ComposedSystem {
    pub system_id: String,
    pub active_plugins: Vec<String>,
    pub system_capabilities: Vec<PluginCapability>,
    pub system_health: SystemHealth,
}

/// System health for composed systems
#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub overall_healthy: bool,
    pub plugin_health: std::collections::HashMap<String, PluginHealth>,
    pub integration_health: std::collections::HashMap<String, bool>,
}
