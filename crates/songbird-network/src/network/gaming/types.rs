//! Core types for the Universal Gaming Network Bridge

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, SystemTime};

/// Unique identifier for a gaming session
pub type GameSessionId = String;

/// Process ID for running games
pub type ProcessId = u32;

/// Game protocol classes that we can universally handle
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameProtocolClass {
    /// Legacy IPX-based games (StarCraft, Age of Empires, C&C, etc.)
    IpxBased,
    /// Microsoft DirectPlay games (Windows 95-XP era)
    DirectPlay,
    /// NetBIOS game discovery protocol
    NetBiosDiscovery,
    /// Simple UDP broadcast games
    UdpBroadcast,
    /// TCP client-server games
    TcpHostClient,
    /// Turn-based TCP games (Chess, card games)
    TurnBasedTcp,
    /// Turn-based UDP games
    TurnBasedUdp,
    /// Real-time UDP games (FPS, racing)
    RealTimeUdp,
    /// Real-time TCP games (Strategy)
    RealTimeTcp,
    /// Games using multiple protocols
    MixedProtocol,
    /// Unknown protocol being learned
    UnknownLearning,

    // ============================================================================
    // RETRO GAMING PROTOCOL EXPANSION - 90%+ Coverage
    // ============================================================================
    /// Battle.net protocol family (Diablo, StarCraft, Warcraft)
    BattleNet,
    /// GameSpy protocols (Quake, Half-Life, Unreal series)
    GameSpy,
    /// MSN Gaming Zone protocols (Age of Empires, card games)
    MsnGamingZone,
    /// Kali IPX-over-Internet tunneling
    KaliIpxTunnel,
    /// Heat.net gaming network
    HeatNet,
    /// MPlayer gaming network
    MPlayer,
    /// TEN (Total Entertainment Network)
    TotalEntertainmentNetwork,

    /// DOS-era modem/serial gaming
    ModemSerial,
    /// Null modem cable gaming
    NullModem,
    /// Direct cable connection gaming
    DirectCable,

    /// Xbox System Link protocol
    XboxSystemLink,
    /// PlayStation Link Cable protocol
    PlayStationLink,
    /// Nintendo network protocols
    NintendoNetwork,
    /// Sega network protocols
    SegaNetwork,

    /// Quake protocol family (Quake, Quake II, III)
    QuakeProtocol,
    /// Doom protocol family (Doom, Doom II, Heretic, Hexen)
    DoomProtocol,
    /// Build Engine games (Duke Nukem 3D, Blood, Shadow Warrior)
    BuildEngineProtocol,
    /// Source Engine games (Half-Life, Counter-Strike)
    SourceEngineProtocol,
    /// Unreal Engine games
    UnrealEngineProtocol,

    /// HTTP-based gaming (web games, early MMOs)
    HttpGaming,
    /// Telnet-based games (MUDs, text adventures)
    TelnetGaming,
    /// IRC-based gaming (chess, card games)
    IrcGaming,

    /// Peer-to-peer gaming networks
    P2pGaming,
    /// LAN party protocols
    LanPartyProtocol,
    /// Arcade cabinet networking
    ArcadeNetwork,

    /// Generic retro protocol (fallback for unknown old games)
    GenericRetro,
    /// Custom protocol that can be taught/learned
    CustomLearnable,
}

