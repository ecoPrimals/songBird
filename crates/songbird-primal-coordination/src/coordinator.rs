// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Primal Coordinator - Songbird's central orchestration
//!
//! **ZERO HARDCODING**: Coordinates by capability, not by primal name

use crate::{
    bridge::{PrimalBridge, PrimalConnection},
    error::{PrimalCoordinationError, Result},
    types::{
        CapabilityType, DeploymentId, Identity, NodeId, PrimalRequest, PrimalResponse,
        WitnessProof, Workload,
    },
};
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

/// Configuration for the coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Maximum connections per capability
    pub max_connections_per_capability: usize,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Health check interval in seconds
    pub health_check_interval_secs: u64,

    /// Enable connection pooling
    pub enable_pooling: bool,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_connections_per_capability: 10,
            connection_timeout_secs: 30,
            health_check_interval_secs: 60,
            enable_pooling: true,
        }
    }
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

    /// Request a capability from any primal that provides it
    ///
    /// **ZERO HARDCODING**: Discovers primal by capability, not name
    ///
    /// # Errors
    ///
    /// Returns an error if no primal with the capability can be found
    pub async fn request_capability(&self, capability: CapabilityType) -> Result<PrimalConnection> {
        tracing::info!("🔍 Requesting capability: {}", capability);

        // Check if we already have a connection for this capability
        let cache_key: Arc<str> = Arc::from(capability.as_str());
        {
            let connections = self.active_connections.read().await;
            if let Some(conn) = connections.get(&cache_key) {
                tracing::debug!("Found cached connection for capability: {}", capability);
                return Ok(conn.clone());
            }
        }

        // Discover and connect to a primal with this capability
        let connection = self.bridge.connect(capability.clone()).await?;

        // Cache the connection
        if self.config.enable_pooling {
            self.active_connections.write().await.insert(cache_key, connection.clone());
        }

        tracing::info!("✅ Connected to primal for capability: {}", capability);
        Ok(connection)
    }

    /// Coordinate a Genesis ceremony
    ///
    /// Songbird orchestrates the steps, delegating specific tasks to capability providers
    ///
    /// # Errors
    ///
    /// Returns an error if coordination fails
    pub async fn coordinate_genesis(&self, new_node_id: NodeId) -> Result<Identity> {
        tracing::info!("🌱 Songbird: Coordinating Genesis ceremony for node: {}", new_node_id);

        // 1. Request security capability (whoever provides it, e.g., a primal with security)
        let security_conn = self.request_capability(CapabilityType::Security).await?;

        // 2. Request key generation (security primal's responsibility)
        let key_gen_request = PrimalRequest::GenerateKeys;
        let key_gen_response = security_conn.send_request(key_gen_request).await?;
        let generated_keys = match key_gen_response {
            PrimalResponse::KeysGenerated(keys) => keys,
            PrimalResponse::Error(e) => {
                return Err(PrimalCoordinationError::PrimalError(format!(
                    "Key generation failed: {e}"
                )));
            }
            _ => {
                return Err(PrimalCoordinationError::UnexpectedResponse(
                    "Expected KeysGenerated response".into(),
                ));
            }
        };
        tracing::debug!("Songbird: Security primal generated keys.");

        // 3. Coordinate witness network (Songbird's responsibility)
        // This involves using Songbird's own P2P and BLE stack
        let witness_proof = self.coordinate_witness_network(&new_node_id).await?;
        tracing::debug!("Songbird: Witness network coordinated.");

        // 4. Request lineage signing (security primal's responsibility)
        let sign_request = PrimalRequest::SignLineage {
            keys: generated_keys.clone(),
            proof: witness_proof.clone(),
            node_id: new_node_id.clone(),
        };
        let sign_response = security_conn.send_request(sign_request).await?;
        let signed_lineage = match sign_response {
            PrimalResponse::LineageSigned(lineage) => lineage,
            PrimalResponse::Error(e) => {
                return Err(PrimalCoordinationError::PrimalError(format!(
                    "Lineage signing failed: {e}"
                )));
            }
            _ => {
                return Err(PrimalCoordinationError::UnexpectedResponse(
                    "Expected LineageSigned response".into(),
                ));
            }
        };
        tracing::debug!("Songbird: Security primal signed lineage.");

        // 5. Construct and return the final identity
        let identity = Identity {
            node_id: new_node_id.clone(),
            public_key: generated_keys.public_key,
            lineage: signed_lineage,
            witness_proof,
        };

        tracing::info!("🎉 Songbird: Genesis ceremony complete for node: {}", identity.node_id);
        Ok(identity)
    }

    /// Coordinate a compute workload deployment
    ///
    /// Songbird orchestrates the deployment, delegating execution to capability providers
    ///
    /// # Errors
    ///
    /// Returns an error if deployment fails
    pub async fn deploy_compute(&self, workload: Workload) -> Result<DeploymentId> {
        tracing::info!("🚀 Songbird: Deploying compute workload: {}", workload.id);

        // 1. Request compute capability (whoever provides it)
        let compute_conn = self.request_capability(CapabilityType::Compute).await?;

        // 2. Check if this primal can handle the workload
        let capabilities = compute_conn.get_capabilities().await;
        if !capabilities.supports_workload(&workload) {
            return Err(PrimalCoordinationError::NoCapablePrimal(format!(
                "No primal found for workload type: {}",
                workload.service_type
            )));
        }

        // 3. Send workload to compute primal (primal's responsibility to execute)
        let deploy_request = PrimalRequest::DeployWorkload(workload.clone());
        let deploy_response = compute_conn.send_request(deploy_request).await?;
        let deployment_id = match deploy_response {
            PrimalResponse::WorkloadDeployed(id) => id,
            PrimalResponse::Error(e) => {
                return Err(PrimalCoordinationError::PrimalError(format!(
                    "Deployment failed: {e}"
                )));
            }
            _ => {
                return Err(PrimalCoordinationError::UnexpectedResponse(
                    "Expected WorkloadDeployed response".into(),
                ));
            }
        };

        tracing::info!(
            "✅ Songbird: Workload {} deployed to compute primal with ID: {}",
            workload.id,
            deployment_id
        );
        Ok(deployment_id)
    }

    /// Coordinate service mesh connection
    ///
    /// Example: Songbird connects a service mesh for primal-to-primal communication
    /// (e.g., Toadstool provides compute for Squirrel analyzing `storage provider` data)
    ///
    /// # Errors
    ///
    /// Returns an error if coordination fails
    pub async fn coordinate_service_mesh(
        &self,
        requester_capability: CapabilityType,
        provider_capability: CapabilityType,
    ) -> Result<MeshConnection> {
        tracing::info!(
            "🕸️ Songbird: Coordinating service mesh: {} → {}",
            requester_capability,
            provider_capability
        );

        // 1. Get connection to requester
        let requester_conn = self.request_capability(requester_capability.clone()).await?;

        // 2. Get connection to provider
        let provider_conn = self.request_capability(provider_capability.clone()).await?;

        // 3. Establish mesh connection (Songbird coordinates, doesn't execute)
        let mesh_connection = MeshConnection {
            id: uuid::Uuid::new_v4().to_string(),
            requester_endpoint: Arc::clone(&requester_conn.endpoint),
            provider_endpoint: Arc::clone(&provider_conn.endpoint),
            requester_capability,
            provider_capability,
        };

        tracing::info!("✅ Songbird: Service mesh established: {}", mesh_connection.id);
        Ok(mesh_connection)
    }

    /// Get health status of all connected primals
    ///
    /// # Errors
    ///
    /// Returns an error if health check fails
    pub async fn health_check_all(&self) -> Result<Vec<PrimalHealthStatus>> {
        let mut statuses = Vec::new();

        for (capability, conn) in self.active_connections.read().await.iter() {
            let response = conn.send_request(PrimalRequest::Status).await;
            let status = match response {
                Ok(PrimalResponse::StatusResponse(s)) => PrimalHealthStatus {
                    capability: Arc::clone(capability),
                    endpoint: Arc::clone(&conn.endpoint),
                    healthy: s.healthy,
                    version: s.version,
                },
                Ok(_) => PrimalHealthStatus {
                    capability: Arc::clone(capability),
                    endpoint: Arc::clone(&conn.endpoint),
                    healthy: false,
                    version: "unknown".to_string(),
                },
                Err(_) => PrimalHealthStatus {
                    capability: Arc::clone(capability),
                    endpoint: Arc::clone(&conn.endpoint),
                    healthy: false,
                    version: "error".to_string(),
                },
            };
            statuses.push(status);
        }

        Ok(statuses)
    }

    /// Internal: Coordinate witness network using capability-based routing
    ///
    /// Uses the coordinator's registry (`active_connections`) to find primals that can
    /// participate in witness attestation. Prefers Networking and Discovery capabilities
    /// for P2P presence; falls back to any connected primal. Physical BLE proximity
    /// would be layered by Songbird's own stack when available.
    async fn coordinate_witness_network(&self, node_id: &NodeId) -> Result<WitnessProof> {
        let connections = self.active_connections.read().await;

        // Capability-based routing: prefer Networking/Discovery for witness (P2P presence)
        let witness_capabilities =
            [CapabilityType::Networking, CapabilityType::Discovery, CapabilityType::Security];

        let mut witness_data = Vec::new();
        for cap in &witness_capabilities {
            let cache_key: Arc<str> = Arc::from(cap.as_str());
            if let Some(conn) = connections.get(&cache_key) {
                let caps = conn.get_capabilities().await;
                witness_data.extend_from_slice(
                    format!("{}:{}", conn.connection_id, caps.services.join(",")).as_bytes(),
                );
                witness_data.push(b';');
            }
        }

        // If no capability-specific witnesses, use any connected primal from registry
        if witness_data.is_empty() {
            for (cap_key, conn) in connections.iter() {
                tracing::debug!("Witness fallback: using {} primal", cap_key);
                witness_data.extend_from_slice(
                    format!("{}:{}", conn.connection_id, conn.endpoint).as_bytes(),
                );
                witness_data.push(b';');
            }
        }

        let connection_count = connections.len();
        drop(connections);

        // Build proof: node_id + capability-derived witness attestations
        let mut proof = format!("genesis_witness:{node_id}:").into_bytes();
        if witness_data.is_empty() {
            proof.extend_from_slice(b"ble_proximity_proof");
        } else {
            proof.extend_from_slice(&witness_data);
        }

        tracing::debug!(
            "Coordinating witness network via capability registry ({} participants)",
            connection_count
        );
        Ok(WitnessProof {
            data: proof,
        })
    }
}

/// Routed link between two capability endpoints (requester ↔ provider).
#[derive(Debug, Clone)]
pub struct MeshConnection {
    /// Unique mesh link identifier.
    pub id: String,
    /// Requester primal base URL.
    pub requester_endpoint: Arc<str>,
    /// Provider primal base URL.
    pub provider_endpoint: Arc<str>,
    /// Capability requested by the initiator.
    pub requester_capability: CapabilityType,
    /// Capability offered by the peer.
    pub provider_capability: CapabilityType,
}

/// Result of a status probe against one cached primal connection.
#[derive(Debug, Clone)]
pub struct PrimalHealthStatus {
    /// Capability key used to cache this connection.
    pub capability: Arc<str>,
    /// Primal base URL.
    pub endpoint: Arc<str>,
    /// Whether the status call reported healthy.
    pub healthy: bool,
    /// Reported version or `"unknown"` / `"error"` on failure.
    pub version: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::bridge::*;
    use crate::error::PrimalCoordinationError;
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
}
