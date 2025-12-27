/// JSON-RPC 2.0 Server for Songbird
///
/// Provides universal, language-agnostic RPC access to Songbird orchestration
/// capabilities over HTTPS. Works with any client supporting JSON-RPC 2.0.
use std::net::SocketAddr;
use std::sync::Arc;

use jsonrpsee::{
    server::{Server, ServerHandle},
    types::ErrorObjectOwned,
    RpcModule,
};
use serde::Deserialize;
use tracing::{debug, info};

use crate::app::SongbirdOrchestrator;
use songbird_network_federation::service_registry::FederatedServiceRegistry;

/// JSON-RPC server configuration
#[derive(Debug, Clone)]
pub struct JsonRpcConfig {
    /// Bind address (typically same as HTTP server)
    pub addr: SocketAddr,
    /// Enable request logging
    pub log_requests: bool,
    /// Maximum request size (bytes)
    pub max_request_size: u32,
    /// Maximum response size (bytes)
    pub max_response_size: u32,
}

impl Default for JsonRpcConfig {
    fn default() -> Self {
        use std::net::{IpAddr, Ipv6Addr, SocketAddr};
        Self {
            // Use direct SocketAddr construction - zero unwraps
            addr: SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 8080),
            log_requests: true,
            max_request_size: 10 * 1024 * 1024,  // 10 MB
            max_response_size: 10 * 1024 * 1024, // 10 MB
        }
    }
}

/// Shared state for JSON-RPC methods
#[derive(Clone)]
pub struct JsonRpcState {
    pub orchestrator: Arc<SongbirdOrchestrator>,
    pub service_registry: Arc<FederatedServiceRegistry>,
    pub start_time: std::time::Instant,
}

/// JSON-RPC 2.0 server for Songbird orchestration
pub struct JsonRpcServer {
    config: JsonRpcConfig,
    state: JsonRpcState,
}

impl JsonRpcServer {
    /// Create a new JSON-RPC server
    pub fn new(
        orchestrator: Arc<SongbirdOrchestrator>,
        service_registry: Arc<FederatedServiceRegistry>,
        config: JsonRpcConfig,
    ) -> Self {
        Self {
            config,
            state: JsonRpcState {
                orchestrator,
                service_registry,
                start_time: std::time::Instant::now(),
            },
        }
    }

    /// Build and start the JSON-RPC server
    pub async fn start(self) -> Result<(ServerHandle, SocketAddr), Box<dyn std::error::Error>> {
        info!("🚀 Starting JSON-RPC 2.0 server on {}", self.config.addr);

        // Build server
        let server = Server::builder().build(self.config.addr).await?;

        let addr = server.local_addr()?;

        // Create RPC module with shared state
        let mut module = RpcModule::new(self.state.clone());

        // Register all JSON-RPC methods
        Self::register_discovery_methods(&mut module)?;
        Self::register_registry_methods(&mut module)?;
        Self::register_health_methods(&mut module)?;
        Self::register_protocol_methods(&mut module)?;

        // Start server
        let handle = server.start(module);

        info!("✅ JSON-RPC 2.0 server listening on {}", addr);
        info!("   Endpoint: http://{}/jsonrpc", addr);
        info!("   Methods: {} registered", Self::method_count());

        Ok((handle, addr))
    }

    /// Register discovery-related JSON-RPC methods
    fn register_discovery_methods(
        module: &mut RpcModule<JsonRpcState>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // songbird.discover - Discover services by capability
        module.register_async_method("songbird.discover", |params, ctx, _ext| async move {
            let capability: String = params.one()?;
            debug!("JSON-RPC: discover({})", capability);

            // Access state from context
            let state = ctx.as_ref();

            // Use real capability-based discovery
            let registrations = state.service_registry.find_by_capability(&capability).await;

            let services: Vec<serde_json::Value> = registrations
                .into_iter()
                .map(|reg| {
                    serde_json::json!({
                        "id": reg.service_id,
                        "service_name": reg.service_name,
                        "service_type": reg.service_type,
                        "tower_id": reg.tower_id,
                        "capability": capability,
                        "endpoint": reg.endpoint,
                        "status": "healthy",
                        "metadata": reg.metadata
                    })
                })
                .collect();

            Ok::<_, ErrorObjectOwned>(services)
        })?;

        // songbird.discoverAll - Discover all available services
        module.register_async_method("songbird.discoverAll", |_params, ctx, _ext| async move {
            debug!("JSON-RPC: discoverAll()");

            // Access state from context
            let state = ctx.as_ref();

            // Use real service discovery
            let all_services = state.service_registry.get_all_services().await;

            let services: Vec<serde_json::Value> = all_services
                .into_iter()
                .map(|reg| {
                    serde_json::json!({
                        "id": reg.service_id,
                        "service_name": reg.service_name,
                        "service_type": reg.service_type,
                        "tower_id": reg.tower_id,
                        "endpoint": reg.endpoint,
                        "status": "healthy",
                        "capabilities": reg.capabilities,
                        "metadata": reg.metadata
                    })
                })
                .collect();

            Ok::<_, ErrorObjectOwned>(services)
        })?;

        Ok(())
    }

