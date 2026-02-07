//! Tor protocol constants

/// Tor protocol version
pub const TOR_PROTOCOL_VERSION: u16 = 5;

/// Maximum cell payload size
pub const MAX_CELL_PAYLOAD: usize = 507;

/// Maximum relay cell payload size
pub const MAX_RELAY_PAYLOAD: usize = 498;

/// Circuit window size (for SENDME)
pub const CIRCUIT_WINDOW: u16 = 1000;

/// Stream window size (for SENDME)
pub const STREAM_WINDOW: u16 = 500;

/// Default timeout for operations
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
