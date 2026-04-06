// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # tarpc Client for Songbird
//!
//! **HIGH-PERFORMANCE PRIMAL-TO-PRIMAL RPC CLIENT** (v3.12.0)
//!
//! Provides an async tarpc client for connecting to Songbird services.
//!
//! Submodules: `endpoint` (URL parsing), `connection` (TCP + transport), `ops` (typed RPC),
//! `json_api` (dynamic JSON dispatch for adapters).

mod connection;
mod endpoint;
mod json_api;
mod ops;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;

use crate::tarpc_types::SongbirdRpcClient;

/// Modern async tarpc client for Songbird
#[derive(Clone)]
pub struct TarpcClient {
    pub(super) endpoint: String,
    pub(super) addr: SocketAddr,
    pub(super) connection: Arc<RwLock<Option<SongbirdRpcClient>>>,
    pub(super) timeout: Duration,
}

impl TarpcClient {
    /// Create new tarpc client from endpoint (`tarpc://host:port`).
    pub fn new(endpoint: &str) -> songbird_types::SongbirdResult<Self> {
        endpoint::new_client(endpoint)
    }

    /// Set request timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl std::fmt::Debug for TarpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TarpcClient")
            .field("endpoint", &self.endpoint)
            .field("addr", &self.addr)
            .field("timeout", &self.timeout)
            .field("connection", &"<connection>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::tarpc_types::{
        HealthStatus, ProtocolInfo, RegistrationResult, ServiceInfo, ServiceRegistration,
        VersionInfo,
    };
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_endpoint_parsing_valid() {
        let addr = TarpcClient::parse_endpoint("tarpc://localhost:9001").unwrap();
        assert_eq!(addr.port(), 9001);
    }

    #[test]
    fn test_endpoint_parsing_with_ip() {
        let addr = TarpcClient::parse_endpoint("tarpc://127.0.0.1:9002").unwrap();
        assert_eq!(addr.port(), 9002);
    }

    #[test]
    fn test_endpoint_parsing_invalid_no_prefix() {
        let result = TarpcClient::parse_endpoint("localhost:9001");
        assert!(result.is_err());
    }

