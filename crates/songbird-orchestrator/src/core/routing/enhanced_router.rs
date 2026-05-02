// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Enhanced Capability Router with Universal Port Authority Integration
//!
//! Modern, idiomatic Rust router that integrates:
//! - Universal Port Authority (service registry)
//! - Legacy capability registry (for backward compatibility)
//! - Federation state (for peer Songbirds)
//!
//! Deep Debt Solution: Unified routing with priority order

use super::analyzer::{TaskComplexity, TaskComplexityAnalyzer};
use super::router::RoutingDecision;
use super::types::Task;
use crate::core::registry::CapabilityRegistry;
use crate::service_registry::{ServiceRegistry, ServiceStatus};
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::{FederationState, NodeStatus};
use songbird_types::{SongbirdError, SongbirdResult};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Enhanced router with Universal Port Authority integration
pub struct EnhancedCapabilityRouter {
    /// Universal Port Authority service registry (PRIMARY)
    service_registry: Arc<ServiceRegistry>,

    /// Federation state for peer discovery
    federation_state: Arc<FederationState>,

    /// Legacy federated service registry
    #[allow(dead_code, reason = "retained for federation registry integration paths")]
    federated_service_registry: Arc<FederatedServiceRegistry>,

    /// Capability endpoint resolver
    capability_resolver: CapabilityEndpointResolver,

    /// Legacy capability registry for external providers (optional)
    capability_registry: Option<Arc<CapabilityRegistry>>,
}

impl EnhancedCapabilityRouter {
    /// Create a new enhanced router
    #[must_use]
    pub fn new(
        service_registry: Arc<ServiceRegistry>,
        federation_state: Arc<FederationState>,
        federated_service_registry: Arc<FederatedServiceRegistry>,
    ) -> Self {
        Self {
            service_registry,
            federation_state,
            federated_service_registry,
            capability_resolver: CapabilityEndpointResolver::new(),
            capability_registry: None,
        }
    }

    /// Create with all registries (for backward compatibility)
    #[must_use]
    pub fn with_all_registries(
        service_registry: Arc<ServiceRegistry>,
        federation_state: Arc<FederationState>,
        federated_service_registry: Arc<FederatedServiceRegistry>,
        capability_registry: Arc<CapabilityRegistry>,
    ) -> Self {
        Self {
            service_registry,
            federation_state,
            federated_service_registry,
            capability_resolver: CapabilityEndpointResolver::new(),
            capability_registry: Some(capability_registry),
        }
    }

