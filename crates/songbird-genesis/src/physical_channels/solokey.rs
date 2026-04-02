// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `SoloKey` hardware key support (FIDO2/WebAuthn)

use crate::error::Result;
use crate::types::{PhysicalChannelType, ProximityProof, TrustLevel};
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

impl PhysicalChannelProvider for SoloKeyChannel {
    async fn verify_proximity(&self) -> Result<ProximityProof> {
        // Placeholder attestation: real SoloKey/FIDO2 verification is not invoked here.
        Ok(ProximityProof {
            channel_type: PhysicalChannelType::HardwareKey,
            timestamp: Utc::now(),
            proof_data: b"solokey_proof".to_vec(),
            attestation: Some(b"hardware_attestation".to_vec()),
        })
    }

    async fn secure_exchange(&self) -> Result<Vec<u8>> {
        // Returns static demo material: hardware key exchange path is stubbed.
        Ok(b"solokey_genesis_creds".to_vec())
    }

    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Maximum
    }

    fn channel_type(&self) -> PhysicalChannelType {
        PhysicalChannelType::HardwareKey
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::SoloKeyChannel;
    use crate::physical_channels::PhysicalChannelProvider;
    use crate::types::{PhysicalChannelType, TrustLevel};

    #[test]
    fn solokey_channel_constructors() {
        let a = SoloKeyChannel::new();
        let b = SoloKeyChannel::default();
        assert_eq!(a.channel_type(), b.channel_type());
        assert_eq!(a.channel_type(), PhysicalChannelType::HardwareKey);
        assert_eq!(a.trust_level(), TrustLevel::Maximum);
    }

    #[tokio::test]
    async fn solokey_placeholder_proximity_and_exchange() {
        let ch = SoloKeyChannel::new();
        let proof = ch.verify_proximity().await.expect("placeholder proximity");
        assert_eq!(proof.channel_type, PhysicalChannelType::HardwareKey);
        assert!(proof.attestation.is_some());

        let creds = ch.secure_exchange().await.expect("placeholder exchange");
        assert_eq!(creds, b"solokey_genesis_creds");
    }
}
