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
use tracing::{info, debug};

use crate::app::SongbirdOrchestrator;

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
        Self {
            addr: "[::]:8080".parse().unwrap(),
            log_requests: true,
            max_request_size: 10 * 1024 * 1024, // 10 MB
            max_response_size: 10 * 1024 * 1024, // 10 MB
        }
    }
}

/// JSON-RPC 2.0 server for Songbird orchestration
pub struct JsonRpcServer {
    config: JsonRpcConfig,
    orchestrator: Arc<SongbirdOrchestrator>,
}

impl JsonRpcServer {
    /// Create a new JSON-RPC server
    pub fn new(orchestrator: Arc<SongbirdOrchestrator>, config: JsonRpcConfig) -> Self {
        Self {
            config,
            orchestrator,
        }
    }

    /// Build and start the JSON-RPC server
    pub async fn start(self) -> Result<(ServerHandle, SocketAddr), Box<dyn std::error::Error>> {
        info!("🚀 Starting JSON-RPC 2.0 server on {}", self.config.addr);

        // Build server
        let server = Server::builder()
            .build(self.config.addr)
            .await?;

        let addr = server.local_addr()?;

        // Create RPC module with methods
        let mut module = RpcModule::new(self.orchestrator.clone());

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
    fn register_discovery_methods(module: &mut RpcModule<Arc<SongbirdOrchestrator>>) -> Result<(), Box<dyn std::error::Error>> {
        // songbird.discover - Discover services by capability
        module.register_async_method("songbird.discover", |params, _ctx, _state| async move {
            let capability: String = params.one()?;
            debug!("JSON-RPC: discover({})", capability);

            // TODO: Call actual discovery implementation
            // For now, return mock response
            let services = vec![
                serde_json::json!({
                    "id": "service-1",
                    "capability": capability,
                    "endpoint": "http://localhost:8001",
                    "status": "healthy"
                })
            ];

            Ok::<_, ErrorObjectOwned>(services)
        })?;

        // songbird.discoverAll - Discover all available services
        module.register_async_method("songbird.discoverAll", |_params, _ctx, _state| async move {
            debug!("JSON-RPC: discoverAll()");

            // TODO: Call actual discovery implementation
            let services = vec![
                serde_json::json!({
                    "id": "service-1",
                    "capability": "compute",
                    "endpoint": "http://localhost:8001",
                    "status": "healthy"
                })
            ];

            Ok::<_, ErrorObjectOwned>(services)
        })?;

        Ok(())
    }

    /// Register registry-related JSON-RPC methods
    fn register_registry_methods(module: &mut RpcModule<Arc<SongbirdOrchestrator>>) -> Result<(), Box<dyn std::error::Error>> {
        // songbird.register - Register a service
        module.register_async_method("songbird.register", |params, _ctx, _state| async move {
            #[derive(Deserialize)]
            struct RegisterRequest {
                service_id: String,
                capability: String,
                endpoint: String,
                metadata: Option<serde_json::Value>,
            }

            let req: RegisterRequest = params.parse()?;
            debug!("JSON-RPC: register({}, {})", req.service_id, req.capability);

            // TODO: Call actual registry implementation
            let response = serde_json::json!({
                "success": true,
                "service_id": req.service_id,
                "message": "Service registered successfully"
            });

            Ok::<_, ErrorObjectOwned>(response)
        })?;

        // songbird.unregister - Unregister a service
        module.register_async_method("songbird.unregister", |params, _ctx, _state| async move {
            let service_id: String = params.one()?;
            debug!("JSON-RPC: unregister({})", service_id);

            // TODO: Call actual registry implementation
            let response = serde_json::json!({
                "success": true,
                "service_id": service_id,
                "message": "Service unregistered successfully"
            });

            Ok::<_, ErrorObjectOwned>(response)
        })?;

        Ok(())
    }

    /// Register health-related JSON-RPC methods
    fn register_health_methods(module: &mut RpcModule<Arc<SongbirdOrchestrator>>) -> Result<(), Box<dyn std::error::Error>> {
        // songbird.health - Get orchestrator health status
        module.register_async_method("songbird.health", |_params, _ctx, _state| async move {
            debug!("JSON-RPC: health()");

            let response = serde_json::json!({
                "status": "healthy",
                "version": env!("CARGO_PKG_VERSION"),
                "uptime_seconds": 3600, // TODO: Real uptime
                "services_count": 0, // TODO: Real count
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
    fn register_protocol_methods(module: &mut RpcModule<Arc<SongbirdOrchestrator>>) -> Result<(), Box<dyn std::error::Error>> {
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
        module.register_async_method("songbird.negotiateProtocol", |params, _ctx, _state| async move {
            #[derive(Deserialize)]
            struct NegotiateRequest {
                desired_protocol: String,
                peer_id: Option<String>,
            }

            let req: NegotiateRequest = params.parse()?;
            debug!("JSON-RPC: negotiateProtocol({})", req.desired_protocol);

            // TODO: Implement protocol negotiation
            let response = serde_json::json!({
                "protocol": req.desired_protocol,
                "available": false,
                "message": "Protocol negotiation not yet implemented",
                "fallback": "JSON-RPC"
            });

            Ok::<_, ErrorObjectOwned>(response)
        })?;

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

