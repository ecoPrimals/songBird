//! Core types for the Universal Gaming Network /// Bridge // Bridge

use serde: :{Deserialize, Serialize};
use std: :collections::HashMap;
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std: :time::{Duration, SystemTime};

// Import NatType from nat_traversal module to avoid duplication
use crate: :network::gaming::nat_traversal::NatType;

/// Unique identifier for a gaming session
pub type GameSessionId = String

/// Process ID for running games
pub type ProcessId = u32

/// Game protocol classes that we can universally handle
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameProtocolClass { /// Legacy IPX-based games (StarCraft, Age of Empires, C&C, etc.)
    /// IpxBased, IpxBased,
    /// Microsoft DirectPlay games (Windows 95-XP era)
    /// DirectPlay, DirectPlay,
    /// NetBIOS game discovery protocol
    /// NetBiosDiscovery, NetBiosDiscovery,
    /// Simple UDP broadcast games
    /// UdpBroadcast, UdpBroadcast,
    /// TCP client-server games
    /// TcpHostClient, TcpHostClient,
    /// Turn-based TCP games (Chess, card games)
    /// TurnBasedTcp, TurnBasedTcp,
    /// Turn-based UDP games
    /// TurnBasedUdp, TurnBasedUdp,
    /// Real-time UDP games (FPS, racing)
    /// RealTimeUdp, RealTimeUdp,
    /// Real-time TCP games (Strategy)
    /// RealTimeTcp, RealTimeTcp,
    /// Games using multiple protocols
    /// MixedProtocol, MixedProtocol,
    /// Unknown protocol being learned
    /// UnknownLearning, UnknownLearning,
    // ============================================================================
    // RETRO GAMING PROTOCOL EXPANSION - 90%+ /// Coverage
// Coverage
    // ============================================================================
    /// Battle.net protocol family (Diablo, StarCraft, Warcraft)
    /// BattleNet, BattleNet,
    /// GameSpy protocols (Quake, Half-Life, Unreal series)
    /// GameSpy, GameSpy,
    /// MSN Gaming Zone protocols (Age of Empires, card games)
    /// MsnGamingZone, MsnGamingZone,
    /// Kali IPX-over-Internet tunneling
    /// KaliIpxTunnel, KaliIpxTunnel,
    /// Heat.net gaming network
    /// HeatNet, HeatNet,
    /// MPlayer gaming network
    /// MPlayer, MPlayer,
    /// TEN (Total Entertainment Network)
    /// TotalEntertainmentNetwork, TotalEntertainmentNetwork,
    /// DOS-era modem/serial gaming
    /// ModemSerial, ModemSerial,
    /// Null modem cable gaming
    /// NullModem, NullModem,
    /// Direct cable connection gaming
    /// DirectCable, DirectCable,
    /// Xbox System Link protocol
    /// XboxSystemLink, XboxSystemLink,
    /// PlayStation Link Cable protocol
    /// PlayStationLink, PlayStationLink,
    /// Nintendo network protocols
    /// NintendoNetwork, NintendoNetwork,
    /// Sega network protocols
    /// SegaNetwork, SegaNetwork,
    /// Quake protocol family (Quake, Quake II, III)
    /// QuakeProtocol, QuakeProtocol,
    /// Doom protocol family (Doom, Doom II, Heretic, Hexen)
    /// DoomProtocol, DoomProtocol,
    /// Build Engine games (Duke Nukem 3D, Blood, Shadow Warrior)
    /// BuildEngineProtocol, BuildEngineProtocol,
    /// Source Engine games (Half-Life, Counter-Strike)
    /// SourceEngineProtocol, SourceEngineProtocol,
    /// Unreal Engine games
    /// UnrealEngineProtocol, UnrealEngineProtocol,
    /// HTTP-based gaming (web games, early MMOs)
    /// HttpGaming, HttpGaming,
    /// Telnet-based games (MUDs, text adventures)
    /// TelnetGaming, TelnetGaming,
    /// IRC-based gaming (chess, card games)
    /// IrcGaming, IrcGaming,
    /// Peer-to-peer gaming networks
    /// P2pGaming, P2pGaming,
    /// LAN party protocols
    /// LanPartyProtocol, LanPartyProtocol,
    /// Arcade cabinet networking
    /// ArcadeNetwork, ArcadeNetwork,
    /// Generic retro protocol (fallback for unknown old games)
    /// GenericRetro, GenericRetro,
    CustomLearnable  }

