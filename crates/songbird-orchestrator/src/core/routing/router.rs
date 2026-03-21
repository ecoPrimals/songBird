// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Intelligent Capability Router
//!
//! Routes tasks to optimal execution location based on complexity and requirements.

use super::analyzer::{TaskComplexity, TaskComplexityAnalyzer};
use super::types::Task;
use crate::core::registry::CapabilityRegistry;
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::{FederationState, NodeStatus};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Routing decision for a task
#[derive(Debug, Clone)]
pub enum RoutingDecision {
    /// Execute the task locally on this Songbird instance
    ExecuteLocally,

    /// Route to another Songbird instance in the federation
    RouteToSongbird {
        /// ID of the target node
        node_id: String,
        /// RPC endpoint of the target node
        endpoint: String,
    },

    /// Route to a registered service (Universal Port Authority)
    /// NEW: Modern routing via service registry
    RouteToRegisteredService {
        /// Service ID from registry
        service_id: String,
        /// Service name (e.g., "Toadstool")
        service_name: String,
        /// Full endpoint URL
        endpoint: String,
        /// Port assigned by UPA
        port: u16,
    },

    /// Route to a specialized capability provider
    RouteToCapability {
        /// Type of capability (Compute, Security, AI, Storage)
        capability_type: CapabilityType,
        /// Endpoint of the capability provider
        provider_endpoint: String,
    },

    /// Route to an external provider registered in capability registry
    RouteToExternalProvider {
        /// Provider ID
        provider_id: String,
        /// Full execution endpoint URL
        execution_endpoint: String,
        /// Capability being used
        capability_name: String,
    },
}

/// Intelligent router for task distribution
pub struct CapabilityRouter {
    /// Federation state for peer discovery
    federation_state: Arc<FederationState>,

    /// Service registry for capability discovery
    service_registry: Arc<FederatedServiceRegistry>,

    /// Capability endpoint resolver
    capability_resolver: CapabilityEndpointResolver,

    /// Capability registry for external providers (optional)
    capability_registry: Option<Arc<CapabilityRegistry>>,
}

impl CapabilityRouter {
    /// Create a new capability router
    #[must_use]
    pub fn new(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
    ) -> Self {
        Self {
            federation_state,
            service_registry,
            capability_resolver: CapabilityEndpointResolver::new(),
            capability_registry: None,
        }
    }

    /// Create a new capability router with external provider registry
    #[must_use]
    pub fn with_capability_registry(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
        capability_registry: Arc<CapabilityRegistry>,
    ) -> Self {
        Self {
            federation_state,
            service_registry,
            capability_resolver: CapabilityEndpointResolver::new(),
            capability_registry: Some(capability_registry),
        }
    }

    /// Router with fixed capability endpoints (tests, embedders) — no process env for those URLs.
    #[must_use]
    pub fn with_capability_endpoint_overrides(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
        overrides: HashMap<CapabilityType, String>,
    ) -> Self {
        Self {
            federation_state,
            service_registry,
            capability_resolver: CapabilityEndpointResolver::with_endpoint_overrides(overrides),
            capability_registry: None,
        }
    }

    /// Route a task to the optimal execution location
    ///
    /// # Routing Strategy
    ///
    /// - **Lightweight**: Execute locally if capacity available, else route to peer
    /// - **Moderate**: Prefer peer Songbird, fallback to capability if needed
    /// - **Heavy**: Always route to specialized capability (Toadstool, security provider, etc.)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let router = CapabilityRouter::new(federation_state, service_registry);
    /// let task = Task::builder("ml_training").with_gpu().build();
    /// let decision = router.route_task(&task).await?;
    /// ```
    pub async fn route_task(&self, task: &Task) -> SongbirdResult<RoutingDecision> {
        info!("Routing task: {}", task.task_type);

        // 1. Analyze task complexity
        let complexity = TaskComplexityAnalyzer::analyze(task);
        debug!("Task complexity: {:?}", complexity);

        // 2. Route based on complexity
        match complexity {
            TaskComplexity::Lightweight => self.route_lightweight_task(task).await,

            TaskComplexity::Moderate => self.route_moderate_task(task).await,

            TaskComplexity::Heavy => self.route_heavy_task(task).await,
        }
    }

