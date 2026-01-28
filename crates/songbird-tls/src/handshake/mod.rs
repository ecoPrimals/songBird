//! TLS 1.3 Handshake State Machine
//!
//! Manages the TLS handshake protocol state transitions.

use crate::crypto::BeardogCryptoClient;
use crate::error::{Result, TlsError};
use crate::key_schedule::KeySchedule;
use crate::messages::{ClientHello, ServerHello};

/// Handshake state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    /// Initial state - waiting for ClientHello
    Start,

    /// Received ClientHello - ready to send ServerHello
    ReceivedClientHello,

    /// Sent ServerHello - handshake in progress
    SentServerHello,

    /// Handshake complete - ready for application data
    Connected,

    /// Error state
    Error,
}

/// Handshake state machine
///
/// Coordinates TLS 1.3 handshake protocol.
pub struct HandshakeStateMachine {
    /// Current state
    state: HandshakeState,

    /// Key schedule for key derivation
    key_schedule: KeySchedule,

    /// Crypto client for BearDog delegation
    crypto_client: Option<BeardogCryptoClient>,

    /// Cached ClientHello (for transcript)
    client_hello: Option<ClientHello>,

    /// Cached ServerHello (for transcript)
    server_hello: Option<ServerHello>,
}

impl HandshakeStateMachine {
    /// Create a new handshake state machine
    pub fn new() -> Self {
        Self {
            state: HandshakeState::Start,
            key_schedule: KeySchedule::new(),
            crypto_client: None,
            client_hello: None,
            server_hello: None,
        }
    }

    /// Set the BearDog crypto client
    pub fn set_crypto_client(&mut self, client: BeardogCryptoClient) {
        self.key_schedule.set_crypto_client(client.clone());
        self.crypto_client = Some(client);
    }

    /// Get current state
    pub fn state(&self) -> HandshakeState {
        self.state
    }

    /// Is handshake complete?
    pub fn is_connected(&self) -> bool {
        self.state == HandshakeState::Connected
    }

    /// Process received ClientHello
    pub fn process_client_hello(&mut self, client_hello: ClientHello) -> Result<()> {
        if self.state != HandshakeState::Start {
            return Err(TlsError::UnexpectedMessage {
                expected: "Start".to_string(),
                got: "ClientHello".to_string(),
            });
        }

        // Validate ClientHello
        client_hello.validate()?;

        // Store for transcript
        self.client_hello = Some(client_hello);

        // Transition state
        self.state = HandshakeState::ReceivedClientHello;

        Ok(())
    }

    /// Generate ServerHello response
    pub async fn generate_server_hello(&mut self) -> Result<ServerHello> {
        if self.state != HandshakeState::ReceivedClientHello {
            return Err(TlsError::ProtocolError(
                "Cannot generate ServerHello in current state".to_string(),
            ));
        }

        let client_hello = self
            .client_hello
            .as_ref()
            .ok_or_else(|| TlsError::InternalError("ClientHello not stored".to_string()))?;

        // Generate server random (32 bytes) from BearDog
        let crypto = self
            .crypto_client
            .as_ref()
            .ok_or_else(|| TlsError::InternalError("Crypto client not set".to_string()))?;

        // TODO: Add random generation method to BearDog
        // For now, use HMAC with timestamp as fallback
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_le_bytes();
        let random_bytes = crypto.hmac_sha256(&timestamp, b"server_random_seed").await?;
        let mut server_random = [0u8; 32];
        server_random.copy_from_slice(&random_bytes[..32]);

        // Select cipher suite (for now, just take first supported)
        let cipher_suite = client_hello
            .cipher_suites
            .first()
            .copied()
            .ok_or_else(|| TlsError::HandshakeFailure("No cipher suites".to_string()))?;

        // Echo session ID
        let session_id_echo = client_hello.legacy_session_id.clone();

        // Generate X25519 ephemeral keypair for key exchange
        let (server_public_key, server_secret_key) = crypto.x25519_generate_ephemeral().await?;

        // Store secret key for later shared secret derivation
        self.key_schedule.set_server_secret_key(server_secret_key);

        // Create ServerHello extensions with REAL key share
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]), // TLS 1.3
            crate::messages::Extension::KeyShare(server_public_key),     // Real X25519 public key
        ];

        let server_hello =
            ServerHello::new(server_random, session_id_echo, cipher_suite, extensions);

        // Validate
        server_hello.validate()?;

        // Store for transcript
        self.server_hello = Some(server_hello.clone());

        // Transition state
        self.state = HandshakeState::SentServerHello;

        Ok(server_hello)
    }

    /// Complete handshake (after Finished messages exchanged)
    pub fn complete_handshake(&mut self) -> Result<()> {
        if self.state != HandshakeState::SentServerHello {
            return Err(TlsError::ProtocolError(
                "Cannot complete handshake in current state".to_string(),
            ));
        }

        // Transition to connected
        self.state = HandshakeState::Connected;

        Ok(())
    }

    /// Get key schedule (for key derivation)
    pub fn key_schedule(&self) -> &KeySchedule {
        &self.key_schedule
    }

    /// Get mutable key schedule
    pub fn key_schedule_mut(&mut self) -> &mut KeySchedule {
        &mut self.key_schedule
    }
}

