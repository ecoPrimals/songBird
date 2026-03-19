//! `SoloKey` hardware key support (FIDO2/WebAuthn)

use crate::error::Result;
use crate::types::{PhysicalChannelType, ProximityProof, TrustLevel};
use async_trait::async_trait;
use chrono::Utc;

use super::PhysicalChannelProvider;

/// `SoloKey` hardware key channel
///
/// **Status:** Placeholder implementation for future Pure Rust FIDO2/WebAuthn support
///
/// **Future Implementation:**
/// - Use Pure Rust FIDO2/WebAuthn library (when available)
/// - Or delegate to `BearDog` for hardware key operations via IPC
/// - Zero C dependencies (no OpenSSL)
#[derive(Debug)]
pub struct SoloKeyChannel {
    // Future: Add Pure Rust FIDO2/WebAuthn integration
}

impl SoloKeyChannel {
    /// Create new `SoloKey` channel
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for SoloKeyChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PhysicalChannelProvider for SoloKeyChannel {
    async fn verify_proximity(&self) -> Result<ProximityProof> {
        // TODO: Implement actual SoloKey/FIDO2 verification
        Ok(ProximityProof {
            channel_type: PhysicalChannelType::HardwareKey,
            timestamp: Utc::now(),
            proof_data: b"solokey_proof".to_vec(),
            attestation: Some(b"hardware_attestation".to_vec()),
        })
    }

    async fn secure_exchange(&self) -> Result<Vec<u8>> {
        // TODO: Implement actual key exchange via SoloKey
        Ok(b"solokey_genesis_creds".to_vec())
    }

    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Maximum
    }

    fn channel_type(&self) -> PhysicalChannelType {
        PhysicalChannelType::HardwareKey
    }
}
