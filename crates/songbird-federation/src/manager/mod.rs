//! Federation manager coordinating all federation components

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::{mpsc, RwLock};
use tracing::info;
use uuid::Uuid;

use crate::deployment::DeploymentManager;
use crate::discovery::DiscoveryEngine;
use crate::routing::RouteOptimizer;
use crate::security::SecurityManager;
use crate::types::*;
use songbird_errors::{Result, SongbirdError};

/// Main federation manager coordinating all components
pub struct FederationManager {
    /// Local node information
    local_node: Arc<RwLock<FederationNode>>,
    /// Discovered federation nodes
    nodes: Arc<RwLock<HashMap<Uuid, FederationNode>>>,
    /// Network topology cache
    topology: Arc<RwLock<NetworkTopology>>,
    /// Discovery engine
    discovery: Arc<DiscoveryEngine>,
    /// Route optimizer
    router: Arc<RouteOptimizer>,
    /// Deployment manager
    deployment: Arc<DeploymentManager>,
    /// Security manager
    security: Arc<SecurityManager>,
    /// Configuration
    config: FederationConfig,
    /// Event channels
    event_tx: mpsc::UnboundedSender<FederationEvent>,
}

impl FederationManager {
    /// Create new federation manager
    pub async fn new(config: FederationConfig) -> Result<Self> {
        let local_node = Arc::new(RwLock::new(Self::create_local_node(&config).await?));
        let nodes = Arc::new(RwLock::new(HashMap::new()));
        let topology = Arc::new(RwLock::new(NetworkTopology::new()));

        // Initialize components
        let discovery = Arc::new(DiscoveryEngine::new(config.discovery.clone()).await?);
        let router = Arc::new(RouteOptimizer::new(
            config.performance.route_strategy.clone(),
        ));
        let deployment = Arc::new(DeploymentManager::new(config.clone()).await?);
        let security = Arc::new(SecurityManager::new(config.security.clone()).await?);

        // Create event channel
        let (event_tx, _event_rx) = mpsc::unbounded_channel();

        Ok(Self {
            local_node,
            nodes,
            topology,
            discovery,
            router,
            deployment,
            security,
            config,
            event_tx,
        })
    }

    /// Start federation manager
    pub async fn start(&self) -> Result<()> {
        info!("Starting Songbird Federation Manager");

        // Start all components
        self.discovery.start().await?;
        self.security.start().await?;
        self.deployment.start().await?;

        // Start background tasks
        self.start_topology_monitoring().await?;
        self.start_route_optimization().await?;

        info!("Federation Manager started successfully");
        Ok(())
    }

    /// Discover nodes using proximity-first strategy
    pub async fn discover_nodes(&self) -> Result<Vec<FederationNode>> {
        info!("Starting proximity-first node discovery");

        let discovered_nodes = self.discovery.discover_nodes().await?;

        // Update local node cache
        {
            let mut nodes = self.nodes.write().await;
            for node in &discovered_nodes {
                nodes.insert(node.node_id, node.clone());
            }
        }

        // Update topology
        self.update_topology(&discovered_nodes).await?;

        info!("Total discovered nodes: {}", discovered_nodes.len());
        Ok(discovered_nodes)
    }

    /// Establish secure connection using security manager
    pub async fn establish_secure_connection(&self, node_id: Uuid) -> Result<SecuritySession> {
        let node = {
            let nodes = self.nodes.read().await;
            nodes.get(&node_id).cloned().ok_or_else(|| {
                SongbirdError::service_error("federation", "Node not found".to_string())
            })?
        };

        info!("Establishing secure connection to node: {}", node.name);

        let session = self.security.establish_session(&node).await?;

        // Update node with security session
        {
            let mut nodes = self.nodes.write().await;
            if let Some(node) = nodes.get_mut(&node_id) {
                node.security_session = Some(session.clone());
            }
        }

        info!("Secure connection established with node: {}", node.name);
        Ok(session)
    }

    /// Find optimal route to destination node
    pub async fn find_optimal_route(&self, destination: Uuid) -> Result<RouteInfo> {
        let local_node_id = {
            let local_node = self.local_node.read().await;
            local_node.node_id
        };

        let topology = self.topology.read().await;
        let nodes = self.nodes.read().await;

        self.router
            .calculate_optimal_route(local_node_id, destination, &topology, &nodes)
            .await
    }

    /// Deploy BYOB across federation
    pub async fn deploy_byob_federated(
        &self,
        team_id: String,
        requirements: FederatedDeploymentRequirements,
    ) -> Result<FederatedDeploymentResult> {
        info!("Starting federated BYOB deployment for team: {}", team_id);

        // Use deployment manager
        let result = self
            .deployment
            .deploy_federated(team_id, requirements)
            .await?;

        info!("Federated deployment completed: {}", result.deployment_id);
        Ok(result)
    }

