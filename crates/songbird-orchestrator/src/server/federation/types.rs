// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Federation Shared Types
//!
//! Common types and state used across federation endpoints

use serde::{Deserialize, Serialize};
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use std::sync::Arc;

use crate::core::registry::CapabilityRegistry;
use crate::trust::TrustEscalationManager;

/// Shared application state for federation
#[derive(Debug, Clone)]
pub struct FederationAppState {
    pub federation_state: Arc<FederationState>,
    pub service_registry: Arc<FederatedServiceRegistry>,
    pub capability_registry: Option<Arc<CapabilityRegistry>>,
    pub trust_manager: Option<Arc<TrustEscalationManager>>,
}

/// POST /api/federation/heartbeat - Send heartbeat
#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    pub node_id: String,
    #[allow(dead_code, reason = "heartbeat JSON fields reserved for federation telemetry")]
    pub timestamp: String,
    pub status: Option<String>,
    #[allow(dead_code, reason = "heartbeat JSON fields reserved for federation telemetry")]
    pub metrics: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct FederationHeartbeatResponse {
    pub acknowledged: bool,
    pub federation_status: String,
}
