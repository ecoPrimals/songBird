//! SoloKey hardware key support (FIDO2/WebAuthn)

use crate::{error::*, types::*};
use async_trait::async_trait;
use chrono::Utc;

use super::PhysicalChannelProvider;

/// SoloKey hardware key channel
#[derive(Debug)]
pub struct SoloKeyChannel {
    // TODO: Add webauthn-rs integration
}

impl SoloKeyChannel {
    /// Create new SoloKey channel
    pub fn new() -> Self {
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
