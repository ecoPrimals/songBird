// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Core orchestrator types and state management
//!
//! This module contains the main `SongbirdOrchestrator` struct and its
//! fundamental operations.

mod background_tasks;
mod broadcast;
mod commands;
mod ipc_server;
mod lifecycle;
mod peers;
mod security_provisioning;

use anyhow::Result;
use songbird_discovery::anonymous::AnonymousDiscoveryListener;
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use songbird_network_federation::{FederationConfig, FederationCoordinator};
use songbird_observability::ObservabilityManager;
use songbird_types::config::CanonicalSongbirdConfig;
use std::sync::Arc;
use tracing::info;

use super::connection_manager::ConnectionManager;

use crate::trust::TrustEscalationManager;

/// Main orchestrator application
///
/// Coordinates all Songbird subsystems including federation, discovery,
/// observability, and service registry.
#[expect(dead_code, reason = "reserved for future use: phased subsystem wiring")]
pub struct SongbirdOrchestrator {
    pub(super) _config: CanonicalSongbirdConfig,
    pub(super) _service_registry: Arc<FederatedServiceRegistry>,
    pub(super) service_registry: Arc<crate::service_registry::ServiceRegistry>,
    pub(super) federation_coordinator: Option<Arc<FederationCoordinator>>,
    pub(super) federation_config: Option<FederationConfig>,
    pub(super) federation_state: Arc<FederationState>,
    pub(super) federated_service_registry: Arc<FederatedServiceRegistry>,
    pub(super) observability_manager: Arc<ObservabilityManager>,
    pub(super) trust_manager: Arc<TrustEscalationManager>,
    pub(super) connection_manager: Arc<ConnectionManager>,

    // ✅ MODERN RUST PATTERN (v3.10.3 - Jan 6, 2026): Store listener WITHOUT Arc
    // This enables builder pattern usage (add BirdSong, stats) before Arc wrapping.
    // Arc wrapping happens in start() AFTER all configuration is complete.
    // This follows "build then Arc" pattern, not "Arc then try to mutate" anti-pattern.
    pub(super) discovery_listener_pending: Option<AnonymousDiscoveryListener>,
    pub(super) discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,

    pub(super) discovery_status_manager: Arc<songbird_discovery::DiscoveryStatusManager>,
    pub(super) ipc_server_handle: Option<tokio::task::JoinHandle<()>>,
    pub(super) broker_registry: Option<crate::ipc::universal_broker::SharedServiceRegistry>,
    pub(super) broker_mesh_handler: Option<Arc<songbird_universal_ipc::handlers::MeshHandler>>,
    pub(crate) shared_ipc_handler: Option<Arc<songbird_universal_ipc::service::IpcServiceHandler>>,
    pub(super) shutdown_signal: tokio::sync::broadcast::Receiver<()>,
    pub(super) shutdown_sender: tokio::sync::broadcast::Sender<()>,
    pub(crate) transport_registry: Arc<crate::app::transport_registry::TransportRegistry>,
}

