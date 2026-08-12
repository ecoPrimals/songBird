// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `route.*` — dynamic route configuration for the drawbridge proxy.
//!
//! Enables programmatic route management so services can self-configure
//! their reverse-proxy routes at registration time, eliminating manual
//! Caddy/nginx configuration.

use super::IpcServiceHandler;
use super::http_proxy::{BackendProtocol, ProxyRoute};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::info;

impl IpcServiceHandler {
    /// Handle `route.add` — register or update a proxy route.
    ///
    /// Params:
    /// ```json
    /// {
    ///   "capability": "jupyter",
    ///   "backend_url": "http://localhost:8888",
    ///   "protocol": "http",       // optional: "http" (default) | "jsonrpc_ipc"
    ///   "timeout_ms": 30000,      // optional
    ///   "api_key_env": "...",     // optional
    ///   "headers": { ... }        // optional default headers
    /// }
    /// ```
    pub(super) async fn handle_route_add(&self, params: Value) -> Result<Value, String> {
        let capability = params
            .get("capability")
            .and_then(|v| v.as_str())
            .ok_or_else(|| String::from("Missing 'capability' parameter"))?;

        let backend_url = params
            .get("backend_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| String::from("Missing 'backend_url' parameter"))?;

        let protocol = match params.get("protocol").and_then(|v| v.as_str()) {
            Some("jsonrpc_ipc") => BackendProtocol::JsonRpcIpc,
            _ => BackendProtocol::Http,
        };

        let timeout_ms = params.get("timeout_ms").and_then(Value::as_u64).unwrap_or(30_000);

        let api_key_env = params.get("api_key_env").and_then(Value::as_str).map(String::from);

        let default_headers: HashMap<String, String> = params
            .get("headers")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let route = ProxyRoute {
            base_url: backend_url.to_string(),
            protocol,
            default_headers,
            api_key_env,
            timeout_ms,
        };

        self.capability_router.register(capability, route);
        info!(capability, backend_url, "route.add: registered");

        Ok(json!({
            "status": "added",
            "capability": capability,
            "backend_url": backend_url,
        }))
    }

    /// Handle `route.remove` — unregister a proxy route by capability name.
    ///
    /// Params: `{ "capability": "jupyter" }`
    pub(super) async fn handle_route_remove(&self, params: Value) -> Result<Value, String> {
        let capability = params
            .get("capability")
            .and_then(|v| v.as_str())
            .ok_or_else(|| String::from("Missing 'capability' parameter"))?;

        let removed = self.capability_router.remove(capability);

        if removed {
            info!(capability, "route.remove: unregistered");
        }

        Ok(json!({
            "status": if removed { "removed" } else { "not_found" },
            "capability": capability,
        }))
    }

    /// Handle `route.list` — list all configured routes with details.
    #[allow(clippy::unused_async, reason = "async signature matches JsonRpcHandler dispatch table")]
    pub(super) async fn handle_route_list(&self) -> Result<Value, String> {
        let routes = self.capability_router.list_routes();

        let entries: Vec<Value> = routes
            .into_iter()
            .map(|(capability, route)| {
                json!({
                    "capability": capability,
                    "backend_url": route.base_url,
                    "protocol": match route.protocol {
                        BackendProtocol::Http => "http",
                        BackendProtocol::JsonRpcIpc => "jsonrpc_ipc",
                    },
                    "timeout_ms": route.timeout_ms,
                    "api_key_env": route.api_key_env,
                    "headers": route.default_headers,
                })
            })
            .collect();

        Ok(json!({ "routes": entries }))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use crate::service::http_proxy::CapabilityProxyRouter;

    fn test_router() -> CapabilityProxyRouter {
        CapabilityProxyRouter::new()
    }

    #[test]
    fn route_add_remove_list_cycle() {
        let router = test_router();

        router.register(
            "jupyter",
            ProxyRoute {
                base_url: String::from("http://localhost:8888"),
                protocol: BackendProtocol::Http,
                default_headers: HashMap::new(),
                api_key_env: None,
                timeout_ms: 30_000,
            },
        );

        assert_eq!(router.list_routes().len(), 1);
        assert!(router.remove("jupyter"));
        assert!(router.list_routes().is_empty());
        assert!(!router.remove("jupyter"));
    }

    #[test]
    fn list_routes_returns_details() {
        let router = test_router();
        router.register(
            "inference",
            ProxyRoute {
                base_url: String::from("http://localhost:11434"),
                protocol: BackendProtocol::Http,
                default_headers: HashMap::new(),
                api_key_env: Some(String::from("OLLAMA_KEY")),
                timeout_ms: 60_000,
            },
        );

        let routes = router.list_routes();
        assert_eq!(routes.len(), 1);
        let (cap, route) = &routes[0];
        assert_eq!(cap, "inference");
        assert_eq!(route.base_url, "http://localhost:11434");
        assert_eq!(route.timeout_ms, 60_000);
    }
}
