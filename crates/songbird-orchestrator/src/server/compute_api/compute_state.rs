// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared Axum state for the compute API (routers and active job registry).

use crate::core::registry::CapabilityRegistry;
use crate::core::routing::CapabilityRouter;
use crate::core::routing::enhanced_router::EnhancedCapabilityRouter;
use crate::service_registry::ServiceRegistry;
use songbird_config::capability_endpoints::CapabilityType;
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::compute_types::JobStatus;

/// State for compute API
#[derive(Clone)]
pub struct ComputeApiState {
    /// Capability router for intelligent task routing
    pub(super) router: Arc<CapabilityRouter>,
    /// Enhanced router with Universal Port Authority (optional, preferred)
    pub(super) enhanced_router: Option<Arc<EnhancedCapabilityRouter>>,
    /// Active job tracking
    pub(super) active_jobs: Arc<RwLock<HashMap<Uuid, JobStatus>>>,
}

impl ComputeApiState {
    /// Create new compute API state
    #[must_use]
    pub fn new(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
    ) -> Self {
        let router = Arc::new(CapabilityRouter::new(federation_state, service_registry));
        Self {
            router,
            enhanced_router: None,
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Test/embedder constructor: fixed capability endpoints (no `CAPABILITY_*` env vars).
    #[must_use]
    pub fn new_with_capability_endpoint_overrides(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
        overrides: HashMap<CapabilityType, String>,
    ) -> Self {
        let router = Arc::new(CapabilityRouter::with_capability_endpoint_overrides(
            federation_state,
            service_registry,
            overrides,
        ));
        Self {
            router,
            enhanced_router: None,
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create new compute API state with capability registry
    #[must_use]
    pub fn with_capability_registry(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
        capability_registry: Arc<CapabilityRegistry>,
    ) -> Self {
        let router = Arc::new(CapabilityRouter::with_capability_registry(
            federation_state,
            service_registry,
            capability_registry,
        ));
        Self {
            router,
            enhanced_router: None,
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create new compute API state with Enhanced Router (Universal Port Authority)
    /// This is the modern, preferred constructor that uses the Enhanced Router
    #[must_use]
    pub fn with_enhanced_router(
        federation_state: Arc<FederationState>,
        federated_service_registry: Arc<FederatedServiceRegistry>,
        service_registry: Arc<ServiceRegistry>,
        capability_registry: Option<Arc<CapabilityRegistry>>,
    ) -> Self {
        // Create enhanced router (modern approach with UPA)
        let enhanced_router = Arc::new(EnhancedCapabilityRouter::new(
            service_registry,
            Arc::clone(&federation_state),
            Arc::clone(&federated_service_registry),
        ));

        // Create legacy router for fallback
        let router = if let Some(cap_registry) = capability_registry {
            Arc::new(CapabilityRouter::with_capability_registry(
                federation_state,
                federated_service_registry,
                cap_registry,
            ))
        } else {
            Arc::new(CapabilityRouter::new(federation_state, federated_service_registry))
        };

        Self {
            router,
            enhanced_router: Some(enhanced_router),
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
