//! IPC Service - Songbird's JSON-RPC IPC Broker
//!
//! This module provides Songbird's IPC brokering service. Instead of other
//! primals importing `songbird-universal-ipc` as a library (which would violate
//! primal autonomy), they connect to Songbird's IPC service via JSON-RPC.
//!
//! ## Architecture
//!
//! ```text
//! Primal (BearDog, Squirrel, etc.):
//!   - Uses tokio::net::UnixStream directly
//!   - Connects to /primal/songbird
//!   - Calls JSON-RPC methods for discovery
//!   - Connects directly to discovered services
//!
//! Songbird IPC Service (this module):
//!   - Maintains service registry
//!   - Provides discovery via JSON-RPC
//!   - Manages platform abstraction internally
//!   - NO code embedding - pure service!
//! ```
//!
//! ## JSON-RPC Methods
//!
//! ### `ipc.register`
//! Register a primal with capabilities
//!
//! **Request**:
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "method": "ipc.register",
//!   "params": {
//!     "primal_id": "beardog",
//!     "capabilities": ["crypto", "btsp"],
//!     "endpoint": "/tmp/primal-beardog.sock"
//!   },
//!   "id": 1
//! }
//! ```
//!
//! **Response**:
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "result": {
//!     "virtual_endpoint": "/primal/beardog",
//!     "registered_at": "2026-01-19T12:00:00Z"
//!   },
//!   "id": 1
//! }
//! ```
//!
//! ### `ipc.resolve`
//! Resolve a primal to its native endpoint
//!
//! **Request**:
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "method": "ipc.resolve",
//!   "params": {
//!     "primal_id": "beardog"
//!   },
//!   "id": 2
//! }
//! ```
//!
//! **Response**:
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "result": {
//!     "virtual_endpoint": "/primal/beardog",
//!     "native_endpoint": "/tmp/primal-beardog.sock",
//!     "capabilities": ["crypto", "btsp"]
//!   },
//!   "id": 2
//! }
//! ```
//!
//! ### `ipc.discover`
//! Discover services by capability
//!
//! **Request**:
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "method": "ipc.discover",
//!   "params": {
//!     "capability": "crypto"
//!   },
//!   "id": 3
//! }
//! ```
//!
//! **Response**:
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "result": {
//!     "providers": [
//!       {
//!         "primal_id": "beardog",
//!         "virtual_endpoint": "/primal/beardog",
//!         "native_endpoint": "/tmp/primal-beardog.sock",
//!         "capabilities": ["crypto", "btsp"]
//!       }
//!     ]
//!   },
//!   "id": 3
//! }
//! ```
//!
//! ### `ipc.list`
//! List all registered services
//!
//! **Request**:
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "method": "ipc.list",
//!   "params": {},
//!   "id": 4
//! }
//! ```
//!
//! **Response**:
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "result": {
//!     "services": [
//!       {
//!         "primal_id": "beardog",
//!         "virtual_endpoint": "/primal/beardog",
//!         "capabilities": ["crypto", "btsp"]
//!       },
//!       {
//!         "primal_id": "squirrel",
//!         "virtual_endpoint": "/primal/squirrel",
//!         "capabilities": ["ai", "mcp"]
//!       }
//!     ]
//!   },
//!   "id": 4
//! }
//! ```

use crate::capability::Provider;
use crate::registry::ServiceRegistry;
use crate::tower_atomic::{JsonRpcError, JsonRpcHandler, JsonRpcRequest, JsonRpcResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

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
pub struct IpcServiceHandler {
    registry: Arc<RwLock<ServiceRegistry>>,
}

impl IpcServiceHandler {
    /// Create a new IPC service handler
    pub fn new(registry: Arc<RwLock<ServiceRegistry>>) -> Self {
        Self { registry }
    }

    /// Handle `ipc.register` method
    async fn handle_register(&self, params: Value) -> Result<Value, String> {
        let params: RegisterParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {}", e))?;

        info!(
            "Registering primal: {} at {}",
            params.primal_id, params.endpoint
        );

        // Create provider
        let provider = Provider {
            id: params.primal_id.clone(),
            virtual_endpoint: format!("/primal/{}", params.primal_id),
            native_endpoint: params.endpoint.clone(),
            capabilities: params.capabilities.clone(),
            registered_at: std::time::SystemTime::now(),
            last_seen: std::time::SystemTime::now(),
        };

        // Register in registry
        let mut registry = self.registry.write().await;
        registry.register_provider(provider).await;

        let result = RegisterResult {
            virtual_endpoint: format!("/primal/{}", params.primal_id),
            registered_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))?)
    }

    /// Handle `ipc.resolve` method
    async fn handle_resolve(&self, params: Value) -> Result<Value, String> {
        let params: ResolveParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {}", e))?;

        debug!("Resolving primal: {}", params.primal_id);

        let virtual_path = format!("/primal/{}", params.primal_id);

        // Resolve from registry
        let registry = self.registry.read().await;
        let provider = registry
            .resolve(&virtual_path)
            .await
            .ok_or_else(|| format!("Primal not found: {}", params.primal_id))?;

        let result = ResolveResult {
            virtual_endpoint: provider.virtual_endpoint,
            native_endpoint: provider.native_endpoint,
            capabilities: provider.capabilities,
        };

        Ok(serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))?)
    }

    /// Handle `ipc.discover` method
    async fn handle_discover(&self, params: Value) -> Result<Value, String> {
        let params: DiscoverParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {}", e))?;

        debug!("Discovering capability: {}", params.capability);

        // Discover from registry
        let registry = self.registry.read().await;
        let providers = registry.find_by_capability(&params.capability).await;

        let provider_infos: Vec<ProviderInfo> = providers
            .into_iter()
            .map(|p| ProviderInfo {
                primal_id: p.id,
                virtual_endpoint: p.virtual_endpoint,
                native_endpoint: p.native_endpoint,
                capabilities: p.capabilities,
            })
            .collect();

        let result = DiscoverResult {
            providers: provider_infos,
        };

        Ok(serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))?)
    }

    /// Handle `ipc.list` method
    async fn handle_list(&self, _params: Value) -> Result<Value, String> {
        debug!("Listing all services");

        // List all from registry
        let registry = self.registry.read().await;
        let providers = registry.list_all().await;

        let service_infos: Vec<ServiceInfo> = providers
            .into_iter()
            .map(|p| ServiceInfo {
                primal_id: p.id,
                virtual_endpoint: p.virtual_endpoint,
                capabilities: p.capabilities,
            })
            .collect();

        let result = ListResult {
            services: service_infos,
        };

        Ok(serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))?)
    }
}

#[async_trait]
impl JsonRpcHandler for IpcServiceHandler {
    async fn handle(&self, method: &str, params: Value) -> Result<Value, String> {
        match method {
            "ipc.register" => self.handle_register(params).await,
            "ipc.resolve" => self.handle_resolve(params).await,
            "ipc.discover" => self.handle_discover(params).await,
            "ipc.list" => self.handle_list(params).await,
            _ => Err(format!("Unknown method: {}", method)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            result_value["virtual_endpoint"],
            "/primal/beardog"
        );
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
        assert_eq!(result_value["native_endpoint"], "/tmp/primal-beardog.sock");
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
        for (id, caps) in &[
            ("beardog", vec!["crypto"]),
            ("squirrel", vec!["ai"]),
        ] {
            let params = json!({
                "primal_id": id,
                "capabilities": caps,
                "endpoint": format!("/tmp/primal-{}.sock", id)
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

