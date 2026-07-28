// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]

mod capability;
mod introspection;
mod mesh;
mod network;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

use super::IpcServiceHandler;
use crate::tower_atomic::JsonRpcHandler;
use capability::dispatch_capability;
use introspection::dispatch_introspection;
use mesh::dispatch_mesh;
use network::dispatch_network;
use serde_json::Value;
use songbird_types::json_rpc_method::{
    CapabilitiesMethod, HttpMethod, JsonRpcMethod, RouteMethod, TowerMethod,
};

/// Domain-routed JSON-RPC dispatch for [`IpcServiceHandler`].
pub trait IpcServiceDispatch: Send + Sync {
    /// Route a parsed JSON-RPC method to the appropriate domain handler.
    fn handle(
        &self,
        method: JsonRpcMethod,
        params: Value,
    ) -> impl std::future::Future<Output = Result<Value, String>> + Send;
}

impl IpcServiceDispatch for IpcServiceHandler {
    async fn handle(&self, method: JsonRpcMethod, params: Value) -> Result<Value, String> {
        match method {
            JsonRpcMethod::Primal(_)
            | JsonRpcMethod::Rpc(_)
            | JsonRpcMethod::DiscoverCapabilities
            | JsonRpcMethod::Health(_)
            | JsonRpcMethod::Capabilities(CapabilitiesMethod::List | CapabilitiesMethod::Methods)
            | JsonRpcMethod::Identity
            | JsonRpcMethod::IdentityGet(_)
            | JsonRpcMethod::Btsp(_)
            | JsonRpcMethod::Lifecycle(_) => dispatch_introspection(self, method, params).await,

            JsonRpcMethod::Http(HttpMethod::Request) => self.handle_http_request(params).await,
            JsonRpcMethod::Http(HttpMethod::Get) => self.handle_http_get(params).await,
            JsonRpcMethod::Http(HttpMethod::Post) => self.handle_http_post(params).await,
            JsonRpcMethod::Http(HttpMethod::Put) => self.handle_http_put(params).await,
            JsonRpcMethod::Http(HttpMethod::Delete) => self.handle_http_delete(params).await,
            JsonRpcMethod::Http(HttpMethod::Proxy) => self.handle_http_proxy(params).await,

            JsonRpcMethod::Route(RouteMethod::Add) => self.handle_route_add(params).await,
            JsonRpcMethod::Route(RouteMethod::Remove) => self.handle_route_remove(params).await,
            JsonRpcMethod::Route(RouteMethod::List) => self.handle_route_list().await,

            JsonRpcMethod::Tower(TowerMethod::Health) => self.handle_tower_health().await,
            JsonRpcMethod::Tower(TowerMethod::MeshStatus) => self.handle_tower_mesh_status().await,

            JsonRpcMethod::Ipc(_)
            | JsonRpcMethod::Capabilities(CapabilitiesMethod::Resolve | CapabilitiesMethod::Call)
            | JsonRpcMethod::Discovery(_)
            | JsonRpcMethod::Rendezvous(_)
            | JsonRpcMethod::Peer(_) => dispatch_capability(self, method, params).await,

            JsonRpcMethod::Mesh(_) => dispatch_mesh(self, method, params).await,

            JsonRpcMethod::Stun(_)
            | JsonRpcMethod::Igd(_)
            | JsonRpcMethod::Relay(_)
            | JsonRpcMethod::Birdsong(_)
            | JsonRpcMethod::Punch(_)
            | JsonRpcMethod::Onion(_)
            | JsonRpcMethod::Federation(_)
            | JsonRpcMethod::Tor(_) => dispatch_network(self, method, params).await,

            other => Err(format!("Unknown method: {other}")),
        }
    }
}

impl JsonRpcHandler for IpcServiceHandler {
    async fn handle(&self, method: &str, params: Value) -> Result<Value, String> {
        let method = match JsonRpcMethod::parse_ipc(method) {
            Ok(m) => m,
            Err(e) => return Err(e.into_message()),
        };
        <Self as IpcServiceDispatch>::handle(self, method, params).await
    }
}