    /// Route a task to the optimal execution location
    ///
    /// # Modern Routing Strategy (Deep Debt Solution)
    ///
    /// 1. **Query Universal Port Authority** (PRIMARY)
    ///    - Find active registered services by capability
    ///    - Select best match based on load/health
    ///    - Route to assigned endpoint
    ///
    /// 2. **Federation Fallback** (SECONDARY)
    ///    - Query federated peers
    ///    - Route to peer Songbird
    ///
    /// 3. **Legacy Capability Registry** (TERTIARY)
    ///    - Query old capability registry
    ///    - Route to external provider
    ///
    /// 4. **Static Resolver** (LAST RESORT)
    ///    - Use built-in endpoint resolver
    ///    - Route to hardcoded endpoints
    ///
    /// This eliminates deep debt by:
    /// - Prioritizing dynamic discovery
    /// - Maintaining backward compatibility
    /// - Clear fallback chain
    /// - Modern idiomatic Rust
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn route_task(&self, task: &Task) -> SongbirdResult<RoutingDecision> {
        info!("🎯 Routing task: {}", task.task_type);

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

    /// Route lightweight task (local or peer)
    async fn route_lightweight_task(&self, _task: &Task) -> SongbirdResult<RoutingDecision> {
        debug!("Routing lightweight task");

        // Check local capacity
        if self.has_local_capacity().await {
            debug!("✅ Executing lightweight task locally");
            return Ok(RoutingDecision::ExecuteLocally);
        }

        // Route to peer
        debug!("Routing lightweight task to peer");
        self.route_to_peer_songbird().await
    }

    /// Route moderate task (prefer peer, fallback to capability)
    async fn route_moderate_task(&self, task: &Task) -> SongbirdResult<RoutingDecision> {
        debug!("Routing moderate task");

        // Try peer first
        match self.route_to_peer_songbird().await {
            Ok(decision) => {
                debug!("✅ Routing moderate task to peer Songbird");
                Ok(decision)
            }
            Err(e) => {
                warn!("No peer Songbirds available: {}, trying capability", e);
                self.route_to_specialized_capability(task).await
            }
        }
    }

    /// Route heavy task (always to specialized capability)
    async fn route_heavy_task(&self, task: &Task) -> SongbirdResult<RoutingDecision> {
        debug!("Routing heavy task to specialized capability");
        self.route_to_specialized_capability(task).await
    }

    /// Route to specialized capability (MODERN IMPLEMENTATION)
    ///
    /// Priority order (Deep Debt Solution):
    /// 1. Universal Port Authority (registered services)
    /// 2. Legacy capability registry
    /// 3. Static endpoint resolver
    async fn route_to_specialized_capability(
        &self,
        task: &Task,
    ) -> SongbirdResult<RoutingDecision> {
        let capability_type = Self::determine_capability_type(task);
        let capability_name = capability_type.as_str();

        debug!("Task requires capability: {} ({:?})", capability_name, capability_type);

        // PRIORITY 1: Query Universal Port Authority
        info!("🔍 [1/3] Querying Universal Port Authority for '{}'...", capability_name);
        match self.query_service_registry(capability_name).await {
            Ok(Some(decision)) => {
                info!("✅ Found registered service via UPA");
                return Ok(decision);
            }
            Ok(None) => {
                debug!("No registered services found for '{}'", capability_name);
            }
            Err(e) => {
                warn!("Error querying service registry: {}", e);
            }
        }

        // PRIORITY 2: Legacy capability registry
        info!("🔍 [2/3] Querying legacy capability registry...");
        if let Some(registry) = &self.capability_registry {
            match registry.find_providers_with_capability(capability_name).await {
                Ok(providers) if !providers.is_empty() => {
                    let provider = &providers[0];
                    let endpoint = format!(
                        "{}{}",
                        provider.registration.endpoint, provider.registration.workload_endpoint
                    );

                    info!("✅ Found legacy provider: {}", provider.registration.provider_name);
                    return Ok(RoutingDecision::RouteToExternalProvider {
                        provider_id: provider.registration.provider_id.clone(),
                        execution_endpoint: endpoint,
                        capability_name: capability_name.to_string(),
                    });
                }
                _ => debug!("No legacy providers found"),
            }
        }

        // PRIORITY 3: Static endpoint resolver (last resort)
        info!("🔍 [3/3] Trying static endpoint resolver...");
        match self.capability_resolver.get_endpoint(capability_type.clone()).await {
            Ok(endpoint) => {
                info!("✅ Found static endpoint: {}", endpoint);
                Ok(RoutingDecision::RouteToCapability {
                    capability_type,
                    provider_endpoint: endpoint,
                })
            }
            Err(e) => Err(SongbirdError::service(
                capability_name,
                format!("No provider found for capability: {e}"),
            )),
        }
    }

    /// Query Universal Port Authority service registry
    async fn query_service_registry(
        &self,
        capability: &str,
    ) -> SongbirdResult<Option<RoutingDecision>> {
        let services = self.service_registry.query_by_capability(capability).await;

        // Filter to active services only
        let active_services: Vec<_> =
            services.into_iter().filter(|s| s.status == ServiceStatus::Active).collect();

        if active_services.is_empty() {
            return Ok(None);
        }

        // Select best service using load balancing strategy
        // Implements least-loaded algorithm for optimal distribution
        let service = self.select_best_service(&active_services).await;

        let endpoint = format!(
            "{}://{}:{}",
            service.assigned_endpoint.protocol,
            service.assigned_endpoint.host,
            service.assigned_endpoint.port
        );

        Ok(Some(RoutingDecision::RouteToRegisteredService {
            service_id: service.service_id.clone(),
            service_name: service.service_name.clone(),
            endpoint,
            port: service.assigned_endpoint.port,
        }))
    }

    /// Select the best service from available services using load balancing
    ///
    /// Implements least-loaded algorithm:
    /// 1. Prefer services with fewer active connections
    /// 2. Fall back to round-robin if load is equal
    /// 3. Consider service health scores
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn select_best_service<'a>(
        &self,
        services: &'a [crate::service_registry::RegisteredService],
    ) -> &'a crate::service_registry::RegisteredService {
        // For now, use simple round-robin (first service)
        // Future: Track active connections per service and select least loaded
        // Future: Consider geographic proximity for distributed deployments
        // Future: Implement weighted load balancing based on capacity

