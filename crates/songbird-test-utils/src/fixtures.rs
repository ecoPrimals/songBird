// Test fixtures and data structures
///
/// Provides standardized test fixtures, mock data, and common
/// test utilities for use across the Songbird ecosystem.
use songbird_errors::{SongbirdError, Result as SongbirdResult};
use std::net::SocketAddr;

/// Create a test socket address with a random port
/// Note: This function is preserved for backward compatibility with existing tests.
/// New tests should consider using the constants from
/// `songbird_config::constants::testing` for consistency across the ecosystem.
pub fn test_socket_addr() -> SocketAddr {
    let port = 0; // OS will assign available port
    format!("127.0.0.1:{port}")
        .parse()
        .expect("Failed to parse test socket address - this should never happen with 127.0.0.1:0")
}

/// Create a test socket address with a specific port
pub fn test_socket_addr_with_port(port: u16) -> SocketAddr {
    format!("127.0.0.1:{port}")
        .parse()
        .expect("Failed to parse test socket address - this should never happen with valid port")
}

/// Create a temporary directory for testing
pub fn create_test_temp_dir() -> SongbirdResult<tempfile::TempDir> {
    tempfile::tempdir().map_err(|e| {
        SongbirdError::service_error("test", format!("Failed to create temp dir: {e}"))
    })
}

/// Mock peer information for testing
#[derive(Debug, Clone)]
pub struct MockPeer {
    /// Peer identifier
    pub id: String,
    /// Peer address
    pub address: SocketAddr,
    /// Peer capabilities
    pub capabilities: Vec<String>,
}

impl MockPeer {
    /// Create a new mock peer
    pub fn new(id: String, address: SocketAddr) -> Self {
        Self {
            id,
            address,
            capabilities: vec!["mock".to_string()],
        }
    }

    /// Add a capability to this peer
    pub fn with_capability(mut self, capability: String) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Get peer capabilities
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
}

/// Mock network message for testing
#[derive(Debug, Clone)]
pub struct MockMessage {
    /// Message sender
    pub from: String,
    /// Message recipient
    pub to: String,
    /// Message payload
    pub payload: Vec<u8>,
}

impl MockMessage {
    /// Create a new mock message
    pub fn new(from: String, to: String, payload: Vec<u8>) -> Self {
        Self { from, to, payload }
    }

    /// Create a text message
    pub fn text(from: String, to: String, text: String) -> Self {
        Self::new(from, to, text.into_bytes())
    }

    /// Get message payload as string
    pub fn as_text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.payload.clone())
    }

    /// Convert payload to string
    pub fn payload_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.payload.to_vec()) // Use to_vec() for clarity
    }

    /// Get payload as a slice to avoid cloning when possible
    pub fn payload_slice(&self) -> &[u8] {
        &self.payload
    }
}
