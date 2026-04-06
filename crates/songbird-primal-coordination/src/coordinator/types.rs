// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::types::CapabilityType;
use std::sync::Arc;

/// Configuration for the coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Maximum connections per capability
    pub max_connections_per_capability: usize,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Health check interval in seconds
    pub health_check_interval_secs: u64,

    /// Enable connection pooling
    pub enable_pooling: bool,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_connections_per_capability: 10,
            connection_timeout_secs: 30,
            health_check_interval_secs: 60,
            enable_pooling: true,
        }
    }
}

/// Routed link between two capability endpoints (requester ↔ provider).
#[derive(Debug, Clone)]
pub struct MeshConnection {
    /// Unique mesh link identifier.
    pub id: String,
    /// Requester primal base URL.
    pub requester_endpoint: Arc<str>,
    /// Provider primal base URL.
    pub provider_endpoint: Arc<str>,
    /// Capability requested by the initiator.
    pub requester_capability: CapabilityType,
    /// Capability offered by the peer.
    pub provider_capability: CapabilityType,
}

/// Result of a status probe against one cached primal connection.
#[derive(Debug, Clone)]
pub struct PrimalHealthStatus {
    /// Capability key used to cache this connection.
    pub capability: Arc<str>,
    /// Primal base URL.
    pub endpoint: Arc<str>,
    /// Whether the status call reported healthy.
    pub healthy: bool,
    /// Reported version or `"unknown"` / `"error"` on failure.
    pub version: String,
}
