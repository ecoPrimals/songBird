// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::{
    bridge::PrimalConnection,
    error::{PrimalCoordinationError, Result},
    types::{
        CapabilityType, DeploymentId, Identity, NodeId, PrimalRequest, PrimalResponse,
        WitnessProof, Workload,
    },
};
use std::sync::Arc;

use super::PrimalCoordinator;
use super::types::{MeshConnection, PrimalHealthStatus};

impl PrimalCoordinator {
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
        let connection = self.bridge.connect(&capability)?;

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
    /// (e.g., compute capability provides cycles for an AI capability provider analyzing `storage provider` data)
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
                    version: String::from("unknown"),
                },
                Err(_) => PrimalHealthStatus {
                    capability: Arc::clone(capability),
                    endpoint: Arc::clone(&conn.endpoint),
                    healthy: false,
                    version: String::from("error"),
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::PrimalCoordinator;
    use crate::bridge::{CoordinatorTestBridge, PrimalBridge};
    use crate::coordinator::CoordinatorConfig;
    use crate::error::PrimalCoordinationError;
    use crate::types::{CapabilityType, NodeId, Workload};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test(start_paused = true)]
    async fn request_capability_caches_distinct_capabilities_separately() {
        let bridge = Arc::new(PrimalBridge::CoordinatorTest(CoordinatorTestBridge::Mock));
        let coordinator = PrimalCoordinator::new(Arc::clone(&bridge));
        let sec = coordinator.request_capability(CapabilityType::Security).await.expect("security");
        let compute =
            coordinator.request_capability(CapabilityType::Compute).await.expect("compute");
        assert_ne!(sec.connection_id, compute.connection_id);
        assert!(sec.endpoint.contains("security"));
        assert!(compute.endpoint.contains("compute"));
    }

    #[tokio::test(start_paused = true)]
    async fn coordinate_service_mesh_wires_distinct_endpoints() {
        let bridge = Arc::new(PrimalBridge::CoordinatorTest(CoordinatorTestBridge::Mock));
        let coordinator = PrimalCoordinator::new(bridge);
        let mesh = coordinator
            .coordinate_service_mesh(CapabilityType::Compute, CapabilityType::Security)
            .await
            .expect("mesh");
        assert_ne!(
            mesh.requester_endpoint.as_ref(),
            mesh.provider_endpoint.as_ref(),
            "requester and provider should resolve to different mock URLs"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn health_check_reports_one_row_per_cached_capability() {
        let bridge = Arc::new(PrimalBridge::CoordinatorTest(CoordinatorTestBridge::Mock));
        let coordinator = PrimalCoordinator::new(bridge);
        coordinator.request_capability(CapabilityType::Security).await.expect("seed security");
        coordinator.request_capability(CapabilityType::Compute).await.expect("seed compute");
        let statuses = coordinator.health_check_all().await.expect("health");
        assert_eq!(statuses.len(), 2);
        let caps: Vec<_> = statuses.iter().map(|s| s.capability.as_ref()).collect();
        assert!(caps.contains(&"security"));
        assert!(caps.contains(&"compute"));
        assert!(statuses.iter().all(|s| !s.healthy));
    }

    #[tokio::test(start_paused = true)]
    async fn request_capability_without_pooling_does_not_seed_health_registry() {
        let bridge = Arc::new(PrimalBridge::CoordinatorTest(CoordinatorTestBridge::Mock));
        let config = CoordinatorConfig {
            enable_pooling: false,
            ..CoordinatorConfig::default()
        };
        let coordinator = PrimalCoordinator::with_config(bridge, config);
        coordinator.request_capability(CapabilityType::Security).await.expect("connect");
        let statuses = coordinator.health_check_all().await.expect("health");
        assert!(
            statuses.is_empty(),
            "without pooling, connections should not remain in active_connections"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn request_capability_compute_connects_mock_endpoint() {
        let bridge = Arc::new(PrimalBridge::CoordinatorTest(CoordinatorTestBridge::Mock));
        let coordinator = PrimalCoordinator::new(bridge);
        let conn =
            coordinator.request_capability(CapabilityType::Compute).await.expect("compute connect");
        assert!(conn.supports_capability(&CapabilityType::Compute).await);
        assert!(conn.endpoint.contains("compute"));
    }

    #[tokio::test(start_paused = true)]
    async fn coordinate_service_mesh_same_capability_reuses_cached_connection() {
        let bridge = Arc::new(PrimalBridge::CoordinatorTest(CoordinatorTestBridge::Mock));
        let coordinator = PrimalCoordinator::new(bridge);
        let mesh = coordinator
            .coordinate_service_mesh(CapabilityType::Security, CapabilityType::Security)
            .await
            .expect("mesh");
        assert_eq!(
            mesh.requester_endpoint.as_ref(),
            mesh.provider_endpoint.as_ref(),
            "same capability should resolve to the same cached endpoint"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn deploy_compute_with_matching_service_fails_at_ipc_transport() {
        let bridge = Arc::new(PrimalBridge::CoordinatorTest(CoordinatorTestBridge::Mock));
        let coordinator = PrimalCoordinator::new(bridge);
        let workload = Workload {
            id: "ops-deploy".into(),
            service_type: "compute".into(),
            requirements: std::collections::HashMap::new(),
            payload: serde_json::json!({}),
        };
        let err = coordinator
            .deploy_compute(workload)
            .await
            .expect_err("capability gate passes then IPC fails");
        assert!(matches!(err, PrimalCoordinationError::Internal(_)));
    }

    #[tokio::test(start_paused = true)]
    async fn request_capability_without_pooling_invokes_bridge_each_time() {
        let counter = Arc::new(AtomicUsize::new(0));
        let bridge = Arc::new(PrimalBridge::CoordinatorTest(CoordinatorTestBridge::Counting(
            Arc::clone(&counter),
        )));
        let config = CoordinatorConfig {
            enable_pooling: false,
            ..CoordinatorConfig::default()
        };
        let coordinator = PrimalCoordinator::with_config(bridge, config);
        coordinator.request_capability(CapabilityType::Security).await.expect("first");
        coordinator.request_capability(CapabilityType::Security).await.expect("second");
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn coordinate_genesis_fails_before_witness_when_keygen_unreachable() {
        let bridge = Arc::new(PrimalBridge::CoordinatorTest(CoordinatorTestBridge::Mock));
        let coordinator = PrimalCoordinator::with_config(
            bridge,
            CoordinatorConfig {
                enable_pooling: false,
                ..CoordinatorConfig::default()
            },
        );
        let err = coordinator
            .coordinate_genesis(NodeId("genesis-ops".into()))
            .await
            .expect_err("GenerateKeys requires live IPC HTTP");
        assert!(matches!(err, PrimalCoordinationError::Internal(_)));
    }
}
