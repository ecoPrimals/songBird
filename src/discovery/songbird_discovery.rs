use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::pin::Pin;
use futures_util::Stream;
use tokio::sync::{RwLock, broadcast};
use chrono::Utc;
use uuid::Uuid;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket as TokioUdpSocket;

use crate::errors::Result;
use crate::traits::discovery::{ServiceDiscovery, ServiceQuery, ServiceEvent, ServiceHealthStatus};
use crate::traits::service::ServiceInfo;

// Import our modular components
use super::types::*;
use super::config::*;
use super::resources::ResourceDetector;
use super::monitoring::ResourceMonitor;
use super::network::NetworkManager;

/// Songbird Discovery Service - Custom Rust-native service discovery
/// Optimized for scientific computing federation scenarios
#[derive(Clone)]
pub struct SongbirdDiscovery {
    /// Local node information
    local_node: LocalNode,
    
    /// Known nodes in the federation
    known_nodes: Arc<RwLock<HashMap<NodeId, NodeInfo>>>,
    
    /// Services registered on this node
    local_services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    
    /// Event broadcasting for service changes
    event_sender: broadcast::Sender<ServiceEvent>,
    
    /// Configuration
    config: SongbirdDiscoveryConfig,
    
    /// Network timing configuration
    network_timing: NetworkTimingConfig,
}

impl SongbirdDiscovery {
    /// Create new Songbird Discovery Service
    pub fn new(config: SongbirdDiscoveryConfig) -> Self {
        let node_id = config.node_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        
        let local_node = LocalNode {
            id: node_id.clone(),
            node_type: config.node_type.clone(),
            institution: config.institution.clone(),
            resources: ResourceDetector::detect_local_resources(),
            network_location: ResourceDetector::detect_network_location(),
            created_at: Utc::now(),
        };

        let (event_sender, _) = broadcast::channel(1000);

        Self {
            local_node,
            known_nodes: Arc::new(RwLock::new(HashMap::new())),
            local_services: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            config,
            network_timing: NetworkTimingConfig::default(),
        }
    }

    /// Get local node information
    pub fn local_node(&self) -> &LocalNode {
        &self.local_node
    }

    /// Register a node in the federation
    pub async fn register_node(&self, node: NodeInfo) -> Result<()> {
        self.known_nodes.write().await.insert(node.id.clone(), node.clone());
        
        // Broadcast node registration event
        let _ = self.event_sender.send(ServiceEvent::ServiceRegistered {
            service: ServiceInfo {
                id: format!("node-{}", node.id),
                name: format!("Federation Node {}", node.id),
                version: "1.0.0".to_string(),
                service_type: format!("{:?}", node.node_type),
                description: format!("Federation node: {:?}", node.node_type),
                endpoints: Vec::new(),
                capabilities: vec!["federation".to_string(), "discovery".to_string()],
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("node_type".to_string(), format!("{:?}", node.node_type));
                    if let Some(inst) = &node.institution {
                        tags.insert("institution".to_string(), inst.clone());
                    }
                    tags
                },
                metadata: HashMap::new(),
            }
        });

