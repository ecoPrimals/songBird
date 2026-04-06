// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
//! Federation setup discovers configuration at runtime via environment variables
//! OR accepts explicit configuration via dependency injection.
//! No hardcoded endpoints - all configuration is external and dynamic.
//!
//! ## Modern Rust Patterns
//!
//! This module follows modern Rust practices:
//! - Dependency injection via `FederationOptions`
//! - Builder pattern for test fixtures
//! - Zero global state coupling
//! - Fully concurrent and async-safe
//! - Clear Option handling for conditional federation
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

/// Federation configuration options for dependency injection
///
/// This allows tests to pass explicit configuration without modifying
/// global environment variables, enabling fully concurrent test execution.
#[derive(Debug, Clone, Default)]
pub struct FederationOptions {
    /// Enable federation (None = read from env)
    pub enabled: Option<bool>,
    /// Bootstrap node address (None = read from env)
    pub bootstrap_address: Option<String>,
    /// Rendezvous server URL (None = read from env)
    pub rendezvous_url: Option<String>,
    /// Node's public address (None = read from env or auto-detect)
    pub node_address: Option<String>,
    /// Node's port (None = read from env or use default)
    pub port: Option<u16>,
}

impl FederationOptions {
    /// Create options from environment variables (production use)
    #[must_use]
    pub fn from_env() -> Self {
        Self::default() // All None = read from env
    }

    /// Create options for testing with explicit values
    #[cfg(test)]
    #[must_use]
    pub fn for_testing() -> FederationOptionsBuilder {
        FederationOptionsBuilder::default()
    }
}

/// Builder for `FederationOptions` (test fixture pattern)
#[cfg(test)]
#[derive(Default)]
pub struct FederationOptionsBuilder {
    options: FederationOptions,
}

#[cfg(test)]
impl FederationOptionsBuilder {
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.options.enabled = Some(enabled);
        self
    }

    pub fn bootstrap_address(mut self, addr: impl Into<String>) -> Self {
        self.options.bootstrap_address = Some(addr.into());
        self
    }

    pub fn rendezvous_url(mut self, url: impl Into<String>) -> Self {
        self.options.rendezvous_url = Some(url.into());
        self
    }

    pub fn node_address(mut self, addr: impl Into<String>) -> Self {
        self.options.node_address = Some(addr.into());
        self
    }

    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.options.port = Some(port);
        self
    }

    #[must_use]
    pub fn build(self) -> FederationOptions {
        self.options
    }
}

/// Federation setup result containing coordinator and config
pub struct FederationSetup {
    pub coordinator: Option<Arc<FederationCoordinator>>,
    pub config: Option<FederationConfig>,
}

/// Setup federation coordinator and configuration
///
/// Creates the federation coordinator if federation is enabled.
/// Configuration can be provided explicitly (for tests) or read from
/// environment variables (for production).
///
/// # Modern Dependency Injection Pattern
///
/// ```rust,ignore
/// // Production: read from environment
/// let setup = setup_federation(&identity, state, FederationOptions::from_env())?;
///
/// // Tests: explicit configuration (zero global state!)
/// let setup = setup_federation(
///     &identity,
///     state,
///     FederationOptions::for_testing()
///         .enabled(true)
///         .bootstrap_address("http://localhost:8000")
///         .build()
/// )?;
/// ```
///
/// # Zero Hardcoding
///
/// All configuration comes from:
/// 1. Explicit options (dependency injection)
/// 2. Environment variables (if options are None)
/// 3. Auto-detection (for IP/port)
///
/// # Returns
///
/// - `Some(coordinator, config)` if federation enabled
/// - `None` if federation disabled (standalone mode)
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn setup_federation(
    node_identity: &NodeIdentity,
    federation_state: Arc<FederationState>,
    options: FederationOptions,
) -> Result<FederationSetup> {
    // Resolve federation enabled from options OR env
    let enabled =
        options.enabled.unwrap_or_else(|| SafeEnv::get_bool("SONGBIRD_FEDERATION_ENABLED", false));

    if !enabled {
        info!("🏠 Running in standalone mode (federation disabled)");
        return Ok(FederationSetup {
            coordinator: None,
            config: None,
        });
    }

    info!("🌐 Federation mode enabled");

    // Resolve node address from options OR env OR auto-detect
    let node_address = options.node_address.unwrap_or_else(|| {
        SafeEnv::get_or_default(
            "SONGBIRD_NODE_ADDRESS",
            detect_primary_ip().unwrap_or_else(|| songbird_types::constants::LOCALHOST.to_string()),
        )
    });

    // Resolve port from options OR env OR default
    let port = options.port.unwrap_or_else(|| {
        SafeEnv::get_or_default(
            "SONGBIRD_PORT",
            songbird_config::defaults::ports::orchestrator_port().to_string(),
        )
        .parse::<u16>()
        .unwrap_or_else(|_| songbird_config::defaults::ports::orchestrator_port())
    });

    // Build self registration using STABLE node_id
    // This ensures the node has consistent identity across restarts
    let self_registration = NodeRegistration {
        node_id: node_identity.node_id.to_string(),
        node_name: node_identity.node_name.clone(),
        node_address: format!("{node_address}:{port}"),
        endpoints: None, // Will be populated in start() after we know the actual port
        capabilities: vec!["orchestrator".to_string()],
        cpu_cores: std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
        memory_gb: detect_memory_gb(),
        gpu_model: SongbirdOrchestrator::detect_gpu(),
        storage_gb: SongbirdOrchestrator::detect_storage_capacity(),
        status: NodeStatus::Active,
        joined_at: chrono::Utc::now(),
        last_heartbeat: chrono::Utc::now(),
    };

    // Resolve bootstrap address from options OR env
    let bootstrap_address = options
        .bootstrap_address
        .or_else(|| SafeEnv::get_required("SONGBIRD_BOOTSTRAP_ADDRESS").ok());

    // Resolve rendezvous URL from options OR env
    let rendezvous_url =
        options.rendezvous_url.or_else(|| SafeEnv::get_required("SONGBIRD_RENDEZVOUS_URL").ok());

    // Create federation config
    let config = FederationConfig {
        enabled: true,
        bootstrap_address,
        self_registration: Some(self_registration),
        heartbeat_interval_secs: 30,
        node_timeout_secs: 60,
        rendezvous_url,
        discovery_mode: None, // Auto-detect based on security provider availability
    };

    // Log bootstrap if configured
    if let Some(ref bootstrap) = config.bootstrap_address {
        info!("🔗 Will join federation via bootstrap: {}", bootstrap);
    }

    // Create coordinator with state (with_state is now async and returns Result)
    let coordinator = FederationCoordinator::with_state(Arc::clone(&federation_state)).await?;

    Ok(FederationSetup {
        coordinator: Some(Arc::new(coordinator)),
        config: Some(config),
    })
}

