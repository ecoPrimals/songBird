//! Core orchestrator types and state management
//!
//! This module contains the main `SongbirdOrchestrator` struct and its
//! fundamental operations.

use anyhow::Result;
use songbird_config::canonical::primals::{
    PrimalCapability, PrimalConfiguration, PrimalEndpoint, QosMetrics,
};
use songbird_discovery::anonymous_discovery::{
    AnonymousDiscoveryBroadcaster, AnonymousDiscoveryListener,
};
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::{FederationState, NodeRegistration};
use songbird_network_federation::{FederationConfig, FederationCoordinator};
use songbird_observability::ObservabilityManager;
use songbird_types::config::CanonicalSongbirdConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::trust::{TrustEscalationManager, TrustTimeouts};

/// Main orchestrator application
///
/// Coordinates all Songbird subsystems including federation, discovery,
/// observability, and service registry.
#[allow(dead_code)]
pub struct SongbirdOrchestrator {
    pub(super) _config: CanonicalSongbirdConfig,
    pub(super) _service_registry: Arc<FederatedServiceRegistry>,
    pub(super) service_registry: Arc<crate::service_registry::ServiceRegistry>,
    // gaming_manager: Arc<GamingManager>, // Temporarily disabled
    // federation_manager: Arc<CanonicalFederation>, // Temporarily disabled
    pub(super) federation_coordinator: Option<Arc<FederationCoordinator>>,
    pub(super) federation_config: Option<FederationConfig>,
    pub(super) federation_state: Arc<FederationState>,
    pub(super) federated_service_registry: Arc<FederatedServiceRegistry>,
    pub(super) observability_manager: Arc<ObservabilityManager>,
    // security_integration: Arc<UniversalSecurityIntegration>, // Temporarily disabled
    pub(super) trust_manager: Arc<TrustEscalationManager>,
    pub(super) discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,
    pub(super) shutdown_signal: tokio::sync::broadcast::Receiver<()>,
    pub(super) shutdown_sender: tokio::sync::broadcast::Sender<()>,
}

// Re-export the implementation
// The actual impl block will remain in mod.rs temporarily during migration
// to avoid breaking existing code. This will be moved in Phase 2.
