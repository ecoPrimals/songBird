//! Gaming Constants Module
//!
//! Gaming-specific constants for the Songbird configuration system.
//! Extracted from the main constants module for better organization.

use std::time::Duration;

// ============================================================================
// GAMING CONSTANTS
// ============================================================================

/// Gaming protocol types
pub const PROTOCOL_IPX: &str = "IPX";
pub const PROTOCOL_UDP: &str = "UDP";
pub const PROTOCOL_TCP: &str = "TCP";
pub const PROTOCOL_DIRECTPLAY: &str = "DirectPlay";

/// Gaming session limits
pub const MAX_PLAYERS_PER_SESSION: usize = 8;
pub const MIN_PLAYERS_PER_SESSION: usize = 2;
pub const MAX_CONCURRENT_SESSIONS: usize = 100;
pub const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

/// Gaming network timeouts
pub const GAMING_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10);
pub const GAMING_CONNECTION_TIMEOUT: Duration = Duration::from_secs(15);
pub const GAMING_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);
pub const GAMING_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// Gaming buffer sizes
pub const GAMING_BUFFER_SIZE: usize = 4096;
pub const MAX_GAMING_PACKET_SIZE: usize = 1500; // MTU size
pub const GAMING_RECEIVE_BUFFER_SIZE: usize = 8192;

/// Gaming port ranges
pub const GAMING_PORT_START: u16 = 6112;
pub const GAMING_PORT_END: u16 = 6200;
pub const IPX_PORT_RANGE_START: u16 = 213;
pub const IPX_PORT_RANGE_END: u16 = 215;
pub const DIRECTPLAY_PORT_RANGE_START: u16 = 2300;
pub const DIRECTPLAY_PORT_RANGE_END: u16 = 2400;

/// Gaming protocol identifiers
pub const IPX_NETWORK_ID: u32 = 0x1234_5678;
pub const BROADCAST_NODE_ID: [u8; 6] = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
pub const DEFAULT_SOCKET_ID: u16 = 0x869A;

/// Gaming message types
pub const MSG_TYPE_DISCOVERY: u8 = 0x01;
pub const MSG_TYPE_JOIN_REQUEST: u8 = 0x02;
pub const MSG_TYPE_JOIN_RESPONSE: u8 = 0x03;
pub const MSG_TYPE_GAME_DATA: u8 = 0x04;
pub const MSG_TYPE_HEARTBEAT: u8 = 0x05;
pub const MSG_TYPE_DISCONNECT: u8 = 0x06;
pub const MSG_TYPE_BROADCAST: u8 = 0x07;

/// Gaming protocol magic numbers
pub const IPX_MAGIC: u32 = 0x4950_5800; // "IPX\0"
pub const DIRECTPLAY_MAGIC: u32 = 0x4450_4C59; // "DPLY"
pub const STARCRAFT_MAGIC: u32 = 0x5354_4152; // "STAR"
pub const WARCRAFT_MAGIC: u32 = 0x5741_5233; // "WAR3"

/// Gaming session states
pub const SESSION_STATE_WAITING: u8 = 0;
pub const SESSION_STATE_ACTIVE: u8 = 1;
pub const SESSION_STATE_PAUSED: u8 = 2;
pub const SESSION_STATE_ENDED: u8 = 3;

/// Gaming performance constants
pub const TARGET_FPS: u32 = 60;
pub const MAX_FRAME_TIME_MS: u32 = 16; // ~60 FPS
pub const NETWORK_UPDATE_RATE_HZ: u32 = 20;
pub const MAX_NETWORK_LATENCY_MS: u32 = 100;

/// Gaming encryption constants
pub const GAMING_ENCRYPTION_KEY_SIZE: usize = 32; // 256-bit
pub const GAMING_IV_SIZE: usize = 16; // 128-bit
pub const GAMING_MAC_SIZE: usize = 32; // 256-bit

/// Helper functions
#[must_use]
pub fn is_valid_gaming_port(port: u16) -> bool {
    (GAMING_PORT_START..=GAMING_PORT_END).contains(&port)
        || (IPX_PORT_RANGE_START..=IPX_PORT_RANGE_END).contains(&port)
        || (DIRECTPLAY_PORT_RANGE_START..=DIRECTPLAY_PORT_RANGE_END).contains(&port)
}

#[must_use]
pub fn get_protocol_magic(protocol: &str) -> Option<u32> {
    match protocol.to_uppercase().as_str() {
        "IPX" => Some(IPX_MAGIC),
        "DIRECTPLAY" => Some(DIRECTPLAY_MAGIC),
        "STARCRAFT" => Some(STARCRAFT_MAGIC),
        "WARCRAFT" => Some(WARCRAFT_MAGIC),
        _ => None,
    }
}

#[must_use]
pub fn is_valid_session_state(state: u8) -> bool {
    matches!(
        state,
        SESSION_STATE_WAITING | SESSION_STATE_ACTIVE | SESSION_STATE_PAUSED | SESSION_STATE_ENDED
    )
}

#[must_use]
pub fn get_default_timeout_for_operation(operation: &str) -> Duration {
    match operation {
        "discovery" => GAMING_DISCOVERY_TIMEOUT,
        "session" => DEFAULT_SESSION_TIMEOUT,
        _ => GAMING_CONNECTION_TIMEOUT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaming_port_validation() {
        assert!(is_valid_gaming_port(6112));
        assert!(is_valid_gaming_port(213));
        assert!(is_valid_gaming_port(2300));
        assert!(!is_valid_gaming_port(8080));
    }

    #[test]
    fn test_protocol_magic_lookup() {
        assert_eq!(get_protocol_magic("IPX"), Some(IPX_MAGIC));
        assert_eq!(get_protocol_magic("DIRECTPLAY"), Some(DIRECTPLAY_MAGIC));
        assert_eq!(get_protocol_magic("unknown"), None);
    }

    #[test]
    fn test_session_state_validation() {
        assert!(is_valid_session_state(SESSION_STATE_WAITING));
        assert!(is_valid_session_state(SESSION_STATE_ACTIVE));
        assert!(!is_valid_session_state(255));
    }

    #[test]
    fn test_timeout_lookup() {
        assert_eq!(
            get_default_timeout_for_operation("discovery"),
            GAMING_DISCOVERY_TIMEOUT
        );
        assert_eq!(
            get_default_timeout_for_operation("connection"),
            GAMING_CONNECTION_TIMEOUT
        );
        assert_eq!(
            get_default_timeout_for_operation("unknown"),
            GAMING_CONNECTION_TIMEOUT
        );
    }

    #[test]
    fn test_gaming_constants() {
        assert_eq!(MAX_PLAYERS_PER_SESSION, 8);
        assert_eq!(MIN_PLAYERS_PER_SESSION, 2);
        const _: () = assert!(GAMING_BUFFER_SIZE > 0);
        const _: () = assert!(MAX_GAMING_PACKET_SIZE <= 1500);
    }
}
