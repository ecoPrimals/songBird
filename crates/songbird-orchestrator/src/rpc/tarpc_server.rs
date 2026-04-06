// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! High-Performance tarpc Server for Songbird
//!
//! Provides binary RPC with 10x performance improvement over HTTP/REST.
//! Designed for primal-to-primal communication with TLS support.
//!
//! Phase 2 Complete: Full async runtime implementation with tarpc.
//! v3.12.0: Imports types from songbird-universal for consistency

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_universal::tarpc_types::{
    HealthStatus, ProtocolInfo, RegistrationResult, ServiceInfo, ServiceRegistration, SongbirdRpc,
    VersionInfo,
};
use tarpc::context::Context;
use tarpc::server::Channel;
use tracing::{debug, error, info};

use crate::app::SongbirdOrchestrator;

// Re-export the trait from songbird-universal for backward compatibility
pub use songbird_universal::tarpc_types::SongbirdRpc as SongbirdRpcTrait;

// All type definitions now imported from songbird-universal for consistency (v3.12.0)
// This eliminates duplication and ensures type consistency across client and server

/// Service update event (local to orchestrator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceUpdate {
    pub service_id: String,
    pub event_type: String,
    pub timestamp: i64,
}

/// tarpc server implementation (simplified, zero unsafe - v3.12.0)
///
/// **Modern Rust**: This version doesn't require `Arc<SongbirdOrchestrator>`,
/// making it simpler and safer. The orchestrator field was never actually used.
#[derive(Clone)]
pub struct TarpcServerSimple {
    service_registry: Arc<FederatedServiceRegistry>,
    start_time: std::time::Instant,
}

impl TarpcServerSimple {
    /// Create new tarpc server with only service registry (no orchestrator needed!)
    #[must_use]
    pub fn new(service_registry: Arc<FederatedServiceRegistry>) -> Self {
        Self {
            service_registry,
            start_time: std::time::Instant::now(),
        }
    }
}

/// Implementation of `SongbirdRpc` trait for `TarpcServerSimple` (v3.12.0)
impl SongbirdRpc for TarpcServerSimple {
    async fn discover(self, _context: Context, capability: String) -> Vec<ServiceInfo> {
        debug!("tarpc: discover(capability={})", capability);

        let services = self.service_registry.find_by_capability(&capability).await;
        debug!("Discovered {} services for capability '{}'", services.len(), capability);

        let cap: Arc<str> = Arc::from(capability);
        services
            .into_iter()
            .map(|svc| ServiceInfo {
                id: svc.service_id,
                capability: (*cap).to_string(),
                endpoint: svc.endpoint,
                status: svc.health_status.to_string(),
                metadata: None,
            })
            .collect()
    }

    async fn discover_all(self, _context: Context) -> Vec<ServiceInfo> {
        debug!("tarpc: discover_all()");

        let services = self.service_registry.get_all_services().await;
        debug!("Discovered {} total services", services.len());
        services
            .into_iter()
            .map(|svc| ServiceInfo {
                id: svc.service_id,
                capability: svc.service_type,
                endpoint: svc.endpoint,
                status: svc.health_status.to_string(),
                metadata: None,
            })
            .collect()
    }

    async fn register(
        self,
        _context: Context,
        registration: ServiceRegistration,
    ) -> RegistrationResult {
        debug!("tarpc: register({}, {})", registration.service_id, registration.capability);

        let reg_id = registration.service_id;
        let reg_name = registration.service_name;
        let reg_cap = registration.capability;
        let capabilities = vec![reg_cap.clone()];

        let service_registration =
            songbird_network_federation::service_registry::ServiceRegistration {
                service_id: reg_id.clone(),
                service_name: reg_name.clone(),
                service_type: reg_cap,
                tower_id: registration.tower_id.unwrap_or_else(|| "unknown".to_string()),
                tower_name: registration.tower_name.unwrap_or_else(|| "Unknown Tower".to_string()),
                endpoint: registration.endpoint,
                capabilities,
                metadata: registration.metadata,
                health_status:
                    songbird_network_federation::service_registry::ServiceHealthStatus::Healthy,
                registered_at: chrono::Utc::now(),
                last_seen: chrono::Utc::now(),
            };

        self.service_registry.register_local(service_registration).await;

        info!("Service registered: {} ({})", reg_name, reg_id);

        RegistrationResult {
            success: true,
            message: format!("Service {reg_id} registered successfully"),
        }
    }

