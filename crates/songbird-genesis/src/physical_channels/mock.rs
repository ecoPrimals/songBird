// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Mock physical channel for testing
//!
//! **Test-Only Module**: This mock is only available during testing
//! and should not be used in production code.
//!
//! For production, use actual physical channel providers:
//! - `SoloKeyChannel` for hardware keys
//! - `QrCodeChannel` for QR code verification
//! - `BluetoothChannel` for Bluetooth proximity

use crate::{error::*, types::*};
use chrono::Utc;

use super::PhysicalChannelProvider;

/// Mock physical channel for testing
#[derive(Debug)]
pub struct MockPhysicalChannel {
    /// Simulate which channel type this is
    pub channel_type: PhysicalChannelType,

    /// Should verification succeed?
    pub should_succeed: bool,
}

impl MockPhysicalChannel {
    /// Create new mock channel (defaults to hardware key, always succeeds)
    pub fn new() -> Self {
        Self {
            channel_type: PhysicalChannelType::HardwareKey,
            should_succeed: true,
        }
    }

    /// Create mock channel with specific type
    pub fn with_channel_type(channel_type: PhysicalChannelType) -> Self {
        Self {
            channel_type,
            should_succeed: true,
        }
    }

    /// Create mock channel that fails verification
    pub fn failing() -> Self {
        Self {
            channel_type: PhysicalChannelType::HardwareKey,
            should_succeed: false,
        }
    }
}

impl Default for MockPhysicalChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalChannelProvider for MockPhysicalChannel {
    async fn verify_proximity(&self) -> Result<ProximityProof> {
        if !self.should_succeed {
            return Err(GenesisError::ProximityVerificationFailed(
                "Mock verification failed".to_string(),
            ));
        }

        Ok(ProximityProof {
            channel_type: self.channel_type,
            timestamp: Utc::now(),
            proof_data: b"mock_proof_data".to_vec(),
            attestation: Some(b"mock_attestation".to_vec()),
        })
    }

    async fn secure_exchange(&self) -> Result<Vec<u8>> {
        if !self.should_succeed {
            return Err(GenesisError::PhysicalChannelError("Mock exchange failed".to_string()));
        }

        // Return mock genesis credentials
        Ok(b"mock_genesis_credentials".to_vec())
    }

    fn trust_level(&self) -> TrustLevel {
        self.channel_type.trust_level().into()
    }

    fn channel_type(&self) -> PhysicalChannelType {
        self.channel_type
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[tokio::test]
    async fn test_mock_channel_success() {
        let channel = MockPhysicalChannel::new();

        let proof = channel.verify_proximity().await;
        assert!(proof.is_ok());

        let creds = channel.secure_exchange().await;
        assert!(creds.is_ok());

        assert_eq!(channel.trust_level(), TrustLevel::Maximum);
    }

    #[tokio::test]
    async fn test_mock_channel_failure() {
        let channel = MockPhysicalChannel::failing();

        let proof = channel.verify_proximity().await;
        assert!(proof.is_err());

        let creds = channel.secure_exchange().await;
        assert!(creds.is_err());
    }

    #[tokio::test]
    async fn test_different_channel_types() {
        let hw = MockPhysicalChannel::with_channel_type(PhysicalChannelType::HardwareKey);
        assert_eq!(hw.trust_level(), TrustLevel::Maximum);

        let qr = MockPhysicalChannel::with_channel_type(PhysicalChannelType::QrCodeWithOob);
        assert_eq!(qr.trust_level(), TrustLevel::High);

        let bt = MockPhysicalChannel::with_channel_type(PhysicalChannelType::Bluetooth);
        assert_eq!(bt.trust_level(), TrustLevel::Medium);
    }
}
