//! TLS 1.3 Handshake State Machine
//!
//! Manages the TLS handshake protocol state transitions.

use crate::error::{Result, TlsError};
use crate::messages::{ClientHello, ServerHello};
use crate::key_schedule::KeySchedule;
use crate::crypto::BeardogCryptoClient;

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
    pub fn generate_server_hello(&mut self) -> Result<ServerHello> {
        if self.state != HandshakeState::ReceivedClientHello {
            return Err(TlsError::ProtocolError(
                "Cannot generate ServerHello in current state".to_string(),
            ));
        }

        let client_hello = self.client_hello.as_ref()
            .ok_or_else(|| TlsError::InternalError("ClientHello not stored".to_string()))?;

        // Generate server random (32 bytes)
        // In production, this would use secure random from BearDog
        let server_random = [99u8; 32]; // Placeholder

        // Select cipher suite (for now, just take first supported)
        let cipher_suite = client_hello.cipher_suites.get(0)
            .copied()
            .ok_or_else(|| TlsError::HandshakeFailure("No cipher suites".to_string()))?;

        // Echo session ID
        let session_id_echo = client_hello.legacy_session_id.clone();

        // Create ServerHello extensions
        // In production, this would include KeyShare with X25519 public key from BearDog
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]), // TLS 1.3
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]), // Placeholder key share
        ];

        let server_hello = ServerHello::new(
            server_random,
            session_id_echo,
            cipher_suite,
            extensions,
        );

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

    #[test]
    fn test_generate_server_hello() {
        let mut hsm = HandshakeStateMachine::new();
        
        // First, process ClientHello
        let random = [42u8; 32];
        let cipher_suites = vec![0x1303];
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]),
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
        ];
        
        let client_hello = ClientHello::new(random, cipher_suites, extensions);
        hsm.process_client_hello(client_hello).unwrap();
        
        // Generate ServerHello
        let server_hello = hsm.generate_server_hello().unwrap();
        assert_eq!(hsm.state(), HandshakeState::SentServerHello);
        assert_eq!(server_hello.cipher_suite, 0x1303);
    }

    #[test]
    fn test_complete_handshake() {
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
        
        // Generate ServerHello
        hsm.generate_server_hello().unwrap();
        
        // Complete handshake
        hsm.complete_handshake().unwrap();
        assert_eq!(hsm.state(), HandshakeState::Connected);
        assert!(hsm.is_connected());
    }

    #[test]
    fn test_invalid_state_transition() {
        let mut hsm = HandshakeStateMachine::new();
        
        // Try to generate ServerHello without ClientHello
        let result = hsm.generate_server_hello();
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
}