        tracing::info!("Registered federation node: {} ({:?})", node.id, node.node_type);
        Ok(())
    }

    /// Find optimal nodes for a resource query
    pub async fn find_optimal_nodes(&self, query: ResourceQuery) -> Result<Vec<NodeInfo>> {
        let nodes = self.known_nodes.read().await;
        let mut matching_nodes = Vec::new();

        for node in nodes.values() {
            if self.node_matches_query(node, &query) {
                matching_nodes.push(node.clone());
            }
        }

        // Sort by optimization criteria
        matching_nodes.sort_by(|a, b| {
            // Simple sorting by available resources and reputation
            let score_a = a.resources.memory_available_gb as f64 + a.reputation_score;
            let score_b = b.resources.memory_available_gb as f64 + b.reputation_score;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(matching_nodes)
    }

    /// Check if a node matches a resource query
    fn node_matches_query(&self, node: &NodeInfo, query: &ResourceQuery) -> bool {
        // Check minimum CPU cores
        if let Some(min_cpu) = query.min_cpu_cores {
            if node.resources.cpu_cores < min_cpu {
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
        if let Some(required_type) = &query.required_node_type {
            if &node.node_type != required_type {
                return false;
            }
        }

        // Check institution filter
        if let Some(required_institution) = &query.institution_filter {
            if node.institution.as_ref() != Some(required_institution) {
                return false;
            }
        }

        // Check minimum trust level
        if query.min_trust_level > node.trust_level {
            return false;
        }

        true
    }

    /// Get federation statistics
    pub async fn get_federation_stats(&self) -> FederationStats {
        let nodes = self.known_nodes.read().await;
        let services = self.local_services.read().await;
        
        let mut stats = FederationStats::default();
        stats.total_nodes = nodes.len() as u32;
        stats.total_services = services.len() as u32;
        
        // Count by node type
        for node in nodes.values() {
            match node.node_type {
                NodeType::Compute => stats.compute_nodes += 1,
                NodeType::Storage => stats.storage_nodes += 1,
                NodeType::Gateway => stats.gateway_nodes += 1,
                NodeType::Hybrid => stats.hybrid_nodes += 1,
                NodeType::Orchestrator => stats.orchestrator_nodes += 1,
            }
            
            // Aggregate resources
            stats.total_cpu_cores += node.resources.cpu_cores;
            stats.total_memory_gb += node.resources.memory_total_gb;
            stats.total_storage_gb += node.storage_capacity.total_capacity_gb;
        }

        stats
    }

    /// Start federation discovery background tasks
    pub async fn start_federation(&self) -> Result<()> {
        if !self.config.federation_enabled {
            return Ok(());
        }

        tracing::info!("Starting Songbird Discovery Federation for node: {}", self.local_node.id);
        
        // Start background tasks
        self.start_health_monitor().await?;
        self.start_node_discovery().await?;
        self.start_resource_monitor().await?;
        self.start_network_listener().await?;
        self.start_federation_announcer().await?;
        
        Ok(())
    }

    /// Start health monitoring background task
    async fn start_health_monitor(&self) -> Result<()> {
        let known_nodes = Arc::clone(&self.known_nodes);
        let health_interval = self.config.health_check_interval_secs;
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(health_interval));

            loop {
                interval.tick().await;
                
                let nodes = known_nodes.read().await;
                let now = Utc::now();
                
                for (node_id, node) in nodes.iter() {
                    // Check if node was last seen too long ago
                    let seconds_since_last_seen = now
                        .signed_duration_since(node.last_seen)
                        .num_seconds();
                    
                    if seconds_since_last_seen > (health_interval * 3) as i64 {
                        tracing::warn!("Node {} appears unhealthy (last seen {} seconds ago)", 
                                     node_id, seconds_since_last_seen);
                        
                        let _ = event_sender.send(ServiceEvent::ServiceHealthChanged {
                            service_id: format!("node-{}", node_id),
                            health: ServiceHealthStatus::Unhealthy,
                        });
                    }
                }
            }
        });

        Ok(())
    }

    /// Start node discovery background task
    async fn start_node_discovery(&self) -> Result<()> {
        let known_nodes = Arc::clone(&self.known_nodes);
        let local_node = self.local_node.clone();
        let discovery_interval = self.config.node_discovery_interval_secs;
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(discovery_interval));

            loop {
                interval.tick().await;
                
                // Send discovery requests and announcements
                NetworkManager::broadcast_node_discovery(&local_node, &config.network).await;
                
                // Measure network to known nodes
                Self::measure_network_to_known_nodes(&known_nodes).await;
                
                tracing::debug!("Node discovery heartbeat for node: {}", local_node.id);
            }
        });

        Ok(())
    }

    /// Measure network performance to all known nodes
    async fn measure_network_to_known_nodes(known_nodes: &Arc<RwLock<HashMap<NodeId, NodeInfo>>>) {
        let nodes = known_nodes.read().await.clone();
        
        for (node_id, node_info) in nodes {
            tokio::spawn(async move {
                // Extract IP from address (format: "ip:port")
                let target_ip = if let Some(colon_pos) = node_info.address.find(':') {
                    &node_info.address[..colon_pos]
                } else {
                    &node_info.address
                };

                // Measure latency and bandwidth
                if let Ok((latency, bandwidth)) = NetworkManager::perform_network_measurement(target_ip).await {
                    tracing::debug!("Network to {}: {:.2}ms, {:.2} Mbps", node_id, latency, bandwidth);
                }
            });
        }
    }

    /// Start network listener for federation messages
    async fn start_network_listener(&self) -> Result<()> {
        let known_nodes = Arc::clone(&self.known_nodes);
        let local_services = Arc::clone(&self.local_services);
        let event_sender = self.event_sender.clone();
        let local_node_id = self.local_node.id.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let bind_addr = format!("{}:{}", config.network.bind_address, config.network.federation_port);
            if let Ok(socket) = TokioUdpSocket::bind(&bind_addr).await {
                tracing::info!("Federation network listener started on {}", bind_addr);
                
                let mut buffer = vec![0u8; config.network.max_packet_size];
                
                loop {
                    if let Ok((len, addr)) = socket.recv_from(&mut buffer).await {
                        if let Ok(message_str) = std::str::from_utf8(&buffer[..len]) {
                            if let Ok(message) = serde_json::from_str::<FederationMessage>(message_str) {
                                Self::handle_federation_message(
                                    message, 
                                    addr, 
                                    &known_nodes, 
                                    &local_services,
                                    &event_sender,
                                    &local_node_id,
                                    &config
                                ).await;
                            }
                        }
                    }
                    
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            } else {
                tracing::error!("Failed to bind UDP socket for federation discovery on {}", bind_addr);
            }
        });

        Ok(())
    }

    /// Handle incoming federation messages
    async fn handle_federation_message(
        message: FederationMessage,
        sender_addr: SocketAddr,
        known_nodes: &Arc<RwLock<HashMap<NodeId, NodeInfo>>>,
        _local_services: &Arc<RwLock<HashMap<String, ServiceInfo>>>,
        event_sender: &broadcast::Sender<ServiceEvent>,
        local_node_id: &str,
        config: &SongbirdDiscoveryConfig,
    ) {
        match message {
            FederationMessage::NodeAnnouncement { node, timestamp: _ } => {
                tracing::info!("Received node announcement from {} at {}", node.id, sender_addr);
                
                // Update node information
                let mut nodes = known_nodes.write().await;
                nodes.insert(node.id.clone(), node.clone());
                
                // Broadcast node registration event
                let _ = event_sender.send(ServiceEvent::ServiceRegistered {
                    service: ServiceInfo {
                        id: format!("node-{}", node.id),
                        name: format!("Federation Node {}", node.id),
                        version: "1.0.0".to_string(),
                        service_type: format!("{:?}", node.node_type),
                        description: format!("Federation node: {:?}", node.node_type),
                        endpoints: Vec::new(),
                        capabilities: vec!["federation".to_string(), "discovery".to_string()],
                        tags: {
                            let mut tags = HashMap::new();
                            tags.insert("node_type".to_string(), format!("{:?}", node.node_type));
                            if let Some(inst) = &node.institution {
                                tags.insert("institution".to_string(), inst.clone());
                            }
                            tags
                        },
                        metadata: HashMap::new(),
                    }
                });
            }
            
            FederationMessage::NodeDiscoveryRequest { sender_id, .. } => {
                if sender_id != local_node_id {
                    tracing::debug!("Received discovery request from {}", sender_id);
                    
                    // Send back our known nodes
                    let nodes: Vec<NodeInfo> = known_nodes.read().await.values().cloned().collect();
                    NetworkManager::send_discovery_response(nodes, sender_addr, &config.network).await;
                }
            }
            
            FederationMessage::Heartbeat { node_id, resource_usage, .. } => {
                if let Some(node) = known_nodes.write().await.get_mut(&node_id) {
                    node.current_load = resource_usage;
                    node.last_seen = Utc::now();
                    
                    tracing::debug!("Updated heartbeat for node {}", node_id);
                }
            }
            
            FederationMessage::ServiceAdvertisement { node_id, services, .. } => {
                tracing::info!("Received service advertisement from {}: {} services", node_id, services.len());
                
                // Update node's service list
                if let Some(node) = known_nodes.write().await.get_mut(&node_id) {
                    node.services = services.iter().map(|s| s.id.clone()).collect();
                    node.last_seen = Utc::now();
                }
            }
            
            _ => {
                tracing::debug!("Received other federation message");
            }
        }
    }

    /// Start federation announcer - periodically announce this node
    async fn start_federation_announcer(&self) -> Result<()> {
        let local_node = self.local_node.clone();
        let local_services = Arc::clone(&self.local_services);
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(config.network.announcement_interval_secs)
            );

            loop {
                interval.tick().await;
                
                // Create current resource usage
                let current_load = ResourceMonitor::create_resource_usage(
                    &config.monitoring, 
                    config.network.default_bandwidth_mbps
                ).await;
                
                // Send node announcement
                NetworkManager::send_node_announcement(&local_node, &config.network, current_load).await;
                
                // Send service advertisement
                let services: Vec<ServiceInfo> = local_services.read().await.values().cloned().collect();
                if !services.is_empty() {
                    NetworkManager::send_service_advertisement(&local_node.id, services, &config.network).await;
                }
            }
        });

        Ok(())
    }

    /// Start resource monitoring background task
    async fn start_resource_monitor(&self) -> Result<()> {
        let local_node_id = self.local_node.id.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(config.monitoring.resource_update_interval_secs)
            );

            loop {
                interval.tick().await;
                
                // Create resource usage for heartbeat
                let resource_usage = ResourceMonitor::create_resource_usage(
                    &config.monitoring, 
                    config.network.default_bandwidth_mbps
                ).await;
                
                // Send heartbeat to federation
                NetworkManager::send_heartbeat(&local_node_id, resource_usage, &config.network).await;
                
                tracing::debug!("Resource monitoring update for node: {}", local_node_id);
            }
        });

        Ok(())
    }

    /// Measure network performance to a node
    pub async fn measure_network_performance(&self, node_id: &str, target_address: &str) -> Result<(f64, f64)> {
        let (latency_ms, bandwidth_mbps) = NetworkManager::perform_network_measurement(target_address).await?;

        // Update measurements in node info
        if let Some(node) = self.known_nodes.write().await.get_mut(node_id) {
            node.latency_measurements.insert(self.local_node.id.clone(), latency_ms);
            node.bandwidth_measurements.insert(self.local_node.id.clone(), bandwidth_mbps);
            node.last_seen = Utc::now();
        }

        Ok((latency_ms, bandwidth_mbps))
    }

    /// Get nodes filtered by institution
    pub async fn get_nodes_by_institution(&self, institution: &str) -> Result<Vec<NodeInfo>> {
        let nodes = self.known_nodes.read().await;
        Ok(nodes.values()
            .filter(|node| node.institution.as_deref() == Some(institution))
            .cloned()
            .collect())
    }

    /// Get nodes by trust level or higher
    pub async fn get_trusted_nodes(&self, min_trust_level: TrustLevel) -> Result<Vec<NodeInfo>> {
        let nodes = self.known_nodes.read().await;
        Ok(nodes.values()
            .filter(|node| node.trust_level >= min_trust_level)
            .cloned()
            .collect())
    }
}

