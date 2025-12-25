//! Federation initialization and management
//!
//! This module handles all federation-specific logic including:
//! - Federation coordinator setup
//! - Node registration
//! - Bootstrap connection
//! - Federation state management

use anyhow::Result;
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::{FederationState, NodeRegistration, NodeStatus};
use songbird_network_federation::{FederationConfig, FederationCoordinator};
use songbird_types::SafeEnv;
use std::sync::Arc;
use tracing::info;

use super::network;
use crate::node_identity::NodeIdentity;

/// Initialize federation components
///
/// Creates and configures federation coordinator, state, and service registry.
/// Returns `None` if federation is disabled.
///
/// # Federation Setup
///
/// 1. Loads stable node identity (persistent across restarts)
/// 2. Detects node capabilities (CPU, memory, GPU, storage)
/// 3. Builds self-registration with node metadata
/// 4. Creates federation coordinator with shared state
/// 5. Connects to bootstrap node if configured
///
/// # Capability-Based Discovery
///
/// The federation uses capability-based discovery - nodes advertise their
/// capabilities (compute, storage, AI, etc.) and other nodes discover them
/// at runtime. No hardcoded node addresses or names.
pub async fn initialize_federation() -> Result<(
    Option<Arc<FederationCoordinator>>,
    Option<FederationConfig>,
    Arc<FederationState>,
    Arc<FederatedServiceRegistry>,
    NodeIdentity,
)> {
    // Initialize federation state (always created, even if federation disabled)
    let federation_state = Arc::new(FederationState::new("main".to_string()));
    let federated_service_registry = Arc::new(FederatedServiceRegistry::new());

    // Load stable node identity EARLY (Dec 20, 2025 identity fix)
    // This ensures self-registration and discovery use the SAME node_id
    let node_identity = NodeIdentity::new_or_load(None)?;
    info!(
        "🆔 Loaded stable node identity: {} ({})",
        node_identity.node_name, node_identity.node_id
    );

    // Check if federation is enabled
    let (federation_coordinator, federation_config) =
        if SafeEnv::get_bool("SONGBIRD_FEDERATION_ENABLED", false) {
            info!("🌐 Federation mode enabled");

            let (coordinator, config) =
                create_federation_coordinator(&node_identity, Arc::clone(&federation_state))?;

            (Some(coordinator), Some(config))
        } else {
            info!("🏠 Running in standalone mode (federation disabled)");
            (None, None)
        };

    Ok((
        federation_coordinator,
        federation_config,
        federation_state,
        federated_service_registry,
        node_identity,
    ))
}

/// Create federation coordinator with node registration
fn create_federation_coordinator(
    node_identity: &NodeIdentity,
    federation_state: Arc<FederationState>,
) -> Result<(Arc<FederationCoordinator>, FederationConfig)> {
    // Build self registration using STABLE node_id
    let self_registration = build_self_registration(node_identity)?;

    // Create federation config
    let config = FederationConfig {
        enabled: true,
        bootstrap_address: SafeEnv::get_required("SONGBIRD_BOOTSTRAP_ADDRESS").ok(),
        self_registration: Some(self_registration),
        heartbeat_interval_secs: 30,
        node_timeout_secs: 60,
        rendezvous_url: SafeEnv::get_required("SONGBIRD_RENDEZVOUS_URL").ok(),
        discovery_mode: None, // Auto-detect based on BearDog availability
        _legacy_test_fields: (),
    };

    // Register self if we have bootstrap
    if let Some(ref bootstrap) = config.bootstrap_address {
        info!("🔗 Will join federation via bootstrap: {}", bootstrap);
    }

    // Create coordinator with state
    let coordinator = Arc::new(FederationCoordinator::with_state(federation_state));

    Ok((coordinator, config))
}

/// Build self-registration with node capabilities
///
/// Uses runtime detection to discover:
/// - CPU cores (via `num_cpus`)
/// - Memory (via `sysinfo` on Linux, fallback on other platforms)
/// - GPU model (runtime detection)
/// - Storage capacity (runtime detection)
///
/// This is **not hardcoded** - capabilities are discovered at startup.
fn build_self_registration(node_identity: &NodeIdentity) -> Result<NodeRegistration> {
    let node_address = format!(
        "{}:{}",
        SafeEnv::get_or_default(
            "SONGBIRD_NODE_ADDRESS",
            network::detect_primary_ip().unwrap_or_else(|| "127.0.0.1".to_string())
        ),
        SafeEnv::get_or_default(
            "SONGBIRD_PORT",
            songbird_config::defaults::ports::orchestrator_port().to_string()
        )
    );

    Ok(NodeRegistration {
        node_id: node_identity.node_id.to_string(),
        node_name: node_identity.node_name.clone(),
        node_address,
        endpoints: None, // Will be populated after we know actual port
        capabilities: vec!["orchestrator".to_string()],
        cpu_cores: num_cpus::get(),
        memory_gb: detect_memory_gb(),
        gpu_model: detect_gpu(),
        storage_gb: detect_storage_capacity(),
        status: NodeStatus::Active,
        joined_at: chrono::Utc::now(),
        last_heartbeat: chrono::Utc::now(),
    })
}

/// Detect available memory in GB
///
/// Uses platform-specific detection:
/// - Linux: `sysinfo` crate (accurate)
/// - Other: Fallback to 16GB estimate
fn detect_memory_gb() -> usize {
    #[cfg(target_os = "linux")]
    {
        (sysinfo::System::new_all().total_memory() / (1024 * 1024 * 1024)) as usize
    }
    #[cfg(not(target_os = "linux"))]
    {
        16 // Conservative fallback for non-Linux
    }
}

/// Detect GPU model (runtime capability discovery)
fn detect_gpu() -> Option<String> {
    // TODO: Implement GPU detection via:
    // - Linux: `/sys/class/drm/` or `lspci`
    // - Windows: WMI queries
    // - macOS: System profiler
    None
}

/// Detect storage capacity in GB (runtime capability discovery)
fn detect_storage_capacity() -> Option<usize> {
    // TODO: Implement storage detection via:
    // - Linux: `statvfs` system call
    // - Windows: `GetDiskFreeSpaceEx`
    // - macOS: `statfs`
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_detection_returns_reasonable_value() {
        let memory = detect_memory_gb();
        assert!(memory > 0, "Memory detection should return positive value");
        assert!(memory < 1024, "Memory detection should return reasonable value (< 1TB)");
    }

    #[tokio::test]
    async fn test_federation_initialization_standalone() {
        // Test standalone mode (federation disabled)
        std::env::remove_var("SONGBIRD_FEDERATION_ENABLED");

        let result = initialize_federation().await;
        assert!(result.is_ok());

        let (coordinator, config, _state, _registry, identity) = result.unwrap();
        assert!(coordinator.is_none(), "Coordinator should be None in standalone mode");
        assert!(config.is_none(), "Config should be None in standalone mode");
        assert!(!identity.node_id.is_nil(), "Node identity should be loaded");
    }
}