    #[test]
    fn test_endpoint_parsing_invalid_address() {
        let result = TarpcClient::parse_endpoint("tarpc://invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_client_creation() {
        let client = TarpcClient::new("tarpc://localhost:9001").unwrap();
        assert_eq!(client.endpoint, "tarpc://localhost:9001");
        assert_eq!(client.addr.port(), 9001);
    }

    #[test]
    fn test_with_timeout_builder() {
        let client = TarpcClient::new("tarpc://localhost:9001")
            .unwrap()
            .with_timeout(Duration::from_secs(10));

        assert_eq!(client.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_debug_impl() {
        let client = TarpcClient::new("tarpc://localhost:9001").unwrap();
        let debug_str = format!("{:?}", client);
        assert!(debug_str.contains("TarpcClient"));
        assert!(debug_str.contains("localhost:9001"));
    }

    #[test]
    fn test_parse_endpoint_localhost_localdomain() {
        let addr = TarpcClient::parse_endpoint("tarpc://localhost.localdomain:9003")
            .expect("localhost.localdomain resolves");
        assert_eq!(addr, "127.0.0.1:9003".parse().expect("socket addr"));
    }

    #[test]
    fn test_parse_endpoint_missing_port() {
        let err = TarpcClient::parse_endpoint("tarpc://localhost").expect_err("port required");
        assert!(err.to_string().contains("port") || err.to_string().contains("Invalid"));
    }

    #[test]
    fn test_parse_endpoint_invalid_port() {
        let err = TarpcClient::parse_endpoint("tarpc://127.0.0.1:notaport").expect_err("bad port");
        assert!(err.to_string().contains("port") || err.to_string().contains("Invalid"));
    }

    #[test]
    fn test_parse_endpoint_unknown_hostname_rejected() {
        let err = TarpcClient::parse_endpoint("tarpc://not-a-resolvable-name.example:9000")
            .expect_err("non-IP host must be localhost");
        assert!(err.to_string().contains("Invalid") || err.to_string().contains("hostname"));
    }

    #[test]
    fn test_parse_endpoint_ipv6() {
        let addr = TarpcClient::parse_endpoint("tarpc://[::1]:9004").expect("IPv6 bracket addr");
        assert_eq!(addr.port(), 9004);
    }

    #[tokio::test]
    async fn test_call_method_unknown_no_connection() {
        let client = TarpcClient::new("tarpc://127.0.0.1:59998").expect("client");
        let err = client.call_method("unknown_method", None).await.expect_err("unknown method");
        assert!(err.to_string().contains("Unknown method"));
    }

    #[tokio::test]
    async fn test_call_method_discover_missing_capability() {
        let client = TarpcClient::new("tarpc://127.0.0.1:59997").expect("client");
        let err =
            client.call_method("discover", Some(json!({}))).await.expect_err("missing capability");
        assert!(err.to_string().contains("capability") || err.to_string().contains("Missing"));
    }

    #[tokio::test]
    async fn test_call_method_register_missing_body() {
        let client = TarpcClient::new("tarpc://127.0.0.1:59996").expect("client");
        let err = client.call_method("register", None).await.expect_err("missing registration");
        assert!(err.to_string().contains("registration") || err.to_string().contains("Missing"));
    }

    #[tokio::test]
    async fn test_call_method_unregister_missing_service_id() {
        let client = TarpcClient::new("tarpc://127.0.0.1:59995").expect("client");
        let err = client
            .call_method("unregister", Some(json!({})))
            .await
            .expect_err("missing service_id");
        assert!(err.to_string().contains("service_id") || err.to_string().contains("Missing"));
    }

    #[tokio::test]
    async fn test_call_method_register_invalid_json() {
        let client = TarpcClient::new("tarpc://127.0.0.1:59994").expect("client");
        let err = client
            .call_method("register", Some(json!("not-an-object")))
            .await
            .expect_err("invalid registration");
        assert!(
            err.to_string().contains("registration")
                || err.to_string().contains("serialize")
                || err.to_string().contains("Invalid")
        );
    }

    #[test]
    fn test_parse_endpoint_preserves_port_max() {
        let addr = TarpcClient::parse_endpoint("tarpc://127.0.0.1:65535").expect("addr");
        assert_eq!(addr.port(), 65535);
    }

    #[test]
    fn test_client_clone_shares_state() {
        let a = TarpcClient::new("tarpc://127.0.0.1:9001").expect("client");
        let b = a.clone();
        assert_eq!(a.endpoint, b.endpoint);
        assert_eq!(a.addr, b.addr);
    }

    #[tokio::test]
    async fn test_call_method_health_requires_network() {
        let client = TarpcClient::new("tarpc://127.0.0.1:59993").expect("client");
        let err = client.call_method("health", None).await.expect_err("no server");
        assert!(err.to_string().contains("tarpc") || err.to_string().contains("connect"));
    }

    #[tokio::test]
    async fn test_call_method_version_requires_network() {
        let client = TarpcClient::new("tarpc://127.0.0.1:59992").expect("client");
        let err = client.call_method("version", None).await.expect_err("no server");
        assert!(err.to_string().contains("tarpc") || err.to_string().contains("connect"));
    }

    #[tokio::test]
    async fn test_call_method_protocols_requires_network() {
        let client = TarpcClient::new("tarpc://127.0.0.1:59991").expect("client");
        let err = client.call_method("protocols", None).await.expect_err("no server");
        assert!(err.to_string().contains("tarpc") || err.to_string().contains("connect"));
    }

    #[tokio::test]
    async fn test_call_method_discover_all_requires_network() {
        let client = TarpcClient::new("tarpc://127.0.0.1:59990").expect("client");
        let err = client.call_method("discover_all", None).await.expect_err("no server");
        assert!(err.to_string().contains("tarpc") || err.to_string().contains("connect"));
    }

    #[test]
    fn test_new_strips_tarpc_prefix_only() {
        let c = TarpcClient::new("tarpc://localhost:5555").expect("client");
        assert!(c.endpoint.starts_with("tarpc://"));
        assert_eq!(c.addr.port(), 5555);
    }

    #[test]
    fn tarpc_service_info_json_roundtrip() {
        let info = ServiceInfo {
            id: "svc-1".into(),
            capability: "compute".into(),
            endpoint: "tarpc://127.0.0.1:9001".into(),
            status: "active".into(),
            metadata: Some(json!({"k": "v"})),
        };
        let json = serde_json::to_string(&info).expect("serialize");
        let back: ServiceInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, info.id);
        assert_eq!(back.capability, info.capability);
    }

    #[test]
    fn service_registration_json_roundtrip() {
        let mut meta = HashMap::new();
        meta.insert("a".into(), "b".into());
        let reg = ServiceRegistration {
            service_id: "r1".into(),
            service_name: "Reg".into(),
            capability: "storage".into(),
            endpoint: "http://x".into(),
            metadata: meta,
            tower_id: Some("t1".into()),
            tower_name: None,
        };
        let json = serde_json::to_string(&reg).expect("serialize");
        let back: ServiceRegistration = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.service_id, reg.service_id);
        assert_eq!(back.tower_id, reg.tower_id);
    }

    #[test]
    fn registration_result_json_roundtrip() {
        let r = RegistrationResult {
            success: true,
            message: "ok".into(),
        };
        let json = serde_json::to_string(&r).expect("serialize");
        let back: RegistrationResult = serde_json::from_str(&json).expect("deserialize");
        assert!(back.success);
        assert_eq!(back.message, "ok");
    }

    #[test]
    fn health_status_tarpc_type_json_roundtrip() {
        let h = HealthStatus {
            status: "healthy".into(),
            version: "1.0.0".into(),
            uptime_seconds: 42,
            services_count: 7,
        };
        let json = serde_json::to_string(&h).expect("serialize");
        let back: HealthStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.uptime_seconds, 42);
        assert_eq!(back.services_count, 7);
    }

    #[test]
    fn version_info_json_roundtrip() {
        let v = VersionInfo {
            version: "3.0".into(),
            protocol: "bincode".into(),
            capabilities: vec!["a".into(), "b".into()],
        };
        let json = serde_json::to_string(&v).expect("serialize");
        let back: VersionInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.capabilities, v.capabilities);
    }

    #[test]
    fn protocol_info_json_roundtrip() {
        let mut info_map = HashMap::new();
        info_map.insert("x".into(), "y".into());
        let p = ProtocolInfo {
            name: "tarpc".into(),
            port: 9001,
            enabled: true,
            info: info_map,
        };
        let json = serde_json::to_string(&p).expect("serialize");
        let back: ProtocolInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.port, 9001);
        assert!(back.enabled);
    }
}