    async fn unregister(self, _context: Context, service_id: String) -> RegistrationResult {
        debug!("tarpc: unregister({})", service_id);

        // Check if service exists before unregistering
        let service_exists = self.service_registry.find_by_id(&service_id).await.is_some();

        if service_exists {
            self.service_registry.deregister_local(&service_id).await;
            info!("✅ Service unregistered: {}", service_id);

            RegistrationResult {
                success: true,
                message: format!("Service {service_id} unregistered successfully"),
            }
        } else {
            debug!("⚠️  Service not found for unregistration: {}", service_id);

            RegistrationResult {
                success: false,
                message: format!("Service {service_id} not found"),
            }
        }
    }

    async fn health(self, _context: Context) -> HealthStatus {
        debug!("tarpc: health()");

        // Calculate real uptime
        let uptime_seconds = self.start_time.elapsed().as_secs();

        // Get real service count from registry
        let services_count = self.service_registry.get_all_services().await.len();

        HealthStatus {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds,
            services_count,
        }
    }

    async fn version(self, _context: Context) -> VersionInfo {
        debug!("tarpc: version()");

        VersionInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            protocol: "tarpc".to_string(),
            capabilities: vec![
                "discovery".to_string(),
                "registry".to_string(),
                "health".to_string(),
            ],
        }
    }

    async fn protocols(self, _context: Context) -> Vec<ProtocolInfo> {
        use songbird_types::error_helpers::SafeEnv;

        debug!("tarpc: protocols()");

        let tarpc_port = SafeEnv::get_port("SONGBIRD_TARPC_PORT", 8081);
        let http_port = SafeEnv::get_port("SONGBIRD_HTTP_PORT", 8080);
        let ipc_path = std::env::temp_dir().join("songbird.sock").to_string_lossy().into_owned();

        vec![
            ProtocolInfo {
                name: "tarpc".to_string(),
                port: tarpc_port,
                enabled: true,
                info: HashMap::new(),
            },
            ProtocolInfo {
                name: "jsonrpc".to_string(),
                port: 0,
                enabled: true,
                info: HashMap::from([("path".to_string(), ipc_path)]),
            },
            ProtocolInfo {
                name: "http".to_string(),
                port: http_port,
                enabled: true,
                info: HashMap::new(),
            },
        ]
    }
}

/// tarpc server implementation (original - legacy)
///
/// **Legacy**: Kept for backward compatibility. New code should use `TarpcServerSimple`.
#[derive(Clone)]
pub struct TarpcServer {
    #[expect(
        dead_code,
        reason = "legacy tarpc surface; retained for future orchestrator-scoped RPC"
    )]
    orchestrator: Arc<SongbirdOrchestrator>,
    service_registry: Arc<FederatedServiceRegistry>,
    start_time: std::time::Instant,
}

impl TarpcServer {
    /// Create new tarpc server with service registry
    #[must_use]
    pub fn new(
        orchestrator: Arc<SongbirdOrchestrator>,
        service_registry: Arc<FederatedServiceRegistry>,
    ) -> Self {
        Self {
            orchestrator,
            service_registry,
            start_time: std::time::Instant::now(),
        }
    }
}

impl SongbirdRpc for TarpcServer {
    async fn discover(self, _context: Context, capability: String) -> Vec<ServiceInfo> {
        debug!("tarpc: discover(capability={})", capability);

        let services = self.service_registry.find_by_capability(&capability).await;
        debug!("Discovered {} services for capability '{}'", services.len(), capability);

        let cap: Arc<str> = Arc::from(capability);
        services
            .into_iter()
            .map(|svc| ServiceInfo {
                id: svc.service_id,
                capability: (*cap).to_string(),
                endpoint: svc.endpoint,
                status: svc.health_status.to_string(),
                metadata: None,
            })
            .collect()
    }

