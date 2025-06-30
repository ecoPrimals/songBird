/*!
 * Gaming Network Demo - Songbird Orchestrator
 *
 * This example demonstrates how to create virtual LANs for legacy gaming
 * and optimize modern gaming traffic using Songbird's existing architecture.
 *
 * 🎮 LEGACY GAMING SUPPORT:
 * - StarCraft 1 over internet (IPX emulation)
 * - Age of Empires LAN party simulation
 * - Quake/Doom UDP broadcast bridging
 * - NetBIOS name resolution
 *
 * 🌐 MODERN GAMING OPTIMIZATION:
 * - Traffic prioritization for competitive games
 * - Jitter reduction and latency optimization
 * - NAT traversal for peer-to-peer gaming
 * - Game server selection and load balancing
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use songbird_gaming_bridge::communication::ProtocolRouter;
use songbird_gaming_bridge::config::OrchestratorConfig;
use songbird_gaming_bridge::errors::SongbirdError;
use songbird_gaming_bridge::orchestrator::Orchestrator;
use songbird_gaming_bridge::traits::service_id::{
    ResponseStatus, ServiceEndpoint, ServiceInfo, ServiceMetrics, ServiceRequest, ServiceResponse,
    UniversalService,
};

/// Gaming network bridge for legacy and modern games
#[derive(Clone)]
pub struct GamingNetworkBridge {
    id: String,
    name: String,
    protocol_router: Arc<ProtocolRouter>,
    virtual_lans: HashMap<String, VirtualLAN>,
    game_optimizers: HashMap<GameProtocol, GameOptimizer>,
    nat_traversal: NatTraversalManager,
}

/// Supported game protocols
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameProtocol {
    // Legacy RTS games
    StarCraft { version: StarCraftVersion },
    AgeOfEmpires { version: AoeVersion },
    CommandAndConquer { version: CncVersion },

    // Legacy FPS games
    Quake { version: QuakeVersion },
    Doom { version: DoomVersion },
    HalfLife { mod_name: Option<String> },

    // Modern competitive games
    CounterStrike2,
    Valorant,
    LeagueOfLegends,
    Minecraft { version: String },

    // Generic protocols
    DirectPlay,
    IPX,
    NetBIOS,
    UDPBroadcast,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StarCraftVersion {
    Original,
    BroodWar,
    Remastered,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AoeVersion {
    AgeOfEmpires,
    AgeOfKings,
    AgeOfMythology,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CncVersion {
    TiberianDawn,
    RedAlert,
    TiberianSun,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuakeVersion {
    Quake1,
    Quake2,
    Quake3Arena,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DoomVersion {
    Doom,
    Doom2,
    FinalDoom,
}

/// Virtual LAN configuration for gaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualLAN {
    pub lan_id: String,
    pub game_protocol: GameProtocol,
    pub participants: Vec<GameParticipant>,
    pub network_config: VirtualNetworkConfig,
    pub optimization_settings: OptimizationSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameParticipant {
    pub player_id: String,
    pub display_name: String,
    pub real_address: SocketAddr,
    pub virtual_address: Ipv4Addr,
    pub connection_quality: ConnectionQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualNetworkConfig {
    pub subnet: String,
    pub broadcast_address: Ipv4Addr,
    pub dhcp_range: (Ipv4Addr, Ipv4Addr),
    pub dns_servers: Vec<Ipv4Addr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSettings {
    pub max_latency_ms: u32,
    pub jitter_buffer_ms: u32,
    pub packet_prioritization: bool,
    pub traffic_shaping: bool,
    pub anti_cheat_integration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionQuality {
    pub latency_ms: f32,
    pub jitter_ms: f32,
    pub packet_loss_percent: f32,
    pub bandwidth_mbps: f32,
}

/// Game-specific optimizer
#[derive(Debug, Clone)]
pub struct GameOptimizer {
    pub protocol: GameProtocol,
    pub latency_target: Duration,
    pub jitter_reduction: bool,
    pub packet_prioritization: bool,
}

/// NAT traversal manager for peer-to-peer gaming
#[derive(Debug, Clone)]
pub struct NatTraversalManager {
    pub stun_servers: Vec<String>,
    pub turn_servers: Vec<String>,
    pub upnp_enabled: bool,
    pub ice_enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GamingNetworkConfig {
    pub bridge_name: String,
    pub supported_games: Vec<GameProtocol>,
    pub default_optimization: OptimizationSettings,
    pub nat_traversal: NatTraversalConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NatTraversalConfig {
    pub stun_servers: Vec<String>,
    pub turn_servers: Vec<String>,
    pub enable_upnp: bool,
    pub enable_ice: bool,
}

impl GamingNetworkBridge {
    pub fn new(id: String, config: GamingNetworkConfig) -> Self {
        println!(
            "🎮 Initializing Gaming Network Bridge: {}",
            config.bridge_name
        );

        let mut game_optimizers = HashMap::new();
        for game in &config.supported_games {
            game_optimizers.insert(
                game.clone(),
                GameOptimizer {
                    protocol: game.clone(),
                    latency_target: Duration::from_millis(50),
                    jitter_reduction: true,
                    packet_prioritization: true,
                },
            );
        }

        Self {
            id,
            name: config.bridge_name,
            protocol_router: Arc::new(ProtocolRouter::new()),
            virtual_lans: HashMap::new(),
            game_optimizers,
            nat_traversal: NatTraversalManager {
                stun_servers: config.nat_traversal.stun_servers,
                turn_servers: config.nat_traversal.turn_servers,
                upnp_enabled: config.nat_traversal.enable_upnp,
                ice_enabled: config.nat_traversal.enable_ice,
            },
        }
    }

    /// Create a virtual LAN for a specific game
    pub async fn create_virtual_lan(
        &mut self,
        game: GameProtocol,
        players: Vec<String>,
    ) -> Result<String, SongbirdError> {
        let lan_id = format!(
            "vlan_{}_{}",
            self.id,
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        );

        println!("🌐 Creating virtual LAN '{}' for game: {:?}", lan_id, game);

        // Create participants with virtual IP addresses
        let mut participants = Vec::new();
        let base_ip = Ipv4Addr::new(192, 168, 100, 10);

        for (i, player) in players.iter().enumerate() {
            let virtual_ip = Ipv4Addr::new(192, 168, 100, 10 + i as u8);
            participants.push(GameParticipant {
                player_id: player.clone(),
                display_name: format!("Player_{}", i + 1),
                real_address: SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    8000 + i as u16,
                ),
                virtual_address: virtual_ip,
                connection_quality: ConnectionQuality {
                    latency_ms: 20.0,
                    jitter_ms: 2.0,
                    packet_loss_percent: 0.1,
                    bandwidth_mbps: 100.0,
                },
            });
        }

        let virtual_lan = VirtualLAN {
            lan_id: lan_id.clone(),
            game_protocol: game.clone(),
            participants,
            network_config: VirtualNetworkConfig {
                subnet: "192.168.100.0/24".to_string(),
                broadcast_address: Ipv4Addr::new(192, 168, 100, 255),
                dhcp_range: (
                    Ipv4Addr::new(192, 168, 100, 10),
                    Ipv4Addr::new(192, 168, 100, 250),
                ),
                dns_servers: vec![Ipv4Addr::new(8, 8, 8, 8)],
            },
            optimization_settings: OptimizationSettings {
                max_latency_ms: 50,
                jitter_buffer_ms: 10,
                packet_prioritization: true,
                traffic_shaping: true,
                anti_cheat_integration: matches!(
                    game,
                    GameProtocol::CounterStrike2 | GameProtocol::Valorant
                ),
            },
        };

        // Apply game-specific optimizations
        self.apply_game_optimizations(&game).await?;

        self.virtual_lans.insert(lan_id.clone(), virtual_lan);

        println!(
            "✅ Virtual LAN '{}' created successfully with {} players",
            lan_id,
            players.len()
        );
        Ok(lan_id)
    }

    /// Apply game-specific network optimizations
    async fn apply_game_optimizations(&self, game: &GameProtocol) -> Result<(), SongbirdError> {
        match game {
            GameProtocol::StarCraft { .. } => {
                println!("🔧 Applying StarCraft optimizations: IPX emulation, broadcast bridging");
                // Enable IPX packet translation to UDP
                // Set up NetBIOS name resolution
                // Configure broadcast-to-unicast conversion
            }

            GameProtocol::CounterStrike2 => {
                println!(
                    "🔧 Applying CS2 optimizations: Ultra-low latency, anti-cheat integration"
                );
                // Enable ultra-low latency mode
                // Integrate with VAC anti-cheat
                // Prioritize game packets
            }

            GameProtocol::Minecraft { .. } => {
                println!("🔧 Applying Minecraft optimizations: Chunk loading, multiplayer sync");
                // Optimize chunk loading traffic
                // Reduce multiplayer synchronization overhead
            }

            _ => {
                println!("🔧 Applying generic gaming optimizations");
                // Apply general low-latency optimizations
            }
        }

        Ok(())
    }

    /// Handle legacy protocol translation (e.g., IPX to UDP)
    pub async fn translate_legacy_packet(
        &self,
        protocol: GameProtocol,
        packet_data: &[u8],
    ) -> Result<Vec<u8>, SongbirdError> {
        match protocol {
            GameProtocol::IPX => {
                println!("🔄 Translating IPX packet to UDP");
                // Convert IPX packet format to UDP
                // Maintain game compatibility
                Ok(packet_data.to_vec()) // Simplified
            }

            GameProtocol::NetBIOS => {
                println!("🔄 Translating NetBIOS packet");
                // Handle NetBIOS name resolution
                // Convert to modern DNS-like resolution
                Ok(packet_data.to_vec()) // Simplified
            }

            _ => Ok(packet_data.to_vec()),
        }
    }

    /// Perform NAT traversal for peer-to-peer connections
    pub async fn establish_p2p_connection(
        &self,
        local_player: &str,
        remote_player: &str,
    ) -> Result<SocketAddr, SongbirdError> {
        println!(
            "🌐 Establishing P2P connection between {} and {}",
            local_player, remote_player
        );

        // STUN server discovery
        for stun_server in &self.nat_traversal.stun_servers {
            println!("🔍 Trying STUN server: {}", stun_server);
            // Attempt STUN binding request
        }

        // ICE candidate gathering
        if self.nat_traversal.ice_enabled {
            println!("🧊 Gathering ICE candidates");
            // Collect host, server reflexive, and relay candidates
        }

        // UPnP port mapping
        if self.nat_traversal.upnp_enabled {
            println!("🔓 Attempting UPnP port mapping");
            // Try to open ports via UPnP
        }

        // Return successful connection endpoint
        Ok(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)),
            12345,
        ))
    }
}

#[async_trait::async_trait]
impl UniversalService for GamingNetworkBridge {
    type Config = GamingNetworkConfig;
    type Health = String;
    type Error = SongbirdError;

    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.name = config.bridge_name.clone();
        println!(
            "🔧 Initializing Gaming Network Bridge with {} supported games",
            config.supported_games.len()
        );
        Ok(())
    }

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("🚀 Starting Gaming Network Bridge: {}", self.name);

        // Start protocol router
        self.protocol_router.start_all().await?;

        // Initialize game protocol handlers
        for game in self.game_optimizers.keys() {
            println!("🎮 Registering handler for: {:?}", game);
        }

        println!("✅ Gaming Network Bridge started successfully");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        println!("🛑 Stopping Gaming Network Bridge: {}", self.name);

        // Clean up virtual LANs
        for (lan_id, _) in &self.virtual_lans {
            println!("🧹 Cleaning up virtual LAN: {}", lan_id);
        }
        self.virtual_lans.clear();

        // Stop protocol router
        self.protocol_router.stop_all().await?;

        println!("✅ Gaming Network Bridge stopped");
        Ok(())
    }

    async fn health_check(&self) -> Result<Self::Health, Self::Error> {
        let active_lans = self.virtual_lans.len();
        let status = if active_lans > 0 {
            format!("Healthy - {} active virtual LANs", active_lans)
        } else {
            "Healthy - Ready for gaming sessions".to_string()
        };
        Ok(status)
    }

    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> Result<ServiceResponse, Self::Error> {
        println!(
            "🎮 Gaming Network Bridge handling request: {} {}",
            request.method, request.path
        );

        let response_payload = match request.path.as_str() {
            "/create_lan" => {
                let game_type = request
                    .body
                    .get("game")
                    .and_then(|g| g.as_str())
                    .unwrap_or("StarCraft");
                let players: Vec<String> = request
                    .body
                    .get("players")
                    .and_then(|p| p.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_else(|| vec!["Player1".to_string(), "Player2".to_string()]);

                serde_json::json!({
                    "action": "create_virtual_lan",
                    "game": game_type,
                    "players": players,
                    "lan_id": format!("vlan_{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
                    "status": "Virtual LAN created successfully",
                    "network": {
                        "subnet": "192.168.100.0/24",
                        "broadcast": "192.168.100.255"
                    }
                })
            }

            "/optimize_game" => {
                let game = request
                    .body
                    .get("game")
                    .and_then(|g| g.as_str())
                    .unwrap_or("CounterStrike2");

                serde_json::json!({
                    "action": "optimize_gaming_traffic",
                    "game": game,
                    "optimizations": {
                        "latency_reduction": true,
                        "jitter_control": true,
                        "packet_prioritization": true,
                        "target_latency_ms": 10
                    },
                    "status": "Game traffic optimized"
                })
            }

            "/establish_p2p" => {
                let local_player = request
                    .body
                    .get("local_player")
                    .and_then(|p| p.as_str())
                    .unwrap_or("Player1");
                let remote_player = request
                    .body
                    .get("remote_player")
                    .and_then(|p| p.as_str())
                    .unwrap_or("Player2");

                serde_json::json!({
                    "action": "establish_p2p_connection",
                    "local_player": local_player,
                    "remote_player": remote_player,
                    "connection_endpoint": "203.0.113.1:12345",
                    "nat_traversal": {
                        "method": "STUN",
                        "ice_candidates": 3,
                        "upnp_success": true
                    },
                    "status": "P2P connection established"
                })
            }

            "/list_lans" => {
                serde_json::json!({
                    "action": "list_virtual_lans",
                    "active_lans": self.virtual_lans.len(),
                    "supported_games": ["StarCraft", "Age of Empires", "Quake", "Counter-Strike 2", "Minecraft"],
                    "status": "Virtual LANs listed"
                })
            }

            _ => {
                serde_json::json!({
                    "error": "Unknown gaming network operation",
                    "available_operations": ["/create_lan", "/optimize_game", "/establish_p2p", "/list_lans"]
                })
            }
        };

        Ok(ServiceResponse {
            request_id: request.id,
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            body: response_payload,
            timestamp: chrono::Utc::now(),
            processing_time: std::time::Duration::from_millis( Duration::from_millis(10),
            processing_time: std::time::Duration::from_millis(10),
            
        })
    }

    async fn update_config(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        println!("🔧 Updating Gaming Network Bridge configuration");
        self.name = config.bridge_name;
        Ok(())
    }

    fn get_info(&self) -> ServiceInfo {
        ServiceInfo {
            service_id: self.id.clone(),
            name: self.name.clone(),
            service_type: "gaming_network_bridge".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Virtual LAN creation and gaming traffic optimization").to_string(),
            endpoints: vec![
                ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                    path: "/create_lan".to_string(),
                    method: "POST".to_string(),
                    description: Some("Create virtual LAN for gaming").to_string(),
                },
                ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                    path: "/optimize_game".to_string(),
                    method: "POST".to_string(),
                    description: Some("Optimize traffic for specific games").to_string(),
                },
                ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                    path: "/establish_p2p".to_string(),
                    method: "POST".to_string(),
                    description: Some("Establish peer-to-peer connections").to_string(),
                },
            ],
            tags: vec![
                "virtual_lan_creation".to_string(),
                "legacy_game_support".to_string(),
                "traffic_optimization".to_string(),
                "nat_traversal".to_string(),
                "ipx_emulation".to_string(),
            ],
            dependencies: vec!["protocol_router".to_string()],
            resource_requirements: HashMap::new(),
            
        }
    }

    async fn get_metrics(&self) -> Result<ServiceMetrics, Self::Error> {
        Ok(ServiceMetrics {
            requests_handled: 100,
            average_response_time: Duration::from_millis(15),
            error_rate: 0.01,
            resource_usage: HashMap::new(),
            custom_metrics: HashMap::from([
                (
                    "active_virtual_lans".to_string(),
                    serde_json::json!(self.virtual_lans.len()),
                ),
                (
                    "supported_games".to_string(),
                    serde_json::json!(self.game_optimizers.len()),
                ),
            ]),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Gaming Network Bridge Demo - Songbird Orchestrator");
    println!("=======================================================");

    // Create gaming network configuration
    let gaming_config = GamingNetworkConfig {
        bridge_name: "Songbird Gaming Bridge".to_string(),
        supported_games: vec![
            GameProtocol::StarCraft {
                version: StarCraftVersion::Original,
            },
            GameProtocol::AgeOfEmpires {
                version: AoeVersion::AgeOfKings,
            },
            GameProtocol::Quake {
                version: QuakeVersion::Quake3Arena,
            },
            GameProtocol::CounterStrike2,
            GameProtocol::Minecraft {
                version: "1.20.4".to_string(),
            },
        ],
        default_optimization: OptimizationSettings {
            max_latency_ms: 50,
            jitter_buffer_ms: 10,
            packet_prioritization: true,
            traffic_shaping: true,
            anti_cheat_integration: false,
        },
        nat_traversal: NatTraversalConfig {
            stun_servers: vec![
                "stun.l.google.com:19302".to_string(),
                "stun1.l.google.com:19302".to_string(),
            ],
            turn_servers: vec!["turn.songbird.local:3478".to_string()],
            enable_upnp: true,
            enable_ice: true,
        },
    };

    // Create and configure orchestrator
    let config = OrchestratorConfig::default();
    let mut orchestrator = Orchestrator::new(config).await?;

    // Create gaming network bridge service
    let mut gaming_bridge = GamingNetworkBridge::new("gaming_bridge_1".to_string(), gaming_config);

    // Initialize and start the gaming bridge
    gaming_bridge
        .initialize(GamingNetworkConfig {
            bridge_name: "Demo Gaming Bridge".to_string(),
            supported_games: vec![GameProtocol::StarCraft {
                version: StarCraftVersion::Original,
            }],
            default_optimization: OptimizationSettings {
                max_latency_ms: 20,
                jitter_buffer_ms: 5,
                packet_prioritization: true,
                traffic_shaping: true,
                anti_cheat_integration: false,
            },
            nat_traversal: NatTraversalConfig {
                stun_servers: vec!["stun.l.google.com:19302".to_string()],
                turn_servers: vec![],
                enable_upnp: true,
                enable_ice: true,
            },
        })
        .await?;

    gaming_bridge.start().await?;

    // Register the gaming bridge with orchestrator
    orchestrator.register_service(gaming_bridge, ()).await?;

    println!("\n🎮 Gaming Network Bridge Demo Scenarios:");
    println!("==========================================");

    // Demo 1: Create StarCraft virtual LAN
    println!("\n1️⃣  Creating StarCraft virtual LAN...");
    let starcraft_request = ServiceRequest {
        id: "starcraft_lan_1".to_string(),
        method: "POST".to_string(),
        path: "/create_lan".to_string(),
        headers: HashMap::new(),
        body: serde_json::json!({
            "game": "StarCraft",
            "players": ["Alice", "Bob", "Charlie", "Diana"]
        }),
        timeout: Some(Duration::from_secs(10)),
        timestamp: chrono::Utc::now(),
        
    };

    let response = orchestrator
        .handle_service_request("gaming_bridge_1", starcraft_request)
        .await?;
    println!(
        "✅ StarCraft LAN Response: {}",
        serde_json::to_string_pretty(&response.body)?
    );

    sleep(Duration::from_secs(1)).await;

    // Demo 2: Optimize Counter-Strike 2 traffic
    println!("\n2️⃣  Optimizing Counter-Strike 2 traffic...");
    let cs2_request = ServiceRequest {
        id: "cs2_optimization_1".to_string(),
        method: "POST".to_string(),
        path: "/optimize_game".to_string(),
        headers: HashMap::new(),
        body: serde_json::json!({
            "game": "CounterStrike2"
        }),
        timeout: Some(Duration::from_secs(5)),
        timestamp: chrono::Utc::now(),
        
    };

    let response = orchestrator
        .handle_service_request("gaming_bridge_1", cs2_request)
        .await?;
    println!(
        "✅ CS2 Optimization Response: {}",
        serde_json::to_string_pretty(&response.body)?
    );

    sleep(Duration::from_secs(1)).await;

    // Demo 3: Establish P2P connection
    println!("\n3️⃣  Establishing P2P connection...");
    let p2p_request = ServiceRequest {
        id: "p2p_connection_1".to_string(),
        method: "POST".to_string(),
        path: "/establish_p2p".to_string(),
        headers: HashMap::new(),
        body: serde_json::json!({
            "local_player": "Alice",
            "remote_player": "Bob"
        }),
        timeout: Some(Duration::from_secs(15)),
        timestamp: chrono::Utc::now(),
        
    };

    let response = orchestrator
        .handle_service_request("gaming_bridge_1", p2p_request)
        .await?;
    println!(
        "✅ P2P Connection Response: {}",
        serde_json::to_string_pretty(&response.body)?
    );

    sleep(Duration::from_secs(1)).await;

    // Demo 4: List active virtual LANs
    println!("\n4️⃣  Listing active virtual LANs...");
    let list_request = ServiceRequest {
        id: "list_lans_1".to_string(),
        method: "GET".to_string(),
        path: "/list_lans".to_string(),
        headers: HashMap::new(),
        body: serde_json::json!({}),
        timeout: Some(Duration::from_secs(5)),
        timestamp: chrono::Utc::now(),
        
    };

    let response = orchestrator
        .handle_service_request("gaming_bridge_1", list_request)
        .await?;
    println!(
        "✅ Virtual LANs List: {}",
        serde_json::to_string_pretty(&response.body)?
    );

    // Health check
    println!("\n🏥 Health Check:");
    let health_request = ServiceRequest {
        id: "health_check_1".to_string(),
        method: "GET".to_string(),
        path: "/health".to_string(),
        headers: HashMap::new(),
        body: serde_json::json!({}),
        timeout: Some(Duration::from_secs(5)),
        timestamp: chrono::Utc::now(),
        
    };

    let response = orchestrator
        .handle_service_request("gaming_bridge_1", health_request)
        .await?;
    println!(
        "✅ Health Status: {}",
        serde_json::to_string_pretty(&response.body)?
    );

    println!("\n🎉 Gaming Network Bridge Demo completed successfully!");
    println!("\n💡 This demonstrates how Songbird's existing architecture");
    println!("   can be extended to support gaming networks and legacy protocols.");
    println!("   The trait-based design makes it easy to add new game protocols");
    println!("   and optimization strategies without changing core code.");

    Ok(())
}
