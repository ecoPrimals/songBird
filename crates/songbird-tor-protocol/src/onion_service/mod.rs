// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Onion service protocol - Host .onion addresses
//!
//! **Phase 2D**: Onion Service

mod descriptor;
mod introduction;
mod rendezvous;

pub use descriptor::{OnionServiceDescriptor, OnionServiceKeys};
pub use introduction::IntroductionPoint;
pub use rendezvous::RendezvousPoint;

use crate::circuit::Circuit;
use crate::error::{Error, Result};
use songbird_crypto_provider::CryptoProvider;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Onion service state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    /// Initializing
    Initializing,
    /// Publishing descriptor
    Publishing,
    /// Running and accepting connections
    Running,
    /// Stopped
    Stopped,
}

/// Onion service manager
pub struct OnionServiceManager {
    /// `BearDog` crypto client
    beardog: Arc<CryptoProvider>,
    /// Service keys
    keys: Arc<RwLock<Option<OnionServiceKeys>>>,
    /// Introduction points
    intro_points: Arc<RwLock<Vec<IntroductionPoint>>>,
    /// Active rendezvous circuits
    rendezvous_circuits: Arc<RwLock<HashMap<[u8; 20], Circuit>>>,
    /// Service state
    state: Arc<RwLock<ServiceState>>,
    /// Port to expose (used when full relay mode is implemented)
    port: u16,
}

