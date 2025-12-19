//! Standalone tarpc Server for Albatross Benchmarking
//!
//! This is a minimal tarpc server that implements the SongbirdRpc trait
//! for benchmarking purposes. It provides the same interface as the full
//! Songbird orchestrator tarpc server.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tarpc::{
    context::Context,
    server::{self, Channel},
};
use tokio::net::TcpListener;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get address from args or use default
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:8091".to_string())
        .parse::<SocketAddr>()?;

    info!("🚀 Starting standalone tarpc server on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    info!("✅ tarpc server listening on {}", addr);

    let server = BenchmarkTarpcServer;

    // Accept connections in a loop
    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                continue;
            }
        };

        debug!("New tarpc connection from {}", peer_addr);

        let server_clone = server.clone();
        tokio::spawn(async move {
            use futures::StreamExt;

            let transport = tarpc::serde_transport::Transport::from((
                stream,
                tarpc::tokio_serde::formats::Bincode::default(),
            ));
            let channel = server::BaseChannel::with_defaults(transport);

            // Serve requests on this connection
            channel
                .execute(server_clone.serve())
                .for_each(|response| async move {
                    tokio::spawn(response);
                })
                .await;
        });
    }
}

// tarpc service trait (must match songbird-orchestrator)
#[tarpc::service]
trait SongbirdRpc {
    async fn discover(capability: String) -> Vec<ServiceInfo>;
    async fn discover_all() -> Vec<ServiceInfo>;
    async fn register(registration: ServiceRegistration) -> RegistrationResult;
    async fn unregister(service_id: String) -> RegistrationResult;
    async fn health() -> HealthStatus;
    async fn version() -> VersionInfo;
    async fn protocols() -> Vec<ProtocolInfo>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceInfo {
    id: String,
    capability: String,
    endpoint: String,
    status: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceRegistration {
    service_id: String,
    capability: String,
    endpoint: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistrationResult {
    success: bool,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HealthStatus {
    status: String,
    version: String,
    uptime_seconds: u64,
    services_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionInfo {
    version: String,
    protocol: String,
    capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProtocolInfo {
    name: String,
    port: u16,
    status: String,
    path: Option<String>,
}

#[derive(Clone)]
struct BenchmarkTarpcServer;

impl SongbirdRpc for BenchmarkTarpcServer {
    async fn discover(self, _context: Context, capability: String) -> Vec<ServiceInfo> {
        debug!("tarpc: discover({})", capability);
        vec![ServiceInfo {
            id: "benchmark-service".to_string(),
            capability,
            endpoint: "http://localhost:7878".to_string(),
            status: "healthy".to_string(),
            metadata: None,
        }]
    }

    async fn discover_all(self, _context: Context) -> Vec<ServiceInfo> {
        debug!("tarpc: discover_all()");
        vec![ServiceInfo {
            id: "benchmark-service".to_string(),
            capability: "compute".to_string(),
            endpoint: "http://localhost:7878".to_string(),
            status: "healthy".to_string(),
            metadata: None,
        }]
    }

    async fn register(
        self,
        _context: Context,
        registration: ServiceRegistration,
    ) -> RegistrationResult {
        debug!("tarpc: register({}, {})", registration.service_id, registration.capability);
        RegistrationResult {
            success: true,
            message: format!("Registered: {}", registration.service_id),
        }
    }

    async fn unregister(self, _context: Context, service_id: String) -> RegistrationResult {
        debug!("tarpc: unregister({})", service_id);
        RegistrationResult {
            success: true,
            message: format!("Unregistered: {}", service_id),
        }
    }

    async fn health(self, _context: Context) -> HealthStatus {
        debug!("tarpc: health()");
        HealthStatus {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            uptime_seconds: 3600,
            services_count: 1,
        }
    }

    async fn version(self, _context: Context) -> VersionInfo {
        debug!("tarpc: version()");
        VersionInfo {
            version: "0.1.0".to_string(),
            protocol: "tarpc".to_string(),
            capabilities: vec![
                "discovery".to_string(),
                "registry".to_string(),
                "health".to_string(),
            ],
        }
    }

    async fn protocols(self, _context: Context) -> Vec<ProtocolInfo> {
        debug!("tarpc: protocols()");
        vec![ProtocolInfo {
            name: "tarpc".to_string(),
            port: 8091,
            status: "active".to_string(),
            path: None,
        }]
    }
}
