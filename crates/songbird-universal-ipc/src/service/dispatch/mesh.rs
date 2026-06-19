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
        other => Err(format!("Unknown method: {other}")),
    }
}
