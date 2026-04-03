// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Circuit manager - Build and manage Tor circuits
//!
//! **Phase 2B**: Circuit building

use crate::circuit::{Circuit, CircuitHop, CircuitPurpose, KeyMaterial};
use crate::connection::TorConnection;
use crate::directory::{CircuitPath, Consensus, RelayInfo};
use crate::error::{Error, Result};
use crate::protocol::{Cell, CellCommand};
use songbird_crypto_provider::CryptoProvider;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

/// Circuit manager
pub struct CircuitManager {
    /// `security provider` crypto client
    beardog: Arc<CryptoProvider>,
    /// Network consensus
    consensus: Arc<RwLock<Consensus>>,
    /// Active circuits
    circuits: Arc<RwLock<HashMap<u32, Circuit>>>,
    /// Active connections (`circuit_id` -> connection)
    connections: Arc<tokio::sync::RwLock<HashMap<u32, TorConnection>>>,
    /// Next circuit ID
    next_circuit_id: Arc<RwLock<u32>>,
}

impl CircuitManager {
    /// Create new circuit manager
    #[must_use]
    pub fn new(beardog: CryptoProvider, consensus: Consensus) -> Self {
        Self {
            beardog: Arc::new(beardog),
            consensus: Arc::new(RwLock::new(consensus)),
            circuits: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            next_circuit_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Build a new circuit
    ///
    /// # Arguments
    /// * `purpose` - Circuit purpose (General, `HSDir`, Rendezvous)
    ///
    /// # Returns
    /// * Circuit ID
    ///
    /// # Errors
    /// Returns error if path selection, connection, or handshake fails.
    pub async fn build_circuit(&self, purpose: CircuitPurpose) -> Result<u32> {
        // 1. Allocate circuit ID
        let circuit_id = self.allocate_circuit_id()?;

        // 2. Select path (3 hops: Guard → Middle → Exit/HSDir)
        let mut path = {
            let consensus = self
                .consensus
                .read()
                .map_err(|_| Error::Protocol("Failed to acquire consensus lock".to_string()))?;
            consensus.select_path()?
        };

        // 2.5 Fetch ntor keys for path relays (not in consensus, need descriptors)
        info!("Fetching ntor keys for path relays...");
        Consensus::fetch_path_ntor_keys(&mut path).await?;

        // 3. Create circuit
        let circuit = Circuit::new(circuit_id, purpose);

        // 4. Store circuit
        {
            let mut circuits = self
                .circuits
                .write()
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
    async fn create_first_hop(&self, circuit_id: u32, path: &CircuitPath) -> Result<()> {
        // Get guard relay
        let guard = &path.guard;
        info!(
            "Creating first hop to guard: {} at {}:{}",
            guard.nickname, guard.address, guard.or_port
        );

        // 1. Connect to the guard relay
        let mut connection = TorConnection::new(guard.clone());
        connection.connect().await?;

        // 2. Generate CREATE2 payload via ntor handshake
        let ntor = super::NtorHandshake::new((*self.beardog).clone());

        // Get ntor key from relay (required for ntor handshake)
        let relay_ntor_key = guard
            .ntor_key
            .ok_or_else(|| Error::Protocol(format!("Guard {} has no ntor_key", guard.nickname)))?;

        // Node ID is the 20-byte fingerprint (SHA1 of RSA identity key)
        let node_id = &guard.fingerprint;

        let (create2_payload, state) = ntor.create_handshake(node_id, &relay_ntor_key).await?;

        // 3. Build CREATE2 cell
        // CREATE2 cell payload format:
        // - HTYPE (2 bytes): handshake type (0x0002 = ntor)
        // - HLEN (2 bytes): handshake data length
        // - HDATA: handshake data
        let mut payload = Vec::new();
        payload.extend_from_slice(&[0x00, 0x02]); // HTYPE = ntor
        payload.extend_from_slice(
            &u16::try_from(create2_payload.len())
                .map_err(|_| Error::Protocol("CREATE2 payload too long".to_string()))?
                .to_be_bytes(),
        ); // HLEN
        payload.extend_from_slice(&create2_payload); // HDATA

        debug!(
            "CREATE2 payload: htype=ntor, hlen={}, hdata_len={}",
            create2_payload.len(),
            create2_payload.len()
        );
        debug!(
            "CREATE2 ntor data: node_id[0..4]={:02x?}, ntor_key[0..4]={:02x?}, client_pk[0..4]={:02x?}",
            &node_id[0..4],
            &relay_ntor_key[0..4],
            &create2_payload[52..56]
        );

        let cell = Cell {
            circ_id: circuit_id,
            command: CellCommand::Create2,
            payload,
        };

        // 4. Send CREATE2
        debug!(
            "Sending CREATE2 cell (circuit_id={}, total_payload={})",
            circuit_id,
            cell.payload.len()
        );
        connection.send_cell(&cell).await?;

        // 5. Receive CREATED2
        debug!("Waiting for CREATED2 response...");
        let response = connection.recv_cell().await?;

        if response.command != CellCommand::Created2 {
            return Err(Error::Protocol(format!("Expected CREATED2, got {:?}", response.command)));
        }

        // 6. Parse CREATED2 payload
        // Format: HLEN (2 bytes) | HDATA
        if response.payload.len() < 2 {
            return Err(Error::Protocol("CREATED2 payload too short".to_string()));
        }
        let hlen = u16::from_be_bytes([response.payload[0], response.payload[1]]) as usize;
        if response.payload.len() < 2 + hlen {
            return Err(Error::Protocol(format!(
                "CREATED2 hdata too short: {} < {}",
                response.payload.len() - 2,
                hlen
            )));
        }
        let handshake_response = &response.payload[2..2 + hlen];

        // 7. Complete handshake
        let key_material = ntor.complete_handshake(&state, handshake_response).await?;
        info!("First hop handshake complete with {}", guard.nickname);

        // 8. Add hop to circuit
        self.add_hop_to_circuit(circuit_id, Self::create_hop(guard.clone(), &key_material))?;

        // 9. Store connection
        self.connections.write().await.insert(circuit_id, connection);

        Ok(())
    }

    /// Create `CircuitHop` from key material
    const fn create_hop(relay: RelayInfo, keys: &KeyMaterial) -> CircuitHop {
        CircuitHop::new(
            relay,
            keys.forward_digest,
            keys.backward_digest,
            keys.forward_key,
            keys.backward_key,
        )
    }

    /// Extend circuit by one hop
    async fn extend_circuit_hop(
        &self,
        circuit_id: u32,
        path: &CircuitPath,
        hop_index: usize,
    ) -> Result<()> {
        let next_relay = match hop_index {
            1 => &path.middle,
            2 => &path.exit,
            _ => return Err(Error::Protocol(format!("Invalid hop index: {hop_index}"))),
        };

        let hop_name = match hop_index {
            1 => "middle",
            2 => "exit",
            _ => "unknown",
        };
        info!(
            "Extending circuit {} to {} ({}): {}:{}",
            circuit_id, hop_name, next_relay.nickname, next_relay.address, next_relay.or_port
        );

        // Get current circuit
        let circuit = {
            let circuits = self
                .circuits
                .read()
                .map_err(|_| Error::Protocol("Failed to acquire circuits lock".to_string()))?;
            circuits
                .get(&circuit_id)
                .ok_or_else(|| Error::Protocol(format!("Circuit {circuit_id} not found")))?
                .clone()
        };

        // Create EXTEND2 relay cell
        let extender = super::extend::CircuitExtender::new((*self.beardog).clone());
        let (extend2_relay_cell, state) = extender.create_extend2(&circuit, next_relay).await?;

        // Encode EXTEND2 as relay cell
        let relay_payload = extend2_relay_cell.encode();

        // Encrypt through existing hops (onion encryption)
        let onion = super::OnionCrypto::new((*self.beardog).clone());
        let encrypted_payload = onion.encrypt_forward(&relay_payload, &circuit.hops).await?;

        // Build RELAY_EARLY cell (used for first 8 hops, required for EXTEND2)
        let relay_early_cell = Cell {
            circ_id: circuit_id,
            command: CellCommand::RelayEarly,
            payload: encrypted_payload,
        };

        // Get connection and send
        {
            let mut connections = self.connections.write().await;
            let connection = connections.get_mut(&circuit_id).ok_or_else(|| {
                Error::Protocol(format!("No connection for circuit {circuit_id}"))
            })?;

            debug!("Sending RELAY_EARLY (EXTEND2) cell");
            connection.send_cell(&relay_early_cell).await?;

            // Receive response
            debug!("Waiting for RELAY (EXTENDED2) response...");
            let response_cell = connection.recv_cell().await?;

            if response_cell.command != CellCommand::Relay {
                return Err(Error::Protocol(format!(
                    "Expected RELAY, got {:?}",
                    response_cell.command
                )));
            }

            // Decrypt through existing hops
            let decrypted_payload =
                onion.decrypt_backward(&response_cell.payload, &circuit.hops).await?;

            // Parse EXTENDED2 relay cell
            // Relay cell format: command (1) | recognized (2) | stream_id (2) | digest (4) | length (2) | data
            if decrypted_payload.len() < 11 {
                return Err(Error::Protocol("EXTENDED2 relay cell too short".to_string()));
            }

            let relay_command = decrypted_payload[0];
            if relay_command != 7 {
                // EXTENDED = 7
                return Err(Error::Protocol(format!(
                    "Expected EXTENDED (7), got relay command {relay_command}"
                )));
            }

            let data_len =
                u16::from_be_bytes([decrypted_payload[9], decrypted_payload[10]]) as usize;
            if decrypted_payload.len() < 11 + data_len {
                return Err(Error::Protocol("EXTENDED2 data too short".to_string()));
            }

            // Create mock relay cell for process_extended2
            let response_relay_cell = crate::protocol::RelayCell {
                command: crate::protocol::RelayCommand::Extended,
                recognized: 0,
                stream_id: 0,
                digest: [0u8; 4],
                length: u16::try_from(data_len)
                    .map_err(|_| Error::Protocol("EXTENDED2 data length overflow".to_string()))?,
                data: decrypted_payload[11..11 + data_len].to_vec(),
            };

            // Complete handshake and create hop
            let hop = extender
                .process_extended2(&circuit, &state, &response_relay_cell, next_relay.clone())
                .await?;
            info!("Extended circuit {} to {}", circuit_id, next_relay.nickname);

            // Add hop to circuit
            drop(connections); // Release async lock before sync lock
            self.add_hop_to_circuit(circuit_id, hop)?;
        }

        Ok(())
    }

    /// Allocate new circuit ID
    ///
    /// For link protocol v4+, the initiator (client) must use circuit IDs
    /// with the MSB (bit 31) set to 1. This distinguishes client-initiated
    /// circuits from server-initiated circuits.
    fn allocate_circuit_id(&self) -> Result<u32> {
        let id = {
            let mut next_id = self
                .next_circuit_id
                .write()
                .map_err(|_| Error::Protocol("Failed to acquire circuit ID lock".to_string()))?;
            // Set MSB to 1 for client-initiated circuits (link v4+ requirement)
            let id = 0x8000_0000 | *next_id;
            *next_id += 1;
            id
        };
        Ok(id)
    }

    /// Add hop to circuit
    fn add_hop_to_circuit(&self, circuit_id: u32, hop: CircuitHop) -> Result<()> {
        self.circuits
            .write()
            .map_err(|_| Error::Protocol("Failed to acquire circuits lock".to_string()))?
            .get_mut(&circuit_id)
            .ok_or_else(|| Error::Protocol(format!("Circuit {circuit_id} not found")))?
            .add_hop(hop);
        Ok(())
    }

    /// Get circuit
    ///
    /// # Errors
    /// Returns error if circuit is not found or lock acquisition fails.
    pub fn get_circuit(&self, circuit_id: u32) -> Result<Circuit> {
        let circuits = self
            .circuits
            .read()
            .map_err(|_| Error::Protocol("Failed to acquire circuits lock".to_string()))?;
        circuits
            .get(&circuit_id)
            .ok_or_else(|| Error::Circuit(format!("Circuit {circuit_id} not found")))
            .cloned()
    }

    /// Close circuit by sending DESTROY cells and cleaning up
    ///
    /// Sends a DESTROY cell to tear down the circuit at the first hop,
    /// which cascades to all subsequent hops per Tor spec.
    ///
    /// Destroy reason codes (Tor spec):
    /// - 0: NONE
    /// - 1: PROTOCOL (protocol violation)
    /// - 3: REQUESTED (clean teardown)
    /// - 5: FINISHED (stream finished)
    ///
    /// # Errors
    /// Returns error if lock acquisition fails.
    pub async fn close_circuit(&self, circuit_id: u32) -> Result<()> {
        // Build DESTROY cell for the circuit
        let destroy_cell = crate::protocol::Cell {
            circ_id: circuit_id,
            command: crate::protocol::CellCommand::Destroy,
            // Destroy reason: REQUESTED (0x03) — clean teardown
            payload: vec![0x03],
        };

        // Attempt to send DESTROY cell via the connection
        {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.get_mut(&circuit_id) {
                if let Err(e) = connection.send_cell(&destroy_cell).await {
                    tracing::warn!(
                        "Failed to send DESTROY cell for circuit {}: {} (cleaning up anyway)",
                        circuit_id,
                        e
                    );
                } else {
                    tracing::debug!("Sent DESTROY cell for circuit {}", circuit_id);
                }
            } else {
                tracing::debug!(
                    "No active connection for circuit {} (already disconnected), cleaning up",
                    circuit_id
                );
            }
            connections.remove(&circuit_id);
        }

        // Remove from circuits
        self.circuits
            .write()
            .map_err(|_| Error::Protocol("Failed to acquire circuits lock".to_string()))?
            .remove(&circuit_id);

        tracing::info!("Circuit {circuit_id} closed");
        Ok(())
    }

    /// Get active circuit count
    #[must_use]
    pub fn circuit_count(&self) -> usize {
        self.circuits.read().map(|c| c.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_manager_creation() {
        use std::time::SystemTime;

        let beardog = CryptoProvider::from_env();
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

        let beardog = CryptoProvider::from_env();
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

        // Client-initiated circuits have MSB set (Tor link protocol v4+)
        assert_eq!(id1, 0x8000_0001);
        assert_eq!(id2, 0x8000_0002);
        // Verify MSB is set (client-initiated)
        assert!(id1 & 0x8000_0000 != 0);
        assert!(id2 & 0x8000_0000 != 0);
    }
}
