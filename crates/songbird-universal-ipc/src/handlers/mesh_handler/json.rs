// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! JSON serialization helpers for mesh paths and endpoints.

use serde_json::{Value, json};
use songbird_onion_relay::mesh::{EndpointType, RelayEndpoint};

pub(super) fn path_to_json(path: &RelayEndpoint, found: bool) -> Value {
    let (path_type, address) = endpoint_to_strings(&path.endpoint_type);

    json!({
        "found": found,
        "target_node_id": path.node_id,
        "path_type": path_type,
        "priority": path.endpoint_type.priority(),
        "address": address,
        "estimated_latency_ms": path.latency.map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX)),
        "reachable": path.reachable
    })
}

pub(super) fn endpoint_to_strings(endpoint: &EndpointType) -> (&'static str, Option<String>) {
    match endpoint {
        EndpointType::Local {
            addr,
        } => ("local", Some(addr.to_string())),
        EndpointType::Overlay {
            addr,
            overlay_name,
        } => ("overlay", Some(format!("{overlay_name}://{addr}"))),
        EndpointType::Direct {
            addr,
        } => ("direct", Some(addr.to_string())),
        EndpointType::FamilyRelay {
            relay_node_id,
        } => ("family_relay", Some(relay_node_id.clone())),
        EndpointType::TorOnion {
            onion_addr,
        } => ("onion", Some(onion_addr.clone())),
    }
}