/// Detect available memory in GB via `/proc/meminfo` (pure Rust).
fn detect_memory_gb() -> usize {
    songbird_types::sys_metrics::total_memory_gb().max(16)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_detect_memory_gb_returns_reasonable_value() {
        let memory = detect_memory_gb();
        // Should be at least 1GB and less than 1TB (reasonable bounds)
        assert!(memory >= 1, "Memory should be at least 1GB");
        assert!(memory < 1024, "Memory should be less than 1TB");
    }

    #[tokio::test]
    async fn test_federation_setup_standalone_mode() {
        // Modern pattern: explicit config via dependency injection
        // NO global state modification - fully concurrent!
        let options = FederationOptions::for_testing().enabled(false).build();

        let node_identity = NodeIdentity::new_or_load(None).expect("Failed to load identity");
        let federation_state = Arc::new(FederationState::new("test".to_string()));

        let result = setup_federation(&node_identity, federation_state, options).await;
        assert!(result.is_ok());

        let setup = result.unwrap();
        assert!(setup.coordinator.is_none(), "Coordinator should be None in standalone mode");
        assert!(setup.config.is_none(), "Config should be None in standalone mode");
    }

    #[tokio::test]
    async fn test_federation_setup_enabled() {
        // Modern pattern: explicit config via builder
        // Zero coupling to global environment!
        let options = FederationOptions::for_testing()
            .enabled(true)
            .bootstrap_address("http://localhost:8000")
            .rendezvous_url("http://localhost:8001")
            .node_address("127.0.0.1")
            .port(8080)
            .build();

        let node_identity = NodeIdentity::new_or_load(None).expect("Failed to load identity");
        let federation_state = Arc::new(FederationState::new("test".to_string()));

        let result = setup_federation(&node_identity, federation_state, options).await;
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

        // Verify bootstrap and rendezvous from options
        assert_eq!(config.bootstrap_address, Some("http://localhost:8000".to_string()));
        assert_eq!(config.rendezvous_url, Some("http://localhost:8001".to_string()));
    }

    #[tokio::test]
    async fn test_federation_setup_uses_stable_identity() {
        // Modern pattern: dependency injection
        // Multiple calls in parallel would work fine!
        let options = FederationOptions::for_testing()
            .enabled(true)
            .bootstrap_address("http://localhost:8000")
            .rendezvous_url("http://localhost:8001")
            .node_address("127.0.0.1")
            .port(8080)
            .build();

        let node_identity = NodeIdentity::new_or_load(None).expect("Failed to load identity");
        let federation_state = Arc::new(FederationState::new("test".to_string()));

        let setup1 =
            setup_federation(&node_identity, Arc::clone(&federation_state), options.clone())
                .await
                .expect("First setup should succeed");
        let setup2 = setup_federation(&node_identity, Arc::clone(&federation_state), options)
            .await
            .expect("Second setup should succeed");

        // Both should have the same node_id (stable)
        let id1 = setup1
            .config
            .as_ref()
            .and_then(|c| c.self_registration.as_ref())
            .map(|r| r.node_id.clone())
            .expect("Expected federation config with node_id");
        let id2 = setup2
            .config
            .as_ref()
            .and_then(|c| c.self_registration.as_ref())
            .map(|r| r.node_id.clone())
            .expect("Expected federation config with node_id");
        assert_eq!(id1, id2, "Node ID should be stable across multiple setups");
    }
}
