use async_trait::async_trait;
use futures_util::Stream;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::discovery::config::SongbirdDiscoveryConfig;
use crate::discovery::monitoring::ResourceMonitor;
use crate::discovery::network::NetworkManager;
use crate::discovery::resources::ResourceDetector;
use crate::discovery::types::{FederationStats, LocalNode, NodeId, NodeInfo, NodeType, ResourceQuery,
};
use crate::traits::discovery::{ServiceDiscovery, ServiceEvent, ServiceHealthStatus, ServiceQuery};
use crate::traits::service::ServiceInfo;
use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;

/// Main Songbird Discovery Service
pub struct SongbirdDiscovery {
    config: Arc<SongbirdDiscoveryConfig>,  // ✅ Shared via Arc for zero-copy
    local_node: LocalNode,
    known_nodes: Arc<RwLock<HashMap<NodeId, NodeInfo>>>,
    registered_services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    event_sender: broadcast::Sender<ServiceEvent>,
    #[allow(dead_code)]
    shutdown_sender: Option<tokio::sync::mpsc::Sender<()>>,
}

impl SongbirdDiscovery {
    /// Create a new Songbird Discovery instance
    #[must_use]
    pub fn new(config: SongbirdDiscoveryConfig) -> Self {
        let local_resources = ResourceDetector::detect_local_resources();
        let network_location = NetworkManager::create_network_location();

        let local_node = LocalNode {
            id: config.node_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            node_type: config.node_type.clone(),
            institution: config.institution.clone(),
            resources: local_resources,
            network_location,
            created_at: chrono::Utc::now(),
        };

        let (event_sender, _) = broadcast::channel(1000);

        Self {
            config: Arc::new(config),  // ✅ Wrap in Arc
            local_node,
            known_nodes: Arc::new(RwLock::new(HashMap::new())),
            registered_services: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            shutdown_sender: None,
        }
    }

    /// Get local node information
    #[must_use]
    pub fn local_node(&self) -> &LocalNode {
        &self.local_node
    }

    /// Register a new node in the discovery system
    pub fn register_node(&self, node: NodeInfo) -> Result<()> {
        let node_id = node.id.clone();  // Need clone for logging

        tracing::info!(
            node_id = %node_id,
            node_type = ?node.node_type,
            "Registering node in Songbird Discovery"
        );

        self.known_nodes.write().insert(node_id, node);  // ✅ Move node_id instead of clone

        // Node registered successfully - event broadcasting handled by federation layer
        tracing::debug!("Node registered: {}", node_id);

        Ok(())
    }

    /// Find nodes that match resource requirements
    pub fn find_optimal_nodes(&self, query: ResourceQuery) -> Result<Vec<NodeInfo>> {
        let nodes = self.known_nodes.read();
        let mut matching_nodes = Vec::new();

        for node in nodes.values() {
            if self.node_matches_query(node, &query) {
                matching_nodes.push(node.clone());
            }
        }

        // Sort by some criteria (e.g., resource availability, proximity)
        matching_nodes.sort_by(|a, b| {
            // Simple sorting by available resources
            let a_score = a.resources.cpu_cores + a.resources.memory_total_gb as u32;
            let b_score = b.resources.cpu_cores + b.resources.memory_total_gb as u32;
            b_score.cmp(&a_score)
        });

        Ok(matching_nodes)
    }

    /// Check if a node matches the resource query
    fn node_matches_query(&self, node: &NodeInfo, query: &ResourceQuery) -> bool {
        // Check minimum CPU cores
        if let Some(min_cores) = query.min_cpu_cores {
            if node.resources.cpu_cores < min_cores {
                return false;
            }
        }

        // Check minimum memory
        if let Some(min_memory) = query.min_memory_gb {
            if node.resources.memory_available_gb < min_memory {
                return false;
            }
        }

        // Check node type
        if let Some(ref required_type) = query.required_node_type {
            if node.node_type != *required_type {
                return false;
            }
        }

        // Check institution filter
        if let Some(ref institution_filter) = query.institution_filter {
            if node.institution.as_ref() != Some(institution_filter) {
                return false;
            }
        }

        // Check trust level
        if node.trust_level < query.min_trust_level {
            return false;
        }

        // Check maximum latency (if we have measurement data)
        if let Some(max_latency) = query.max_latency_ms {
            if let Some(latency) = node.latency_measurements.get(&self.local_node.id) {
                if *latency > max_latency {
                    return false;
                }
            }
        }

        // Check required datasets
        for required_dataset in &query.required_datasets {
            if !node.available_datasets.iter().any(|d| &d.id == required_dataset) {
                return false;
            }
        }

        true
    }

