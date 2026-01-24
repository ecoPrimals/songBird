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
}

impl IpcServiceHandler {
    /// Create a new IPC service handler
    pub fn new(registry: Arc<RwLock<ServiceRegistry>>) -> Self {
        Self {
            registry,
        }
    }

    /// Handle `ipc.register` method
    async fn handle_register(&self, params: Value) -> Result<Value, String> {
        let params: RegisterParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {}", e))?;

        info!("Registering primal: {} at {}", params.primal_id, params.endpoint);

        // Parse native endpoint
        let native_endpoint = if params.endpoint.starts_with("/") {
            // Unix socket path
            NativeEndpoint::UnixSocket(params.endpoint.into())
        } else if params.endpoint.starts_with("127.0.0.1:") || params.endpoint.contains(":") {
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
            .map_err(|e| format!("Registration failed: {}", e))?;

        let result = RegisterResult {
            virtual_endpoint: virtual_endpoint.path,
            registered_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))?)
    }

    /// Handle `ipc.resolve` method
    async fn handle_resolve(&self, params: Value) -> Result<Value, String> {
        let params: ResolveParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {}", e))?;

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

        Ok(serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))?)
    }

    /// Handle `ipc.discover` method
    async fn handle_discover(&self, params: Value) -> Result<Value, String> {
        let params: DiscoverParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {}", e))?;

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

        Ok(serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))?)
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
