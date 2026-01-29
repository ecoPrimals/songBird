//! IPC Service - Songbird's JSON-RPC IPC Broker
//!
//! This module provides Songbird's IPC brokering service. Instead of other
//! primals importing `songbird-universal-ipc` as a library (which would violate
//! primal autonomy), they connect to Songbird's IPC service via JSON-RPC.
//!
//! ## TRUE PRIMAL Architecture
//!
//! **Problem**: Library embedding violates primal autonomy
//! **Solution**: Service-based architecture via JSON-RPC
//!
//! ```text
//! Other Primals (BearDog, Squirrel, etc.):
//!   - Use tokio::net::UnixStream (standard library!)
//!   - Connect to /primal/songbird
//!   - Call JSON-RPC methods for discovery
//!   - Connect directly to discovered services
//!   - ZERO Songbird code embedded!
//!
//! Songbird IPC Service (this module):
//!   - Maintains service registry
//!   - Provides discovery via JSON-RPC
//!   - Manages platform abstraction internally
//!   - Pure service - no code embedding!
//! ```

use crate::endpoint::NativeEndpoint;
use crate::handlers::discovery_handler::{DiscoveryHandler, PeerRegistry};
use crate::handlers::http_handler::{HttpHandler, HttpRequestParams};
use crate::handlers::stun_handler::StunHandler;
use crate::handlers::rendezvous_handler::RendezvousHandler;
use crate::handlers::peer_handler::PeerHandler;
use crate::handlers::http_rendezvous_client::HttpRendezvousClient;
use crate::handlers::udp_peer_connector::UdpPeerConnector;
use crate::registry::ServiceRegistry;
use crate::tower_atomic::JsonRpcHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// IPC service request parameters for registration
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterParams {
    pub primal_id: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
}

/// IPC service request parameters for resolution
#[derive(Debug, Clone, Deserialize)]
pub struct ResolveParams {
    pub primal_id: String,
}

/// IPC service request parameters for discovery
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoverParams {
    pub capability: String,
}

/// IPC service response for registration
#[derive(Debug, Clone, Serialize)]
pub struct RegisterResult {
    pub virtual_endpoint: String,
    pub registered_at: String,
}

/// IPC service response for resolution
#[derive(Debug, Clone, Serialize)]
pub struct ResolveResult {
    pub virtual_endpoint: String,
    pub native_endpoint: String,
    pub capabilities: Vec<String>,
}

/// IPC service response for discovery
#[derive(Debug, Clone, Serialize)]
pub struct DiscoverResult {
    pub providers: Vec<ProviderInfo>,
}

/// Provider information
#[derive(Debug, Clone, Serialize)]
pub struct ProviderInfo {
    pub primal_id: String,
    pub virtual_endpoint: String,
    pub native_endpoint: String,
    pub capabilities: Vec<String>,
}

/// IPC service response for listing
#[derive(Debug, Clone, Serialize)]
pub struct ListResult {
    pub services: Vec<ServiceInfo>,
}

/// Service information
#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    pub primal_id: String,
    pub virtual_endpoint: String,
    pub capabilities: Vec<String>,
}

/// Songbird IPC Service Handler
///
/// This handler provides IPC brokering as a JSON-RPC service,
/// allowing other primals to discover and connect to services
/// without embedding Songbird code.
///
/// **TRUE PRIMAL**: Zero code embedding, pure service protocol!
pub struct IpcServiceHandler {
    registry: Arc<RwLock<ServiceRegistry>>,
    http_handler: Arc<HttpHandler>,
    stun_handler: Arc<StunHandler>,
    discovery_handler: Arc<DiscoveryHandler>,
    rendezvous_handler: Arc<RendezvousHandler>,
    peer_handler: Arc<PeerHandler>,
}

