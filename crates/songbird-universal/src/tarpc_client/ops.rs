// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Typed tarpc service calls (discovery, registration, health, version, protocols).

use tracing::debug;

use crate::tarpc_types::{
    HealthStatus, ProtocolInfo, RegistrationResult, ServiceInfo, ServiceRegistration, VersionInfo,
};
use songbird_types::{SongbirdError, SongbirdResult};

use super::TarpcClient;

impl TarpcClient {
    /// Discover services by capability.
    pub async fn discover(&self, capability: &str) -> SongbirdResult<Vec<ServiceInfo>> {
        debug!("Discovering services with capability: {}", capability);
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .discover(ctx, capability.to_string())
            .await
            .map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }

    /// Discover all available services.
    pub async fn discover_all(&self) -> SongbirdResult<Vec<ServiceInfo>> {
        debug!("Discovering all services");
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .discover_all(ctx)
            .await
            .map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }

    /// Register a service with the remote registry.
    pub async fn register(
        &self,
        registration: ServiceRegistration,
    ) -> SongbirdResult<RegistrationResult> {
        debug!("Registering service: {}", registration.service_id);
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .register(ctx, registration)
            .await
            .map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }

    /// Unregister a service by id.
    pub async fn unregister(&self, service_id: &str) -> SongbirdResult<RegistrationResult> {
        debug!("Unregistering service: {}", service_id);
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .unregister(ctx, service_id.to_string())
            .await
            .map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }

    /// Remote health status.
    pub async fn health(&self) -> SongbirdResult<HealthStatus> {
        debug!("Checking health status");
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client.health(ctx).await.map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }

    /// Protocol and build version info.
    pub async fn version(&self) -> SongbirdResult<VersionInfo> {
        debug!("Getting version information");
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client.version(ctx).await.map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }

