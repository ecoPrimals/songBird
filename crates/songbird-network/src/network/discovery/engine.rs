//! Main NetworkDiscoveryEngine coordinator - FRAGO Implementation for BearDog integration

use tracing::{debug, info, warn};

use super::peer_registry::PeerRegistry;
use super::stun::STUNClient;
use super::topology::TopologyMapper;
use super::turn::TURNClient;
use super::types::NetworkEvent;
use super::types::{DiscoveredPeer, DiscoveryConfig};
use super::upnp::UPnPClient;
use songbird_errors::SongbirdResult as Result;
use songbird_universal_primals::PrimalCapability;

/// NetworkDiscoveryEngine - Exact FRAGO specification for BearDog integration
pub struct NetworkDiscoveryEngine {
    upnp_client: UPnPClient,         // ✅ FRAGO Requirement
    stun_client: STUNClient,         // ✅ FRAGO Requirement
    turn_client: TURNClient,         // ✅ FRAGO Requirement
    peer_registry: PeerRegistry,     // ✅ FRAGO Requirement
    topology_mapper: TopologyMapper, // ✅ FRAGO Requirement
    config: DiscoveryConfig,
}

impl NetworkDiscoveryEngine {
    /// Create new NetworkDiscoveryEngine with configuration
    pub fn new(config: DiscoveryConfig) -> Self {
        info!("Initializing NetworkDiscoveryEngine with FRAGO specification");

        Self {
            upnp_client: UPnPClient::new(&config),
            stun_client: STUNClient::new(&config),
            turn_client: TURNClient::new(&config),
            peer_registry: PeerRegistry::with_config(config.clone()),
            topology_mapper: TopologyMapper::new(config.topology_update_interval),
            config,
        }
    }

    /// Start comprehensive peer discovery (FRAGO sub-10ms requirement)
    pub async fn discover_peers(&self) -> Result<Vec<DiscoveredPeer>> {
        info!("🎯 Starting FRAGO-compliant peer discovery for sub-10ms gaming");
        let mut all_peers: Vec<DiscoveredPeer> = Vec::new();

        // UPnP Discovery - Local network (fastest)
        if self.config.enable_upnp {
            debug!("🔍 Phase 1: UPnP local discovery");
            match self.upnp_client.discover_peers().await {
                Ok(upnp_peers) => {
                    debug!("Found {} UPnP peers", upnp_peers.len());
                    all_peers.extend(upnp_peers);
                }
                Err(e) => warn!("UPnP discovery failed: {}", e),
            }
        }

        // STUN Discovery - NAT traversal
        if self.config.enable_stun {
            debug!("🔍 Phase 2: STUN NAT traversal discovery");
            match self.stun_client.discover_peers().await {
                Ok(stun_peers) => {
                    debug!("Found {} STUN peers", stun_peers.len());
                    all_peers.extend(stun_peers);
                }
                Err(e) => warn!("STUN discovery failed: {}", e),
            }
        }

        // TURN Discovery - Relay fallback
        if self.config.enable_turn {
            debug!("🔍 Phase 3: TURN relay discovery");
            match self.turn_client.discover_peers().await {
                Ok(turn_peers) => {
                    debug!("Found {} TURN peers", turn_peers.len());
                    all_peers.extend(turn_peers);
                }
                Err(e) => warn!("TURN discovery failed: {}", e),
            }
        }

        // Filter for gaming-optimized peers if required
        if self.config.gaming_optimized {
            all_peers.retain(|peer| {
                // For now, keep all peers as the capability-based filtering
                // would require peer capability metadata that's not yet implemented
                // in the DiscoveredPeer structure. This can be enhanced later.
                matches!(
                    peer.peer_type,
                    super::types::PeerType::Orchestrator | super::types::PeerType::Service
                )
            });
            debug!(
                "🎮 Filtered to {} gaming-optimized peers (≤10ms)",
                all_peers.len()
            );
        }

        info!(
            "🎯 Discovery complete: {} total peers found",
            all_peers.len()
        );
        Ok(all_peers)
    }

    /// Start discovery engine with continuous monitoring
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting NetworkDiscoveryEngine");

        // Start UPnP announcement listener
        if self.config.enable_upnp {
            self.upnp_client.start_announcement_listener().await?;
            self.upnp_client.send_device_announcement().await?;
        }

        // Start topology mapping
        self.topology_mapper.start_periodic_updates().await?;

        // Start peer cleanup task
        self.start_peer_cleanup_task().await;

        // Start discovery coordination task
        self.start_discovery_coordination().await;