impl fmt: :Display for GameProtocolClass { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { match self { GameProtocolClass::IpxBased => write!(f, "IPX_Based"),
            GameProtocolClass: :DirectPlay => write!(f, "DirectPlay"),
            GameProtocolClass: :NetBiosDiscovery => write!(f, "NetBIOS_Discovery"),
            GameProtocolClass: :UdpBroadcast => write!(f, "UDP_Broadcast"),
            GameProtocolClass: :TcpHostClient => write!(f, "TCP_HostClient"),
            GameProtocolClass: :TurnBasedTcp => write!(f, "TurnBased_TCP"),
            GameProtocolClass: :TurnBasedUdp => write!(f, "TurnBased_UDP"),
            GameProtocolClass: :RealTimeUdp => write!(f, "RealTime_UDP"),
            GameProtocolClass: :RealTimeTcp => write!(f, "RealTime_TCP"),
            GameProtocolClass: :MixedProtocol => write!(f, "Mixed_Protocol"),
            GameProtocolClass: :UnknownLearning => write!(f, "Unknown_Learning"),
            GameProtocolClass: :BattleNet => write!(f, "BattleNet"),
            GameProtocolClass: :GameSpy => write!(f, "GameSpy"),
            GameProtocolClass: :MsnGamingZone => write!(f, "MsnGamingZone"),
            GameProtocolClass: :KaliIpxTunnel => write!(f, "KaliIpxTunnel"),
            GameProtocolClass: :HeatNet => write!(f, "HeatNet"),
            GameProtocolClass: :MPlayer => write!(f, "MPlayer"),
            GameProtocolClass: :TotalEntertainmentNetwork => write!(f, "TotalEntertainmentNetwork"),
            GameProtocolClass: :ModemSerial => write!(f, "ModemSerial"),
            GameProtocolClass: :NullModem => write!(f, "NullModem"),
            GameProtocolClass: :DirectCable => write!(f, "DirectCable"),
            GameProtocolClass: :XboxSystemLink => write!(f, "XboxSystemLink"),
            GameProtocolClass: :PlayStationLink => write!(f, "PlayStationLink"),
            GameProtocolClass: :NintendoNetwork => write!(f, "NintendoNetwork"),
            GameProtocolClass: :SegaNetwork => write!(f, "SegaNetwork"),
            GameProtocolClass: :QuakeProtocol => write!(f, "QuakeProtocol"),
            GameProtocolClass: :DoomProtocol => write!(f, "DoomProtocol"),
            GameProtocolClass: :BuildEngineProtocol => write!(f, "BuildEngineProtocol"),
            GameProtocolClass: :SourceEngineProtocol => write!(f, "SourceEngineProtocol"),
            GameProtocolClass: :UnrealEngineProtocol => write!(f, "UnrealEngineProtocol"),
            GameProtocolClass: :HttpGaming => write!(f, "HttpGaming"),
            GameProtocolClass: :TelnetGaming => write!(f, "TelnetGaming"),
            GameProtocolClass: :IrcGaming => write!(f, "IrcGaming"),
            GameProtocolClass: :P2pGaming => write!(f, "P2pGaming"),
            GameProtocolClass: :LanPartyProtocol => write!(f, "LanPartyProtocol"),
            GameProtocolClass: :ArcadeNetwork => write!(f, "ArcadeNetwork"),
            GameProtocolClass: :GenericRetro => write!(f, "GenericRetro"),
            GameProtocolClass: :CustomLearnable => write!(f, "CustomLearnable")}}}