    /// Supported wire protocols and ports.
    pub async fn protocols(&self) -> SongbirdResult<Vec<ProtocolInfo>> {
        debug!("Getting available protocols");
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .protocols(ctx)
            .await
            .map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use std::collections::HashMap;
    use std::net::SocketAddr;
    use std::time::Duration;

    use futures_util::StreamExt;
    use songbird_types::SongbirdError;
    use tarpc::context::Context;
    use tarpc::server::Channel;

    use crate::tarpc_types::{
        HealthStatus, ProtocolInfo, RegistrationResult, ServiceInfo, ServiceRegistration,
        SongbirdRpc, VersionInfo,
    };

    use super::TarpcClient;

    /// Minimal in-process tarpc server for exercising [`TarpcClient`] RPC wrappers (no mocks in
    /// `songbird-test-utils` for tarpc).
    #[derive(Clone)]
    struct MockSongbirdRpcServer;

    impl SongbirdRpc for MockSongbirdRpcServer {
        async fn discover(self, _ctx: Context, capability: String) -> Vec<ServiceInfo> {
            vec![ServiceInfo {
                id: format!("svc-{capability}"),
                capability,
                endpoint: "tarpc://127.0.0.1:7001".into(),
                status: "active".into(),
                // `serde_json::Value` does not round-trip through bincode the same way as JSON;
                // wire format matches production servers that omit metadata here.
                metadata: None,
            }]
        }

        async fn discover_all(self, _ctx: Context) -> Vec<ServiceInfo> {
            vec![ServiceInfo {
                id: "all-1".into(),
                capability: "any".into(),
                endpoint: "tarpc://127.0.0.1:7002".into(),
                status: "active".into(),
                metadata: None,
            }]
        }

        async fn register(
            self,
            _ctx: Context,
            registration: ServiceRegistration,
        ) -> RegistrationResult {
            RegistrationResult {
                success: true,
                message: format!("registered:{}", registration.service_id),
            }
        }

        async fn unregister(self, _ctx: Context, service_id: String) -> RegistrationResult {
            RegistrationResult {
                success: true,
                message: format!("unregistered:{service_id}"),
            }
        }

        async fn health(self, _ctx: Context) -> HealthStatus {
            HealthStatus {
                status: "healthy".into(),
                version: "9.9.9".into(),
                uptime_seconds: 123,
                services_count: 3,
            }
        }

        async fn version(self, _ctx: Context) -> VersionInfo {
            VersionInfo {
                version: "1.2.3".into(),
                protocol: "bincode".into(),
                capabilities: vec!["a".into(), "b".into()],
            }
        }

        async fn protocols(self, _ctx: Context) -> Vec<ProtocolInfo> {
            let mut info = HashMap::new();
            info.insert("k".into(), "v".into());
            vec![ProtocolInfo {
                name: "tarpc".into(),
                port: 9001,
                enabled: true,
                info,
            }]
        }
    }

    /// Binds `127.0.0.1:0`, serves [`SongbirdRpc`] over bincode + length-delimited frames (same as
    /// production). Abort the returned handle to stop accepting.
    async fn spawn_mock_tarpc_server(
        server: MockSongbirdRpcServer,
    ) -> (SocketAddr, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local_addr");
        let handle = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                let server = server.clone();
                tokio::spawn(async move {
                    let transport = tarpc::serde_transport::new(
                        tokio_util::codec::LengthDelimitedCodec::builder()
                            .max_frame_length(16 * 1024 * 1024)
                            .new_framed(stream),
                        tokio_serde::formats::Bincode::default(),
                    );
                    let channel = tarpc::server::BaseChannel::with_defaults(transport);
                    channel
                        .execute(server.serve())
                        .for_each(|response| async move {
                            tokio::spawn(response);
                        })
                        .await;
                });
            }
        });
        (addr, handle)
    }

    /// Reserves a free TCP port briefly so nothing is listening when the client connects.
    async fn closed_local_addr() -> SocketAddr {
        let s = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
        s.local_addr().expect("addr")
    }

    /// Accepts one TCP connection and sends non-tarpc bytes so the bincode client fails (RPC path).
    async fn spawn_garbage_tcp_peer() -> (SocketAddr, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let h = tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            let (mut stream, _) = listener.accept().await.expect("accept");
            let _ = stream.write_all(b"not-bincode-tarpc").await;
            let _ = stream.shutdown().await;
        });
        (addr, h)
    }

    fn assert_rpc_transport_error(err: SongbirdError) {
        assert!(matches!(err, SongbirdError::Rpc { .. }), "expected RPC error, got {err:?}");
        assert!(err.to_string().contains("tarpc call failed"), "{}", err);
    }

    #[tokio::test]
    async fn discover_happy_path() {
        let (addr, server) = spawn_mock_tarpc_server(MockSongbirdRpcServer).await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        let out = client.discover("compute").await.expect("discover");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "svc-compute");
        assert_eq!(out[0].capability, "compute");
        server.abort();
    }

    #[tokio::test]
    async fn discover_all_happy_path() {
        let (addr, server) = spawn_mock_tarpc_server(MockSongbirdRpcServer).await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        let out = client.discover_all().await.expect("discover_all");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "all-1");
        server.abort();
    }

    #[tokio::test]
    async fn register_happy_path() {
        let (addr, server) = spawn_mock_tarpc_server(MockSongbirdRpcServer).await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        let reg = ServiceRegistration {
            service_id: "sid-1".into(),
            service_name: "n".into(),
            capability: "c".into(),
            endpoint: "http://e".into(),
            metadata: HashMap::from([("m".into(), "d".into())]),
            tower_id: Some("t1".into()),
            tower_name: None,
        };
        let r = client.register(reg.clone()).await.expect("register");
        assert!(r.success);
        assert!(r.message.contains("sid-1"));
        server.abort();
    }

    #[tokio::test]
    async fn unregister_happy_path() {
        let (addr, server) = spawn_mock_tarpc_server(MockSongbirdRpcServer).await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        let r = client.unregister("to-drop").await.expect("unregister");
        assert!(r.success);
        assert!(r.message.contains("to-drop"));
        server.abort();
    }

    #[tokio::test]
    async fn health_happy_path() {
        let (addr, server) = spawn_mock_tarpc_server(MockSongbirdRpcServer).await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        let h = client.health().await.expect("health");
        assert_eq!(h.status, "healthy");
        assert_eq!(h.uptime_seconds, 123);
        server.abort();
    }

    #[tokio::test]
    async fn version_happy_path() {
        let (addr, server) = spawn_mock_tarpc_server(MockSongbirdRpcServer).await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        let v = client.version().await.expect("version");
        assert_eq!(v.version, "1.2.3");
        assert_eq!(v.protocol, "bincode");
        server.abort();
    }

    #[tokio::test]
    async fn protocols_happy_path() {
        let (addr, server) = spawn_mock_tarpc_server(MockSongbirdRpcServer).await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        let p = client.protocols().await.expect("protocols");
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].name, "tarpc");
        assert_eq!(p[0].port, 9001);
        assert_eq!(p[0].info.get("k"), Some(&String::from("v")));
        server.abort();
    }

    #[tokio::test]
    async fn connection_refused_is_network_error() {
        let addr = closed_local_addr().await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        let err = client.discover("x").await.expect_err("refused");
        assert!(
            matches!(err, SongbirdError::Network { .. }),
            "expected network error, got {err:?}"
        );
        let msg = err.to_string();
        assert!(msg.contains("Failed to connect") || msg.contains("connect"), "unexpected: {msg}");
    }

    #[tokio::test]
    async fn zero_connect_timeout_surfaces_network_timeout() {
        let addr = closed_local_addr().await;
        let client = TarpcClient::new(&format!("tarpc://{addr}"))
            .expect("client")
            .with_timeout(Duration::ZERO);
        let err = client.health().await.expect_err("timeout or refused");
        assert!(
            matches!(err, SongbirdError::Network { .. }),
            "expected network error, got {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("timeout") || msg.contains("Timeout") || msg.contains("connect"),
            "unexpected: {msg}"
        );
    }

    /// Virtual time: `start_paused` requires `time::advance` for `sleep` to complete (TCP still uses
    /// wall time; connect failures/timeouts are covered by other tests).
    #[tokio::test(start_paused = true)]
    async fn paused_timer_advances_for_sleep() {
        let start = tokio::time::Instant::now();
        let sleeper = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
        });
        tokio::time::advance(Duration::from_secs(2)).await;
        sleeper.await.expect("sleep task");
        assert!(start.elapsed() >= Duration::from_secs(2));
    }

    #[tokio::test]
    async fn non_tarpc_peer_yields_rpc_error_discover() {
        let (addr, bogus) = spawn_garbage_tcp_peer().await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        assert_rpc_transport_error(client.discover("c").await.expect_err("garbage"));
        let _ = bogus.await;
    }

    #[tokio::test]
    async fn non_tarpc_peer_yields_rpc_error_discover_all() {
        let (addr, bogus) = spawn_garbage_tcp_peer().await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        assert_rpc_transport_error(client.discover_all().await.expect_err("garbage"));
        let _ = bogus.await;
    }

    #[tokio::test]
    async fn non_tarpc_peer_yields_rpc_error_register() {
        let (addr, bogus) = spawn_garbage_tcp_peer().await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        let reg = ServiceRegistration {
            service_id: "s".into(),
            service_name: "n".into(),
            capability: "c".into(),
            endpoint: "e".into(),
            metadata: HashMap::new(),
            tower_id: None,
            tower_name: None,
        };
        assert_rpc_transport_error(client.register(reg).await.expect_err("garbage"));
        let _ = bogus.await;
    }

    #[tokio::test]
    async fn non_tarpc_peer_yields_rpc_error_unregister() {
        let (addr, bogus) = spawn_garbage_tcp_peer().await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        assert_rpc_transport_error(client.unregister("id").await.expect_err("garbage"));
        let _ = bogus.await;
    }

    #[tokio::test]
    async fn non_tarpc_peer_yields_rpc_error_health() {
        let (addr, bogus) = spawn_garbage_tcp_peer().await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        assert_rpc_transport_error(client.health().await.expect_err("garbage"));
        let _ = bogus.await;
    }

    #[tokio::test]
    async fn non_tarpc_peer_yields_rpc_error_version() {
        let (addr, bogus) = spawn_garbage_tcp_peer().await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        assert_rpc_transport_error(client.version().await.expect_err("garbage"));
        let _ = bogus.await;
    }

    #[tokio::test]
    async fn non_tarpc_peer_yields_rpc_error_protocols() {
        let (addr, bogus) = spawn_garbage_tcp_peer().await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        assert_rpc_transport_error(client.protocols().await.expect_err("garbage"));
        let _ = bogus.await;
    }

    #[test]
    fn tarpc_ops_types_bincode_roundtrip() {
        let svc = ServiceInfo {
            id: "i".into(),
            capability: "c".into(),
            endpoint: "e".into(),
            status: "s".into(),
            metadata: None,
        };
        let bytes = bincode::serialize(&svc).expect("serialize svc");
        let back: ServiceInfo = bincode::deserialize(&bytes).expect("deserialize svc");
        assert_eq!(back.id, svc.id);
        assert_eq!(back.metadata, svc.metadata);

        let mut meta = HashMap::new();
        meta.insert("a".into(), "b".into());
        let reg = ServiceRegistration {
            service_id: "r".into(),
            service_name: "n".into(),
            capability: "k".into(),
            endpoint: "ep".into(),
            metadata: meta,
            tower_id: Some("tid".into()),
            tower_name: Some("tn".into()),
        };
        let bytes = bincode::serialize(&reg).expect("serialize reg");
        let back: ServiceRegistration = bincode::deserialize(&bytes).expect("deserialize reg");
        assert_eq!(back.service_id, reg.service_id);
        assert_eq!(back.tower_name, reg.tower_name);

        let rr = RegistrationResult {
            success: false,
            message: "m".into(),
        };
        let bytes = bincode::serialize(&rr).expect("serialize rr");
        let back: RegistrationResult = bincode::deserialize(&bytes).expect("deserialize rr");
        assert!(!back.success);

        let h = HealthStatus {
            status: "h".into(),
            version: "v".into(),
            uptime_seconds: 9,
            services_count: 2,
        };
        let bytes = bincode::serialize(&h).expect("serialize h");
        let back: HealthStatus = bincode::deserialize(&bytes).expect("deserialize h");
        assert_eq!(back.uptime_seconds, 9);

        let v = VersionInfo {
            version: "1".into(),
            protocol: "p".into(),
            capabilities: vec!["x".into()],
        };
        let bytes = bincode::serialize(&v).expect("serialize v");
        let back: VersionInfo = bincode::deserialize(&bytes).expect("deserialize v");
        assert_eq!(back.capabilities, v.capabilities);

        let mut info = HashMap::new();
        info.insert("q".into(), "w".into());
        let p = ProtocolInfo {
            name: "n".into(),
            port: 42,
            enabled: false,
            info,
        };
        let bytes = bincode::serialize(&p).expect("serialize p");
        let back: ProtocolInfo = bincode::deserialize(&bytes).expect("deserialize p");
        assert_eq!(back.port, p.port);
        assert_eq!(back.info.get("q"), Some(&String::from("w")));
    }

    #[test]
    fn service_info_with_metadata_json_roundtrip() {
        use serde_json::json;
        let svc = ServiceInfo {
            id: "id1".into(),
            capability: "cap".into(),
            endpoint: "tarpc://127.0.0.1:1".into(),
            status: "active".into(),
            metadata: Some(json!({ "k": 1 })),
        };
        let json = serde_json::to_string(&svc).expect("json serialize");
        let back: ServiceInfo = serde_json::from_str(&json).expect("json deserialize");
        assert_eq!(back.metadata, svc.metadata);
    }

    #[tokio::test]
    async fn discover_with_empty_capability_string_succeeds() {
        let (addr, server) = spawn_mock_tarpc_server(MockSongbirdRpcServer).await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        let out = client.discover("").await.expect("discover");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "svc-");
        assert_eq!(out[0].capability, "");
        server.abort();
    }

    #[tokio::test]
    async fn unregister_with_empty_service_id_succeeds() {
        let (addr, server) = spawn_mock_tarpc_server(MockSongbirdRpcServer).await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        let r = client.unregister("").await.expect("unregister");
        assert!(r.success);
        assert_eq!(r.message, "unregistered:");
        server.abort();
    }

    #[tokio::test]
    async fn sequential_operations_on_one_client_reuse_connection() {
        let (addr, server) = spawn_mock_tarpc_server(MockSongbirdRpcServer).await;
        let client = TarpcClient::new(&format!("tarpc://{addr}")).expect("client");
        let d = client.discover("reuse").await.expect("discover");
        assert_eq!(d[0].capability, "reuse");
        let h = client.health().await.expect("health");
        assert_eq!(h.status, "healthy");
        let v = client.version().await.expect("version");
        assert_eq!(v.version, "1.2.3");
        let p = client.protocols().await.expect("protocols");
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].name, "tarpc");
        server.abort();
    }

    #[test]
    fn protocol_info_serde_json_roundtrip_empty_info_map() {
        let p = ProtocolInfo {
            name: "grpc".into(),
            port: 50051,
            enabled: false,
            info: HashMap::new(),
        };
        let json = serde_json::to_string(&p).expect("json serialize");
        let back: ProtocolInfo = serde_json::from_str(&json).expect("json deserialize");
        assert!(back.info.is_empty());
        assert_eq!(back.name, "grpc");
        assert_eq!(back.port, 50051);
    }

    #[test]
    fn health_status_serde_json_roundtrip_zero_values() {
        let h = HealthStatus {
            status: String::new(),
            version: String::new(),
            uptime_seconds: 0,
            services_count: 0,
        };
        let json = serde_json::to_string(&h).expect("json serialize");
        let back: HealthStatus = serde_json::from_str(&json).expect("json deserialize");
        assert_eq!(back.uptime_seconds, 0);
        assert_eq!(back.services_count, 0);
        assert!(back.status.is_empty());
        assert!(back.version.is_empty());
    }
}