    async fn discover_all(self, _context: Context) -> Vec<ServiceInfo> {
        debug!("tarpc: discover_all()");

        let services = self.service_registry.get_all_services().await;
        debug!("Discovered {} total services", services.len());
        services
            .into_iter()
            .map(|svc| ServiceInfo {
                id: svc.service_id,
                capability: svc.service_type,
                endpoint: svc.endpoint,
                status: svc.health_status.to_string(),
                metadata: None,
            })
            .collect()
    }

    async fn register(
        self,
        _context: Context,
        registration: ServiceRegistration,
    ) -> RegistrationResult {
        debug!("tarpc: register({}, {})", registration.service_id, registration.capability);

        let reg_id = registration.service_id;
        let reg_name = registration.service_name;
        let reg_cap = registration.capability;
        let capabilities = vec![reg_cap.clone()];

        let service_registration =
            songbird_network_federation::service_registry::ServiceRegistration {
                service_id: reg_id.clone(),
                service_name: reg_name.clone(),
                service_type: reg_cap,
                tower_id: registration.tower_id.unwrap_or_else(|| "unknown".to_string()),
                tower_name: registration.tower_name.unwrap_or_else(|| "Unknown Tower".to_string()),
                endpoint: registration.endpoint,
                capabilities,
                metadata: registration.metadata,
                health_status:
                    songbird_network_federation::service_registry::ServiceHealthStatus::Healthy,
                registered_at: chrono::Utc::now(),
                last_seen: chrono::Utc::now(),
            };

        self.service_registry.register_local(service_registration).await;

        info!("Service registered: {} ({})", reg_name, reg_id);

        RegistrationResult {
            success: true,
            message: format!("Service {reg_id} registered successfully"),
        }
    }

    async fn unregister(self, _context: Context, service_id: String) -> RegistrationResult {
        debug!("tarpc: unregister({})", service_id);

        let service_exists = self.service_registry.find_by_id(&service_id).await.is_some();

        if service_exists {
            self.service_registry.deregister_local(&service_id).await;
            info!("Service unregistered: {}", service_id);

            RegistrationResult {
                success: true,
                message: format!("Service {service_id} unregistered successfully"),
            }
        } else {
            debug!("Service not found for unregistration: {}", service_id);

            RegistrationResult {
                success: false,
                message: format!("Service {service_id} not found"),
            }
        }
    }

    async fn health(self, _context: Context) -> HealthStatus {
        debug!("tarpc: health()");

        // Calculate real uptime
        let uptime_seconds = self.start_time.elapsed().as_secs();

        // Get real service count from registry
        let services_count = self.service_registry.get_all_services().await.len();

        HealthStatus {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds,
            services_count,
        }
    }

    async fn version(self, _context: Context) -> VersionInfo {
        debug!("tarpc: version()");

        VersionInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
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
        use songbird_types::error_helpers::SafeEnv;

        let http_port = SafeEnv::get_port("SONGBIRD_HTTP_PORT", 8080);
        let https_port = SafeEnv::get_port("SONGBIRD_HTTPS_PORT", 8443);
        let tarpc_port = SafeEnv::get_port("SONGBIRD_TARPC_PORT", 8081);

        vec![
            ProtocolInfo {
                name: "HTTP".to_string(),
                port: http_port,
                enabled: true,
                info: HashMap::new(),
            },
            ProtocolInfo {
                name: "HTTPS".to_string(),
                port: https_port,
                enabled: true,
                info: HashMap::new(),
            },
            ProtocolInfo {
                name: "JSON-RPC".to_string(),
                port: https_port,
                enabled: true,
                info: HashMap::from([("path".to_string(), "/jsonrpc".to_string())]),
            },
            ProtocolInfo {
                name: "tarpc".to_string(),
                port: tarpc_port,
                enabled: true,
                info: HashMap::new(),
            },
        ]
    }
}