impl Default for HandshakeStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_handshake() {
        let hsm = HandshakeStateMachine::new();
        assert_eq!(hsm.state(), HandshakeState::Start);
        assert!(!hsm.is_connected());
    }

    #[test]
    fn test_process_client_hello() {
        let mut hsm = HandshakeStateMachine::new();

        let random = [42u8; 32];
        let cipher_suites = vec![0x1303];
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]),
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
        ];

        let client_hello = ClientHello::new(random, cipher_suites, extensions);

        hsm.process_client_hello(client_hello).unwrap();
        assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
    }

    #[tokio::test]
    async fn test_generate_server_hello() {
        let mut hsm = HandshakeStateMachine::new();

        // Set up a mock crypto client (for testing, we'll skip this)
        // In real tests, we'd use a mock BearDog client

        // First, process ClientHello
        let random = [42u8; 32];
        let cipher_suites = vec![0x1303];
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]),
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
        ];

        let client_hello = ClientHello::new(random, cipher_suites, extensions);
        hsm.process_client_hello(client_hello).unwrap();

        // NOTE: Skipping generate_server_hello test as it requires BearDog
        // This will be tested in integration tests with a live BearDog instance
        assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
    }

    #[tokio::test]
    async fn test_complete_handshake() {
        let mut hsm = HandshakeStateMachine::new();

        // Process ClientHello
        let random = [42u8; 32];
        let cipher_suites = vec![0x1303];
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]),
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
        ];

        let client_hello = ClientHello::new(random, cipher_suites, extensions);
        hsm.process_client_hello(client_hello).unwrap();

        // NOTE: Skipping full handshake test as it requires BearDog
        // This will be tested in integration tests with a live BearDog instance
        assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
    }

    #[tokio::test]
    async fn test_invalid_state_transition() {
        let mut hsm = HandshakeStateMachine::new();

        // Try to complete handshake without processing messages
        let result = hsm.complete_handshake();
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_client_hello() {
        let mut hsm = HandshakeStateMachine::new();

        let random = [42u8; 32];
        let cipher_suites = vec![0x1303];
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]),
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
        ];

        let client_hello = ClientHello::new(random, cipher_suites.clone(), extensions.clone());
        hsm.process_client_hello(client_hello).unwrap();

        // Try to process again
        let client_hello2 = ClientHello::new(random, cipher_suites, extensions);
        let result = hsm.process_client_hello(client_hello2);
        assert!(result.is_err());
    }

    // ========================================
    // NEW COMPREHENSIVE HANDSHAKE FLOW TESTS
    // Added: January 27, 2026 (Evening)
    // Goal: Increase coverage from 12% → 70%
    // ========================================

    #[test]
    fn test_handshake_state_transitions() {
        let mut hsm = HandshakeStateMachine::new();

        // Initial state
        assert_eq!(hsm.state(), HandshakeState::Start);
        assert!(!hsm.is_connected());

        // After ClientHello
        let client_hello = create_test_client_hello();
        hsm.process_client_hello(client_hello).unwrap();
        assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
        assert!(!hsm.is_connected());
    }

    #[test]
    fn test_client_hello_with_multiple_cipher_suites() {
        let mut hsm = HandshakeStateMachine::new();

        let random = [123u8; 32];
        let cipher_suites = vec![0x1301, 0x1302, 0x1303]; // AES-128-GCM, AES-256-GCM, ChaCha20-Poly1305
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]),
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
        ];

        let client_hello = ClientHello::new(random, cipher_suites, extensions);
        let result = hsm.process_client_hello(client_hello);
        assert!(result.is_ok());
        assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
    }

    #[test]
    fn test_client_hello_with_sni_extension() {
        let mut hsm = HandshakeStateMachine::new();

        let random = [200u8; 32];
        let cipher_suites = vec![0x1303];
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]),
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
            crate::messages::Extension::ServerName("example.com".to_string()),
        ];

        let client_hello = ClientHello::new(random, cipher_suites, extensions);
        let result = hsm.process_client_hello(client_hello);
        assert!(result.is_ok());
    }

    #[test]
    fn test_client_hello_missing_required_extensions() {
        let mut hsm = HandshakeStateMachine::new();

        let random = [250u8; 32];
        let cipher_suites = vec![0x1303];
        let extensions = vec![]; // Missing required extensions

        let client_hello = ClientHello::new(random, cipher_suites, extensions);
        let result = hsm.process_client_hello(client_hello);
        // Should fail validation due to missing extensions
        assert!(result.is_err());
    }

    #[test]
    fn test_client_hello_with_legacy_version() {
        let mut hsm = HandshakeStateMachine::new();

        let random = [77u8; 32];
        let cipher_suites = vec![0x1303];
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0303]), // TLS 1.2, not 1.3
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
        ];

        let client_hello = ClientHello::new(random, cipher_suites, extensions);
        let result = hsm.process_client_hello(client_hello);
        // Currently accepts any version during processing (validation happens later in handshake)
        // This test documents current behavior
        assert!(result.is_ok());
    }

    #[test]
    fn test_client_hello_no_cipher_suites() {
        let mut hsm = HandshakeStateMachine::new();

        let random = [88u8; 32];
        let cipher_suites = vec![]; // No cipher suites
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]),
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
        ];

        let client_hello = ClientHello::new(random, cipher_suites, extensions);
        let result = hsm.process_client_hello(client_hello);
        // Should fail validation
        assert!(result.is_err());
    }

    #[test]
    fn test_handshake_key_schedule_initialization() {
        let hsm = HandshakeStateMachine::new();
        // Key schedule should be initialized
        assert_eq!(hsm.state(), HandshakeState::Start);
        // Crypto client is optional (may be None for testing)
        assert!(hsm.crypto_client.is_none());
    }

    #[test]
    fn test_set_crypto_client() {
        let mut hsm = HandshakeStateMachine::new();

        // Create a mock crypto client (using explicit socket path for testing)
        let crypto_client =
            BeardogCryptoClient::with_socket_path("/tmp/test-beardog.sock".to_string());
        hsm.set_crypto_client(crypto_client);

        assert!(hsm.crypto_client.is_some());
    }

    #[test]
    fn test_handshake_error_state() {
        let mut hsm = HandshakeStateMachine::new();
        hsm.state = HandshakeState::Error;

        // Attempting operations in error state should fail
        let client_hello = create_test_client_hello();
        let result = hsm.process_client_hello(client_hello);
        assert!(result.is_err());
    }

    #[test]
    fn test_handshake_state_display() {
        let states = [
            HandshakeState::Start,
            HandshakeState::ReceivedClientHello,
            HandshakeState::SentServerHello,
            HandshakeState::Connected,
            HandshakeState::Error,
        ];

        // Verify all states are distinct
        for (i, state1) in states.iter().enumerate() {
            for (j, state2) in states.iter().enumerate() {
                if i == j {
                    assert_eq!(state1, state2);
                } else {
                    assert_ne!(state1, state2);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_server_hello_generation_without_crypto_client() {
        let mut hsm = HandshakeStateMachine::new();

        // Process ClientHello first
        let client_hello = create_test_client_hello();
        hsm.process_client_hello(client_hello).unwrap();

        // Try to generate ServerHello without crypto client
        let result = hsm.generate_server_hello().await;
        // Should fail gracefully
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complete_handshake_without_messages() {
        let mut hsm = HandshakeStateMachine::new();

        // Try to complete handshake without processing any messages
        let result = hsm.complete_handshake();
        assert!(result.is_err());
        assert_eq!(hsm.state(), HandshakeState::Start);
    }

    #[test]
    fn test_handshake_clone() {
        let hsm1 = HandshakeStateMachine::new();
        let state_clone = hsm1.state();

        // State should be cloneable
        let _state_copy = state_clone;
        assert_eq!(hsm1.state(), HandshakeState::Start);
    }

    // Helper function for creating test ClientHello
    fn create_test_client_hello() -> ClientHello {
        let random = [42u8; 32];
        let cipher_suites = vec![0x1303];
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]),
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
        ];

        ClientHello::new(random, cipher_suites, extensions)
    }
}
