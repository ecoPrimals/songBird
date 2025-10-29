// Test fixtures and data structures
///
/// Provides standardized test fixtures, mock data, and common
/// test utilities for Songbird tests.
use songbird_types::{errors::SongbirdResult, SongbirdError};
use std::net::SocketAddr;

/// Create a test socket address with a random port
///
/// Note: This function is preserved for backward compatibility with existing tests.
/// New tests should consider using the constants from
/// `songbird_config::constants::testing` for consistency across the ecosystem.
///
/// # Errors
///
/// Returns an error if the socket address parsing fails (should never occur with `127.0.0.1:0`).
pub fn test_socket_addr() -> SongbirdResult<SocketAddr> {
    let port = 0; // OS will assign available port
    format!("127.0.0.1:{port}").parse().map_err(|e| {
        SongbirdError::configuration(format!(
            "Failed to parse test socket address - this should never happen with 127.0.0.1:0 : {e}"
        ))
    })
}

/// Create a test socket address with a specific port
///
/// # Errors
///
/// Returns an error if the socket address parsing fails (should never occur with valid port).
pub fn test_socket_addr_with_port(port: u16) -> SongbirdResult<SocketAddr> {
    format!("127.0.0.1:{port}").parse().map_err(|e| {
        SongbirdError::configuration(format!(
            "Failed to parse test socket address - this should never happen with valid port : {e}"
        ))
    })
}

/// Create a temporary directory for testing
///
/// # Errors
///
/// Returns an error if the temporary directory creation fails.
pub fn create_test_temp_dir() -> SongbirdResult<tempfile::TempDir> {
    tempfile::tempdir()
        .map_err(|e| SongbirdError::service("test", format!("Failed to create temp dir: {e}")))
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
    #[must_use]
    pub fn new(id: String, address: SocketAddr) -> Self {
        Self {
            id,
            address,
            capabilities: vec!["mock".to_string()],
        }
    }

    /// Add a capability to this peer
    #[must_use]
    pub fn with_capability(mut self, capability: String) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Get peer capabilities
    #[must_use]
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
    #[must_use]
    pub fn new(from: String, to: String, payload: Vec<u8>) -> Self {
        Self {
            from,
            to,
            payload,
        }
    }

    /// Create a text message
    #[must_use]
    pub fn text(from: String, to: String, text: String) -> Self {
        Self::new(from, to, text.into_bytes())
    }

    /// Get message payload as string
    ///
    /// # Errors
    ///
    /// Returns an error if the payload is not valid UTF-8.
    pub fn as_text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.payload.clone())
    }

    /// Convert payload to string
    ///
    /// # Errors
    ///
    /// Returns an error if the payload is not valid UTF-8.
    pub fn payload_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.payload.clone()) // More explicit than clone()
    }

    /// Get payload as a slice to avoid cloning when possible
    #[must_use]
    pub fn payload_slice(&self) -> &[u8] {
        &self.payload
    }
}
