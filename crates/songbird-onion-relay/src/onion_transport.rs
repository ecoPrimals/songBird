// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Onion Transport - Sovereign Onion Service Integration
//!
//! This module integrates the Songbird Sovereign Onion Service with the onion relay.
//! It delegates to `songbird-sovereign-onion` for all connection and crypto operations.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │  Onion Relay Coordinator                            │
//! │  ├─ Signaling via Sovereign Onion                   │
//! │  ├─ STUN for address discovery                      │
//! │  └─ Direct UDP for data transfer                    │
//! └─────────────┬───────────────────────────────────────┘
//!               │
//!     ┌─────────▼─────────────────┐
//!     │ Sovereign Onion Service    │
//!     │ (Complete Implementation)  │
//!     │ ├─ .onion addresses       │
//!     │ ├─ TCP listener (Phase 3) │
//!     │ ├─ Connector (Phase 4)    │
//!     │ ├─ BearDog crypto          │
//!     │ └─ Sled persistence        │
//!     └────────────────────────────┘
//! ```
//!
//! ## Deep Debt Compliance
//!
//! - ✅ Zero `unimplemented!()` — all stubs evolved to real implementations
//! - ✅ Delegates to `songbird-sovereign-onion` (no duplicated logic)
//! - ✅ BearDog crypto delegation (TRUE PRIMAL)
//! - ✅ Persistent identity via Sled storage

use anyhow::{Context, Result};
use songbird_sovereign_onion::OnionStorage;
use std::path::Path;
use tracing::info;

/// Onion transport for NAT traversal signaling
///
/// Provides identity management and delegates connection operations
/// to `songbird-sovereign-onion::OnionService` and `OnionConnector`.
///
/// ## Usage
///
/// ```rust,no_run
/// use songbird_onion_relay::onion_transport::OnionTransport;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let transport = OnionTransport::new("./data/onion")?;
///     let address = transport.onion_address();
///     println!("Our address: {}", address);
///     Ok(())
/// }
/// ```
pub struct OnionTransport {
    /// Our .onion address (cached)
    onion_address: String,

    /// Ed25519 verifying key bytes (cached)
    verifying_key_bytes: Vec<u8>,

    /// Storage for identity and peer information
    storage: OnionStorage,
}

impl OnionTransport {
    /// Create a new onion transport with persistent storage
    ///
    /// Loads an existing identity from storage, or generates a new one.
    ///
    /// # Arguments
    ///
    /// * `storage_path` - Path to Sled database for persistent storage
    ///
    /// # Errors
    ///
    /// Returns error if storage initialization or identity generation fails.
    pub fn new<P: AsRef<Path>>(storage_path: P) -> Result<Self> {
        info!("Initializing Sovereign Onion Transport...");

        let storage = OnionStorage::open(storage_path.as_ref())
            .context("Failed to initialize onion storage")?;

        let identity = storage
            .load_or_generate_identity()
            .context("Failed to load/generate onion identity")?;

        let onion_address = identity.onion_address().to_string();
        let verifying_key_bytes = identity.public_key_bytes().to_vec();

        info!("Onion address: {}", onion_address);

        Ok(Self {
            onion_address,
            verifying_key_bytes,
            storage,
        })
    }

    /// Get our .onion address (Tor v3 format)
    ///
    /// This address can be shared with peers for them to connect to us.
    /// Format: `<56 chars>.onion`
    #[must_use]
    pub fn onion_address(&self) -> &str {
        &self.onion_address
    }

    /// Get our Ed25519 verifying (public) key
    ///
    /// This is the cryptographic identity behind the .onion address.
    #[must_use]
    pub fn verifying_key_bytes(&self) -> &[u8] {
        &self.verifying_key_bytes
    }

    /// Get a reference to the underlying storage
    ///
    /// Allows callers to access peer storage and identity management
    /// directly when needed for relay coordination.
    #[must_use]
    pub const fn storage(&self) -> &OnionStorage {
        &self.storage
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_onion_transport() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path();

        let transport = OnionTransport::new(storage_path);

        if let Ok(transport) = transport {
            let address = transport.onion_address();

            // Verify .onion address format
            assert!(
                std::path::Path::new(address)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("onion"))
            );
            assert_eq!(address.len(), 62); // 56 chars + ".onion"
        }
    }

    #[test]
    fn test_persistent_identity() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path();

        let transport1 = OnionTransport::new(storage_path);
        if let Ok(transport1) = transport1 {
            let address1 = transport1.onion_address().to_string();
            let key1 = transport1.verifying_key_bytes().to_vec();
            drop(transport1);

            // Second transport should load same identity
            let transport2 = OnionTransport::new(storage_path).unwrap();
            let address2 = transport2.onion_address();
            let key2 = transport2.verifying_key_bytes();

            assert_eq!(address1, address2);
            assert_eq!(key1, key2);
        }
    }

    #[test]
    fn test_storage_accessible() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path();

        if let Ok(transport) = OnionTransport::new(storage_path) {
            let _storage = transport.storage();
            // Storage is accessible for relay coordination
        }
    }

    #[test]
    fn verifying_key_length_matches_ed25519_public() {
        let temp_dir = TempDir::new().unwrap();
        let t = OnionTransport::new(temp_dir.path()).unwrap();
        assert_eq!(t.verifying_key_bytes().len(), 32);
    }

    #[test]
    fn onion_address_suffix_and_accessor() {
        let temp_dir = TempDir::new().unwrap();
        let t = OnionTransport::new(temp_dir.path()).unwrap();
        let addr = t.onion_address();
        assert!(
            std::path::Path::new(addr)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("onion"))
        );
        assert_eq!(addr, t.onion_address());
    }
}