    /// Route a lightweight task (execute locally or on peer)
    async fn route_lightweight_task(&self, _task: &Task) -> SongbirdResult<RoutingDecision> {
        debug!("Routing lightweight task");

        // Check if we have local capacity
        // For now, assume we always have capacity for lightweight tasks
        if self.has_local_capacity().await {
            debug!("Executing lightweight task locally");
            return Ok(RoutingDecision::ExecuteLocally);
        }

        // Otherwise, route to a peer Songbird
        debug!("Routing lightweight task to peer");
        self.route_to_peer_songbird().await
    }

    /// Route a moderate task (prefer peer, fallback to capability)
    async fn route_moderate_task(&self, task: &Task) -> SongbirdResult<RoutingDecision> {
        debug!("Routing moderate task");

        // Try to route to a peer Songbird first
        match self.route_to_peer_songbird().await {
            Ok(decision) => {
                debug!("Routing moderate task to peer Songbird");
                Ok(decision)
            }
            Err(e) => {
                warn!("No peer Songbirds available: {}, falling back to capability", e);
                // Fallback: route to specialized capability
                self.route_to_specialized_capability(task).await
            }
        }
    }

    /// Route a heavy task (always to specialized capability)
    async fn route_heavy_task(&self, task: &Task) -> SongbirdResult<RoutingDecision> {
        debug!("Routing heavy task to specialized capability");
        self.route_to_specialized_capability(task).await
    }

    /// Route to a specialized capability (Toadstool, security provider, etc.)
    async fn route_to_specialized_capability(
        &self,
        task: &Task,
    ) -> SongbirdResult<RoutingDecision> {
        // Determine required capability type
        let capability_type = Self::determine_capability_type(task);
        debug!("Task requires capability: {:?}", capability_type);

        // Format capability type for logging and error messages
        let capability_type_str = format!("{capability_type:?}");

        // NEW: First try to find an external provider in the capability registry
        if let Some(registry) = &self.capability_registry {
            let capability_name = Self::capability_type_to_name(&capability_type);

            match registry.find_providers_with_capability(&capability_name).await {
                Ok(providers) if !providers.is_empty() => {
                    // Select best provider (for now, use first healthy one)
                    let provider = &providers[0];
                    let execution_endpoint = format!(
                        "{}{}",
                        provider.registration.endpoint, provider.registration.workload_endpoint
                    );

                    info!(
                        "Routing to external provider '{}' ({}) at: {}",
                        provider.registration.provider_name,
                        provider.registration.provider_id,
                        execution_endpoint
                    );

                    return Ok(RoutingDecision::RouteToExternalProvider {
                        provider_id: provider.registration.provider_id.clone(),
                        execution_endpoint,
                        capability_name,
                    });
                }
                Ok(_) => {
                    debug!("No external providers found for capability: {}", capability_name);
                }
                Err(e) => {
                    warn!("Error querying capability registry: {}", e);
                }
            }
        }

        // Fallback to static capability endpoint resolver
        let endpoint = self
            .capability_resolver
            .get_endpoint(capability_type.clone()) // Clone for the async call
            .await
            .map_err(|e| {
                SongbirdError::service(
                    capability_type_str.clone(),
                    format!("No capability provider found: {e}"),
                )
            })?;

        info!("Routing to {} capability at: {}", capability_type_str, endpoint);

        Ok(RoutingDecision::RouteToCapability {
            capability_type,
            provider_endpoint: endpoint,
        })
    }

