//! Onion service (listen mode) - STUB for Phase 3
//!
//! ⚠️ **TRUE PRIMAL NOTE**: This STUB uses standalone crypto for testing.
//! Production implementation (Phase 3) will use BearDog delegation.

use crate::error::Result;
use crate::keys::OnionIdentity;

#[cfg(any(test, feature = "standalone"))]
use crate::storage::OnionStorage;

/// Onion service (creates reachable .onion address)
///
/// **Status**: STUB - To be implemented in Phase 3
/// ⚠️ **Requires `standalone` feature** (uses direct crypto for testing)
#[cfg(any(test, feature = "standalone"))]
pub struct OnionService {
    identity: OnionIdentity,
    #[allow(dead_code)] // Will be used in Phase 3
    storage: OnionStorage,
    port: u16,
}

#[cfg(any(test, feature = "standalone"))]
impl OnionService {
    /// Create new onion service
    ///
    /// **Status**: STUB
    pub async fn new(port: u16) -> Result<Self> {
        let storage = OnionStorage::open("./data/sovereign-onion")?;
        let identity = storage.load_or_generate_identity()?;

        tracing::info!(
            onion_address = %identity.onion_address(),
            port = port,
            "Onion service created (STUB)"
        );

        Ok(Self {
            identity,
            storage,
            port,
        })
    }

    /// Get our .onion address
    pub fn onion_address(&self) -> &str {
        self.identity.onion_address()
    }

    /// Get listen port
    pub fn port(&self) -> u16 {
        self.port
    }

    // TODO Phase 3: Implement accept(), handshake(), run()
}