    /// Register registry-related JSON-RPC methods
    fn register_registry_methods(
        module: &mut RpcModule<JsonRpcState>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // songbird.register - Register a service
        module.register_async_method("songbird.register", |params, ctx, _ext| async move {
            #[derive(Deserialize)]
            struct RegisterRequest {
                service_id: String,
                capability: String,
                endpoint: String,
                metadata: Option<serde_json::Value>,
            }

            let req: RegisterRequest = params.parse()?;
            debug!("JSON-RPC: register({}, {})", req.service_id, req.capability);

            // Access state from context
            let state = ctx.as_ref();

            // Call actual registry implementation
            use songbird_network_federation::service_registry::ServiceRegistration as RegEntry;
            let registration = RegEntry {
                service_id: req.service_id.clone(),
                service_name: req.service_id.clone(),
                service_type: req.capability.clone(),
                tower_id: "jsonrpc-client".to_string(),
                tower_name: "JSON-RPC Client".to_string(),
                endpoint: req.endpoint,
                capabilities: vec![req.capability],
                metadata: req
                    .metadata
                    .and_then(|v| {
                        v.as_object()
                            .map(|o| o.iter().map(|(k, v)| (k.clone(), v.to_string())).collect())
                    })
                    .unwrap_or_default(),
                health_status:
                    songbird_network_federation::service_registry::ServiceHealthStatus::Healthy,
                registered_at: chrono::Utc::now(),
                last_seen: chrono::Utc::now(),
            };

            state.service_registry.register_local(registration).await;

            let response = serde_json::json!({
                "success": true,
                "service_id": req.service_id,
                "message": "Service registered successfully via JSON-RPC"
            });

            Ok::<_, ErrorObjectOwned>(response)
        })?;

        // songbird.unregister - Unregister a service
        module.register_async_method("songbird.unregister", |params, ctx, _ext| async move {
            let service_id: String = params.one()?;
            debug!("JSON-RPC: unregister({})", service_id);

            // Access state from context
            let state = ctx.as_ref();

            // Call actual registry implementation
            state.service_registry.deregister_local(&service_id).await;

            let response = serde_json::json!({
                "success": true,
                "service_id": service_id,
                "message": "Service unregistered successfully via JSON-RPC"
            });

            Ok::<_, ErrorObjectOwned>(response)
        })?;

        Ok(())
    }

    /// Register health-related JSON-RPC methods
    fn register_health_methods(
        module: &mut RpcModule<JsonRpcState>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // songbird.health - Get orchestrator health status
        module.register_async_method("songbird.health", |_params, ctx, _ext| async move {
            debug!("JSON-RPC: health()");

            // Access state from context
            let state = ctx.as_ref();

            // Calculate real uptime
            let uptime_seconds = state.start_time.elapsed().as_secs();

            // Get real service count from registry
            let services_count = state.service_registry.get_all_services().await.len();

            let response = serde_json::json!({
                "status": "healthy",
                "version": env!("CARGO_PKG_VERSION"),
                "uptime_seconds": uptime_seconds,
                "services_count": services_count,
            });

            Ok::<_, ErrorObjectOwned>(response)
        })?;

        // songbird.version - Get Songbird version
        module.register_async_method("songbird.version", |_params, _ctx, _state| async move {
            debug!("JSON-RPC: version()");

            let response = serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "protocol": "JSON-RPC 2.0",
                "capabilities": ["discovery", "registry", "health", "protocol_negotiation"]
            });

            Ok::<_, ErrorObjectOwned>(response)
        })?;

        Ok(())
    }

    /// Register protocol-related JSON-RPC methods
    fn register_protocol_methods(
        module: &mut RpcModule<JsonRpcState>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // songbird.protocols - Get available protocols
        module.register_async_method("songbird.protocols", |_params, _ctx, _state| async move {
            debug!("JSON-RPC: protocols()");

            let response = serde_json::json!({
                "supported": [
                    {
                        "name": "HTTP",
                        "port": 8080,
                        "status": "active"
                    },
                    {
                        "name": "HTTPS",
                        "port": 8443,
                        "status": "active"
                    },
                    {
                        "name": "JSON-RPC",
                        "port": 8080,
                        "path": "/jsonrpc",
                        "status": "active"
                    },
                    {
                        "name": "tarpc",
                        "port": 8081,
                        "status": "planned"
                    }
                ]
            });

            Ok::<_, ErrorObjectOwned>(response)
        })?;

        // songbird.negotiateProtocol - Negotiate protocol upgrade
        module.register_async_method(
            "songbird.negotiateProtocol",
            |params, _ctx, _state| async move {
                #[derive(Deserialize)]
                struct NegotiateRequest {
                    desired_protocol: String,
                    peer_id: Option<String>,
                }

                let req: NegotiateRequest = params.parse()?;
                debug!("JSON-RPC: negotiateProtocol({})", req.desired_protocol);

                // Protocol negotiation implementation
                // Supports: JSON-RPC (current), tarpc (future), WebSocket (future)
                let (available, message) = match req.desired_protocol.as_str() {
                    "JSON-RPC" | "jsonrpc" => (true, "JSON-RPC is available"),
                    "tarpc" => (false, "tarpc support planned for future release"),
                    "WebSocket" | "websocket" => (false, "WebSocket support planned for future release"),
                    "HTTP" | "http" => (true, "HTTP/JSON-RPC is available"),
                    _ => (false, "Unknown protocol"),
                };
                
                let response = serde_json::json!({
                    "protocol": req.desired_protocol,
                    "available": available,
                    "message": message,
                    "fallback": "JSON-RPC",
                    "supported_protocols": ["JSON-RPC", "HTTP"]
                });

                Ok::<_, ErrorObjectOwned>(response)
            },
        )?;

        Ok(())
    }

    /// Get total method count (for logging)
    fn method_count() -> usize {
        9 // Total registered methods
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_config_default() {
        let config = JsonRpcConfig::default();
        assert_eq!(config.addr.port(), 8080);
        assert!(config.log_requests);
        assert_eq!(config.max_request_size, 10 * 1024 * 1024);
    }

    // TODO: Add integration tests with actual JSON-RPC client
}
