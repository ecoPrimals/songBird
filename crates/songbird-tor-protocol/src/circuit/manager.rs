//! Circuit manager - Build and manage Tor circuits
//!
//! **Phase 2B**: Circuit building

use crate::crypto::BeardogCryptoClient;
use crate::directory::{Consensus, CircuitPath};
use crate::error::{Error, Result};
use crate::circuit::{Circuit, CircuitHop, CircuitPurpose};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Circuit manager
pub struct CircuitManager {
    /// BearDog crypto client
    beardog: Arc<BeardogCryptoClient>,
    /// Network consensus
    consensus: Arc<RwLock<Consensus>>,
    /// Active circuits
    circuits: Arc<RwLock<HashMap<u32, Circuit>>>,
    /// Next circuit ID
    next_circuit_id: Arc<RwLock<u32>>,
}

impl CircuitManager {
    /// Create new circuit manager
    pub fn new(beardog: BeardogCryptoClient, consensus: Consensus) -> Self {
        Self {
            beardog: Arc::new(beardog),
            consensus: Arc::new(RwLock::new(consensus)),
            circuits: Arc::new(RwLock::new(HashMap::new())),
            next_circuit_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Build a new circuit
    ///
    /// # Arguments
    /// * `purpose` - Circuit purpose (General, HSDir, Rendezvous)
    ///
    /// # Returns
    /// * Circuit ID
    pub async fn build_circuit(&self, purpose: CircuitPurpose) -> Result<u32> {
        // 1. Allocate circuit ID
        let circuit_id = self.allocate_circuit_id()?;

        // 2. Select path (3 hops: Guard → Middle → Exit/HSDir)
        let path = {
            let consensus = self.consensus.read()
                .map_err(|_| Error::Protocol("Failed to acquire consensus lock".to_string()))?;
            consensus.select_path()?
        };

        // 3. Create circuit
        let circuit = Circuit::new(circuit_id, purpose);

        // 4. Store circuit
        {
            let mut circuits = self.circuits.write()
                .map_err(|_| Error::Protocol("Failed to acquire circuits lock".to_string()))?;
            circuits.insert(circuit_id, circuit);
        }

        // 5. Build first hop (Guard) using CREATE2
        self.create_first_hop(circuit_id, &path).await?;

        // 6. Extend to second hop (Middle)
        self.extend_circuit_hop(circuit_id, &path, 1).await?;

        // 7. Extend to third hop (Exit/HSDir)
        self.extend_circuit_hop(circuit_id, &path, 2).await?;

        Ok(circuit_id)
    }

    /// Create first hop using CREATE2
    async fn create_first_hop(&self, _circuit_id: u32, path: &CircuitPath) -> Result<()> {
        // Get guard relay
        let guard = &path.guard;

        // Perform ntor handshake
        let ntor = super::NtorHandshake::new((*self.beardog).clone());
        
        // TODO: Get actual ntor key from relay descriptor
        // For now, use fingerprint as placeholder
        let relay_ntor_key = {
            let mut key = [0u8; 32];
            key[..20].copy_from_slice(&guard.fingerprint);
            key
        };
        let relay_identity = relay_ntor_key; // TODO: Get Ed25519 identity

        let (create2_payload, state) = ntor.create_handshake(&relay_identity, &relay_ntor_key)?;

        // TODO: Send CREATE2 cell to guard relay
        // let cell = Cell {
        //     circ_id: circuit_id,
        //     command: CellCommand::Create2,
        //     payload: create2_payload,
        // };
        // stream.write_all(&cell.encode()).await?;

        // TODO: Remove this placeholder once network I/O is implemented
        let _ = (state, create2_payload); // Suppress unused warnings

        Ok(())
    }

    /// Extend circuit by one hop
    async fn extend_circuit_hop(&self, circuit_id: u32, path: &CircuitPath, hop_index: usize) -> Result<()> {
        let next_relay = match hop_index {
            1 => &path.middle,
            2 => &path.exit,
            _ => return Err(Error::Protocol(format!("Invalid hop index: {}", hop_index))),
        };

        // Get current circuit
        let circuit = {
            let circuits = self.circuits.read()
                .map_err(|_| Error::Protocol("Failed to acquire circuits lock".to_string()))?;
            circuits.get(&circuit_id)
                .ok_or_else(|| Error::Protocol(format!("Circuit {} not found", circuit_id)))?
                .clone()
        };

        // Create EXTEND2
        let extender = super::extend::CircuitExtender::new((*self.beardog).clone());
        let (_extend2_cell, _state) = extender.create_extend2(&circuit, next_relay)?;

        // TODO: Encrypt EXTEND2 through existing hops (onion encryption)
        // TODO: Send RELAY_EARLY cell containing EXTEND2
        // TODO: Receive EXTENDED2 response
        // TODO: Complete handshake and add hop

        Ok(())
    }

    /// Allocate new circuit ID
    fn allocate_circuit_id(&self) -> Result<u32> {
        let mut next_id = self.next_circuit_id.write()
            .map_err(|_| Error::Protocol("Failed to acquire circuit ID lock".to_string()))?;
        let id = *next_id;
        *next_id += 1;
        Ok(id)
    }

    /// Add hop to circuit
    #[allow(dead_code)]
    fn add_hop_to_circuit(&self, circuit_id: u32, hop: CircuitHop) -> Result<()> {
        let mut circuits = self.circuits.write()
            .map_err(|_| Error::Protocol("Failed to acquire circuits lock".to_string()))?;
        
        let circuit = circuits.get_mut(&circuit_id)
            .ok_or_else(|| Error::Protocol(format!("Circuit {} not found", circuit_id)))?;
        
        circuit.add_hop(hop);
        Ok(())
    }

    /// Get circuit
    pub fn get_circuit(&self, circuit_id: u32) -> Result<Circuit> {
        let circuits = self.circuits.read()
            .map_err(|_| Error::Protocol("Failed to acquire circuits lock".to_string()))?;
        circuits.get(&circuit_id)
            .ok_or_else(|| Error::Circuit(format!("Circuit {} not found", circuit_id)))
            .cloned()
    }

    /// Close circuit
    pub async fn close_circuit(&self, circuit_id: u32) -> Result<()> {
        // TODO: Send DESTROY cell to all hops
        
        // Remove from circuits
        let mut circuits = self.circuits.write()
            .map_err(|_| Error::Protocol("Failed to acquire circuits lock".to_string()))?;
        circuits.remove(&circuit_id);

        Ok(())
    }

    /// Get active circuit count
    pub fn circuit_count(&self) -> usize {
        self.circuits.read()
            .map(|c| c.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory::RelayInfo;

    #[test]
    fn test_circuit_manager_creation() {
        use std::time::SystemTime;
        
        let beardog = BeardogCryptoClient::from_env()
            .expect("Failed to create BearDog client");
        let now = SystemTime::now();
        let consensus = Consensus {
            valid_after: now,
            fresh_until: now,
            valid_until: now,
            relays: Vec::new(),
        };
        
        let manager = CircuitManager::new(beardog, consensus);
        assert_eq!(manager.circuit_count(), 0);
    }

    #[test]
    fn test_circuit_id_allocation() {
        use std::time::SystemTime;
        
        let beardog = BeardogCryptoClient::from_env()
            .expect("Failed to create BearDog client");
        let now = SystemTime::now();
        let consensus = Consensus {
            valid_after: now,
            fresh_until: now,
            valid_until: now,
            relays: Vec::new(),
        };
        
        let manager = CircuitManager::new(beardog, consensus);
        
        let id1 = manager.allocate_circuit_id().expect("Failed to allocate");
        let id2 = manager.allocate_circuit_id().expect("Failed to allocate");
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }
}