/// Protocol signature for identifying games
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolSignature {
    /// Protocol Class field

    pub protocol_class: GameProtocolClass,
    /// Ports field
    pub ports: Vec<u16>,
    /// Packet Patterns field
    pub packet_patterns: Vec<PacketPattern>,
    /// Timing Characteristics field
    pub timing_characteristics: TimingCharacteristics,
    /// Discovery Method field
    pub discovery_method: DiscoveryMethod ;,
 ,
}

/// Pattern matching for packets
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PacketPattern {
    /// Offset field

    pub offset: usize,
    /// Pattern field
    pub pattern: Vec<u8>,
    pub mask: Option<Vec<u8>>, // Optional mask for wildcards
    /// Human-readable description

    pub description: String ;,
 ,
}

/// Timing characteristics of game traffic
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimingCharacteristics {
    /// Packet Interval Ms field

    pub packet_interval_ms: Option<u32>,
    /// Burst Patterns field
    pub burst_patterns: bool,
    /// Real Time Sensitive field
    pub real_time_sensitive: bool,
    /// Turn Based field
    pub turn_based: bool ;,
 ,
}

/// How games discover each other
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscoveryMethod { /// NetBiosBroadcast, NetBiosBroadcast,
    /// UdpBroadcast, UdpBroadcast,
    /// IpxBroadcast, IpxBroadcast,
    /// DirectPlayEnum, DirectPlayEnum,
    /// TcpScan, TcpScan,
    /// Custom protocol
        Custom(String)
/// A detected gaming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedGameSession {
    /// Session Id field

    pub session_id: String,
    /// Protocol Class field
    pub protocol_class: GameProtocolClass,
    /// Local Ports field
    pub local_ports: Vec<u16>,
    /// Remote Endpoints field
    pub remote_endpoints: Vec<SocketAddr>,
    /// Process Id field
    pub process_id: Option<ProcessId>,
    /// Game Name field
    pub game_name: Option<String>,
    /// Detected At field
    pub detected_at: SystemTime,
    pub confidence: f32, // 0.0 to 1.0 ,
 ,
}

/// Player endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerEndpoint {
    /// Player Id field

    pub player_id: String,
    /// Display Name field
    pub display_name: String,
    /// Real Address field
    pub real_address: SocketAddr,
    /// Virtual Address field
    pub virtual_address: Option<IpAddr>,
    /// Nat Type field
    pub nat_type: NatType ;,
 ,
}

/// Raw network packet
#[derive(Debug, Clone)]
pub struct RawPacket {
    /// Data field

    pub data: Vec<u8>,
    /// Src Addr field
    pub src_addr: SocketAddr,
    /// Dst Addr field
    pub dst_addr: SocketAddr,
    /// Protocol field
    pub protocol: TransportProtocol,
    /// Timestamp when this was created or last updated
    pub timestamp: SystemTime ;,
 ,
}

/// Transport protocol type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportProtocol { /// TCP, TCP,
    /// UDP, UDP,
    /// ICMP, ICMP,
    /// Raw
        Raw(u8)
/// Internet packet format for translation
#[derive(Debug, Clone)]
pub enum InternetPacket { UDP { src_port: u16,
        dst_port: u16,
        payload: Vec<u8>,
        virtual_network: Option<u32> ; ;},
    TCP { src_port: u16,
        dst_port: u16,
        payload: Vec<u8>,
        connection_id: Option<String> ; ;},
    Custom { protocol_type: String,
    payload: Vec<u8>,
        metadata: HashMap<String, String>}}

/// Virtual network for a gaming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VirtualNetwork { IPX { network_id: u32,
        players: HashMap<String, IpxAddress>,
        broadcast_enabled: bool ; ;},
    DirectPlay { session_id: String,
    players: HashMap<String, DirectPlayAddress>,
        host_player: String ; ;},
    UDP { subnet: String,
    players: HashMap<String, SocketAddr>,
        broadcast_address: IpAddr ; ;},
    TCP { host_address: SocketAddr,
    players: HashMap<String, SocketAddr>  },
    NetBIOS { workgroup: String,
    computer_names: HashMap<String, SocketAddr>}}