    /// Start federation discovery
    pub async fn start_federation(&self) -> Result<()> {
        if !self.config.federation_enabled {
            return Ok(());
        }

        tracing::info!("Starting federation discovery");

        // Start federation subsystems
        self.start_resource_monitoring()?;

        self.start_network_monitoring()?;

        Ok(())
    }

    /// Start resource monitoring
    fn start_resource_monitoring(&self) -> Result<()> {
        let node_id = self.local_node.id.clone();
        let config = Arc::clone(&self.config);  // ✅ Cheap Arc clone instead of data clone
        let (_shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel(1);

        tokio::spawn(async move {
            ResourceMonitor::start_monitoring(node_id, config, shutdown_rx).await;
        });

        Ok(())
    }

    /// Start network monitoring
    fn start_network_monitoring(&self) -> Result<()> {
        let node_id = self.local_node.id.clone();
        let target_nodes: Vec<(String, String)> = self
            .known_nodes
            .read()
            .values()
            .map(|node| (node.id.clone(), node.address.clone()))
            .collect();

        let (_shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel(1);

        tokio::spawn(async move {
            let _ = NetworkManager::start_network_monitoring(node_id, target_nodes, shutdown_rx);
        });

        Ok(())
    }

    /// Get federation statistics
    #[must_use]
    pub fn get_federation_stats(&self) -> FederationStats {
        let nodes = self.known_nodes.read();
        let services = self.registered_services.read();

        let mut stats = FederationStats {
            total_nodes: u32::try_from(nodes.len()).unwrap_or(u32::MAX) + 1, // +1 for local node
            total_services: u32::try_from(services.len()).unwrap_or(u32::MAX),
            ..Default::default()
        };

        // Count node types
        for node in nodes.values() {
            match node.node_type {
                NodeType::Compute => stats.compute_nodes += 1,
                NodeType::Storage => stats.storage_nodes += 1,
                NodeType::Gateway => stats.gateway_nodes += 1,
                NodeType::Hybrid => stats.hybrid_nodes += 1,
                NodeType::Orchestrator => stats.orchestrator_nodes += 1,
            }

            stats.total_cpu_cores += node.resources.cpu_cores;
            stats.total_memory_gb += node.resources.memory_total_gb;
            stats.total_storage_gb += node.storage_capacity.total_capacity_gb;
        }

        // Include local node in counts
        match self.local_node.node_type {
            NodeType::Compute => stats.compute_nodes += 1,
            NodeType::Storage => stats.storage_nodes += 1,
            NodeType::Gateway => stats.gateway_nodes += 1,
            NodeType::Hybrid => stats.hybrid_nodes += 1,
            NodeType::Orchestrator => stats.orchestrator_nodes += 1,
        }

        stats.total_cpu_cores += self.local_node.resources.cpu_cores;
        stats.total_memory_gb += self.local_node.resources.memory_total_gb;

        stats
    }
}

#[async_trait]
impl ServiceDiscovery for SongbirdDiscovery {
    // ServiceDiscovery trait methods

    async fn register(&self, service: ServiceInfo) -> Result<()> {
        let service_id = service.service_id.clone();

        tracing::info!(
            service_id = %service_id,
            "Registering service with Songbird discovery"
        );

        self.registered_services.write().insert(service_id, service);
        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> Result<()> {
        tracing::info!(
            service_id = %service_id,
            "Unregistering service from Songbird discovery"
        );

        self.registered_services.write().remove(service_id);
        Ok(())
    }

    async fn discover(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        let services = self.registered_services.read();
        let mut results = Vec::new();

        for service in services.values() {
            if Self::service_matches_query(service, &query) {
                results.push(service.clone());
            }
        }

        Ok(results)
    }

    async fn watch(
        &self,
        _query: ServiceQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        let receiver = self.event_sender.subscribe();

        // Create a simple stream from the broadcast receiver
        use futures_util::stream;
        let stream = stream::unfold(receiver, |mut rx| async move {
            match rx.recv().await {
                Ok(event) => Some((event, rx)),
                Err(_) => None,
            }
        });

        Ok(Box::pin(stream))
    }

    async fn update_health(&self, service_id: &str, health: ServiceHealthStatus) -> Result<()> {
        if let Some(service) = self.registered_services.write().get_mut(service_id) {
            service.metadata.insert("health_status".to_string(), format!("{:?}", health).into());
        }

        // Send health update event
        let _ = self.event_sender.send(ServiceEvent::ServiceHealthChanged {
            service_id: service_id.to_string(),
            health,
        });

        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<ServiceInfo>> {
        Ok(self.registered_services.read().values().cloned().collect())
    }

    async fn exists(&self, service_id: &str) -> Result<bool> {
        Ok(self.registered_services.read().contains_key(service_id))
    }

    async fn is_registered(&self, service_id: &str) -> Result<bool> {
        self.exists(service_id).await
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        if let Some(service) = self.registered_services.write().get_mut(service_id) {
            for (key, value) in metadata {
                service.metadata.insert(key, value.into());
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SongbirdDiscovery {
    /// Check if a service matches the query criteria
    fn service_matches_query(service: &ServiceInfo, query: &ServiceQuery) -> bool {
        // Check service ID filter
        if let Some(ref service_id) = query.service_id {
            if service.service_id != *service_id {
                return false;
            }
        }

        // Check service type filter
        if let Some(ref service_type) = query.service_type {
            if service.service_type != *service_type {
                return false;
            }
        }

        // Check name filter
        if let Some(ref name_filter) = query.name {
            if !service.name.to_lowercase().contains(&name_filter.to_lowercase()) {
                return false;
            }
        }

        // Check tag filters
        for required_tag in &query.tags {
            if !service.tags.contains(required_tag) {
                return false;
            }
        }

        // Check version requirements
        if let Some(ref version_req) = query.version {
            if !Self::check_version_requirement(&service.version, version_req) {
                return false;
            }
        }

        // Check metadata filters
        for (key, expected_value) in &query.metadata {
            if let Some(service_value) = service.metadata.get(key) {
                if service_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Check if a service version matches a requirement
    fn check_version_requirement(service_version: &str, requirement: &str) -> bool {
        if let Some(req_version) = requirement.strip_prefix(">=") {
            service_version >= req_version
        } else if let Some(req_version) = requirement.strip_prefix("<=") {
            service_version <= req_version
        } else if let Some(req_version) = requirement.strip_prefix(">") {
            service_version > req_version
        } else if let Some(req_version) = requirement.strip_prefix("<") {
            service_version < req_version
        } else if let Some(req_version) = requirement.strip_prefix("=") {
            service_version == req_version
        } else {
            // If no operator, assume exact match
            service_version == requirement
        }
    }
}

// Additional service management methods (separate from ServiceDiscovery trait)
impl SongbirdDiscovery {
    /// Register a service with the discovery system
    ///
    /// # Errors
    ///
    /// Returns an error if the service registration fails or if there are
    /// issues with the underlying discovery backend.
    pub fn register_service(&self, service: &ServiceInfo) -> Result<()> {
        let service_id = service.service_id.clone();

        tracing::info!(
            service_id = %service_id,
            service_type = %service.service_type,
            "Registering service with Songbird Discovery"
        );

        self.registered_services.write().insert(service_id.clone(), service.clone());

        // Service registered successfully - event broadcasting handled by federation layer
        tracing::debug!("Service registered: {}", service_id);

        Ok(())
    }

    /// Unregister a service from the discovery system
    ///
    /// # Errors
    ///
    /// Returns an error if the service deregistration fails or if there are
    /// issues with the underlying discovery backend.
    pub fn unregister_service(&self, service_id: &str) -> Result<()> {
        tracing::info!(service_id = %service_id, "Unregistering service from Songbird Discovery");

        self.registered_services.write().remove(service_id);

        // Service unregistered successfully - event broadcasting handled by federation layer
        tracing::debug!("Service unregistered: {}", service_id);

        Ok(())
    }
}
