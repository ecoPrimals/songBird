// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Orchestrator Initialization Module
//!
//! Handles the initialization of all core components:
//! - Service registries (federated and universal)
//! - Observability manager
//! - Trust escalation manager
//! - Connection manager
//! - Federation state
//! - Node identity loading
//! - Discovery listener (pending configuration)
//!
//! ## Modern Rust Patterns
//!
//! This module follows the "build then Arc" pattern:
//! 1. Create components
//! 2. Configure them
//! 3. Wrap in Arc when configuration complete
//! 4. Return for orchestrator use
//!
//! The discovery listener is returned as `Option<AnonymousDiscoveryListener>` (not Arc'd)
//! to allow further configuration in `start()` before Arc wrapping.

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use songbird_config::CanonicalSongbirdConfig;
use songbird_discovery::anonymous::AnonymousDiscoveryListener;
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use songbird_observability::ObservabilityManager;

use crate::app::connection_manager::ConnectionManager;
use crate::node_identity::NodeIdentity;
use crate::trust::{TrustEscalationManager, TrustTimeouts};

/// Initialized components ready for orchestrator use
pub struct InitializedComponents {
    pub service_registry: Arc<FederatedServiceRegistry>,
    pub universal_service_registry: Arc<crate::service_registry::ServiceRegistry>,
    pub observability_manager: Arc<ObservabilityManager>,
    pub trust_manager: Arc<TrustEscalationManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub federation_state: Arc<FederationState>,
    pub federated_service_registry: Arc<FederatedServiceRegistry>,
    pub node_identity: NodeIdentity,
    /// Discovery listener WITHOUT Arc wrapping (will be configured + Arc'd in `start()`)
    pub discovery_listener_pending: Option<AnonymousDiscoveryListener>,
}