impl IpcServiceHandler {
    /// Create a new IPC service handler
    ///
    /// ✅ DEEP DEBT COMPLIANT (Jan 29, 2026):
    /// - Real implementations (`HttpRendezvousClient`, `UdpPeerConnector`)
    /// - Mocks isolated to #[cfg(test)] only
    /// - Production-ready defaults
    pub fn new(registry: Arc<RwLock<ServiceRegistry>>) -> Self {
        let http_handler = Arc::new(HttpHandler::with_default_discovery());
        let stun_handler = Arc::new(StunHandler::new());
        let discovery_handler = Arc::new(DiscoveryHandler::new());
        
        // ✅ Production implementations (not mocks!)
        let rendezvous_handler = Arc::new(RendezvousHandler::new(Arc::new(HttpRendezvousClient::new())));
        let peer_handler = Arc::new(PeerHandler::new(Arc::new(UdpPeerConnector::new())));
        
        Self {
            registry,
            http_handler,
            stun_handler,
            discovery_handler,
            rendezvous_handler,
            peer_handler,
        }
    }

    /// Create with discovery peer registry (for connecting to orchestrator's listener)
    ///
    /// ✅ DEEP DEBT COMPLIANT (Jan 29, 2026):
    /// - Real implementations (`HttpRendezvousClient`, `UdpPeerConnector`)
    /// - Runtime peer discovery via `PeerRegistry` trait
    /// - Zero hardcoding
    pub fn with_discovery_registry(
        registry: Arc<RwLock<ServiceRegistry>>,
        peer_registry: Arc<dyn PeerRegistry>,
    ) -> Self {
        let http_handler = Arc::new(HttpHandler::with_default_discovery());
        let stun_handler = Arc::new(StunHandler::new());
        let discovery_handler = Arc::new(DiscoveryHandler::with_registry(peer_registry));
        
        // ✅ Production implementations (not mocks!)
        let rendezvous_handler = Arc::new(RendezvousHandler::new(Arc::new(HttpRendezvousClient::new())));
        let peer_handler = Arc::new(PeerHandler::new(Arc::new(UdpPeerConnector::new())));
        
        Self {
            registry,
            http_handler,
            stun_handler,
            discovery_handler,
            rendezvous_handler,
            peer_handler,
        }
    }

    /// Create with custom HTTP handler (for testing/DI)
    pub fn with_http_handler(
        registry: Arc<RwLock<ServiceRegistry>>,
        http_handler: Arc<HttpHandler>,
    ) -> Self {
        let stun_handler = Arc::new(StunHandler::new());
        let discovery_handler = Arc::new(DiscoveryHandler::new());
        
        // ✅ Production implementations (not mocks!)
        let rendezvous_handler = Arc::new(RendezvousHandler::new(Arc::new(HttpRendezvousClient::new())));
        let peer_handler = Arc::new(PeerHandler::new(Arc::new(UdpPeerConnector::new())));
        
        Self {
            registry,
            http_handler,
            stun_handler,
            discovery_handler,
            rendezvous_handler,
            peer_handler,
        }
    }

