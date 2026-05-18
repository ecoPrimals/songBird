// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
use songbird_types::constants::LOCALHOST;
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
/// # Errors
///
/// Returns an error if the operation fails.
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

    // ✅ SMART REFACTORING (v5.22.0 - Jan 25, 2026): Use shared setup_federation
    // Eliminates code duplication with federation_setup module
    // Modern async pattern with dependency injection
    let setup = super::federation_setup::setup_federation(
        &node_identity,
        Arc::clone(&federation_state),
        super::federation_setup::FederationOptions::from_env(),
    )
    .await?;

    Ok((
        setup.coordinator,
        setup.config,
        federation_state,
        federated_service_registry,
        node_identity,
    ))
}

#[allow(dead_code, reason = "wires when federation startup path goes live")]
async fn create_federation_coordinator(
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
        discovery_mode: None, // Auto-detect based on security provider availability
    };

    // Register self if we have bootstrap
    if let Some(ref bootstrap) = config.bootstrap_address {
        info!("🔗 Will join federation via bootstrap: {}", bootstrap);
    }

    // Create coordinator with state (with_state is now async and returns Result)
    let coordinator = FederationCoordinator::with_state(federation_state).await?;

    Ok((Arc::new(coordinator), config))
}

#[allow(dead_code, reason = "called by create_federation_coordinator")]
fn build_self_registration(node_identity: &NodeIdentity) -> Result<NodeRegistration> {
    let node_address = format!(
        "{}:{}",
        SafeEnv::get_or_default(
            "SONGBIRD_NODE_ADDRESS",
            network::detect_primary_ip().unwrap_or_else(|| LOCALHOST.to_string())
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
        cpu_cores: std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
        memory_gb: detect_memory_gb(),
        gpu_model: detect_gpu(),
        storage_gb: detect_storage_capacity(),
        status: NodeStatus::Active,
        joined_at: chrono::Utc::now(),
        last_heartbeat: chrono::Utc::now(),
    })
}

#[allow(dead_code, reason = "called by build_self_registration")]
fn detect_memory_gb() -> usize {
    songbird_types::sys_metrics::total_memory_gb().max(16)
}

#[allow(dead_code, reason = "called by build_self_registration")]
fn detect_gpu() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        // Try NVIDIA GPU first
        if let Ok(entries) = std::fs::read_dir("/proc/driver/nvidia/gpus") {
            for entry in entries.flatten() {
                if let Ok(info) = std::fs::read_to_string(entry.path().join("information")) {
                    // Parse GPU model from NVIDIA information file
                    for line in info.lines() {
                        if line.starts_with("Model:") {
                            let model = line.trim_start_matches("Model:").trim();
                            return Some(format!("NVIDIA {model}"));
                        }
                    }
                }
            }
        }

        // Try AMD/Intel via DRM
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.file_name()?.to_str()?.starts_with("card") {
                    // Try to read vendor and device
                    if let Ok(vendor) = std::fs::read_to_string(path.join("device/vendor")) {
                        let vendor = vendor.trim();
                        if let Ok(device) = std::fs::read_to_string(path.join("device/device")) {
                            let device = device.trim();
                            return Some(format!("GPU {vendor}/{device}"));
                        }
                    }
                }
            }
        }
    }

    // Fallback: No GPU detected
    tracing::debug!("No GPU detected via platform-specific methods");
    None
}

#[allow(dead_code, reason = "called by build_self_registration")]
fn detect_storage_capacity() -> Option<usize> {
    let disks = songbird_types::sys_metrics::disk_info();
    let total_bytes: u64 = disks.iter().map(|d| d.total_bytes).sum();

    if total_bytes > 0 {
        let total_gb = (total_bytes / (1024 * 1024 * 1024)) as usize;
        tracing::debug!("Detected storage capacity: {} GB across {} disks", total_gb, disks.len());
        Some(total_gb)
    } else {
        tracing::warn!("No storage capacity detected");
        None
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
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
        // ✅ Concurrent-safe: Uses FederationOptions builder (no global state)
        // Tests standalone mode (federation disabled) via dependency injection
        use crate::app::federation_setup;

        let options = federation_setup::FederationOptions::for_testing().enabled(false).build();

        let node_identity = NodeIdentity::new_or_load(None).expect("Failed to load identity");
        let federation_state = Arc::new(FederationState::new("test".to_string()));

        let result: anyhow::Result<federation_setup::FederationSetup> =
            federation_setup::setup_federation(&node_identity, federation_state, options).await;
        assert!(result.is_ok());

        let setup = result.unwrap();
        assert!(setup.coordinator.is_none(), "Coordinator should be None in standalone mode");
        assert!(setup.config.is_none(), "Config should be None in standalone mode");
        assert!(!node_identity.node_id.is_nil(), "Node identity should be loaded");
    }
}