#[async_trait]
impl ServiceDiscovery for SongbirdDiscovery {
    async fn register(&self, service: ServiceInfo) -> Result<()> {
        self.local_services.write().await.insert(service.id.clone(), service.clone());
        
        // Broadcast service registration
        let _ = self.event_sender.send(ServiceEvent::ServiceRegistered { service });
        
        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> Result<()> {
        if self.local_services.write().await.remove(service_id).is_some() {
            let _ = self.event_sender.send(ServiceEvent::ServiceUnregistered {
                service_id: service_id.to_string(),
            });
        }
        Ok(())
    }

    async fn discover(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        let services = self.local_services.read().await;
        let mut matching_services = Vec::new();
        
        for service in services.values() {
            if self.service_matches_query(service, &query) {
                matching_services.push(service.clone());
            }
        }

        // Sort by relevance/priority
        matching_services.sort_by(|a, b| {
            // Prefer services with more capabilities
            b.capabilities.len().cmp(&a.capabilities.len())
        });

        Ok(matching_services)
    }

    async fn watch(&self, query: ServiceQuery) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        let receiver = self.event_sender.subscribe();
        let self_clone = Arc::new(self.clone());
        
        // Create filtered stream
        let filtered_stream = async_stream::stream! {
            let mut rx = receiver;
            while let Ok(event) = rx.recv().await {
                // Filter events based on query
                if self_clone.event_matches_query(&event, &query).await {
                    yield event;
                }
            }
        };
        
        Ok(Box::pin(filtered_stream))
    }