impl fmt::Display for GameProtocolClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameProtocolClass::IpxBased => write!(f, "IPX_Based"),
            GameProtocolClass::DirectPlay => write!(f, "DirectPlay"),
            GameProtocolClass::NetBiosDiscovery => write!(f, "NetBIOS_Discovery"),
            GameProtocolClass::UdpBroadcast => write!(f, "UDP_Broadcast"),
            GameProtocolClass::TcpHostClient => write!(f, "TCP_HostClient"),
            GameProtocolClass::TurnBasedTcp => write!(f, "TurnBased_TCP"),
            GameProtocolClass::TurnBasedUdp => write!(f, "TurnBased_UDP"),
            GameProtocolClass::RealTimeUdp => write!(f, "RealTime_UDP"),
            GameProtocolClass::RealTimeTcp => write!(f, "RealTime_TCP"),
            GameProtocolClass::MixedProtocol => write!(f, "Mixed_Protocol"),
            GameProtocolClass::UnknownLearning => write!(f, "Unknown_Learning"),
            GameProtocolClass::BattleNet => write!(f, "BattleNet"),
            GameProtocolClass::GameSpy => write!(f, "GameSpy"),
            GameProtocolClass::MsnGamingZone => write!(f, "MsnGamingZone"),
            GameProtocolClass::KaliIpxTunnel => write!(f, "KaliIpxTunnel"),
            GameProtocolClass::HeatNet => write!(f, "HeatNet"),
            GameProtocolClass::MPlayer => write!(f, "MPlayer"),
            GameProtocolClass::TotalEntertainmentNetwork => write!(f, "TotalEntertainmentNetwork"),
            GameProtocolClass::ModemSerial => write!(f, "ModemSerial"),
            GameProtocolClass::NullModem => write!(f, "NullModem"),
            GameProtocolClass::DirectCable => write!(f, "DirectCable"),
            GameProtocolClass::XboxSystemLink => write!(f, "XboxSystemLink"),
            GameProtocolClass::PlayStationLink => write!(f, "PlayStationLink"),
            GameProtocolClass::NintendoNetwork => write!(f, "NintendoNetwork"),
            GameProtocolClass::SegaNetwork => write!(f, "SegaNetwork"),
            GameProtocolClass::QuakeProtocol => write!(f, "QuakeProtocol"),
            GameProtocolClass::DoomProtocol => write!(f, "DoomProtocol"),
            GameProtocolClass::BuildEngineProtocol => write!(f, "BuildEngineProtocol"),
            GameProtocolClass::SourceEngineProtocol => write!(f, "SourceEngineProtocol"),
            GameProtocolClass::UnrealEngineProtocol => write!(f, "UnrealEngineProtocol"),
            GameProtocolClass::HttpGaming => write!(f, "HttpGaming"),
            GameProtocolClass::TelnetGaming => write!(f, "TelnetGaming"),
            GameProtocolClass::IrcGaming => write!(f, "IrcGaming"),
            GameProtocolClass::P2pGaming => write!(f, "P2pGaming"),
            GameProtocolClass::LanPartyProtocol => write!(f, "LanPartyProtocol"),
            GameProtocolClass::ArcadeNetwork => write!(f, "ArcadeNetwork"),
            GameProtocolClass::GenericRetro => write!(f, "GenericRetro"),
            GameProtocolClass::CustomLearnable => write!(f, "CustomLearnable"),
        }
    }
}

/// Protocol signature for identifying games
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolSignature {
    pub protocol_class: GameProtocolClass,
    pub ports: Vec<u16>,
    pub packet_patterns: Vec<PacketPattern>,
    pub timing_characteristics: TimingCharacteristics,
    pub discovery_method: DiscoveryMethod,
}

/// Pattern matching for packets
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PacketPattern {
    pub offset: usize,
    pub pattern: Vec<u8>,
    pub mask: Option<Vec<u8>>, // Optional mask for wildcards
    pub description: String,
}

/// Timing characteristics of game traffic
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimingCharacteristics {
    pub packet_interval_ms: Option<u32>,
    pub burst_patterns: bool,
    pub real_time_sensitive: bool,
    pub turn_based: bool,
}

/// How games discover each other
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    NetBiosBroadcast,
    UdpBroadcast,
    IpxBroadcast,
    DirectPlayEnum,
    TcpScan,
    Custom(String),
}

/// A detected gaming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedGameSession {
    pub session_id: String,
    pub protocol_class: GameProtocolClass,
    pub local_ports: Vec<u16>,
    pub remote_endpoints: Vec<SocketAddr>,
    pub process_id: Option<ProcessId>,
    pub game_name: Option<String>,
    pub detected_at: SystemTime,
    pub confidence: f32, // 0.0 to 1.0
}

/// Player endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerEndpoint {
    pub player_id: String,
    pub display_name: String,
    pub real_address: SocketAddr,
    pub virtual_address: Option<IpAddr>,
    pub nat_type: NatType,
}

/// NAT type detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NatType {
    None,
    Open,
    FullCone,
    RestrictedCone,
    PortRestrictedCone,
    Symmetric,
    Unknown,
}