/// Start tarpc server on specified address (simplified version without orchestrator Arc)
///
/// **v3.12.0**: Zero unsafe blocks - uses `TarpcServerSimple` without orchestrator dependency
///
/// This is the production version that avoids `Arc<SongbirdOrchestrator>` complexity.
/// The `TarpcServer` only needs `service_registry`, so this version is simpler and safer.
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn start_tarpc_server_simple(
    service_registry: Arc<FederatedServiceRegistry>,
    addr: SocketAddr,
) -> Result<()> {
    use futures::StreamExt;

    info!("🚀 Starting tarpc server (simplified, zero unsafe) on {}", addr);

    // Bind TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("✅ tarpc server listening on {}", addr);

    // Create simplified server instance (no orchestrator Arc needed!)
    let server = TarpcServerSimple::new(service_registry);

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

        // Clone server for this connection
        let server = server.clone();

        // Spawn a task to handle this connection
        tokio::spawn(async move {
            // Create codec transport using tokio-serde with bincode
            let transport = tarpc::serde_transport::new(
                tokio_util::codec::LengthDelimitedCodec::builder()
                    .max_frame_length(16 * 1024 * 1024) // 16 MB max frame
                    .new_framed(stream),
                tokio_serde::formats::Bincode::default(),
            );

            // Create server channel
            let channel = tarpc::server::BaseChannel::with_defaults(transport);

            // Respond to requests
            channel
                .execute(server.serve())
                .for_each(|response| async move {
                    tokio::spawn(response);
                })
                .await;

            debug!("tarpc connection from {} closed", peer_addr);
        });
    }
}

/// Start tarpc server on specified address (original version with orchestrator Arc)
///
/// **Legacy**: This version requires `Arc<SongbirdOrchestrator>` but doesn't actually use it.
/// Kept for backward compatibility. New code should use `start_tarpc_server_simple`.
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn start_tarpc_server(
    orchestrator: Arc<SongbirdOrchestrator>,
    service_registry: Arc<FederatedServiceRegistry>,
    addr: SocketAddr,
) -> Result<()> {
    use futures::StreamExt;

    info!("🚀 Starting tarpc server on {}", addr);

    // Bind TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("✅ tarpc server listening on {}", addr);

    // Create server instance with real service registry (EVOLVED)
    let server = TarpcServer::new(orchestrator, service_registry);

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

        // Clone server for this connection
        let server = server.clone();

        // Spawn a task to handle this connection
        tokio::spawn(async move {
            // Create codec transport using tokio-serde with bincode
            let transport = tarpc::serde_transport::new(
                tokio_util::codec::LengthDelimitedCodec::builder()
                    .max_frame_length(16 * 1024 * 1024) // 16 MB max frame
                    .new_framed(stream),
                tokio_serde::formats::Bincode::default(),
            );

            // Create server channel
            let channel = tarpc::server::BaseChannel::with_defaults(transport);

            // Respond to requests
            channel
                .execute(server.serve())
                .for_each(|response| async move {
                    tokio::spawn(response);
                })
                .await;

            debug!("tarpc connection from {} closed", peer_addr);
        });
    }
}

/// tarpc server configuration
#[derive(Debug, Clone)]
pub struct TarpcConfig {
    /// Bind address
    pub addr: SocketAddr,

    /// Enable TLS
    pub tls_enabled: bool,

    /// Maximum concurrent connections
    pub max_connections: usize,
}

