//! Introduction point protocol
//!
//! **Phase 2D**: Onion Service

use crate::protocol::RelayCell;

/// Introduction point
#[derive(Debug, Clone)]
pub struct IntroductionPoint {
    /// Relay identity (Ed25519)
    pub relay_identity: [u8; 32],
    
    /// Relay onion key (X25519)
    pub onion_key: [u8; 32],
    
    /// Service-side introduction auth key
    pub service_key: [u8; 32],
    
    /// Circuit ID to this intro point
    pub circuit_id: u32,
}

impl IntroductionPoint {
    /// Create ESTABLISH_INTRO cell
    ///
    /// Sent by service to introduction point to establish it as an intro point.
    pub fn create_establish_intro(&self) -> RelayCell {
        // TODO: Implement ESTABLISH_INTRO cell format
        // - Auth key type
        // - Auth key
        // - Extensions
        // - Handshake auth (MAC)
        
        RelayCell {
            command: crate::protocol::RelayCommand::Introduce1, // Placeholder
            recognized: 0,
            stream_id: 0,
            digest: [0u8; 4],
            length: 0,
            data: Vec::new(),
        }
    }

    /// Parse INTRODUCE2 cell
    ///
    /// Sent from intro point when a client wants to connect.
    /// Contains rendezvous point info and encrypted data for service.
    pub fn parse_introduce2(_cell: &RelayCell) -> crate::error::Result<IntroductionRequest> {
        // TODO: Parse INTRODUCE2 cell
        // - Onion key
        // - Rendezvous point
        // - Rendezvous cookie
        // - Client public key (for ntor)
        
        Ok(IntroductionRequest {
            rendezvous_point: [0u8; 32],
            rendezvous_cookie: [0u8; 20],
            client_public_key: [0u8; 32],
        })
    }
}

/// Introduction request from client
#[derive(Debug, Clone)]
pub struct IntroductionRequest {
    /// Rendezvous point relay identity
    pub rendezvous_point: [u8; 32],
    
    /// Rendezvous cookie (chosen by client)
    pub rendezvous_cookie: [u8; 20],
    
    /// Client's ephemeral public key (for ntor)
    pub client_public_key: [u8; 32],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_introduction_point_creation() {
        let intro = IntroductionPoint {
            relay_identity: [1u8; 32],
            onion_key: [2u8; 32],
            service_key: [3u8; 32],
            circuit_id: 42,
        };

        assert_eq!(intro.circuit_id, 42);
        assert_eq!(intro.relay_identity[0], 1);
    }

    #[test]
    fn test_establish_intro_cell() {
        let intro = IntroductionPoint {
            relay_identity: [1u8; 32],
            onion_key: [2u8; 32],
            service_key: [3u8; 32],
            circuit_id: 42,
        };

        let cell = intro.create_establish_intro();
        // Placeholder returns empty data for now
        assert_eq!(cell.stream_id, 0);
    }

    #[test]
    fn test_introduction_request() {
        let request = IntroductionRequest {
            rendezvous_point: [1u8; 32],
            rendezvous_cookie: [2u8; 20],
            client_public_key: [3u8; 32],
        };

        assert_eq!(request.rendezvous_cookie[0], 2);
        assert_eq!(request.client_public_key.len(), 32);
    }
}