// Implementation of SongbirdOrchestrator
// Moved from mod.rs in Phase 2 refactoring (Dec 25, 2025)
impl SongbirdOrchestrator {
    /// Create new orchestrator instance
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn new(config: CanonicalSongbirdConfig) -> Result<Self> {
        let (shutdown_sender, shutdown_signal) = tokio::sync::broadcast::channel(1);

        // Print secure-by-default configuration
        info!("🔒 Songbird Orchestrator - Secure by Default");

        // ✅ MODERN RUST PATTERN (v3.10.3 - Jan 6, 2026): Smart refactoring
        // Component initialization extracted to separate module for clarity and maintainability.
        // This follows single responsibility principle - initialization.rs handles component creation,
        // core.rs handles orchestration logic.
        info!(
            "   TLS: {} ({})",
            match config.security.tls.cert_policy {
                songbird_types::config::consolidated_canonical::security::TlsCertPolicy::ProvidedOnly => "✅ Enabled (provided certs)",
                songbird_types::config::consolidated_canonical::security::TlsCertPolicy::AutoGenerate => "✅ Enabled (auto-generate)",
                songbird_types::config::consolidated_canonical::security::TlsCertPolicy::AutoGenerateWithSans => "✅ Enabled (auto + SANs)",
            },
            match config.security.security_level {
                songbird_types::config::consolidated_canonical::security::SecurityLevel::Minimal => "minimal",
                songbird_types::config::consolidated_canonical::security::SecurityLevel::Standard => "standard",
                songbird_types::config::consolidated_canonical::security::SecurityLevel::Paranoid => "paranoid (2FA)",
            }
        );
        info!(
            "   Discovery: {} ({})",
            if config.discovery.mode.is_enabled() {
                "✅ Enabled"
            } else {
                "❌ Disabled"
            },
            match config.discovery.mode {
                songbird_types::config::consolidated_canonical::discovery::DiscoveryMode::Disabled => "disabled",
                songbird_types::config::consolidated_canonical::discovery::DiscoveryMode::Anonymous => "anonymous secure",
                songbird_types::config::consolidated_canonical::discovery::DiscoveryMode::CapabilityAware => "capability-aware",
                songbird_types::config::consolidated_canonical::discovery::DiscoveryMode::FullDisclosure => "full disclosure",
            }
        );
        info!(
            "   Federation: {} (trust: {})",
            if config.federation.cluster_name.is_some() {
                "✅ Enabled"
            } else {
                "❌ Disabled"
            },
            match config.federation.trust_escalation_policy {
                songbird_types::config::consolidated_canonical::federation::TrustEscalationPolicy::Disabled => "static",
                songbird_types::config::consolidated_canonical::federation::TrustEscalationPolicy::CapabilityOnly => "capability escalation",
                songbird_types::config::consolidated_canonical::federation::TrustEscalationPolicy::Progressive => "progressive escalation",
            }
        );
        info!("   Trust Model: Zero-trust with progressive escalation");
        info!("   Initial Trust: {} → Escalate on demand", config.federation.initial_trust_level);
        info!("   🌐 Songbird handles complexity, security automatic!");

        // ✅ SMART REFACTORING (v3.10.3 - Jan 6, 2026): Initialize all components
        // Extracted to initialization module for clarity, maintainability, and testability.
        // This reduces core.rs by ~220 lines while improving separation of concerns.
        let components = super::initialization::initialize_components(&config)?;

        // Destructure initialized components for use
        let service_registry = components.service_registry;
        let universal_service_registry = components.universal_service_registry;
        let observability_manager = components.observability_manager;
        let trust_manager = components.trust_manager;
        let connection_manager = components.connection_manager;
        let federation_state = components.federation_state;
        let federated_service_registry = components.federated_service_registry;
        let node_identity = components.node_identity;
        let discovery_listener_pending = components.discovery_listener_pending;

        // ✅ SMART REFACTORING (v3.10.3 - Jan 6, 2026): Setup federation coordinator
        // Extracted to federation_setup module for clarity and testability.
        // This reduces core.rs by ~65 lines while improving separation of concerns.
        //
        // ✅ MODERN ASYNC PATTERN (v5.22.0 - Jan 25, 2026): Dependency injection
        // Production reads from environment via FederationOptions::from_env()
        // Tests pass explicit config - zero global state coupling!
        let federation_setup = super::federation_setup::setup_federation(
            &node_identity,
            Arc::clone(&federation_state),
            super::federation_setup::FederationOptions::from_env(),
        )
        .await?;
        let federation_coordinator = federation_setup.coordinator;
        let federation_config = federation_setup.config;

        // Security provider setup: capability-based runtime discovery.
        // Non-fatal — songbird degrades to unsigned/non-TLS operation when no
        // security provider is available (SB-STARTUP-01 fix).
        let _security_integration = match super::security_setup::setup_security().await {
            Ok(integration) => Some(integration),
            Err(e) => {
                tracing::warn!(
                    "⚠️  Security provider not available — operating in degraded mode: {e}"
                );
                tracing::warn!(
                    "   Set SONGBIRD_SECURITY_PROVIDER or SECURITY_ENDPOINT to enable TLS/signing"
                );
                None
            }
        };

        // Initialize discovery status manager for observability (Jan 5, 2026)
        let discovery_status_manager = Arc::new(songbird_discovery::DiscoveryStatusManager::new(
            config.discovery.mode.is_enabled(),
            format!("{:?}", config.discovery.mode),
            config.discovery.port,
            Some(songbird_types::defaults::network::ecosystem_discovery_multicast_addr()),
        ));

        Ok(Self {
            _config: config,
            _service_registry: service_registry,
            service_registry: universal_service_registry,
            federation_coordinator,
            federation_config,
            federation_state,
            federated_service_registry,
            observability_manager,
            trust_manager,
            connection_manager,
            discovery_listener_pending, // ✅ Holds non-Arc listener for configuration
            discovery_listener: None,   // ✅ Will be set in start() after full config
            discovery_status_manager,
            ipc_server_handle: None,   // Will be set in start()
            broker_registry: None,     // Set in stage_2 when broker starts
            broker_mesh_handler: None, // Set in stage_2 when broker starts
            shared_ipc_handler: None,  // Set in stage_2 before start_ipc_server
            transport_registry: Arc::new(crate::app::transport_registry::TransportRegistry::new()),
            shutdown_signal,
            shutdown_sender,
        })
    }

    /// Get federation state reference
    #[must_use]
    pub const fn federation_state(&self) -> &Arc<FederationState> {
        &self.federation_state
    }

    /// Get federated service registry reference
    #[must_use]
    pub const fn federated_service_registry(&self) -> &Arc<FederatedServiceRegistry> {
        &self.federated_service_registry
    }

    /// Get configuration reference
    #[must_use]
    pub const fn config(&self) -> &CanonicalSongbirdConfig {
        &self._config
    }

    /// Get service registry reference
    #[must_use]
    pub const fn service_registry(&self) -> &Arc<FederatedServiceRegistry> {
        &self._service_registry
    }
}

// Status, health check, and startup functions are now in their respective modules:
// - health::{OrchestratorStatus, HealthCheckReport, run_health_check}
// - startup::{start_orchestrator, Orchestrator}
// They are re-exported at the top of this module for backwards compatibility.

/// Shared `SONGBIRD_BROADCAST_ADDRESSES` mutex for all `discover_broadcast_addresses` tests.
#[cfg(test)]
pub(crate) mod broadcast_test_lock {
    use std::sync::{Mutex, OnceLock};

    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    pub fn guard() -> std::sync::MutexGuard<'static, ()> {
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
#[path = "../core_broadcast_tests.rs"]
mod discover_broadcast_tests;
