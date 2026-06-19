// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Human-readable labels for routing decisions (logging, API responses).

use crate::core::routing::RoutingDecision;

/// Human-readable routing destination label (mirrors `submit_compute_task` mapping).
#[must_use]
pub fn format_compute_routed_destination(decision: &RoutingDecision) -> String {
    match decision {
        RoutingDecision::ExecuteLocally => String::from("local"),
        RoutingDecision::RouteToSongbird {
            node_id,
            ..
        } => format!("songbird:{node_id}"),
        RoutingDecision::RouteToRegisteredService {
            service_name,
            port,
            ..
        } => format!("service:{service_name}:{port}"),
        RoutingDecision::RouteToCapability {
            capability_type,
            provider_endpoint,
        } => format!("{capability_type:?}:{provider_endpoint}"),
        RoutingDecision::RouteToExternalProvider {
            provider_id,
            ..
        } => format!("external:{provider_id}"),
    }
}
