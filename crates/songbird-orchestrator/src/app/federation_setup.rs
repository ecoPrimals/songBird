//! Federation Setup Module
//!
//! Handles the setup and configuration of federation components:
//! - Federation coordinator creation
//! - Self-registration configuration
//! - Federation config building
//! - Hardware detection (CPU, GPU, storage)
//!
//! ## Zero Hardcoding Philosophy
//!
//! Federation setup discovers configuration at runtime via environment variables.
//! No hardcoded endpoints - all configuration is external and dynamic.
//!
//! ## Modern Rust Patterns
//!
//! This module follows modern Rust practices:
//! - Clear Option handling for conditional federation
//! - Safe environment variable access via SafeEnv
//! - Proper Arc wrapping after configuration complete

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use songbird_network_federation::state::{FederationState, NodeRegistration, NodeStatus};
use songbird_network_federation::{FederationConfig, FederationCoordinator};
use songbird_types::SafeEnv;

use crate::app::core::SongbirdOrchestrator;
use crate::app::network::detect_primary_ip;
use crate::node_identity::NodeIdentity;

/// Federation setup result containing coordinator and config
pub struct FederationSetup {
    pub coordinator: Option<Arc<FederationCoordinator>>,
    pub config: Option<FederationConfig>,
}

/// Setup federation coordinator and configuration
///
/// Creates the federation coordinator if federation is enabled via
/// `SONGBIRD_FEDERATION_ENABLED` environment variable. Discovers
/// all configuration at runtime - no hardcoding.
///
/// # Zero Hardcoding
///
/// All configuration comes from environment:
/// - `SONGBIRD_FEDERATION_ENABLED`: Enable/disable federation
/// - `SONGBIRD_BOOTSTRAP_ADDRESS`: Bootstrap node address
/// - `SONGBIRD_RENDEZVOUS_URL`: Rendezvous server URL
/// - `SONGBIRD_NODE_ADDRESS`: Node's public address
/// - `SONGBIRD_PORT`: Node's port
///
/// # Returns
///
/// - `Some(coordinator, config)` if federation enabled
/// - `None` if federation disabled (standalone mode)
pub fn setup_federation(
    node_identity: &NodeIdentity,
    federation_state: Arc<FederationState>,
) -> Result<FederationSetup> {
    if !SafeEnv::get_bool("SONGBIRD_FEDERATION_ENABLED", false) {
        info!("🏠 Running in standalone mode (federation disabled)");
        return Ok(FederationSetup {
            coordinator: None,
            config: None,
        });
    }

    info!("🌐 Federation mode enabled");

    // Build self registration using STABLE node_id
    // This ensures the node has consistent identity across restarts
    let self_registration = NodeRegistration {
        node_id: node_identity.node_id.to_string(),
        node_name: node_identity.node_name.clone(),
        node_address: format!(
            "{}:{}",
            SafeEnv::get_or_default(
                "SONGBIRD_NODE_ADDRESS",
                detect_primary_ip().unwrap_or_else(|| "127.0.0.1".to_string())
            ),
            SafeEnv::get_or_default(
                "SONGBIRD_PORT",
                songbird_config::defaults::ports::orchestrator_port().to_string()
            )
        ),
        endpoints: None, // Will be populated in start() after we know the actual port
        capabilities: vec!["orchestrator".to_string()],
        cpu_cores: num_cpus::get(),
        memory_gb: detect_memory_gb(),
        gpu_model: SongbirdOrchestrator::detect_gpu(),
        storage_gb: SongbirdOrchestrator::detect_storage_capacity(),
        status: NodeStatus::Active,
        joined_at: chrono::Utc::now(),
        last_heartbeat: chrono::Utc::now(),
    };

    // Create federation config
    let config = FederationConfig {
        enabled: true,
        bootstrap_address: SafeEnv::get_required("SONGBIRD_BOOTSTRAP_ADDRESS").ok(),
        self_registration: Some(self_registration),
        heartbeat_interval_secs: 30,
        node_timeout_secs: 60,
        rendezvous_url: SafeEnv::get_required("SONGBIRD_RENDEZVOUS_URL").ok(),
        discovery_mode: None, // Auto-detect based on BearDog availability
    };

    // Log bootstrap if configured
    if let Some(ref bootstrap) = config.bootstrap_address {
        info!("🔗 Will join federation via bootstrap: {}", bootstrap);
    }

    // Create coordinator with state
    let coordinator = Arc::new(FederationCoordinator::with_state(Arc::clone(
        &federation_state,
    )));

    Ok(FederationSetup {
        coordinator: Some(coordinator),
        config: Some(config),
    })
}