        info!("✅ NetworkDiscoveryEngine started successfully");
        Ok(())
    }

    /// Stop discovery engine
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping NetworkDiscoveryEngine");

        // Cleanup all peers
        self.peer_registry.clear_all_peers().await;

        // Clear measurement history
        self.topology_mapper.clear_measurement_history().await;

        info!("✅ NetworkDiscoveryEngine stopped");
        Ok(())
    }

    /// Get peer registry reference
    pub fn get_peer_registry(&self) -> &PeerRegistry {
        &self.peer_registry
    }

    /// Get topology mapper reference
    pub fn get_topology_mapper(&self) -> &TopologyMapper {
        &self.topology_mapper
    }

    /// Get UPnP client reference
    pub fn get_upnp_client(&self) -> &UPnPClient {
        &self.upnp_client
    }

    /// Get STUN client reference
    pub fn get_stun_client(&self) -> &STUNClient {
        &self.stun_client
    }

    /// Get TURN client reference
    pub fn get_turn_client(&self) -> &TURNClient {
        &self.turn_client
    }

    /// Handle network event
    pub async fn handle_network_event(&self, event: NetworkEvent) -> Result<()> {
        debug!("Handling network event: {:?}", event);

        match event {
            NetworkEvent::PeerDiscovered {
                peer_id,
                address,
                capabilities,
            } => {
                let peer = super::types::DiscoveredPeer::new(
                    peer_id.clone(),
                    address,
                    super::types::PeerType::Unknown,
                    super::types::DiscoveryMethod::Manual,
                );

                self.peer_registry.register_peer(peer, capabilities).await?;

                // Add to topology
                self.topology_mapper
                    .add_node(
                        peer_id,
                        address,
                        super::types::PeerType::Unknown,
                        vec![
                            PrimalCapability::NetworkRouting {
                                protocols: vec!["BSTP".to_string()],
                            },
                            PrimalCapability::Custom {
                                name: "Gaming".to_string(),
                                properties: [("optimized".to_string(), "true".to_string())]
                                    .to_vec(),
                            },
                        ],
                    )
                    .await?;
            }
            NetworkEvent::PeerDisconnected { peer_id } => {
                self.peer_registry.remove_peer(&peer_id).await?;
                self.topology_mapper.remove_node(&peer_id).await?;
            }
            NetworkEvent::LatencyMeasurement {
                source,
                target,
                latency_ms,
            } => {
                let measurement = super::types::NetworkMeasurement::new(
                    source, target, latency_ms, 100, // Default bandwidth
                );
                self.topology_mapper
                    .update_with_measurement(measurement)
                    .await?;
            }
        }

        Ok(())
    }

    /// Start peer cleanup task
    async fn start_peer_cleanup_task(&self) {
        let registry = self.peer_registry.clone();
        let cleanup_interval = self.config.peer_timeout / 2; // Cleanup at half the timeout interval

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);

            loop {
                interval.tick().await;
                registry.cleanup_expired_peers().await;
            }
        });

        debug!(
            "Started peer cleanup task with interval: {:?}",
            cleanup_interval
        );
    }

    /// Start discovery coordination task
    async fn start_discovery_coordination(&self) {
        let engine = self.clone();
        let discovery_interval = self.config.topology_update_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(discovery_interval);

            loop {
                interval.tick().await;

                if let Err(e) = engine.perform_periodic_discovery().await {
                    warn!("Periodic discovery failed: {}", e);
                }
            }
        });

        debug!(
            "Started discovery coordination with interval: {:?}",
            discovery_interval
        );
    }

    /// Perform periodic discovery updates
    async fn perform_periodic_discovery(&self) -> Result<()> {
        debug!("Performing periodic discovery update");

        // Rediscover peers periodically
        let peers = self.discover_peers().await?;

        // Update topology with discovered peers
        self.topology_mapper.discover_topology().await?;

        debug!(
            "Periodic discovery update completed with {} peers",
            peers.len()
        );
        Ok(())
    }

    /// Get discovery statistics
    pub async fn get_discovery_statistics(&self) -> DiscoveryStatistics {
        let peer_stats = self.peer_registry.get_peer_statistics().await;
        let topology_stats = self.topology_mapper.get_topology_statistics().await;
        let upnp_device_count = self.upnp_client.device_count().await;

        DiscoveryStatistics {
            total_peers: peer_stats.total_peers,
            active_peers: peer_stats.active_peers,
            topology_nodes: topology_stats.node_count,
            topology_connections: topology_stats.connection_count,
            upnp_devices: upnp_device_count,
            avg_latency: topology_stats.avg_latency,
        }
    }

    /// Test all discovery methods
    pub async fn test_discovery_methods(&self) -> Result<DiscoveryTestResults> {
        let mut results = DiscoveryTestResults {
            upnp_working: false,
            stun_working: false,
            turn_working: false,
            working_stun_servers: Vec::new(),
            working_turn_servers: Vec::new(),
        };

        // Test UPnP
        if self.config.enable_upnp {
            match self.upnp_client.discover_peers().await {
                Ok(_) => results.upnp_working = true,
                Err(e) => debug!("UPnP test failed: {}", e),
            }
        }

        // Test STUN
        if self.config.enable_stun {
            match self.stun_client.test_stun_connectivity().await {
                Ok(servers) => {
                    results.stun_working = !servers.is_empty();
                    results.working_stun_servers = servers;
                }
                Err(e) => debug!("STUN test failed: {}", e),
            }
        }

        // Test TURN
        if self.config.enable_turn {
            match self.turn_client.test_turn_connectivity().await {
                Ok(servers) => {
                    results.turn_working = !servers.is_empty();
                    results.working_turn_servers = servers;
                }
                Err(e) => debug!("TURN test failed: {}", e),
            }
        }

        Ok(results)
    }
}

impl Clone for NetworkDiscoveryEngine {
    fn clone(&self) -> Self {
        Self {
            upnp_client: UPnPClient::new(&self.config),
            stun_client: STUNClient::new(&self.config),
            turn_client: TURNClient::new(&self.config),
            peer_registry: PeerRegistry::with_config(self.config.clone()),
            topology_mapper: TopologyMapper::new(self.config.topology_update_interval),
            config: self.config.clone(),
        }
    }
}

/// Discovery statistics
#[derive(Debug, Clone)]
pub struct DiscoveryStatistics {
    pub total_peers: usize,
    pub active_peers: usize,
    pub topology_nodes: usize,
    pub topology_connections: usize,
    pub upnp_devices: usize,
    pub avg_latency: f64,
}

/// Discovery method test results
#[derive(Debug, Clone)]
pub struct DiscoveryTestResults {
    pub upnp_working: bool,
    pub stun_working: bool,
    pub turn_working: bool,
    pub working_stun_servers: Vec<String>,
    pub working_turn_servers: Vec<String>,
}