    /// Handle `ipc.register` method
    async fn handle_register(&self, params: Value) -> Result<Value, String> {
        let params: RegisterParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        info!("Registering primal: {} at {}", params.primal_id, params.endpoint);

        // Parse native endpoint
        let native_endpoint = if params.endpoint.starts_with('/') {
            // Unix socket path
            NativeEndpoint::UnixSocket(params.endpoint.into())
        } else if params.endpoint.starts_with("127.0.0.1:") || params.endpoint.contains(':') {
            // TCP localhost
            let port: u16 = params
                .endpoint
                .split(':')
                .nth(1)
                .and_then(|p| p.parse().ok())
                .ok_or_else(|| "Invalid TCP port".to_string())?;
            NativeEndpoint::TcpLocal(port)
        } else {
            return Err(format!("Invalid endpoint format: {}", params.endpoint));
        };

        // Register in registry
        let registry = self.registry.write().await;
        let virtual_endpoint = registry
            .register(&params.primal_id, native_endpoint, params.capabilities)
            .await
            .map_err(|e| format!("Registration failed: {e}"))?;

        let result = RegisterResult {
            virtual_endpoint: virtual_endpoint.path,
            registered_at: chrono::Utc::now().to_rfc3339(),
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `ipc.resolve` method
    async fn handle_resolve(&self, params: Value) -> Result<Value, String> {
        let params: ResolveParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        debug!("Resolving primal: {}", params.primal_id);

        // Get service entry from registry
        let registry = self.registry.read().await;
        let entry = registry
            .get_service(&params.primal_id)
            .await
            .ok_or_else(|| format!("Primal not found: {}", params.primal_id))?;

        let result = ResolveResult {
            virtual_endpoint: entry.virtual_endpoint.path,
            native_endpoint: entry.native_endpoint.display(),
            capabilities: entry.capabilities,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `ipc.discover` method
    async fn handle_discover(&self, params: Value) -> Result<Value, String> {
        let params: DiscoverParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        debug!("Discovering capability: {}", params.capability);

        // Discover from registry (returns virtual paths)
        let registry = self.registry.read().await;
        let virtual_paths = registry.find_by_capability(&params.capability).await;

        // Get full service entries for each path
        let mut provider_infos = Vec::new();
        for virtual_path in virtual_paths {
            // Extract service name from virtual path
            if let Some(name) = virtual_path.strip_prefix("/primal/") {
                if let Some(entry) = registry.get_service(name).await {
                    provider_infos.push(ProviderInfo {
                        primal_id: name.to_string(),
                        virtual_endpoint: entry.virtual_endpoint.path,
                        native_endpoint: entry.native_endpoint.display(),
                        capabilities: entry.capabilities,
                    });
                }
            }
        }

        let result = DiscoverResult {
            providers: provider_infos,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `ipc.list` method
    async fn handle_list(&self, _params: Value) -> Result<Value, String> {
        debug!("Listing all services");

        // List all from registry (returns service names)
        let registry = self.registry.read().await;
        let service_names = registry.list_services().await;

        // Get full service entries for each name
        let mut service_infos = Vec::new();
        for name in service_names {
            if let Some(entry) = registry.get_service(&name).await {
                service_infos.push(ServiceInfo {
                    primal_id: name,
                    virtual_endpoint: entry.virtual_endpoint.path,
                    capabilities: entry.capabilities,
                });
            }
        }

        let result = ListResult {
            services: service_infos,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `http.request` method - Full HTTP/HTTPS request
    async fn handle_http_request(&self, params: Value) -> Result<Value, String> {
        let params: HttpRequestParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        info!("HTTP request via IPC: {} {}", params.method, params.url);

        let result = self
            .http_handler
            .handle_request(params)
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `http.get` method - GET request shorthand
    async fn handle_http_get(&self, params: Value) -> Result<Value, String> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'url' parameter".to_string())?;

        info!("HTTP GET via IPC: {}", url);

        let result =
            self.http_handler.handle_get(url).await.map_err(|e| format!("HTTP GET failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `http.post` method - POST request shorthand
    async fn handle_http_post(&self, params: Value) -> Result<Value, String> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'url' parameter".to_string())?;

        let body = params
            .get("body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'body' parameter".to_string())?;

        let content_type = params.get("content_type").and_then(|v| v.as_str());

        // FIX: Extract headers from params (Issue #1 - Jan 28, 2026)
        let headers: std::collections::HashMap<String, String> = params
            .get("headers")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        info!("HTTP POST via IPC: {}", url);

        let result = self
            .http_handler
            .handle_post(url, body, content_type, headers)
            .await
            .map_err(|e| format!("HTTP POST failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `stun.get_public_address` method
    async fn handle_stun_get_public_address(&self, params: Value) -> Result<Value, String> {
        let result = self
            .stun_handler
            .handle_get_public_address(params)
            .await
            .map_err(|e| format!("STUN get_public_address failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `stun.bind` method
    async fn handle_stun_bind(&self, params: Value) -> Result<Value, String> {
        let result = self
            .stun_handler
            .handle_bind(params)
            .await
            .map_err(|e| format!("STUN bind failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `discovery.peers` method
    async fn handle_discovery_peers(&self, params: Value) -> Result<Value, String> {
        let result = self
            .discovery_handler
            .handle_list_peers(params)
            .await
            .map_err(|e| format!("Discovery peers failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `rendezvous.register` method (NEW - Jan 29, 2026)
    async fn handle_rendezvous_register(&self, params: Value) -> Result<Value, String> {
        let result = self
            .rendezvous_handler
            .handle_register(params)
            .await
            .map_err(|e| format!("Rendezvous register failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `rendezvous.lookup` method (NEW - Jan 29, 2026)
    async fn handle_rendezvous_lookup(&self, params: Value) -> Result<Value, String> {
        let result = self
            .rendezvous_handler
            .handle_lookup(params)
            .await
            .map_err(|e| format!("Rendezvous lookup failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `peer.connect` method (NEW - Jan 29, 2026)
    async fn handle_peer_connect(&self, params: Value) -> Result<Value, String> {
        let result = self
            .peer_handler
            .handle_connect(params)
            .await
            .map_err(|e| format!("Peer connect failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }
}

#[async_trait]
impl JsonRpcHandler for IpcServiceHandler {
    async fn handle(&self, method: &str, params: Value) -> Result<Value, String> {
        match method {
            // IPC registry methods
            "ipc.register" => self.handle_register(params).await,
            "ipc.resolve" => self.handle_resolve(params).await,
            "ipc.discover" => self.handle_discover(params).await,
            "ipc.list" => self.handle_list(params).await,

            // HTTP/HTTPS methods
            "http.request" => self.handle_http_request(params).await,
            "http.get" => self.handle_http_get(params).await,
            "http.post" => self.handle_http_post(params).await,

            // STUN/NAT traversal methods (NEW - Jan 29, 2026)
            "stun.get_public_address" => self.handle_stun_get_public_address(params).await,
            "stun.bind" => self.handle_stun_bind(params).await,

            // Discovery methods (NEW - Jan 29, 2026)
            "discovery.peers" => self.handle_discovery_peers(params).await,

            // Rendezvous methods (NEW - Jan 29, 2026 Phase 2)
            "rendezvous.register" => self.handle_rendezvous_register(params).await,
            "rendezvous.lookup" => self.handle_rendezvous_lookup(params).await,

            // Peer connection methods (NEW - Jan 29, 2026 Phase 2)
            "peer.connect" => self.handle_peer_connect(params).await,

            _ => Err(format!("Unknown method: {method}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_ipc_service_register() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());

        let params = json!({
            "primal_id": "beardog",
            "capabilities": ["crypto", "btsp"],
            "endpoint": "/tmp/primal-beardog.sock"
        });

        let result = handler.handle("ipc.register", params).await;
        assert!(result.is_ok());

        let result_value = result.unwrap();
        assert_eq!(result_value["virtual_endpoint"], "/primal/beardog");
    }

    #[tokio::test]
    async fn test_ipc_service_resolve() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());

        // Register first
        let register_params = json!({
            "primal_id": "beardog",
            "capabilities": ["crypto"],
            "endpoint": "/tmp/primal-beardog.sock"
        });
        handler.handle("ipc.register", register_params).await.unwrap();

        // Then resolve
        let resolve_params = json!({
            "primal_id": "beardog"
        });

        let result = handler.handle("ipc.resolve", resolve_params).await;
        assert!(result.is_ok());

        let result_value = result.unwrap();
        assert_eq!(result_value["virtual_endpoint"], "/primal/beardog");
        assert!(result_value["native_endpoint"].as_str().unwrap().contains("beardog"));
    }

    #[tokio::test]
    async fn test_ipc_service_discover() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());

        // Register service with capability
        let register_params = json!({
            "primal_id": "beardog",
            "capabilities": ["crypto", "btsp"],
            "endpoint": "/tmp/primal-beardog.sock"
        });
        handler.handle("ipc.register", register_params).await.unwrap();

        // Discover by capability
        let discover_params = json!({
            "capability": "crypto"
        });

        let result = handler.handle("ipc.discover", discover_params).await;
        assert!(result.is_ok());

        let result_value = result.unwrap();
        let providers = result_value["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0]["primal_id"], "beardog");
    }

    #[tokio::test]
    async fn test_ipc_service_list() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());

        // Register multiple services
        for (id, caps) in &[("beardog", vec!["crypto"]), ("squirrel", vec!["ai"])] {
            let params = json!({
                "primal_id": id,
                "capabilities": caps,
                "endpoint": format!("/tmp/primal-{id}.sock")
            });
            handler.handle("ipc.register", params).await.unwrap();
        }

        // List all
        let result = handler.handle("ipc.list", json!({})).await;
        assert!(result.is_ok());

        let result_value = result.unwrap();
        let services = result_value["services"].as_array().unwrap();
        assert_eq!(services.len(), 2);
    }
}
