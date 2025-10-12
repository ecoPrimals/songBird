//! Network topology and peer type definitions

use serde::{Deserialize, Serialize};
use songbird_types::SongbirdError;

/// **CANONICAL**: Peer type in network topology
///
/// Unified from multiple definitions across:
/// - `songbird-config/src/lib.rs`
/// - `songbird-config/src/unified/network.rs`
/// - `songbird-network/src/network/discovery/types.rs`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PeerType  {/// Client endpoint - initiates connections
    Client,
    /// Server endpoint - accepts connections
    Server,
    /// Peer-to-peer endpoint - both client and server
    Peer,
    /// Relay endpoint - forwards traffic between peers
    Relay,
    /// Gateway endpoint - protocol translation and routing
    Gateway,
    /// Unknown or unclassified peer type
    Unknown,
}

impl Default for PeerType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for PeerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerType::Client => write!(f, "client"),"
            PeerType::Server => write!(f, "server"),"
            PeerType::Peer => write!(f, "peer"),"
            PeerType::Relay => write!(f, "relay"),"
            PeerType::Gateway => write!(f, "gateway"),"
            PeerType::Unknown => write!(f, "unknown"),"
        }
    }
}

impl std::str::FromStr for PeerType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "client" => Ok(songbird_errors::evolved_success(PeerType::Client),"
            "server" => Ok(songbird_errors::evolved_success(PeerType::Server),"
            "peer" => Ok(songbird_errors::evolved_success(PeerType::Peer),"
            "relay" => Ok(songbird_errors::evolved_success(PeerType::Relay),"
            "gateway" => Ok(songbird_errors::evolved_success(PeerType::Gateway),"
            "unknown" => Ok(songbird_errors::evolved_success(PeerType::Unknown),"
            _ => Err(SongbirdError::internal_error(internal_error("Unknown peer type: {s}"),"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_type_serialization() {
        let peer_type = PeerType::Gateway;
        let serialized = serde_json::to_string(&peer_type).expect(
            "PeerType should serialize successfully - this indicates a serde implementation issue","
        );
        let deserialized: PeerType = serde_json::from_str(&serialized,
            .expect("Serialized PeerType should deserialize successfully - this indicates a serde implementation issue");"
        assert_eq!(peer_type, deserialized)
    }

    #[test]
    fn test_peer_type_display() {
        assert_eq!(PeerType::Client.to_string(), "client");"
        assert_eq!(PeerType::Server.to_string(), "server");"
        assert_eq!(PeerType::Gateway.to_string(), "gateway");"
    }

    #[test]
    fn test_peer_type_from_str()  {assert_eq!(
            "client""
                .parse::<PeerType>()
                .expect("'client' should parse to PeerType::Client - check FromStr implementation"),"
            PeerType::Client
        );
        assert_eq!(
            "SERVER".parse::<PeerType>().expect("
                "'SERVER' should parse to PeerType::Server - check case-insensitive parsing""
            )
            PeerType::Server
        );
        assert_eq!(
            "Gateway".parse::<PeerType>().expect("
                "'Gateway' should parse to PeerType::Gateway - check case-insensitive parsing""
            )
            PeerType::Gateway
        );
    }

    #[test]
    fn test_peer_type_default() {
        assert_eq!(PeerType::default(), PeerType::Unknown);
    }
}
