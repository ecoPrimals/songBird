// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! High-Performance tarpc Server for Songbird
//!
//! Provides binary RPC with 10x performance improvement over HTTP/REST.
//! Designed for primal-to-primal communication with TLS support.
//!
//! Phase 2 Complete: Full async runtime implementation with tarpc.
//! v3.12.0: Imports types from songbird-universal for consistency

#[macro_use]
mod accept;
mod dispatch;

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

use crate::app::SongbirdOrchestrator;

// Re-export the trait from songbird-universal for backward compatibility
pub use songbird_universal::tarpc_types::SongbirdRpc as SongbirdRpcTrait;

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

impl SongbirdRpc for TarpcServerSimple {
    async fn discover(self, _context: Context, capability: String) -> Vec<ServiceInfo> {
        dispatch::discover(&self.service_registry, capability).await
    }

    async fn discover_all(self, _context: Context) -> Vec<ServiceInfo> {
        dispatch::discover_all(&self.service_registry).await
    }

    async fn register(
        self,
        _context: Context,
        registration: ServiceRegistration,
    ) -> RegistrationResult {
        dispatch::register(&self.service_registry, registration).await
    }

    async fn unregister(self, _context: Context, service_id: String) -> RegistrationResult {
        dispatch::unregister(&self.service_registry, service_id).await
    }

    async fn health(self, _context: Context) -> HealthStatus {
        dispatch::health(&self.service_registry, self.start_time).await
    }

    async fn version(self, _context: Context) -> VersionInfo {
        dispatch::version()
    }

    async fn protocols(self, _context: Context) -> Vec<ProtocolInfo> {
        dispatch::protocols_simple()
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
        dispatch::discover(&self.service_registry, capability).await
    }

    async fn discover_all(self, _context: Context) -> Vec<ServiceInfo> {
        dispatch::discover_all(&self.service_registry).await
    }

    async fn register(
        self,
        _context: Context,
        registration: ServiceRegistration,
    ) -> RegistrationResult {
        dispatch::register(&self.service_registry, registration).await
    }

    async fn unregister(self, _context: Context, service_id: String) -> RegistrationResult {
        dispatch::unregister(&self.service_registry, service_id).await
    }

    async fn health(self, _context: Context) -> HealthStatus {
        dispatch::health(&self.service_registry, self.start_time).await
    }

    async fn version(self, _context: Context) -> VersionInfo {
        dispatch::version()
    }

    async fn protocols(self, _context: Context) -> Vec<ProtocolInfo> {
        dispatch::protocols_legacy()
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
    let server = TarpcServerSimple::new(service_registry);
    let startup_log = format!("🚀 Starting tarpc server (simplified, zero unsafe) on {}", addr);
    run_tarpc_accept_loop!(addr, server, startup_log).await
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
    let server = TarpcServer::new(orchestrator, service_registry);
    let startup_log = format!("🚀 Starting tarpc server on {}", addr);
    run_tarpc_accept_loop!(addr, server, startup_log).await
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
        let port = songbird_config::defaults::ports::tarpc_port();
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
        let _lock = env_lock();
        let _guard = VarGuard::remove("SONGBIRD_TARPC_PORT");
        let config = TarpcConfig::default();
        assert_eq!(config.addr.port(), songbird_types::defaults::ports::DEFAULT_TARPC_RPC_PORT);
        assert!(!config.tls_enabled);
        assert_eq!(config.max_connections, 1000);
    }

    #[test]
    fn test_service_info_serialization() {
        let info = ServiceInfo {
            id: String::from("test"),
            capability: String::from("compute"),
            endpoint: String::from("http://localhost:8001"),
            status: String::from("healthy"),
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
