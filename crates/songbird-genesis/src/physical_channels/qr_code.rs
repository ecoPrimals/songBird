//! QR code with out-of-band verification

use crate::error::Result;
use crate::types::{PhysicalChannelType, ProximityProof, TrustLevel};
use async_trait::async_trait;
use chrono::Utc;

use super::PhysicalChannelProvider;

/// QR code channel with out-of-band verification
#[derive(Debug)]
pub struct QrCodeChannel {
    // TODO: Add qrcode generation support
}

impl QrCodeChannel {
    /// Create new QR code channel
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for QrCodeChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PhysicalChannelProvider for QrCodeChannel {
    async fn verify_proximity(&self) -> Result<ProximityProof> {
        // TODO: Implement QR code scanning + OOB verification
        Ok(ProximityProof {
            channel_type: PhysicalChannelType::QrCodeWithOob,
            timestamp: Utc::now(),
            proof_data: b"qr_proof".to_vec(),
            attestation: None,
        })
    }

    async fn secure_exchange(&self) -> Result<Vec<u8>> {
        // TODO: Implement secure exchange after QR scan
        Ok(b"qr_genesis_creds".to_vec())
    }

    fn trust_level(&self) -> TrustLevel {
        TrustLevel::High
    }

    fn channel_type(&self) -> PhysicalChannelType {
        PhysicalChannelType::QrCodeWithOob
    }
}
