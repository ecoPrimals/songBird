//! Bluetooth LE pairing

use crate::error::Result;
use crate::types::{PhysicalChannelType, ProximityProof, TrustLevel};
use async_trait::async_trait;
use chrono::Utc;

use super::PhysicalChannelProvider;

/// Bluetooth LE channel
#[derive(Debug)]
pub struct BluetoothChannel {
    // TODO: Add btleplug integration
}

impl BluetoothChannel {
    /// Create new Bluetooth channel
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for BluetoothChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PhysicalChannelProvider for BluetoothChannel {
    async fn verify_proximity(&self) -> Result<ProximityProof> {
        // TODO: Implement Bluetooth pairing
        Ok(ProximityProof {
            channel_type: PhysicalChannelType::Bluetooth,
            timestamp: Utc::now(),
            proof_data: b"bluetooth_proof".to_vec(),
            attestation: None,
        })
    }

    async fn secure_exchange(&self) -> Result<Vec<u8>> {
        // TODO: Implement secure exchange via Bluetooth
        Ok(b"bluetooth_genesis_creds".to_vec())
    }

    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Medium
    }

    fn channel_type(&self) -> PhysicalChannelType {
        PhysicalChannelType::Bluetooth
    }
}
