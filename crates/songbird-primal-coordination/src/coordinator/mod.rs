// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Primal Coordinator - Songbird's central orchestration
//!
//! **ZERO HARDCODING**: Coordinates by capability, not by primal name

mod operations;
mod types;

pub use types::{CoordinatorConfig, MeshConnection, PrimalHealthStatus};

use crate::bridge::{PrimalBridge, PrimalConnection};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The central orchestrator within Songbird
///
/// Manages connections and interactions with primals without knowing their specific names.
/// Everything is capability-based discovery.
pub struct PrimalCoordinator {
    /// Capability-based bridge for discovering primals
    bridge: Arc<dyn PrimalBridge>,

    /// Active connections to primals (by capability) — `Arc<str>` keys share capability strings cheaply.
    active_connections: Arc<RwLock<HashMap<Arc<str>, PrimalConnection>>>,

    /// Connection pool configuration
    config: CoordinatorConfig,
}

impl PrimalCoordinator {
    /// Create a new primal coordinator with default config
    #[must_use]
    pub fn new(bridge: Arc<dyn PrimalBridge>) -> Self {
        Self::with_config(bridge, CoordinatorConfig::default())
    }

    /// Create a new primal coordinator with custom config
    #[must_use]
    pub fn with_config(bridge: Arc<dyn PrimalBridge>, config: CoordinatorConfig) -> Self {
        tracing::info!("🌳 Initializing Primal Coordinator (zero hardcoded knowledge)");
        Self {
            bridge,
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::bridge::*;
    use crate::error::{PrimalCoordinationError, Result};
    use crate::types::{CapabilityType, NodeId, Workload};
    use crate::{PrimalCapabilities, ServiceQuality};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockBridge;

    #[async_trait::async_trait]
    impl PrimalBridge for MockBridge {
        async fn connect(&self, capability: CapabilityType) -> Result<PrimalConnection> {
            let caps = PrimalCapabilities {
                services: vec![capability.as_str().to_string()],
                resources: std::collections::HashMap::new(),
                metadata: std::collections::HashMap::new(),
                quality: ServiceQuality::default(),
            };
            Ok(PrimalConnection::new(
                uuid::Uuid::new_v4().to_string(),
                format!("http://localhost:8080/{}", capability.as_str()),
                caps,
            ))
        }

        async fn discover_capabilities(
            &self,
            _connection: &PrimalConnection,
        ) -> Result<PrimalCapabilities> {
            Ok(PrimalCapabilities {
                services: vec!["security".to_string()],
                resources: std::collections::HashMap::new(),
                metadata: std::collections::HashMap::new(),
                quality: ServiceQuality::default(),
            })
        }

        fn supported_capabilities(&self) -> Vec<CapabilityType> {
            vec![CapabilityType::Security, CapabilityType::Compute]
        }
    }

    struct FailingBridge;

    #[async_trait::async_trait]
    impl PrimalBridge for FailingBridge {
        async fn connect(&self, _capability: CapabilityType) -> Result<PrimalConnection> {
            Err(PrimalCoordinationError::ConnectionFailed("mock".into()))
        }

        async fn discover_capabilities(
            &self,
            _connection: &PrimalConnection,
        ) -> Result<PrimalCapabilities> {
            Err(PrimalCoordinationError::Internal("not used".into()))
        }

        fn supported_capabilities(&self) -> Vec<CapabilityType> {
            vec![]
        }
    }

    struct CountingBridge {
        connects: AtomicUsize,
    }

    impl CountingBridge {
        fn new() -> Self {
            Self {
                connects: AtomicUsize::new(0),
            }
        }

        fn connect_count(&self) -> usize {
            self.connects.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl PrimalBridge for CountingBridge {
        async fn connect(&self, capability: CapabilityType) -> Result<PrimalConnection> {
            self.connects.fetch_add(1, Ordering::SeqCst);
            let caps = PrimalCapabilities {
                services: vec![capability.as_str().to_string()],
                resources: std::collections::HashMap::new(),
                metadata: std::collections::HashMap::new(),
                quality: ServiceQuality::default(),
            };
            Ok(PrimalConnection::new(
                "counting-id".into(),
                format!("http://127.0.0.1:9/{}", capability.as_str()),
                caps,
            ))
        }

        async fn discover_capabilities(
            &self,
            _connection: &PrimalConnection,
        ) -> Result<PrimalCapabilities> {
            Ok(PrimalCapabilities {
                services: vec![],
                resources: std::collections::HashMap::new(),
                metadata: std::collections::HashMap::new(),
                quality: ServiceQuality::default(),
            })
        }

        fn supported_capabilities(&self) -> Vec<CapabilityType> {
            vec![CapabilityType::Security]
        }
    }

    #[tokio::test(start_paused = true)]
    async fn test_coordinator_creation() {
        let bridge = Arc::new(MockBridge);
        let coordinator = PrimalCoordinator::new(bridge);
        assert_eq!(
            coordinator.config.max_connections_per_capability, 10,
            "default CoordinatorConfig::default max_connections_per_capability"
        );
        assert!(coordinator.config.enable_pooling);
    }

    #[tokio::test(start_paused = true)]
    async fn request_capability_propagates_bridge_errors() {
        let coordinator = PrimalCoordinator::new(Arc::new(FailingBridge));
        let err = coordinator
            .request_capability(CapabilityType::Security)
            .await
            .expect_err("failing bridge should surface ConnectionFailed");
        assert!(matches!(err, PrimalCoordinationError::ConnectionFailed(_)), "got {err:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn test_request_capability() {
        let bridge = Arc::new(MockBridge);
        let coordinator = PrimalCoordinator::new(bridge);

        let conn = coordinator
            .request_capability(CapabilityType::Security)
            .await
            .expect("mock bridge connects");
        assert!(
            conn.supports_capability(&CapabilityType::Security).await,
            "connection should advertise security"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn test_capability_caching_when_pooling_enabled() {
        let bridge = Arc::new(MockBridge);
        let coordinator = PrimalCoordinator::new(bridge);

        let conn1 =
            coordinator.request_capability(CapabilityType::Security).await.expect("first request");
        let id1 = conn1.connection_id.clone();

        let conn2 =
            coordinator.request_capability(CapabilityType::Security).await.expect("second request");
        let id2 = conn2.connection_id;

        assert_eq!(id1, id2, "Should return cached connection when pooling is on");
    }

    #[tokio::test(start_paused = true)]
    async fn request_capability_without_pooling_calls_connect_each_time() {
        let inner: Arc<CountingBridge> = Arc::new(CountingBridge::new());
        let bridge: Arc<dyn PrimalBridge> = inner.clone();
        let config = CoordinatorConfig {
            max_connections_per_capability: 10,
            connection_timeout_secs: 30,
            health_check_interval_secs: 60,
            enable_pooling: false,
        };
        let coordinator = PrimalCoordinator::with_config(bridge, config);
        coordinator.request_capability(CapabilityType::Security).await.expect("first");
        coordinator.request_capability(CapabilityType::Security).await.expect("second");
        assert_eq!(
            inner.connect_count(),
            2,
            "without pooling each request_capability should call bridge.connect"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn deploy_compute_errors_when_no_primal_supports_workload() {
        let bridge = Arc::new(MockBridge);
        let coordinator = PrimalCoordinator::new(bridge);
        let workload = Workload {
            id: "w1".into(),
            service_type: "unknown-service".into(),
            requirements: std::collections::HashMap::new(),
            payload: serde_json::json!({}),
        };
        let err = coordinator
            .deploy_compute(workload)
            .await
            .expect_err("compute primal only advertises 'compute' service");
        assert!(
            matches!(err, PrimalCoordinationError::NoCapablePrimal(_)),
            "expected NoCapablePrimal, got {err:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn coordinate_genesis_fails_when_key_generation_unreachable() {
        let bridge = Arc::new(MockBridge);
        let config = CoordinatorConfig {
            enable_pooling: false,
            ..CoordinatorConfig::default()
        };
        let coordinator = PrimalCoordinator::with_config(bridge, config);
        let err = coordinator
            .coordinate_genesis(NodeId("node-a".into()))
            .await
            .expect_err("GenerateKeys requires IPC-backed HTTP");
        assert!(
            matches!(err, PrimalCoordinationError::Internal(_)),
            "expected Internal from transport/IPC, got {err:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn health_check_all_reports_error_when_status_request_fails() {
        let bridge = Arc::new(MockBridge);
        let coordinator = PrimalCoordinator::new(bridge);
        coordinator.request_capability(CapabilityType::Security).await.expect("seed cache");

        let statuses = coordinator
            .health_check_all()
            .await
            .expect("health_check_all should not fail on per-connection errors");
        assert_eq!(statuses.len(), 1, "one cached capability");
        let s = &statuses[0];
        assert_eq!(s.capability.as_ref(), "security");
        assert!(!s.healthy, "status RPC should fail without IPC");
        assert_eq!(s.version, "error", "send_request error maps to version error");
    }

    #[tokio::test(start_paused = true)]
    async fn health_check_all_empty_when_nothing_cached() {
        let bridge = Arc::new(MockBridge);
        let coordinator = PrimalCoordinator::with_config(
            bridge,
            CoordinatorConfig {
                enable_pooling: false,
                ..CoordinatorConfig::default()
            },
        );
        let statuses =
            coordinator.health_check_all().await.expect("empty registry yields empty vec");
        assert!(statuses.is_empty(), "no cached connections means no health rows");
    }

    #[tokio::test(start_paused = true)]
    async fn test_service_mesh_coordination() {
        let bridge = Arc::new(MockBridge);
        let coordinator = PrimalCoordinator::new(bridge);

        let mesh = coordinator
            .coordinate_service_mesh(CapabilityType::Compute, CapabilityType::Security)
            .await
            .expect("mesh coordination");

        assert!(!mesh.id.is_empty(), "mesh id should be a non-empty UUID string");
        assert_eq!(mesh.requester_capability, CapabilityType::Compute);
        assert_eq!(mesh.provider_capability, CapabilityType::Security);
        let clone = mesh.clone();
        assert_eq!(clone.id, mesh.id);
    }

    #[test]
    fn mesh_connection_and_health_status_debug() {
        let m = MeshConnection {
            id: "mid".into(),
            requester_endpoint: Arc::from("http://a/"),
            provider_endpoint: Arc::from("http://b/"),
            requester_capability: CapabilityType::Ai,
            provider_capability: CapabilityType::Storage,
        };
        let dbg = format!("{m:?}");
        assert!(dbg.contains("mid"), "Debug should include id: {dbg}");

        let h = PrimalHealthStatus {
            capability: Arc::from("compute"),
            endpoint: Arc::from("http://c/"),
            healthy: true,
            version: "2".into(),
        };
        assert!(format!("{h:?}").contains("compute"));
    }

    #[tokio::test(start_paused = true)]
    async fn deploy_compute_passes_capability_gate_then_fails_on_ipc() {
        let bridge = Arc::new(MockBridge);
        let coordinator = PrimalCoordinator::new(bridge);
        let workload = Workload {
            id: "w-compute".into(),
            service_type: "compute".into(),
            requirements: std::collections::HashMap::new(),
            payload: serde_json::json!({}),
        };
        let err = coordinator
            .deploy_compute(workload)
            .await
            .expect_err("workload matches mock 'compute' service but IPC HTTP is unavailable");
        assert!(
            matches!(err, PrimalCoordinationError::Internal(_)),
            "expected Internal from send_request/IPC, got {err:?}"
        );
    }

    #[test]
    fn coordinator_config_default_values() {
        let c = CoordinatorConfig::default();
        assert_eq!(c.max_connections_per_capability, 10);
        assert_eq!(c.connection_timeout_secs, 30);
        assert_eq!(c.health_check_interval_secs, 60);
        assert!(c.enable_pooling);
    }
}
