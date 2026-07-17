// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]

use super::super::IpcServiceHandler;
use serde_json::Value;
use songbird_types::json_rpc_method::{
    CapabilitiesMethod, DiscoveryMethod, IpcMethod, JsonRpcMethod, PeerMethod, RendezvousMethod,
};

pub(super) async fn dispatch_capability(
    handler: &IpcServiceHandler,
    method: JsonRpcMethod,
    params: Value,
) -> Result<Value, String> {
    match method {
        JsonRpcMethod::Ipc(IpcMethod::Register) => handler.handle_register(params).await,
        JsonRpcMethod::Ipc(IpcMethod::Resolve) => handler.handle_resolve(params).await,
        JsonRpcMethod::Ipc(IpcMethod::Discover) => handler.handle_discover(params).await,
        JsonRpcMethod::Ipc(IpcMethod::List) => handler.handle_list(params).await,
        JsonRpcMethod::Ipc(IpcMethod::Watch) => handler.handle_watch(params).await,
        JsonRpcMethod::Ipc(IpcMethod::RelayStats) => handler.handle_relay_stats(params).await,

        JsonRpcMethod::Capabilities(CapabilitiesMethod::Resolve) => {
            handler.handle_capability_resolve(params).await
        }
        JsonRpcMethod::Capabilities(CapabilitiesMethod::Call) => {
            handler.handle_capability_call(params).await
        }

        JsonRpcMethod::Discovery(DiscoveryMethod::Peers) => IpcServiceHandler::wrap_result(
            handler.discovery_handler.handle_list_peers(params).await,
            "Discovery peers failed",
        ),
        JsonRpcMethod::Discovery(DiscoveryMethod::Announce) => IpcServiceHandler::wrap_result(
            handler.discovery_handler.handle_announce(params).await,
            "Discovery announce failed",
        ),
        JsonRpcMethod::Discovery(DiscoveryMethod::ContentPeers) => IpcServiceHandler::wrap_result(
            handler.discovery_handler.handle_content_peers(params).await,
            "Discovery content_peers failed",
        ),
        JsonRpcMethod::Discovery(DiscoveryMethod::Topology) => IpcServiceHandler::wrap_result(
            handler.discovery_handler.handle_topology(params).await,
            "Discovery topology failed",
        ),
        JsonRpcMethod::Discovery(DiscoveryMethod::Health) => IpcServiceHandler::wrap_result(
            handler.discovery_handler.handle_health(params).await,
            "Discovery health failed",
        ),
        JsonRpcMethod::Discovery(DiscoveryMethod::Query) => IpcServiceHandler::wrap_result(
            handler.discovery_handler.handle_query(params).await,
            "Discovery query failed",
        ),
        JsonRpcMethod::Discovery(DiscoveryMethod::Bonds) => IpcServiceHandler::wrap_result(
            handler.discovery_handler.handle_bonds(params).await,
            "Discovery bonds failed",
        ),
        JsonRpcMethod::Rendezvous(RendezvousMethod::Register) => IpcServiceHandler::wrap_result(
            handler.rendezvous_handler.handle_register(params).await,
            "Rendezvous register failed",
        ),
        JsonRpcMethod::Rendezvous(RendezvousMethod::Lookup) => IpcServiceHandler::wrap_result(
            handler.rendezvous_handler.handle_lookup(params).await,
            "Rendezvous lookup failed",
        ),
        JsonRpcMethod::Peer(PeerMethod::Connect) => IpcServiceHandler::wrap_result(
            handler.peer_handler.handle_connect(params).await,
            "Peer connect failed",
        ),

        other => Err(format!("Unknown method: {other}")),
    }
}