impl OnionServiceManager {
    /// Get the service port
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Create new onion service manager
    #[must_use]
    pub fn new(beardog: CryptoProvider, port: u16) -> Self {
        Self {
            beardog: Arc::new(beardog),
            keys: Arc::new(RwLock::new(None)),
            intro_points: Arc::new(RwLock::new(Vec::new())),
            rendezvous_circuits: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(ServiceState::Initializing)),
            port,
        }
    }

    /// Initialize service (generate keys, select intro points)
    ///
    /// # Errors
    ///
    /// Returns error if key generation fails.
    pub async fn initialize(&self) -> Result<OnionServiceKeys> {
        // Generate service keys via BearDog
        let keys = OnionServiceKeys::generate(&self.beardog).await?;

        // Store keys
        {
            let mut keys_lock = self
                .keys
                .write()
                .map_err(|_| Error::Protocol("Failed to acquire keys lock".to_string()))?;
            *keys_lock = Some(keys.clone());
        }

        // Update state
        self.set_state(ServiceState::Publishing)?;

        Ok(keys)
    }

    /// Get onion address
    ///
    /// # Errors
    ///
    /// Returns error if lock acquisition fails or service not initialized.
    pub fn onion_address(&self) -> Result<String> {
        let keys = self
            .keys
            .read()
            .map_err(|_| Error::Protocol("Failed to acquire keys lock".to_string()))?;
        Ok(keys
            .as_ref()
            .ok_or_else(|| Error::Protocol("Service not initialized".to_string()))?
            .onion_address
            .clone())
    }

    /// Set up introduction points
    ///
    /// # Errors
    ///
    /// Returns error if lock acquisition fails.
    ///
    /// Selects relays and establishes introduction circuits.
    /// In production, this would:
    /// 1. Select random relays from the consensus
    /// 2. Build 3-hop circuits to each
    /// 3. Send `ESTABLISH_INTRO` cells via BearDog-signed auth
    ///
    /// Currently creates introduction points with generated keys
    /// and prepares `ESTABLISH_INTRO` cells for when circuits are available.
    ///
    /// # Panics
    ///
    /// Panics if `count` does not fit in `u8` indices or circuit IDs do not fit in `u32`.
    pub async fn setup_introduction_points(&self, count: usize) -> Result<()> {
        core::future::ready(()).await;
        let service_keys = {
            let keys = self
                .keys
                .read()
                .map_err(|_| Error::Protocol("Failed to acquire keys lock".to_string()))?;
            keys.clone()
        };

        let mut batch = Vec::with_capacity(count);
        for i in 0..count {
            // Generate introduction point keys
            // In production: BearDog generates unique keys per intro point
            let i_u8 = u8::try_from(i).expect("count fits u8");
            let mut relay_identity = [0u8; 32];
            relay_identity[0] = i_u8;
            // Mix with service identity for uniqueness
            if let Some(ref k) = service_keys {
                for (j, byte) in relay_identity.iter_mut().enumerate().skip(1) {
                    let j_u8 = u8::try_from(j).expect("index fits u8");
                    *byte = k.identity_public[j] ^ i_u8.wrapping_add(j_u8);
                }
            }

            let mut onion_key = [0u8; 32];
            onion_key[0] = i_u8.wrapping_add(0x10);

            let mut service_key = [0u8; 32];
            service_key[0] = i_u8.wrapping_add(0x20);

            let intro = IntroductionPoint {
                relay_identity,
                onion_key,
                service_key,
                circuit_id: u32::try_from(i + 1).expect("circuit id fits u32"),
            };

            // Prepare the ESTABLISH_INTRO cell (ready to send when circuit is built)
            let _establish_cell = intro.create_establish_intro();

            batch.push(intro);
        }

        let mut intro_points = self
            .intro_points
            .write()
            .map_err(|_| Error::Protocol("Failed to acquire intro points lock".to_string()))?;
        intro_points.extend(batch);
        drop(intro_points);

        Ok(())
    }

    /// Publish descriptor to `HSDir`
    ///
    /// # Errors
    ///
    /// Returns an error if locks fail, the service is not initialized, or descriptor construction fails.
    pub async fn publish_descriptor(&self) -> Result<()> {
        let keys_guard = self
            .keys
            .read()
            .map_err(|_| Error::Protocol("Failed to acquire keys lock".to_string()))?;

        let keys = keys_guard
            .as_ref()
            .ok_or_else(|| Error::Protocol("Service not initialized".to_string()))?
            .clone();

        let intro_points = self
            .intro_points
            .read()
            .map_err(|_| Error::Protocol("Failed to acquire intro points lock".to_string()))?
            .clone();

        drop(keys_guard);

        let descriptor = OnionServiceDescriptor::new(&keys, &intro_points, &self.beardog).await?;

        // HSDir upload is not performed; descriptor is built then discarded for this state machine step.
        let _ = descriptor;

        self.set_state(ServiceState::Running)?;

        Ok(())
    }

    /// Handle introduction request (INTRODUCE2 cell received)
    ///
    /// When a client wants to connect:
    /// 1. Parse the INTRODUCE2 cell to extract rendezvous info
    /// 2. Build a circuit to the rendezvous point
    /// 3. Send RENDEZVOUS1 cell with handshake data
    ///
    /// Currently parses the cell and stores the rendezvous circuit.
    /// Circuit building requires relay connections (Phase 3).
    ///
    /// # Errors
    ///
    /// Returns an error if the service state is not running, lock acquisition fails, or the
    /// rendezvous cookie is already registered.
    pub async fn handle_introduction(&self, rendezvous_cookie: &[u8; 20]) -> Result<()> {
        core::future::ready(()).await;
        let state = self.state()?;
        if state != ServiceState::Running {
            return Err(Error::Protocol(format!(
                "Cannot handle introduction: service is {state:?}"
            )));
        }

        // Store the rendezvous cookie for circuit association
        // In production: build circuit to rendezvous point, send RENDEZVOUS1
        let mut circuits = self
            .rendezvous_circuits
            .write()
            .map_err(|_| Error::Protocol("Failed to acquire circuits lock".to_string()))?;

        // Check for duplicate cookie
        if circuits.contains_key(rendezvous_cookie) {
            return Err(Error::Protocol("Duplicate rendezvous cookie".to_string()));
        }

        // Create a placeholder circuit for this rendezvous
        // In production: this would be a real circuit built to the rendezvous point
        let circuit_id = u32::from(rendezvous_cookie[0]) | u32::from(rendezvous_cookie[1]) << 8;
        let circuit = Circuit::new(circuit_id, crate::circuit::CircuitPurpose::Rendezvous);
        circuits.insert(*rendezvous_cookie, circuit);
        drop(circuits);

        Ok(())
    }

    /// Get service state
    ///
    /// # Errors
    ///
    /// Returns an error if the state lock cannot be acquired.
    pub fn state(&self) -> Result<ServiceState> {
        let state = self
            .state
            .read()
            .map_err(|_| Error::Protocol("Failed to acquire state lock".to_string()))?;
        Ok(*state)
    }

    /// Set service state
    fn set_state(&self, new_state: ServiceState) -> Result<()> {
        let mut state = self
            .state
            .write()
            .map_err(|_| Error::Protocol("Failed to acquire state lock".to_string()))?;
        *state = new_state;
        drop(state);
        Ok(())
    }

    /// Stop service and clean up resources
    ///
    /// Performs graceful shutdown:
    /// 1. Set state to Stopped (rejects new introductions)
    /// 2. Clear all rendezvous circuits
    /// 3. Clear introduction points
    ///
    /// # Errors
    ///
    /// Returns an error if a lock cannot be acquired or state cannot be updated.
    pub async fn stop(&self) -> Result<()> {
        core::future::ready(()).await;
        self.set_state(ServiceState::Stopped)?;

        // Clear all rendezvous circuits
        {
            let mut circuits = self
                .rendezvous_circuits
                .write()
                .map_err(|_| Error::Protocol("Failed to acquire circuits lock".to_string()))?;
            let count = circuits.len();
            circuits.clear();
            drop(circuits);
            if count > 0 {
                tracing::info!("Closed {} rendezvous circuits", count);
            }
        }

        // Clear introduction points
        {
            let mut intro_points = self
                .intro_points
                .write()
                .map_err(|_| Error::Protocol("Failed to acquire intro points lock".to_string()))?;
            let count = intro_points.len();
            intro_points.clear();
            drop(intro_points);
            if count > 0 {
                tracing::info!("Closed {} introduction points", count);
            }
        }

        Ok(())
    }

    /// Get introduction point count
    #[must_use]
    pub fn intro_point_count(&self) -> usize {
        self.intro_points.read().map(|ip| ip.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_manager_creation() {
        let beardog = CryptoProvider::from_env();
        let manager = OnionServiceManager::new(beardog, 8080);

        assert_eq!(manager.port(), 8080);
        assert_eq!(manager.intro_point_count(), 0);
    }

    #[test]
    fn test_service_state() {
        let beardog = CryptoProvider::from_env();
        let manager = OnionServiceManager::new(beardog, 8080);

        let state = manager.state().expect("Failed to get state");
        assert_eq!(state, ServiceState::Initializing);
    }

    #[tokio::test]
    async fn test_setup_introduction_points() {
        let beardog = CryptoProvider::from_env();
        let manager = OnionServiceManager::new(beardog, 8080);

        manager.setup_introduction_points(3).await.expect("Failed to setup intro points");

        assert_eq!(manager.intro_point_count(), 3);
    }

    #[test]
    fn test_service_states() {
        assert_eq!(ServiceState::Initializing, ServiceState::Initializing);
        assert_ne!(ServiceState::Initializing, ServiceState::Running);
    }
}