/// Initialize all core orchestrator components
///
/// This function creates and initializes all the foundational components
/// needed by the orchestrator. Components are returned with Arc wrapping
/// where appropriate, except for the discovery listener which is returned
/// pending further configuration.
///
/// # Modern Rust Pattern
///
/// The discovery listener is NOT Arc'd here because it needs further
/// configuration (`BirdSong`, stats) in `start()`. This follows the
/// "build then Arc" pattern to enable proper builder pattern usage.
///
/// # Returns
///
/// All initialized components ready for use by the orchestrator.
/// # Errors
///
/// Returns an error if the operation fails.
pub fn initialize_components(config: &CanonicalSongbirdConfig) -> Result<InitializedComponents> {
    info!("🔧 Initializing orchestrator components...");

    // Initialize service registry (using FederatedServiceRegistry)
    let service_registry = Arc::new(FederatedServiceRegistry::new());
    info!("   ✅ Federated service registry");

    // Initialize Universal Port Authority service registry (Dec 20, 2025)
    let universal_service_registry = Arc::new(crate::service_registry::ServiceRegistry::new());
    info!("   ✅ Universal service registry");

    // Initialize observability manager (no parameters)
    let observability_manager = Arc::new(ObservabilityManager::new());
    info!("   ✅ Observability manager");

    // Initialize trust escalation manager
    let trust_timeouts = TrustTimeouts {
        anonymous: config.federation.trust_timeouts.anonymous,
        capability: config.federation.trust_timeouts.capability,
        identity: config.federation.trust_timeouts.identity,
        hardware: config.federation.trust_timeouts.hardware,
    };
    let trust_manager = Arc::new(TrustEscalationManager::new(trust_timeouts, None));
    info!("   ✅ Trust escalation manager initialized");
    info!(
        "      Timeouts: Anonymous={}s, Capability={}s, Identity={}s, Hardware={}s",
        config.federation.trust_timeouts.anonymous,
        config.federation.trust_timeouts.capability,
        config.federation.trust_timeouts.identity,
        if config.federation.trust_timeouts.hardware == 0 {
            "never".to_string()
        } else {
            format!("{}s", config.federation.trust_timeouts.hardware)
        }
    );

    // Initialize progressive trust connection manager
    let connection_manager = Arc::new(ConnectionManager::new());
    info!("   ✅ Progressive trust connection manager");
    info!("      Trust Levels: None(0), Limited(1), Elevated(2), Highest(3)");
    info!("      Philosophy: Same family = hear the song, NOT enter the nest");

    // Initialize federation (if enabled)
    let federation_state = Arc::new(FederationState::new("main".to_string()));
    let federated_service_registry = Arc::new(FederatedServiceRegistry::new());
    info!("   ✅ Federation state and registry");

    // ✅ IDENTITY FIX (Dec 20, 2025): Load stable node identity EARLY
    // This ensures self-registration and discovery use the SAME node_id
    let node_identity = crate::node_identity::NodeIdentity::new_or_load(None)?;
    info!("   ✅ Stable node identity: {} ({})", node_identity.node_name, node_identity.node_id);

    // ✅ MODERN RUST PATTERN (v3.10.3 - Jan 6, 2026): Create listener WITHOUT Arc wrapping
    // We'll add BirdSong and stats in start(), THEN wrap in Arc.
    // This enables proper builder pattern usage and prevents the "two instances" bug.
    let discovery_listener_pending = if config.discovery.mode.is_enabled() {
        let listener = AnonymousDiscoveryListener::new(
            config.discovery.port,
            60, // 60 second peer timeout
        )
        .with_node_id(node_identity.node_id.to_string()); // v3.10.2 (Jan 5): Self-filtering

        info!(
            "   ✅ Discovery listener created (port {}, self-filtering: {})",
            config.discovery.port, node_identity.node_id
        );
        info!("      Configuration pending: Will add BirdSong + stats in start(), then Arc wrap");
        Some(listener)
    } else {
        info!("   ⏭️  Discovery disabled");
        None
    };

    info!("✅ All core components initialized");

    Ok(InitializedComponents {
        service_registry,
        universal_service_registry,
        observability_manager,
        trust_manager,
        connection_manager,
        federation_state,
        federated_service_registry,
        node_identity,
        discovery_listener_pending,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use songbird_config::CanonicalSongbirdConfig;

    #[test]
    fn test_initialization_creates_all_components() {
        // This test verifies that initialization doesn't panic
        // and returns all expected components
        let config = CanonicalSongbirdConfig::default();
        let result = initialize_components(&config);

        assert!(result.is_ok(), "Component initialization should succeed with default config");

        let components = result.unwrap();

        // Verify all Arc components have ref count of 1 (newly created)
        assert_eq!(
            Arc::strong_count(&components.service_registry),
            1,
            "Service registry should be newly created"
        );
        assert_eq!(
            Arc::strong_count(&components.trust_manager),
            1,
            "Trust manager should be newly created"
        );
        assert_eq!(
            Arc::strong_count(&components.connection_manager),
            1,
            "Connection manager should be newly created"
        );

        // Verify node identity loaded
        assert!(!components.node_identity.node_name.is_empty(), "Node identity should have a name");

        // Discovery listener depends on config
        if config.discovery.mode.is_enabled() {
            assert!(
                components.discovery_listener_pending.is_some(),
                "Discovery listener should be created when enabled"
            );
        }
    }

    #[test]
    fn test_initialization_with_discovery_disabled() {
        let mut config = CanonicalSongbirdConfig::default();
        config.discovery.mode =
            songbird_types::config::consolidated_canonical::discovery::DiscoveryMode::Disabled;

        let result = initialize_components(&config);
        assert!(result.is_ok());

        let components = result.unwrap();
        assert!(
            components.discovery_listener_pending.is_none(),
            "Discovery listener should not be created when disabled"
        );
    }

    #[test]
    fn test_initialization_idempotent() {
        // Calling initialization multiple times should work
        let config = CanonicalSongbirdConfig::default();

        let result1 = initialize_components(&config);
        let result2 = initialize_components(&config);

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Each initialization should create independent components
        let components1 = result1.unwrap();
        let components2 = result2.unwrap();

        // Node identities should be the SAME (loaded from file)
        assert_eq!(
            components1.node_identity.node_id, components2.node_identity.node_id,
            "Node identity should be stable across initializations"
        );
    }
}