/// Raw network packet
#[derive(Debug, Clone)]
pub struct RawPacket {
    pub data: Vec<u8>,
    pub src_addr: SocketAddr,
    pub dst_addr: SocketAddr,
    pub protocol: TransportProtocol,
    pub timestamp: SystemTime,
}

/// Transport protocol type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportProtocol {
    TCP,
    UDP,
    ICMP,
    Raw(u8),
}

/// Internet packet format for translation
#[derive(Debug, Clone)]
pub enum InternetPacket {
    UDP {
        src_port: u16,
        dst_port: u16,
        payload: Vec<u8>,
        virtual_network: Option<u32>,
    },
    TCP {
        src_port: u16,
        dst_port: u16,
        payload: Vec<u8>,
        connection_id: Option<String>,
    },
    Custom {
        protocol_type: String,
        payload: Vec<u8>,
        metadata: HashMap<String, String>,
    },
}

/// Virtual network for a gaming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VirtualNetwork {
    IPX {
        network_id: u32,
        players: HashMap<String, IpxAddress>,
        broadcast_enabled: bool,
    },
    DirectPlay {
        session_id: String,
        players: HashMap<String, DirectPlayAddress>,
        host_player: String,
    },
    UDP {
        subnet: String,
        players: HashMap<String, SocketAddr>,
        broadcast_address: IpAddr,
    },
    TCP {
        host_address: SocketAddr,
        players: HashMap<String, SocketAddr>,
    },
}

/// IPX address structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpxAddress {
    pub network: u32,
    pub node: [u8; 6],
    pub socket: u16,
}

/// DirectPlay address structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectPlayAddress {
    pub service_user: String,
    pub address_data: Vec<u8>,
}

/// Traffic pattern for analysis
#[derive(Debug, Clone)]
pub struct TrafficPattern {
    pub packets: Vec<RawPacket>,
    pub duration: Duration,
    pub total_bytes: u64,
    pub packet_intervals: Vec<Duration>,
    pub unique_ports: Vec<u16>,
}

/// Discovery response for game discovery
#[derive(Debug, Clone)]
pub enum DiscoveryResponse {
    NetBIOS {
        game_sessions: Vec<NetBIOSGameSession>,
    },
    DirectPlay {
        sessions: Vec<DirectPlaySession>,
    },
    UDP {
        broadcasts: Vec<UDPBroadcast>,
    },
    LegacyGames {
        games: Vec<LegacyGameInfo>,
    },
    Custom {
        protocol: String,
        data: Vec<u8>,
    },
}

/// NetBIOS game session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBIOSGameSession {
    pub name: String,
    pub address: SocketAddr,
    pub players: u8,
    pub max_players: u8,
}

/// DirectPlay session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectPlaySession {
    pub session_name: String,
    pub session_id: String,
    pub host_address: SocketAddr,
    pub current_players: u8,
    pub max_players: u8,
    pub password_required: bool,
}

/// UDP broadcast message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UDPBroadcast {
    pub port: u16,
    pub message: Vec<u8>,
    pub sender: SocketAddr,
}

/// Game session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub id: GameSessionId,
    pub protocol_class: GameProtocolClass,
    pub virtual_network: VirtualNetwork,
    pub players: Vec<PlayerEndpoint>,
    pub created_at: SystemTime,
    pub status: GameSessionStatus,
}

/// Status of a gaming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameSessionStatus {
    Creating,
    Active,
    Waiting,
    Error(String),
    Closed,
}

/// Bridge status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatus {
    pub active_sessions: u32,
    pub protocols_active: Vec<GameProtocolClass>,
    pub total_players: u32,
    pub uptime: Duration,
}

/// Result of a bridge operation
#[derive(Debug, Clone)]
pub enum BridgeResult {
    Success,
    ProtocolLearned(ProtocolSignature),
    RequiresUserInput(String),
    Error(String),
}

/// Learning session for new protocols
#[derive(Debug, Clone)]
pub struct LearningSession {
    pub packets: Vec<RawPacket>,
    pub duration: Duration,
    pub game_name: String,
    pub user_hints: Vec<String>,
}

/// Legacy game information for discovery responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyGameInfo {
    pub name: String,
    pub protocol: String,
    pub players: u8,
    pub max_players: u8,
    pub host_address: String,
}