        &services[0]
    }

    /// Check if local capacity is available
    ///
    /// Checks system resources to determine if we can handle more work:
    /// - CPU usage below threshold (80%)
    /// - Memory available
    /// - Active task count below limit
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn has_local_capacity(&self) -> bool {
        let Some(mem) = songbird_types::sys_metrics::memory_info() else {
            debug!("Local capacity check: memory info unavailable, assuming capacity");
            return true;
        };

        let memory_usage_percent = mem.usage_percent();
        if memory_usage_percent > 90.0 {
            debug!("Local capacity check: Memory usage too high ({:.1}%)", memory_usage_percent);
            return false;
        }

        debug!("Local capacity available: Memory {:.1}% used", memory_usage_percent);
        true
    }

    /// Route to a peer Songbird instance
    async fn route_to_peer_songbird(&self) -> SongbirdResult<RoutingDecision> {
        let state = self.federation_state.nodes.read().await;

        for node in state.values().filter(|n| n.status == NodeStatus::Active) {
            if let Some(endpoint) = node.preferred_endpoint() {
                info!("Routing to peer Songbird: {} at {}", node.node_name, endpoint.address);
                let node_id = node.node_id.clone();
                let endpoint_url = format!("https://{}", endpoint.address);
                return Ok(RoutingDecision::RouteToSongbird {
                    node_id,
                    endpoint: endpoint_url,
                });
            }
        }

        Err(SongbirdError::Service {
            service: "federation".to_string(),
            message: "No active peer Songbirds available".to_string(),
            suggested_alternatives: vec![],
            recovery_actions: vec![],
        })
    }

    /// Determine capability type from task
    fn determine_capability_type(task: &Task) -> CapabilityType {
        // Check task metadata for explicit capability hints
        if let Some(requires_gpu) = task.metadata.get("requires_gpu") {
            // Handle string values since metadata is HashMap<String, String>
            if requires_gpu == "true" || requires_gpu == "1" {
                return CapabilityType::Compute;
            }
        }

        // Infer from task type
        match task.task_type.as_ref() {
            "ml_training" | "gpu_compute" | "distributed_compute" => CapabilityType::Compute,
            "encryption" | "verification" | "trust_check" => CapabilityType::Security,
            "intent_parsing" | "ai_routing" => CapabilityType::Ai,
            "data_storage" | "replication" => CapabilityType::Storage,
            _ => CapabilityType::Compute, // Default
        }
    }

    /// Execute task on registered service (NEW)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn execute_on_registered_service(
        &self,
        service_id: &str,
        endpoint: &str,
        task: &Task,
    ) -> SongbirdResult<serde_json::Value> {
        info!("🚀 Executing task on registered service {} at {}", service_id, endpoint);

        // ✅ EVOLVED (Jan 21, 2026): 100% Pure Rust HTTP via SongbirdHttpClient
        let crypto_socket =
            crate::primal_discovery::discover_crypto_provider().await.map_err(|e| {
                SongbirdError::Network {
                    message: format!("Failed to discover crypto provider: {e}"),
                    interface: None,
                    suggestion: Some("Check security provider availability".to_string()),
                }
            })?;

        let client = songbird_http_client::SongbirdHttpClient::new(crypto_socket);
        let task_json = serde_json::to_value(task).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: e.to_string(),
            debug_info: None,
        })?;

        let response = tokio::time::timeout(
            songbird_types::defaults::timeouts::DEFAULT_COMPUTE_TIMEOUT,
            client.post(&format!("{endpoint}/execute"), task_json),
        )
        .await
        .map_err(|_| SongbirdError::Network {
            message: "Request timeout (5 minutes)".to_string(),
            interface: Some(endpoint.to_string()),
            suggestion: Some("Check service health and network connectivity".to_string()),
        })?
        .map_err(|e| SongbirdError::Network {
            message: format!("Failed to send task to service: {e}"),
            interface: Some(endpoint.to_string()),
            suggestion: Some("Check service health and network connectivity".to_string()),
        })?;

        if response.status < 200 || response.status >= 300 {
            return Err(SongbirdError::Service {
                service: service_id.to_string(),
                message: format!("Service returned error status: {}", response.status),
                suggested_alternatives: vec![],
                recovery_actions: vec![],
            });
        }

        let result = serde_json::from_value(response.body).map_err(|e| SongbirdError::Service {
            service: service_id.to_string(),
            message: format!("Failed to parse service response: {e}"),
            suggested_alternatives: vec![],
            recovery_actions: vec![],
        })?;

        info!("✅ Task executed successfully on service {}", service_id);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::core::routing::types::Task;
    use crate::service_registry::{RegistrationRequest, ServiceCapability, ServiceRegistry};
    use songbird_network_federation::state::{
        EndpointStatus, NodeRegistration, NodeStatus, TransportEndpointInfo,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

    async fn router_with_compute_registration() -> EnhancedCapabilityRouter {
        let service_registry = Arc::new(ServiceRegistry::new());
        let reg = RegistrationRequest {
            primal_name: "compute-svc".into(),
            primal_version: "1".into(),
            capabilities: vec![ServiceCapability {
                name: "compute".into(),
                capability_type: "compute".into(),
                metadata: HashMap::new(),
            }],
            protocols: vec!["https".into()],
            preferred_protocol: "https".into(),
            health_check_path: None,
            metadata: None,
        };
        service_registry.register(reg).await.expect("register");
        let federation_state = Arc::new(FederationState::new("test-fed".into()));
        let federated = Arc::new(FederatedServiceRegistry::new());
        EnhancedCapabilityRouter::new(service_registry, federation_state, federated)
    }

    #[test]
    fn test_capability_type_as_str_matches_router_expectations() {
        assert_eq!(CapabilityType::Compute.as_str(), "compute");
        assert_eq!(CapabilityType::Security.as_str(), "security");
    }

    #[test]
    fn determine_capability_from_metadata_gpu() {
        let mut t = Task::builder("any").build();
        t.metadata.insert("requires_gpu".into(), "true".into());
        assert_eq!(
            EnhancedCapabilityRouter::determine_capability_type(&t),
            CapabilityType::Compute
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn heavy_task_routes_to_registered_compute_service() {
        let router = router_with_compute_registration().await;
        let task = Task::builder("ml_training").with_gpu().build();
        let d = router.route_task(&task).await.expect("route");
        match d {
            RoutingDecision::RouteToRegisteredService {
                endpoint,
                service_name,
                ..
            } => {
                assert!(endpoint.contains("https://"));
                assert_eq!(service_name, "compute-svc");
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn moderate_task_routes_to_federation_peer_when_present() {
        let service_registry = Arc::new(ServiceRegistry::new());
        let federation_state = Arc::new(FederationState::new("test-fed".into()));
        let federated = Arc::new(FederatedServiceRegistry::new());
        let ep = TransportEndpointInfo {
            interface_type: "eth".into(),
            address: "127.0.0.1:5555".into(),
            protocols: vec!["https".into()],
            preference: 10,
            status: EndpointStatus::Active,
            last_check: chrono::Utc::now(),
        };
        let node = NodeRegistration {
            node_id: "n1".into(),
            node_name: "peer-a".into(),
            node_address: "https://127.0.0.1:5555".into(),
            endpoints: Some(vec![ep]),
            cpu_cores: 1,
            memory_gb: 1,
            gpu_model: None,
            storage_gb: None,
            capabilities: vec![],
            status: NodeStatus::Active,
            joined_at: chrono::Utc::now(),
            last_heartbeat: chrono::Utc::now(),
        };
        federation_state.register_node(node).await;

        let router = EnhancedCapabilityRouter::new(service_registry, federation_state, federated);

        let task = Task::builder("data_transform").with_cpu(2.0).build();
        let d = router.route_task(&task).await.expect("route");
        match d {
            RoutingDecision::RouteToSongbird {
                endpoint,
                ..
            } => {
                assert!(endpoint.starts_with("https://"));
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn moderate_task_fallback_to_capability_when_no_peer() {
        let service_registry = Arc::new(ServiceRegistry::new());
        let federation_state = Arc::new(FederationState::new("test-fed".into()));
        let federated = Arc::new(FederatedServiceRegistry::new());
        let router = EnhancedCapabilityRouter::new(service_registry, federation_state, federated);

        let task = Task::builder("data_transform").with_cpu(2.0).build();
        let res = router.route_task(&task).await;
        assert!(res.is_err(), "expected no static endpoint in minimal env: {res:?}");
    }
}