    /// Execute a task on an external provider
    ///
    /// Sends the task to the provider's execution endpoint and waits for results
    pub async fn execute_on_external_provider(
        &self,
        endpoint: &str,
        task: &Task,
    ) -> SongbirdResult<serde_json::Value> {
        info!("Executing task on external provider: {}", endpoint);

        // ✅ EVOLVED (Jan 21, 2026): 100% Pure Rust HTTP via SongbirdHttpClient
        let crypto_socket =
            crate::primal_discovery::discover_crypto_provider().await.map_err(|e| {
                SongbirdError::Network {
                    message: format!("Failed to discover crypto provider: {e}"),
                    interface: None,
                    suggestion: Some("Check BearDog availability".to_string()),
                }
            })?;

        let client = songbird_http_client::SongbirdHttpClient::new(crypto_socket);
        let task_json = serde_json::to_value(task).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: e.to_string(),
            debug_info: None,
        })?;

        let response =
            tokio::time::timeout(Duration::from_secs(300), client.post(endpoint, task_json))
                .await
                .map_err(|_| SongbirdError::Network {
                    message: "Request timeout (5 minutes)".to_string(),
                    interface: Some(endpoint.to_string()),
                    suggestion: Some(
                        "Check provider endpoint and network connectivity".to_string(),
                    ),
                })?
                .map_err(|e| SongbirdError::Network {
                    message: format!("Failed to send task to external provider: {e}"),
                    interface: Some(endpoint.to_string()),
                    suggestion: Some(
                        "Check provider endpoint and network connectivity".to_string(),
                    ),
                })?;

        if response.status < 200 || response.status >= 300 {
            return Err(SongbirdError::Service {
                service: "external_provider".to_string(),
                message: format!("Provider returned error status: {}", response.status),
                suggested_alternatives: vec![],
                recovery_actions: vec!["retry".to_string(), "route_to_fallback".to_string()],
            });
        }

        let result =
            serde_json::from_value(response.body).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Failed to parse provider response: {e}"),
                debug_info: None,
            })?;

        info!("Task execution completed successfully on external provider");
        Ok(result)
    }

    /// Convert `CapabilityType` enum to capability name string
    fn capability_type_to_name(cap_type: &CapabilityType) -> String {
        match cap_type {
            CapabilityType::Compute => "compute_heavy".to_string(),
            CapabilityType::Security => "security".to_string(),
            CapabilityType::Ai => "ai_inference".to_string(),
            CapabilityType::Storage => "storage".to_string(),
            CapabilityType::Orchestration => "orchestration".to_string(),
            CapabilityType::Observability => "observability".to_string(),
            CapabilityType::Networking => "networking".to_string(),
            CapabilityType::Custom(name) => name.clone(),
        }
    }

    /// Route to a peer Songbird instance
    async fn route_to_peer_songbird(&self) -> SongbirdResult<RoutingDecision> {
        let nodes = self.federation_state.nodes.read().await;

        // Find a healthy, available peer
        for (node_id, registration) in nodes.iter() {
            if registration.status == NodeStatus::Active {
                debug!("Found available peer: {} at {}", node_id, registration.node_address);
                return Ok(RoutingDecision::RouteToSongbird {
                    node_id: node_id.clone(),
                    endpoint: registration.node_address.clone(),
                });
            }
        }

        Err(SongbirdError::service("federation", "No available peer Songbirds found in federation"))
    }

    /// Determine required capability type from task
    ///
    /// Takes `&Task` but doesn't need `&self` - made into associated function for clarity
    fn determine_capability_type(task: &Task) -> CapabilityType {
        // GPU required → Compute (Toadstool)
        if task.resource_requirements.as_ref().is_some_and(|r| r.gpu_required) {
            return CapabilityType::Compute;
        }

        // Match on task type (Arc<str> derefs to &str)
        match task.task_type.as_ref() {
            // Compute tasks (Toadstool)
            "ml_training" | "gpu_compute" | "batch_processing" | "video_processing" => {
                CapabilityType::Compute
            }

            // Security tasks (security provider)
            "encrypt" | "decrypt" | "sign" | "verify" | "auth" => CapabilityType::Security,

            // AI tasks (Squirrel)
            "inference" | "ai_query" | "model_serve" => CapabilityType::Ai,

            // Storage tasks (NestGate)
            "store" | "retrieve" | "backup" => CapabilityType::Storage,

            // Default to Compute for unknown heavy tasks
            _ => CapabilityType::Compute,
        }
    }

    /// Check if local Songbird instance has capacity
    ///
    /// FUTURE ENHANCEMENT: Implement resource-aware capacity checking
    ///
    /// Planned metrics:
    /// - Current CPU usage (via sysinfo crate)
    /// - Available memory (RAM check)
    /// - Active task count (from job manager)
    /// - Load average (system load)
    ///
    /// Priority: Medium (Week 4-5 optimization phase)
    async fn has_local_capacity(&self) -> bool {
        // SIMPLIFIED: Currently assumes capacity is always available
        // In production, this would check actual system resources
        true
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use chrono::Utc;
    use songbird_config::capability_endpoints::CapabilityType;
    use songbird_network_federation::state::{NodeRegistration, NodeStatus};
    use std::collections::HashMap;

    fn ep_overrides() -> HashMap<CapabilityType, String> {
        [
            (CapabilityType::Compute, "https://compute.test".into()),
            (CapabilityType::Security, "https://security.test".into()),
            (CapabilityType::Ai, "https://ai.test".into()),
            (CapabilityType::Storage, "https://storage.test".into()),
            (CapabilityType::Orchestration, "https://orch.test".into()),
            (CapabilityType::Observability, "https://obs.test".into()),
            (CapabilityType::Networking, "https://net.test".into()),
        ]
        .into_iter()
        .collect()
    }

    async fn register_active_peer(state: &FederationState, id: &str, addr: &str) {
        let reg = NodeRegistration {
            node_id: id.to_string(),
            node_name: "peer".to_string(),
            node_address: addr.to_string(),
            endpoints: None,
            cpu_cores: 4,
            memory_gb: 8,
            gpu_model: None,
            storage_gb: None,
            capabilities: vec![],
            status: NodeStatus::Active,
            joined_at: Utc::now(),
            last_heartbeat: Utc::now(),
        };
        state.register_node(reg).await;
    }

    #[test]
    fn test_determine_capability_type_gpu() {
        let task = Task::builder("ml_training").with_gpu().build();
        assert_eq!(CapabilityRouter::determine_capability_type(&task), CapabilityType::Compute);
    }

    #[test]
    fn test_determine_capability_type_ml() {
        let task = Task::new("ml_training");
        assert_eq!(CapabilityRouter::determine_capability_type(&task), CapabilityType::Compute);
    }

    #[test]
    fn test_determine_capability_type_security() {
        let task = Task::new("encrypt");
        assert_eq!(CapabilityRouter::determine_capability_type(&task), CapabilityType::Security);
    }

    #[test]
    fn test_determine_capability_type_ai() {
        let task = Task::new("inference");
        assert_eq!(CapabilityRouter::determine_capability_type(&task), CapabilityType::Ai);
    }

    #[test]
    fn test_determine_capability_type_storage() {
        let task = Task::new("store");
        assert_eq!(CapabilityRouter::determine_capability_type(&task), CapabilityType::Storage);
    }

    fn create_test_router() -> CapabilityRouter {
        let federation_state = Arc::new(FederationState::new("default".to_string()));
        let service_registry = Arc::new(FederatedServiceRegistry::new());
        CapabilityRouter::new(federation_state, service_registry)
    }

    #[test]
    fn determine_capability_batch_and_video_map_to_compute() {
        let t1 = Task::new("batch_processing");
        let t2 = Task::new("video_processing");
        assert_eq!(CapabilityRouter::determine_capability_type(&t1), CapabilityType::Compute);
        assert_eq!(CapabilityRouter::determine_capability_type(&t2), CapabilityType::Compute);
    }

    #[test]
    fn determine_capability_decrypt_and_verify_map_to_security() {
        let t1 = Task::new("decrypt");
        let t2 = Task::new("verify");
        assert_eq!(CapabilityRouter::determine_capability_type(&t1), CapabilityType::Security);
        assert_eq!(CapabilityRouter::determine_capability_type(&t2), CapabilityType::Security);
    }

    #[test]
    fn determine_capability_retrieve_maps_to_storage() {
        let t = Task::new("retrieve");
        assert_eq!(CapabilityRouter::determine_capability_type(&t), CapabilityType::Storage);
    }

    #[test]
    fn determine_capability_model_serve_maps_to_ai() {
        let t = Task::new("model_serve");
        assert_eq!(CapabilityRouter::determine_capability_type(&t), CapabilityType::Ai);
    }

    #[test]
    fn determine_capability_unknown_string_defaults_to_compute() {
        let t = Task::new("unknown_workload");
        assert_eq!(CapabilityRouter::determine_capability_type(&t), CapabilityType::Compute);
    }

    #[test]
    fn capability_type_to_name_custom_variant() {
        let ct = CapabilityType::Custom("my_cap".to_string());
        assert_eq!(CapabilityRouter::capability_type_to_name(&ct), "my_cap");
    }

    #[test]
    fn capability_type_to_name_compute() {
        assert_eq!(
            CapabilityRouter::capability_type_to_name(&CapabilityType::Compute),
            "compute_heavy"
        );
    }

    #[tokio::test]
    async fn route_lightweight_executes_locally() {
        let fs = Arc::new(FederationState::new("f".into()));
        let sr = Arc::new(FederatedServiceRegistry::new());
        let router = CapabilityRouter::with_capability_endpoint_overrides(fs, sr, ep_overrides());
        let task = Task::new("ping");
        let d = router.route_task(&task).await.unwrap();
        assert!(matches!(d, RoutingDecision::ExecuteLocally));
    }

    #[tokio::test]
    async fn route_moderate_prefers_peer_when_available() {
        let fs = Arc::new(FederationState::new("f".into()));
        register_active_peer(&fs, "n1", "https://peer:1").await;
        let sr = Arc::new(FederatedServiceRegistry::new());
        let router = CapabilityRouter::with_capability_endpoint_overrides(fs, sr, ep_overrides());
        let task = Task::builder("work").with_cpu(2.0).build();
        let d = router.route_task(&task).await.unwrap();
        assert!(matches!(
            d,
            RoutingDecision::RouteToSongbird {
                node_id,
                ..
            } if node_id == "n1"
        ));
    }

    #[tokio::test]
    async fn route_moderate_falls_back_to_capability_without_peer() {
        let fs = Arc::new(FederationState::new("f".into()));
        let sr = Arc::new(FederatedServiceRegistry::new());
        let router = CapabilityRouter::with_capability_endpoint_overrides(fs, sr, ep_overrides());
        let task = Task::builder("work").with_cpu(2.0).build();
        let d = router.route_task(&task).await.unwrap();
        assert!(matches!(
            d,
            RoutingDecision::RouteToCapability {
                capability_type: CapabilityType::Compute,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn route_heavy_targets_specialized_compute() {
        let fs = Arc::new(FederationState::new("f".into()));
        let sr = Arc::new(FederatedServiceRegistry::new());
        let router = CapabilityRouter::with_capability_endpoint_overrides(fs, sr, ep_overrides());
        let task = Task::builder("ml_training").with_gpu().build();
        let d = router.route_task(&task).await.unwrap();
        assert!(matches!(
            d,
            RoutingDecision::RouteToCapability {
                capability_type: CapabilityType::Compute,
                provider_endpoint,
                ..
            } if provider_endpoint.contains("compute.test")
        ));
    }

    #[tokio::test]
    async fn route_encrypt_heavy_security_endpoint() {
        let fs = Arc::new(FederationState::new("f".into()));
        let sr = Arc::new(FederatedServiceRegistry::new());
        let router = CapabilityRouter::with_capability_endpoint_overrides(fs, sr, ep_overrides());
        let task = Task::builder("encrypt").with_memory(8192).build();
        let d = router.route_task(&task).await.unwrap();
        assert!(matches!(
            d,
            RoutingDecision::RouteToCapability {
                capability_type: CapabilityType::Security,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn route_inference_to_ai_capability() {
        let fs = Arc::new(FederationState::new("f".into()));
        let sr = Arc::new(FederatedServiceRegistry::new());
        let router = CapabilityRouter::with_capability_endpoint_overrides(fs, sr, ep_overrides());
        let task = Task::builder("inference").with_memory(8192).build();
        let d = router.route_task(&task).await.unwrap();
        assert!(matches!(
            d,
            RoutingDecision::RouteToCapability {
                capability_type: CapabilityType::Ai,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn route_store_to_storage_capability() {
        let fs = Arc::new(FederationState::new("f".into()));
        let sr = Arc::new(FederatedServiceRegistry::new());
        let router = CapabilityRouter::with_capability_endpoint_overrides(fs, sr, ep_overrides());
        let task = Task::builder("store").with_memory(8192).build();
        let d = router.route_task(&task).await.unwrap();
        assert!(matches!(
            d,
            RoutingDecision::RouteToCapability {
                capability_type: CapabilityType::Storage,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn route_long_duration_is_heavy_even_if_type_light() {
        let fs = Arc::new(FederationState::new("f".into()));
        let sr = Arc::new(FederatedServiceRegistry::new());
        let router = CapabilityRouter::with_capability_endpoint_overrides(fs, sr, ep_overrides());
        let task = Task::builder("long").with_duration(400).build();
        let d = router.route_task(&task).await.unwrap();
        assert!(matches!(d, RoutingDecision::RouteToCapability { .. }));
    }
}
