// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Intelligent Capability Router
//!
//! Routes tasks to optimal execution location based on complexity and requirements.

mod decision;
mod execution;
mod routing;

pub use decision::RoutingDecision;

use super::analyzer::{TaskComplexity, TaskComplexityAnalyzer};
use super::types::Task;
use crate::core::registry::CapabilityRegistry;
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use songbird_types::SongbirdResult;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Intelligent router for task distribution
pub struct CapabilityRouter {
    /// Federation state for peer discovery
    federation_state: Arc<FederationState>,

    /// Service registry for capability discovery
    #[expect(dead_code, reason = "retained for capability discovery wiring")]
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
    /// - **Heavy**: Always route to specialized capability (compute, security, storage, etc.)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let router = CapabilityRouter::new(federation_state, service_registry);
    /// let task = Task::builder("ml_training").with_gpu().build();
    /// let decision = router.route_task(&task).await?;
    /// ```
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn route_task(&self, task: &Task) -> SongbirdResult<RoutingDecision> {
        info!("Routing task: {}", task.task_type);

        let complexity = TaskComplexityAnalyzer::analyze(task);
        debug!("Task complexity: {:?}", complexity);

        match complexity {
            TaskComplexity::Lightweight => self.route_lightweight_task(task).await,

            TaskComplexity::Moderate => self.route_moderate_task(task).await,

            TaskComplexity::Heavy => self.route_heavy_task(task).await,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
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

    #[test]
    fn determine_capability_gpu_compute_prefers_compute() {
        let t = Task::new("gpu_compute");
        assert_eq!(CapabilityRouter::determine_capability_type(&t), CapabilityType::Compute);
    }

    #[test]
    fn determine_capability_auth_maps_security() {
        let t = Task::new("auth");
        assert_eq!(CapabilityRouter::determine_capability_type(&t), CapabilityType::Security);
    }

    #[test]
    fn determine_capability_ai_query_maps_ai() {
        let t = Task::new("ai_query");
        assert_eq!(CapabilityRouter::determine_capability_type(&t), CapabilityType::Ai);
    }

    #[test]
    fn determine_capability_backup_maps_storage() {
        let t = Task::new("backup");
        assert_eq!(CapabilityRouter::determine_capability_type(&t), CapabilityType::Storage);
    }

    #[test]
    fn determine_capability_sign_maps_security() {
        let t = Task::new("sign");
        assert_eq!(CapabilityRouter::determine_capability_type(&t), CapabilityType::Security);
    }

    #[test]
    fn capability_type_to_name_orchestration() {
        assert_eq!(
            CapabilityRouter::capability_type_to_name(&CapabilityType::Orchestration),
            "orchestration"
        );
    }

    #[test]
    fn capability_type_to_name_observability() {
        assert_eq!(
            CapabilityRouter::capability_type_to_name(&CapabilityType::Observability),
            "observability"
        );
    }

    #[test]
    fn capability_type_to_name_networking() {
        assert_eq!(
            CapabilityRouter::capability_type_to_name(&CapabilityType::Networking),
            "networking"
        );
    }

    #[test]
    fn capability_type_to_name_storage_ai_security() {
        assert_eq!(CapabilityRouter::capability_type_to_name(&CapabilityType::Storage), "storage");
        assert_eq!(CapabilityRouter::capability_type_to_name(&CapabilityType::Ai), "ai_inference");
        assert_eq!(
            CapabilityRouter::capability_type_to_name(&CapabilityType::Security),
            "security"
        );
    }

    #[test]
    fn create_test_router_builds() {
        let _ = create_test_router();
    }
}