    /// Get federation status
    pub async fn get_federation_status(&self) -> Result<FederationStatus> {
        let nodes = self.nodes.read().await;
        let topology = self.topology.read().await;
        let local_node = self.local_node.read().await;

        Ok(FederationStatus {
            local_node: local_node.clone(),
            total_nodes: nodes.len() as u32,
            online_nodes: nodes
                .values()
                .filter(|n| n.status == NodeStatus::Online)
                .count() as u32,
            topology_edges: topology
                .graph
                .values()
                .map(|edges| edges.len())
                .sum::<usize>() as u32,
            federation_health: self.calculate_federation_health(&nodes).await,
            last_updated: Utc::now(),
        })
    }

    /// Get all nodes in federation
    pub async fn get_nodes(&self) -> HashMap<Uuid, FederationNode> {
        let nodes = self.nodes.read().await;
        nodes.clone()
    }

    /// Get network topology
    pub async fn get_topology(&self) -> NetworkTopology {
        let topology = self.topology.read().await;
        topology.clone()
    }

    // Private implementation methods

    async fn create_local_node(config: &FederationConfig) -> Result<FederationNode> {
        let node_id = Uuid::new_v4();

        Ok(FederationNode {
            node_id,
            name: config.local_node.name.clone(),
            node_type: config.local_node.node_type.clone(),
            addresses: config
                .local_node
                .listen_addresses
                .iter()
                .map(|addr| NodeAddress {
                    addr: *addr,
                    addr_type: AddressType::Local,
                    latency_ms: Some(0),
                    bandwidth_mbps: Some(1000),
                    preference: 100,
                })
                .collect(),
            proximity: NetworkProximity::Localhost,
            security_session: None,
            metrics: NodeMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                network_latency_ms: 0,
                bandwidth_usage_mbps: 0,
                active_deployments: 0,
                load_score: 0.0,
            },
            last_seen: Utc::now(),
            status: NodeStatus::Online,
        })
    }

    async fn start_topology_monitoring(&self) -> Result<()> {
        let topology = self.topology.clone();
        let nodes = self.nodes.clone();
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;

                // Update topology based on current nodes
                Self::update_topology_with_nodes(&topology, &nodes).await;

                // Send topology update event
                let _ = event_tx.send(FederationEvent::DataSync {
                    event: DataSyncEvent::ReplicationCompleted {
                        source: "topology".to_string(),
                        target: "cache".to_string(),
                    },
                });
            }
        });

        Ok(())
    }

    async fn start_route_optimization(&self) -> Result<()> {
        let router = self.router.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
            loop {
                interval.tick().await;

                // Clean up expired routes
                router.cleanup_cache().await;
            }
        });

        Ok(())
    }

    async fn update_topology(&self, nodes: &[FederationNode]) -> Result<()> {
        let mut topology = self.topology.write().await;

        // Build topology graph from nodes
        for node in nodes {
            topology
                .graph
                .entry(node.node_id)
                .or_insert_with(HashSet::new);

            // Add connections to other nodes based on proximity
            for other_node in nodes {
                if node.node_id != other_node.node_id {
                    // Connect if in same proximity level or closer
                    if node.proximity <= other_node.proximity {
                        topology
                            .graph
                            .entry(node.node_id)
                            .or_insert_with(HashSet::new)
                            .insert(other_node.node_id);
                    }
                }
            }
        }

        topology.last_updated = std::time::Instant::now();
        Ok(())
    }

    async fn calculate_federation_health(&self, nodes: &HashMap<Uuid, FederationNode>) -> f32 {
        if nodes.is_empty() {
            return 0.0;
        }

        let healthy_nodes = nodes
            .values()
            .filter(|node| matches!(node.status, NodeStatus::Online | NodeStatus::Busy))
            .count();

        healthy_nodes as f32 / nodes.len() as f32
    }

    async fn update_topology_with_nodes(
        topology: &Arc<RwLock<NetworkTopology>>,
        nodes: &Arc<RwLock<HashMap<Uuid, FederationNode>>>,
    ) {
        let nodes_guard = nodes.read().await;
        let mut topology_guard = topology.write().await;

        // Update connections based on current nodes
        for (node_id, node) in nodes_guard.iter() {
            if node.status == NodeStatus::Online {
                topology_guard
                    .graph
                    .entry(*node_id)
                    .or_insert_with(HashSet::new);
            }
        }

        topology_guard.last_updated = std::time::Instant::now();
    }
}

use std::collections::HashSet;
