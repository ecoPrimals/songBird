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
use std::time::Duration;
use tracing::{debug, info, warn};

/// Enhanced router with Universal Port Authority integration
pub struct EnhancedCapabilityRouter {
    /// Universal Port Authority service registry (PRIMARY)
    service_registry: Arc<ServiceRegistry>,

    /// Federation state for peer discovery
    federation_state: Arc<FederationState>,

    /// Legacy federated service registry
    federated_service_registry: Arc<FederatedServiceRegistry>,

    /// Capability endpoint resolver
    capability_resolver: CapabilityEndpointResolver,

    /// Legacy capability registry for external providers (optional)
    capability_registry: Option<Arc<CapabilityRegistry>>,
}

impl EnhancedCapabilityRouter {
    /// Create a new enhanced router
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
        let capability_name = Self::capability_type_to_name(&capability_type);
        
        debug!("Task requires capability: {} ({})", capability_name, format!("{:?}", capability_type));

        // PRIORITY 1: Query Universal Port Authority
        info!("🔍 [1/3] Querying Universal Port Authority for '{}'...", capability_name);
        match self.query_service_registry(&capability_name).await {
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
            match registry.find_providers_with_capability(&capability_name).await {
                Ok(providers) if !providers.is_empty() => {
                    let provider = &providers[0];
                    let endpoint = format!(
                        "{}{}",
                        provider.registration.endpoint,
                        provider.registration.workload_endpoint
                    );

                    info!("✅ Found legacy provider: {}", provider.registration.provider_name);
                    return Ok(RoutingDecision::RouteToExternalProvider {
                        provider_id: provider.registration.provider_id.clone(),
                        execution_endpoint: endpoint,
                        capability_name,
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
            Err(e) => {
                Err(SongbirdError::service(
                    capability_name,
                    format!("No provider found for capability: {}", e),
                ))
            }
        }
    }

    /// Query Universal Port Authority service registry
    async fn query_service_registry(
        &self,
        capability: &str,
    ) -> SongbirdResult<Option<RoutingDecision>> {
        let services = self.service_registry.query_by_capability(capability).await;

        // Filter to active services only
        let active_services: Vec<_> = services
            .into_iter()
            .filter(|s| s.status == ServiceStatus::Active)
            .collect();

        if active_services.is_empty() {
            return Ok(None);
        }

        // Select best service (for now, first active)
        // TODO: Implement load balancing (least loaded, geographic, etc.)
        let service = &active_services[0];

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

    /// Check if local capacity is available
    async fn has_local_capacity(&self) -> bool {
        // TODO: Implement actual capacity checking
        // For now, assume we always have capacity
        true
    }

    /// Route to a peer Songbird instance
    async fn route_to_peer_songbird(&self) -> SongbirdResult<RoutingDecision> {
        let state = self.federation_state.nodes.read().await;
        let nodes = state.values().cloned().collect::<Vec<_>>();
        
        // Filter to active nodes only
        let active_nodes: Vec<_> = nodes
            .into_iter()
            .filter(|n| n.status == NodeStatus::Active)
            .collect();

        // Find a healthy peer
        for node in active_nodes {
            // Get preferred endpoint
            if let Some(endpoint) = node.preferred_endpoint() {
                info!("Routing to peer Songbird: {} at {}", node.node_name, endpoint.address);
                // Clone to avoid borrow checker issues
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

    /// Convert capability type to string name
    fn capability_type_to_name(capability_type: &CapabilityType) -> String {
        match capability_type {
            CapabilityType::Compute => "compute".to_string(),
            CapabilityType::Security => "security".to_string(),
            CapabilityType::Ai => "ai".to_string(),
            CapabilityType::Storage => "storage".to_string(),
            CapabilityType::Orchestration => "orchestration".to_string(),
            CapabilityType::Observability => "observability".to_string(),
            CapabilityType::Networking => "networking".to_string(),
            CapabilityType::Custom(name) => name.clone(),
        }
    }

    /// Execute task on registered service (NEW)
    pub async fn execute_on_registered_service(
        &self,
        service_id: &str,
        endpoint: &str,
        task: &Task,
    ) -> SongbirdResult<serde_json::Value> {
        info!("🚀 Executing task on registered service {} at {}", service_id, endpoint);

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true) // Dev mode
            .timeout(Duration::from_secs(300))
            .build()
            .map_err(|e| SongbirdError::Network {
                message: format!("Failed to create HTTP client: {}", e),
                interface: None,
                suggestion: Some("Check system network configuration".to_string()),
            })?;

        let response = client
            .post(format!("{}/execute", endpoint))
            .json(task)
            .send()
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Failed to send task to service: {}", e),
                interface: Some(endpoint.to_string()),
                suggestion: Some("Check service health and network connectivity".to_string()),
            })?;

        if !response.status().is_success() {
            return Err(SongbirdError::Service {
                service: service_id.to_string(),
                message: format!("Service returned error status: {}", response.status()),
                suggested_alternatives: vec![],
                recovery_actions: vec![],
            });
        }

        let result = response
            .json()
            .await
            .map_err(|e| SongbirdError::Service {
                service: service_id.to_string(),
                message: format!("Failed to parse service response: {}", e),
                suggested_alternatives: vec![],
                recovery_actions: vec![],
            })?;

        info!("✅ Task executed successfully on service {}", service_id);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_type_to_name() {
        assert_eq!(
            EnhancedCapabilityRouter::capability_type_to_name(&CapabilityType::Compute),
            "compute"
        );
        assert_eq!(
            EnhancedCapabilityRouter::capability_type_to_name(&CapabilityType::Security),
            "security"
        );
    }
}