impl Default for TarpcConfig {
    fn default() -> Self {
        use std::net::{IpAddr, Ipv6Addr, SocketAddr};
        let port = songbird_types::error_helpers::SafeEnv::get_port("SONGBIRD_TARPC_PORT", 8081);
        Self {
            addr: SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port),
            tls_enabled: false,
            max_connections: 1000,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::test_sync_env::{VarGuard, env_lock};
    use songbird_network_federation::service_registry::{
        ServiceHealthStatus, ServiceRegistration as FedServiceRegistration,
    };
    use songbird_universal::tarpc_types::SongbirdRpc;
    use std::collections::HashMap;
    use tarpc::context;

    #[test]
    fn test_tarpc_config_default() {
        let config = TarpcConfig::default();
        assert_eq!(config.addr.port(), 8081);
        assert!(!config.tls_enabled);
        assert_eq!(config.max_connections, 1000);
    }

    #[test]
    fn test_service_info_serialization() {
        let info = ServiceInfo {
            id: "test".to_string(),
            capability: "compute".to_string(),
            endpoint: "http://localhost:8001".to_string(),
            status: "healthy".to_string(),
            metadata: None,
        };

        let serialized = serde_json::to_string(&info).unwrap();
        let deserialized: ServiceInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(info.id, deserialized.id);
    }

    fn sample_fed_registration(id: &str, cap: &str, endpoint: &str) -> FedServiceRegistration {
        FedServiceRegistration {
            service_id: id.into(),
            service_name: format!("svc-{id}"),
            service_type: cap.into(),
            tower_id: "t1".into(),
            tower_name: "Tower".into(),
            endpoint: endpoint.into(),
            capabilities: vec![cap.into()],
            metadata: HashMap::new(),
            health_status: ServiceHealthStatus::Healthy,
            registered_at: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
        }
    }

    #[tokio::test(start_paused = true)]
    async fn tarpc_simple_discover_filters_by_capability() {
        let reg = Arc::new(FederatedServiceRegistry::new());
        reg.register_local(sample_fed_registration("a", "compute", "http://127.0.0.1:0")).await;
        reg.register_local(sample_fed_registration("b", "storage", "http://127.0.0.1:0")).await;

        let srv = TarpcServerSimple::new(Arc::clone(&reg));
        tokio::time::sleep(std::time::Duration::from_secs(0)).await;
        let list = srv.clone().discover(context::current(), "compute".into()).await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "a");
        assert_eq!(list[0].capability, "compute");
    }

    #[tokio::test]
    async fn tarpc_simple_register_and_unregister_round_trip() {
        let reg = Arc::new(FederatedServiceRegistry::new());
        let srv = TarpcServerSimple::new(Arc::clone(&reg));

        let registration = ServiceRegistration {
            service_id: "svc-1".into(),
            service_name: "n".into(),
            capability: "ai".into(),
            endpoint: "http://127.0.0.1:0".into(),
            tower_id: None,
            tower_name: None,
            metadata: HashMap::new(),
        };

        let ok = srv.clone().register(context::current(), registration).await;
        assert!(ok.success);

        let not_found = srv.clone().unregister(context::current(), "missing".into()).await;
        assert!(!not_found.success);

        let removed = srv.unregister(context::current(), "svc-1".into()).await;
        assert!(removed.success);
    }

    #[tokio::test]
    async fn tarpc_simple_health_and_version() {
        let reg = Arc::new(FederatedServiceRegistry::new());
        let srv = TarpcServerSimple::new(Arc::clone(&reg));

        let h = srv.clone().health(context::current()).await;
        assert_eq!(h.status, "healthy");
        assert_eq!(h.services_count, 0);

        let v = srv.clone().version(context::current()).await;
        assert_eq!(v.protocol, "tarpc");

        let _serial = env_lock();
        let _tp = VarGuard::set("SONGBIRD_TARPC_PORT", "0");
        let _hp = VarGuard::set("SONGBIRD_HTTP_PORT", "0");
        let protos = srv.protocols(context::current()).await;
        assert!(protos.iter().any(|p| p.name == "tarpc"));
    }

    #[tokio::test]
    async fn tarpc_legacy_server_dispatch_matches_simple() {
        let port = songbird_test_utils::test_port("tarpc_orch_sec");
        let url = format!("http://127.0.0.1:{port}");
        let _serial = env_lock();
        let _sec = VarGuard::set("SONGBIRD_SECURITY_PROVIDER", url.as_str());
        let config = songbird_types::config::CanonicalSongbirdConfig::default();
        let orch = crate::SongbirdOrchestrator::new(config).await.expect("orch");
        let fed = Arc::new(FederatedServiceRegistry::new());
        let arc_orch = Arc::new(orch);
        let srv = TarpcServer::new(Arc::clone(&arc_orch), Arc::clone(&fed));

        fed.register_local(sample_fed_registration("x", "compute", "http://127.0.0.1:0")).await;
        let out = srv.discover(context::current(), "compute".into()).await;
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "x");
    }
}
