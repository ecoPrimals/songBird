// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]

use super::super::IpcServiceHandler;
use serde_json::Value;
use songbird_types::json_rpc_method::{JsonRpcMethod, MeshMethod};

pub(super) async fn dispatch_mesh(
    handler: &IpcServiceHandler,
    method: JsonRpcMethod,
    params: Value,
) -> Result<Value, String> {
    match method {
        JsonRpcMethod::Mesh(MeshMethod::Init) => handler.mesh_handler.handle_init(params).await,
        JsonRpcMethod::Mesh(MeshMethod::Status) => handler.mesh_handler.handle_status(params).await,
        JsonRpcMethod::Mesh(MeshMethod::FindPath) => {
            handler.mesh_handler.handle_find_path(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::Announce) => {
            handler.mesh_handler.handle_announce(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::Peers) => handler.mesh_handler.handle_peers(params).await,
        JsonRpcMethod::Mesh(MeshMethod::Topology) => {
            handler.mesh_handler.handle_topology(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::HealthCheck) => {
            handler.mesh_handler.handle_health_check(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::AutoDiscover) => {
            handler.mesh_handler.handle_auto_discover(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::ProbeLatency) => {
            handler.mesh_handler.handle_probe_latency(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::CapabilitiesAnnounce) => {
            handler.mesh_handler.handle_capabilities_announce(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::CapabilitiesRevoke) => {
            handler.mesh_handler.handle_capabilities_revoke(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::DiscoverRemotes) => {
            handler.mesh_handler.handle_discover_remotes(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::Mirror) => handler.mesh_handler.handle_mirror(params).await,
        JsonRpcMethod::Mesh(MeshMethod::Publish) => {
            handler.mesh_handler.handle_publish(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::Subscribe) => {
            handler.mesh_handler.handle_subscribe(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::Enroll) => handler.mesh_handler.handle_enroll(params).await,
        JsonRpcMethod::Mesh(MeshMethod::GateEnroll) => {
            handler.mesh_handler.handle_gate_enroll(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::PruneStale) => {
            handler.mesh_handler.handle_prune_stale(params).await
        }
        other => Err(format!("Unknown method: {other}")),
    }
}
