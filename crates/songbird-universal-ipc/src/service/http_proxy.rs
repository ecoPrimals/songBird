// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-based HTTP proxy router for `http.proxy`.
//!
//! Routes inbound proxy requests to backend services by capability name,
//! injecting credentials and default headers from environment configuration.

use std::collections::HashMap;
use std::sync::RwLock;
use tracing::info;

/// Backend protocol for capability routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendProtocol {
    /// Standard HTTP proxy (forward request as-is to HTTP backend)
    Http,
    /// JSON-RPC via IPC — translate HTTP path to JSON-RPC method, route via UDS
    JsonRpcIpc,
}

/// A registered proxy route for a capability.
#[derive(Debug, Clone)]
pub struct ProxyRoute {
    /// Base URL of the backend (e.g. `http://localhost:8000`)
    pub base_url: String,
    /// Backend protocol (HTTP forward proxy vs JSON-RPC IPC translation)
    pub protocol: BackendProtocol,
    /// Default headers merged into every proxied request
    pub default_headers: HashMap<String, String>,
    /// Environment variable name holding the API key (injected as Bearer token)
    pub api_key_env: Option<String>,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

/// Capability-based proxy router.
///
/// Maps capability names to backend route configurations.
/// Thread-safe for runtime registration via `register()`.
pub struct CapabilityProxyRouter {
    routes: RwLock<HashMap<String, ProxyRoute>>,
}

impl CapabilityProxyRouter {
    /// Create an empty router (routes registered at runtime or from config).
    #[must_use]
    pub fn new() -> Self {
        Self {
            routes: RwLock::new(HashMap::new()),
        }
    }

    /// Create a router pre-populated from `SONGBIRD_PROXY_ROUTES` env var.
    ///
    /// Format: `capability=url,capability=url,...`
    /// Example: `jupyter=http://localhost:8000,inference=http://localhost:11434`
    #[must_use]
    pub fn from_env() -> Self {
        let router = Self::new();

        if let Ok(routes_env) = songbird_process_env::var("SONGBIRD_PROXY_ROUTES") {
            for entry in routes_env.split(',') {
                let entry = entry.trim();
                if let Some((cap, url)) = entry.split_once('=') {
                    let cap = cap.trim();
                    let url = url.trim();
                    if !cap.is_empty() && !url.is_empty() {
                        let (protocol, base_url) = if url.starts_with("jsonrpc://") {
                            (
                                BackendProtocol::JsonRpcIpc,
                                url.strip_prefix("jsonrpc://").unwrap_or(url).to_string(),
                            )
                        } else {
                            (BackendProtocol::Http, url.to_string())
                        };
                        let protocol_label = match protocol {
                            BackendProtocol::Http => "HTTP",
                            BackendProtocol::JsonRpcIpc => "JSON-RPC/IPC",
                        };
                        router.register(
                            cap,
                            ProxyRoute {
                                base_url,
                                protocol,
                                default_headers: HashMap::new(),
                                api_key_env: None,
                                timeout_ms: 30_000,
                            },
                        );
                        info!("http.proxy route registered: {cap} → {url} ({protocol_label})");
                    }
                }
            }
        }

        router
    }

    /// Register a capability route.
    pub fn register(&self, capability: &str, route: ProxyRoute) {
        #[expect(clippy::unwrap_used, reason = "RwLock poisoning is unrecoverable")]
        self.routes.write().unwrap().insert(capability.to_string(), route);
    }

    /// Look up a route by capability name.
    #[must_use]
    pub fn route(&self, capability: &str) -> Option<ProxyRoute> {
        #[expect(clippy::unwrap_used, reason = "RwLock poisoning is unrecoverable")]
        self.routes.read().unwrap().get(capability).cloned()
    }

    /// List all registered capabilities.
    #[must_use]
    pub fn list_capabilities(&self) -> Vec<String> {
        #[expect(clippy::unwrap_used, reason = "RwLock poisoning is unrecoverable")]
        self.routes.read().unwrap().keys().cloned().collect()
    }
}

impl Default for CapabilityProxyRouter {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn register_and_route_capability() {
        let router = CapabilityProxyRouter::new();
        router.register(
            "jupyter",
            ProxyRoute {
                base_url: String::from("http://localhost:8000"),
                protocol: BackendProtocol::Http,
                default_headers: HashMap::new(),
                api_key_env: None,
                timeout_ms: 30_000,
            },
        );

        let route = router.route("jupyter").unwrap();
        assert_eq!(route.base_url, "http://localhost:8000");
        assert_eq!(route.protocol, BackendProtocol::Http);
        assert_eq!(route.timeout_ms, 30_000);
    }

    #[test]
    fn unknown_capability_returns_none() {
        let router = CapabilityProxyRouter::new();
        assert!(router.route("nonexistent").is_none());
    }

    #[test]
    fn list_capabilities_returns_registered() {
        let router = CapabilityProxyRouter::new();
        router.register(
            "inference",
            ProxyRoute {
                base_url: String::from("http://localhost:11434"),
                protocol: BackendProtocol::Http,
                default_headers: HashMap::new(),
                api_key_env: Some(String::from("OLLAMA_API_KEY")),
                timeout_ms: 60_000,
            },
        );
        router.register(
            "jupyter",
            ProxyRoute {
                base_url: String::from("http://localhost:8000"),
                protocol: BackendProtocol::Http,
                default_headers: HashMap::new(),
                api_key_env: None,
                timeout_ms: 30_000,
            },
        );

        let caps = router.list_capabilities();
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&String::from("inference")));
        assert!(caps.contains(&String::from("jupyter")));
    }

    #[test]
    fn from_env_parses_and_empty_when_unset() {
        songbird_process_env::remove_var("SONGBIRD_PROXY_ROUTES");
        let empty_router = CapabilityProxyRouter::from_env();
        assert!(empty_router.list_capabilities().is_empty());

        songbird_process_env::set_var(
            "SONGBIRD_PROXY_ROUTES",
            "jupyter=http://localhost:8000,inference=http://localhost:11434",
        );

        let router = CapabilityProxyRouter::from_env();
        let jupyter = router.route("jupyter").unwrap();
        assert_eq!(jupyter.base_url, "http://localhost:8000");
        let inference = router.route("inference").unwrap();
        assert_eq!(inference.base_url, "http://localhost:11434");

        songbird_process_env::remove_var("SONGBIRD_PROXY_ROUTES");
    }

    #[test]
    fn from_env_parses_jsonrpc_scheme() {
        songbird_process_env::set_var(
            "SONGBIRD_PROXY_ROUTES",
            "network=jsonrpc://songbird.sock,jupyter=http://localhost:8000",
        );

        let router = CapabilityProxyRouter::from_env();
        let network = router.route("network").unwrap();
        assert_eq!(network.base_url, "songbird.sock");
        assert_eq!(network.protocol, BackendProtocol::JsonRpcIpc);

        let jupyter = router.route("jupyter").unwrap();
        assert_eq!(jupyter.base_url, "http://localhost:8000");
        assert_eq!(jupyter.protocol, BackendProtocol::Http);

        songbird_process_env::remove_var("SONGBIRD_PROXY_ROUTES");
    }
}