/// Detect available memory in GB
///
/// Platform-specific detection with safe fallback.
fn detect_memory_gb() -> usize {
    #[cfg(target_os = "linux")]
    {
        (sysinfo::System::new_all().total_memory() / (1024 * 1024 * 1024)) as usize
    }
    #[cfg(not(target_os = "linux"))]
    {
        16 // Fallback for non-Linux systems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_memory_gb_returns_reasonable_value() {
        let memory = detect_memory_gb();
        // Should be at least 1GB and less than 1TB (reasonable bounds)
        assert!(memory >= 1, "Memory should be at least 1GB");
        assert!(memory < 1024, "Memory should be less than 1TB");
    }

    #[test]
    fn test_federation_setup_standalone_mode() {
        // Without SONGBIRD_FEDERATION_ENABLED, should return None
        std::env::remove_var("SONGBIRD_FEDERATION_ENABLED");

        let node_identity = NodeIdentity::new_or_load(None).expect("Failed to load identity");
        let federation_state = Arc::new(FederationState::new("test".to_string()));

        let result = setup_federation(&node_identity, federation_state);
        assert!(result.is_ok());

        let setup = result.unwrap();
        assert!(setup.coordinator.is_none(), "Coordinator should be None in standalone mode");
        assert!(setup.config.is_none(), "Config should be None in standalone mode");
    }

    #[test]
    fn test_federation_setup_enabled() {
        // Set federation enabled
        std::env::set_var("SONGBIRD_FEDERATION_ENABLED", "true");

        let node_identity = NodeIdentity::new_or_load(None).expect("Failed to load identity");
        let federation_state = Arc::new(FederationState::new("test".to_string()));

        let result = setup_federation(&node_identity, federation_state);
        assert!(result.is_ok());

        let setup = result.unwrap();
        assert!(setup.coordinator.is_some(), "Coordinator should be Some when federation enabled");
        assert!(setup.config.is_some(), "Config should be Some when federation enabled");

        // Verify config
        let config = setup.config.unwrap();
        assert!(config.enabled, "Federation should be enabled in config");
        assert!(config.self_registration.is_some(), "Self registration should be present");

        // Verify self registration uses stable identity
        let self_reg = config.self_registration.unwrap();
        assert_eq!(self_reg.node_id, node_identity.node_id.to_string());
        assert_eq!(self_reg.node_name, node_identity.node_name);
        assert!(!self_reg.capabilities.is_empty(), "Should have capabilities");

        // Clean up
        std::env::remove_var("SONGBIRD_FEDERATION_ENABLED");
    }

    #[test]
    fn test_federation_setup_uses_stable_identity() {
        std::env::set_var("SONGBIRD_FEDERATION_ENABLED", "true");

        let node_identity = NodeIdentity::new_or_load(None).expect("Failed to load identity");
        let federation_state = Arc::new(FederationState::new("test".to_string()));

        let setup1 = setup_federation(&node_identity, Arc::clone(&federation_state))
            .expect("First setup should succeed");
        let setup2 = setup_federation(&node_identity, Arc::clone(&federation_state))
            .expect("Second setup should succeed");

        // Both should have the same node_id (stable)
        let id1 = setup1.config.unwrap().self_registration.unwrap().node_id;
        let id2 = setup2.config.unwrap().self_registration.unwrap().node_id;
        assert_eq!(id1, id2, "Node ID should be stable across multiple setups");

        std::env::remove_var("SONGBIRD_FEDERATION_ENABLED");
    }
}

