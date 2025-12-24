//! Primal Coordinator - Songbird's central orchestration
//!
//! **ZERO HARDCODING**: Coordinates by capability, not by primal name

use crate::{
    bridge::{PrimalBridge, PrimalConnection},
    error::{PrimalCoordinationError, Result},
    types::*,
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
    
    /// Active connections to primals (by capability)
    active_connections: Arc<RwLock<HashMap<String, PrimalConnection>>>,
    
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
        let cache_key = capability.as_str().to_string();
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
                )))
            }
            _ => {
                return Err(PrimalCoordinationError::UnexpectedResponse(
                    "Expected KeysGenerated response".into(),
                ))
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
                )))
            }
            _ => {
                return Err(PrimalCoordinationError::UnexpectedResponse(
                    "Expected LineageSigned response".into(),
                ))
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
                )))
            }
            _ => {
                return Err(PrimalCoordinationError::UnexpectedResponse(
                    "Expected WorkloadDeployed response".into(),
                ))
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
    /// (e.g., Toadstool provides compute for Squirrel analyzing NestGate data)
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
            requester_endpoint: requester_conn.endpoint.clone(),
            provider_endpoint: provider_conn.endpoint.clone(),
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
        let connections = self.active_connections.read().await;
        let mut statuses = Vec::new();

        for (capability, conn) in connections.iter() {
            let response = conn.send_request(PrimalRequest::Status).await;
            let status = match response {
                Ok(PrimalResponse::StatusResponse(s)) => PrimalHealthStatus {
                    capability: capability.clone(),
                    endpoint: conn.endpoint.clone(),
                    healthy: s.healthy,
                    version: s.version,
                },
                Ok(_) => PrimalHealthStatus {
                    capability: capability.clone(),
                    endpoint: conn.endpoint.clone(),
                    healthy: false,
                    version: "unknown".to_string(),
                },
                Err(_) => PrimalHealthStatus {
                    capability: capability.clone(),
                    endpoint: conn.endpoint.clone(),
                    healthy: false,
                    version: "error".to_string(),
                },
            };
            statuses.push(status);
        }

        Ok(statuses)
    }

    /// Internal: Coordinate witness network using Songbird's own capabilities
    async fn coordinate_witness_network(&self, _node_id: &NodeId) -> Result<WitnessProof> {
        // This would involve using the pure Rust BLE stack for physical proximity
        // and other Songbird networking capabilities.
        // For now, placeholder implementation
        tracing::debug!("Coordinating witness network via Songbird's BLE/P2P stack");
        Ok(WitnessProof {
            data: b"ble_proximity_proof".to_vec(),
        })
    }
}

/// Service mesh connection between primals
#[derive(Debug, Clone)]
pub struct MeshConnection {
    pub id: String,
    pub requester_endpoint: String,
    pub provider_endpoint: String,
    pub requester_capability: CapabilityType,
    pub provider_capability: CapabilityType,
}

/// Health status of a primal
#[derive(Debug, Clone)]
pub struct PrimalHealthStatus {
    pub capability: String,
    pub endpoint: String,
    pub healthy: bool,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::*;

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

    #[tokio::test]
    async fn test_coordinator_creation() {
        let bridge = Arc::new(MockBridge);
        let coordinator = PrimalCoordinator::new(bridge);
        
        // Coordinator should be created successfully
        assert_eq!(coordinator.config.max_connections_per_capability, 10);
    }

    #[tokio::test]
    async fn test_request_capability() {
        let bridge = Arc::new(MockBridge);
        let coordinator = PrimalCoordinator::new(bridge);
        
        let conn = coordinator.request_capability(CapabilityType::Security).await;
        assert!(conn.is_ok());
        
        let conn = conn.unwrap();
        assert!(conn.supports_capability(&CapabilityType::Security).await);
    }

    #[tokio::test]
    async fn test_capability_caching() {
        let bridge = Arc::new(MockBridge);
        let coordinator = PrimalCoordinator::new(bridge);
        
        // First request
        let conn1 = coordinator.request_capability(CapabilityType::Security).await.unwrap();
        let id1 = conn1.connection_id.clone();
        
        // Second request should return cached connection
        let conn2 = coordinator.request_capability(CapabilityType::Security).await.unwrap();
        let id2 = conn2.connection_id;
        
        assert_eq!(id1, id2, "Should return cached connection");
    }

    #[tokio::test]
    async fn test_service_mesh_coordination() {
        let bridge = Arc::new(MockBridge);
        let coordinator = PrimalCoordinator::new(bridge);
        
        let mesh = coordinator
            .coordinate_service_mesh(CapabilityType::Compute, CapabilityType::Security)
            .await;
        
        assert!(mesh.is_ok());
        let mesh = mesh.unwrap();
        assert!(!mesh.id.is_empty());
        assert_eq!(mesh.requester_capability, CapabilityType::Compute);
        assert_eq!(mesh.provider_capability, CapabilityType::Security);
    }
}

