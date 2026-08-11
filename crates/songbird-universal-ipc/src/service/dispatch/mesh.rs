// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]

use super::super::IpcServiceHandler;
use serde_json::Value;
use songbird_types::json_rpc_method::{JsonRpcMethod, MeshMethod};
use tracing::debug;

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
            dispatch_gossip_delegated(handler, "mesh.capabilities_announce", &params).await;
            handler.mesh_handler.handle_capabilities_announce(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::CapabilitiesRevoke) => {
            dispatch_gossip_delegated(handler, "mesh.capabilities_revoke", &params).await;
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
        JsonRpcMethod::Mesh(MeshMethod::ConnectivityCheck) => {
            handler.mesh_handler.handle_connectivity_check(params).await
        }
        JsonRpcMethod::Mesh(MeshMethod::Throughput) => {
            handler.mesh_handler.handle_throughput(params).await
        }
        other => Err(format!("Unknown method: {other}")),
    }
}

/// Forward gossip-concern methods to swarmVine when available (fire-and-forget).
///
/// Vertebrate evolution (Wave 157d): gossip propagation belongs to swarmVine.
/// songBird still runs its local handler (for backward compat and fallback),
/// but also injects the payload into swarmVine's gossip engine when reachable.
async fn dispatch_gossip_delegated(_handler: &IpcServiceHandler, method: &str, params: &Value) {
    #[cfg(unix)]
    {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let socket = super::super::swarmvine_gossip::discover_swarmvine_socket();
        let Some(socket_path) = socket else {
            debug!(target: "songbird::delegation", method, "swarmVine not available — gossip handled locally");
            return;
        };

        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "gossip.forward",
            "params": {
                "original_method": method,
                "payload": params,
            },
            "id": 1
        });

        match UnixStream::connect(&socket_path).await {
            Ok(stream) => {
                let (reader, mut writer) = stream.into_split();
                let msg = format!("{payload}\n");
                if writer.write_all(msg.as_bytes()).await.is_ok() {
                    let mut response = String::new();
                    let mut buf_reader = BufReader::new(reader);
                    let _ = buf_reader.read_line(&mut response).await;
                    debug!(target: "songbird::delegation", method, "forwarded to swarmVine");
                }
            }
            Err(e) => {
                debug!(target: "songbird::delegation", method, error = %e, "swarmVine unreachable — local fallback");
            }
        }
    }

    #[cfg(not(unix))]
    {
        let _ = params;
        debug!(target: "songbird::delegation", method, "swarmVine delegation not available on this platform");
    }
}