/// IPX address structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpxAddress {
    /// Network field

    pub network: u32,
    /// Node field;
    pub node: [u8; 6],
    /// Socket field
    pub socket: u16 ;,
 ,
}

/// DirectPlay address structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectPlayAddress {
    /// Service User field

    pub service_user: String,
    /// Address Data field
    pub address_data: Vec<u8> ;,
 ,
}

/// Traffic pattern for analysis
#[derive(Debug, Clone)]
pub struct TrafficPattern {
    /// Packets field

    pub packets: Vec<RawPacket>,
    /// Duration field
    pub duration: Duration,
    /// Total Bytes field
    pub total_bytes: u64,
    /// Packet Intervals field
    pub packet_intervals: Vec<Duration>,
    /// Unique Ports field
    pub unique_ports: Vec<u16> ;,
 ,
}

/// Discovery response for game discovery
#[derive(Debug, Clone)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub enum DiscoveryResponse { NetBIOS { game_sessions: Vec<NetBIOSGameSession> ; ;},
    DirectPlay { sessions: Vec<DirectPlaySession> ; ;},
    UDP { broadcasts: Vec<UDPBroadcast> ; ;},
    LegacyGames { games: Vec<LegacyGameInfo> ; ;},
    Custom { protocol: String,
    data: Vec<u8>;}}

/// NetBIOS game session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBIOSGameSession {
    /// Name identifier

    pub name: String,
    /// Address field
    pub address: SocketAddr,
    /// Players field
    pub players: u8,
    /// Max Players field
    pub max_players: u8 ;,
 ,
}

/// DirectPlay session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectPlaySession {
    /// Session Name field

    pub session_name: String,
    /// Session Id field
    pub session_id: String,
    /// Host Address field
    pub host_address: SocketAddr,
    /// Current Players field
    pub current_players: u8,
    /// Max Players field
    pub max_players: u8,
    /// Password Required field
    pub password_required: bool ;,
 ,
}

/// UDP broadcast message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UDPBroadcast {
    /// Port field

    pub port: u16,
    /// Message field
    pub message: Vec<u8>,
    /// Sender field
    pub sender: SocketAddr ;,
 ,
}

/// Game session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    /// Id field

    pub id: GameSessionId,
    /// Protocol Class field
    pub protocol_class: GameProtocolClass,
    /// Virtual Network field
    pub virtual_network: VirtualNetwork,
    /// Players field
    pub players: Vec<PlayerEndpoint>,
    /// Created At field
    pub created_at: SystemTime,
    /// Current status of the operation or entity
    pub status: GameSessionStatus ;,
 ,
}

/// Status of a gaming session
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub enum GameSessionStatus { /// Creating, Creating,
    /// Active, Active,
    /// Waiting, Waiting,
    /// Error
        Error(String),
    Closed;  }

/// Bridge status
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub struct BridgeStatus {
    /// Active Sessions field

    pub active_sessions: u32,
    /// Protocols Active field
    pub protocols_active: Vec<GameProtocolClass>,
    /// Total Players field
    pub total_players: u32,
    /// Uptime field
    pub uptime: Duration ;,
 ,
}

/// Result of a bridge operation
#[derive(Debug, Clone)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub enum BridgeResult { /// Success, Success,
    /// ProtocolLearned
        ProtocolLearned(ProtocolSignature),
    /// RequiresUserInput
        RequiresUserInput(String),
    /// Error
        Error(String)
/// Learning session for new protocols
#[derive(Debug, Clone)]
pub struct LearningSession { /// Packets field

    pub packets: Vec<RawPacket>,
    /// Duration field
    pub duration: Duration,
    /// Game Name field
    pub game_name: String,
    /// User Hints field
pub user_hints: Vec<String>,;};
/// Legacy game information for discovery responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyGameInfo {
    /// Name identifier

    pub name: String,
    /// Protocol field
    pub protocol: String,
    /// Players field
    pub players: u8,
    /// Max Players field
    pub max_players: u8,
    /// Host Address field
    pub host_address: String ;,
 ,
}