    async fn update_health(&self, service_id: &str, health: ServiceHealthStatus) -> Result<()> {
        if let Some(service) = self.local_services.write().await.get_mut(service_id) {
            service.metadata.insert("health_status".to_string(), format!("{:?}", health).into());
            
            let _ = self.event_sender.send(ServiceEvent::ServiceHealthChanged {
                service_id: service_id.to_string(),
                health,
            });
        }
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<ServiceInfo>> {
        Ok(self.local_services.read().await.values().cloned().collect())
    }

    async fn exists(&self, service_id: &str) -> Result<bool> {
        Ok(self.local_services.read().await.contains_key(service_id))
    }

    async fn is_registered(&self, service_id: &str) -> Result<bool> {
        Ok(self.local_services.read().await.contains_key(service_id))
    }

    async fn update_metadata(&self, service_id: &str, metadata: HashMap<String, String>) -> Result<()> {
        if let Some(service) = self.local_services.write().await.get_mut(service_id) {
            // Convert String values to serde_json::Value
            for (key, value) in metadata {
                service.metadata.insert(key, value.into());
            }
            
            let _ = self.event_sender.send(ServiceEvent::ServiceMetadataUpdated {
                service_id: service_id.to_string(),
            });
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SongbirdDiscovery {
    /// Check if a service matches a service query
    fn service_matches_query(&self, service: &ServiceInfo, query: &ServiceQuery) -> bool {
        // Check service type filter
        if let Some(ref service_type) = query.service_type {
            if service.service_type != *service_type {
                return false;
            }
        }

        // Check name filter (case-insensitive substring match)
        if let Some(ref name_filter) = query.name {
            if !service.name.to_lowercase().contains(&name_filter.to_lowercase()) {
                return false;
            }
        }

        // Check tag filters - service must have all required tags
        for required_tag in &query.tags {
            if !service.tags.contains_key(required_tag) {
                return false;
            }
        }

        // Check version requirements (basic comparison)
        if let Some(ref version_req) = query.version {
            if !self.version_matches_requirement(&service.version, version_req) {
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
                return false; // Required metadata not found
            }
        }

        true
    }

    /// Check if a service version matches a requirement
    fn version_matches_requirement(&self, service_version: &str, requirement: &str) -> bool {
        // Simplified version matching
        if requirement.starts_with(">=") {
            let req_version = &requirement[2..];
            service_version >= req_version
        } else if requirement.starts_with("<=") {
            let req_version = &requirement[2..];
            service_version <= req_version
        } else if requirement.starts_with(">") {
            let req_version = &requirement[1..];
            service_version > req_version
        } else if requirement.starts_with("<") {
            let req_version = &requirement[1..];
            service_version < req_version
        } else if requirement.starts_with("=") {
            let req_version = &requirement[1..];
            service_version == req_version
        } else {
            // Exact match
            service_version == requirement
        }
    }

    /// Check if an event matches a service query (for filtering watch streams)
    async fn event_matches_query(&self, event: &ServiceEvent, query: &ServiceQuery) -> bool {
        match event {
            ServiceEvent::ServiceRegistered { service } => {
                self.service_matches_query(service, query)
            }
            ServiceEvent::ServiceUnregistered { service_id } => {
                // Check if the unregistered service was of interest
                if let Ok(services) = self.list_all().await {
                    for service in services {
                        if service.id == *service_id {
                            return self.service_matches_query(&service, query);
                        }
                    }
                }
                false
            }
            ServiceEvent::ServiceHealthChanged { service_id, .. } => {
                // Always pass through health changes for watched services
                if let Ok(services) = self.list_all().await {
                    for service in services {
                        if service.id == *service_id {
                            return self.service_matches_query(&service, query);
                        }
                    }
                }
                false
            }
            ServiceEvent::ServiceMetadataUpdated { service_id } => {
                // Check if the updated service still matches the query
                if let Ok(services) = self.list_all().await {
                    for service in services {
                        if service.id == *service_id {
                            return self.service_matches_query(&service, query);
                        }
                    }
                }
                false
            }
        }
    }
} 