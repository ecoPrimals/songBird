// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_config::capability_endpoints::CapabilityType;

/// Routing decision for a task
#[derive(Debug, Clone)]
pub enum RoutingDecision {
    /// Execute the task locally on this Songbird instance
    ExecuteLocally,

    /// Route to another Songbird instance in the federation
    RouteToSongbird {
        /// ID of the target node
        node_id: String,
        /// RPC endpoint of the target node
        endpoint: String,
    },

    /// Route to a registered service (Universal Port Authority)
    /// NEW: Modern routing via service registry
    RouteToRegisteredService {
        /// Service ID from registry
        service_id: String,
        /// Registered service display name (may be a primal name; routing uses capabilities)
        service_name: String,
        /// Full endpoint URL
        endpoint: String,
        /// Port assigned by UPA
        port: u16,
    },

    /// Route to a specialized capability provider
    RouteToCapability {
        /// Type of capability (Compute, Security, AI, Storage)
        capability_type: CapabilityType,
        /// Endpoint of the capability provider
        provider_endpoint: String,
    },

    /// Route to an external provider registered in capability registry
    RouteToExternalProvider {
        /// Provider ID
        provider_id: String,
        /// Full execution endpoint URL
        execution_endpoint: String,
        /// Capability being used
        capability_name: String,
    },
}
