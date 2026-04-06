// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
#![allow(async_fn_in_trait, reason = "native async trait methods; not used as trait objects")]

//! Physical channel implementations for genesis

use crate::error::Result;
use crate::types::{PhysicalChannelType, ProximityProof, TrustLevel};

#[cfg(feature = "solokey")]
pub mod solokey;

pub mod qr_code;

#[cfg(feature = "legacy-bluetooth")]
pub mod bluetooth;

#[cfg(feature = "pure-bluetooth")]
pub mod bluetooth_pure;

// Mock implementation — tests only, or `--features testing` for integration tests.
#[cfg(any(test, feature = "testing"))]
pub mod mock;
#[cfg(any(test, feature = "testing"))]
pub use mock::MockPhysicalChannel;

/// Physical channel trait for genesis ceremonies
pub trait PhysicalChannelProvider: Send + Sync {
    /// Verify physical proximity
    async fn verify_proximity(&self) -> Result<ProximityProof>;

    /// Exchange genesis credentials securely
    async fn secure_exchange(&self) -> Result<Vec<u8>>;

    /// Get trust level of this channel
    fn trust_level(&self) -> TrustLevel;

    /// Get channel type
    fn channel_type(&self) -> PhysicalChannelType;
}

/// Physical channel enum for genesis
pub enum PhysicalChannel {
    /// Hardware security key (`SoloKey`, `YubiKey`, etc.)
    #[cfg(feature = "solokey")]
    HardwareKey(solokey::SoloKeyChannel),

    /// QR code with out-of-band verification
    QrCode(qr_code::QrCodeChannel),

    /// Bluetooth LE pairing (legacy btleplug)
    #[cfg(feature = "legacy-bluetooth")]
    #[expect(deprecated, reason = "calling deprecated API until migration completes")]
    // `bluetooth` module is deprecated; variant kept for callers using `legacy-bluetooth`
    Bluetooth(bluetooth::BluetoothChannel),

    /// Pure Rust Bluetooth LE pairing
    #[cfg(feature = "pure-bluetooth")]
    BluetoothPure(bluetooth_pure::PureRustBluetoothChannel),

    /// Mock channel for testing (not present in default production builds).
    #[cfg(any(test, feature = "testing"))]
    Mock(MockPhysicalChannel),
}

impl PhysicalChannelProvider for PhysicalChannel {
    async fn verify_proximity(&self) -> Result<ProximityProof> {
        match self {
            #[cfg(feature = "solokey")]
            Self::HardwareKey(ch) => ch.verify_proximity().await,

            Self::QrCode(ch) => ch.verify_proximity().await,

            #[cfg(feature = "legacy-bluetooth")]
            Self::Bluetooth(ch) => ch.verify_proximity().await,

            #[cfg(feature = "pure-bluetooth")]
            Self::BluetoothPure(ch) => ch.verify_proximity().await,

            #[cfg(any(test, feature = "testing"))]
            Self::Mock(ch) => ch.verify_proximity().await,
        }
    }

    async fn secure_exchange(&self) -> Result<Vec<u8>> {
        match self {
            #[cfg(feature = "solokey")]
            Self::HardwareKey(ch) => ch.secure_exchange().await,

            Self::QrCode(ch) => ch.secure_exchange().await,

            #[cfg(feature = "legacy-bluetooth")]
            Self::Bluetooth(ch) => ch.secure_exchange().await,

            #[cfg(feature = "pure-bluetooth")]
            Self::BluetoothPure(ch) => ch.secure_exchange().await,

            #[cfg(any(test, feature = "testing"))]
            Self::Mock(ch) => ch.secure_exchange().await,
        }
    }

    fn trust_level(&self) -> TrustLevel {
        match self {
            #[cfg(feature = "solokey")]
            Self::HardwareKey(ch) => ch.trust_level(),

            Self::QrCode(ch) => ch.trust_level(),

            #[cfg(feature = "legacy-bluetooth")]
            Self::Bluetooth(ch) => ch.trust_level(),

            #[cfg(feature = "pure-bluetooth")]
            Self::BluetoothPure(ch) => ch.trust_level(),

            #[cfg(any(test, feature = "testing"))]
            Self::Mock(ch) => ch.trust_level(),
        }
    }

    fn channel_type(&self) -> PhysicalChannelType {
        match self {
            #[cfg(feature = "solokey")]
            Self::HardwareKey(ch) => ch.channel_type(),

            Self::QrCode(ch) => ch.channel_type(),

            #[cfg(feature = "legacy-bluetooth")]
            Self::Bluetooth(ch) => ch.channel_type(),

            #[cfg(feature = "pure-bluetooth")]
            Self::BluetoothPure(ch) => ch.channel_type(),

            #[cfg(any(test, feature = "testing"))]
            Self::Mock(ch) => ch.channel_type(),
        }
    }
}

#[cfg(test)]
mod enum_dispatch_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::{PhysicalChannel, PhysicalChannelProvider};
    use crate::physical_channels::mock::MockPhysicalChannel;
    use crate::physical_channels::qr_code::QrCodeChannel;
    use crate::types::{PhysicalChannelType, TrustLevel};

    #[tokio::test]
    async fn physical_channel_enum_delegates_to_inner_provider() {
        let mock = PhysicalChannel::Mock(MockPhysicalChannel::new());
        assert_eq!(mock.channel_type(), PhysicalChannelType::HardwareKey);
        assert_eq!(mock.trust_level(), TrustLevel::Maximum);
        let proof = mock.verify_proximity().await.expect("mock proximity");
        assert_eq!(proof.channel_type, PhysicalChannelType::HardwareKey);

        let qr = PhysicalChannel::QrCode(QrCodeChannel::new());
        assert_eq!(qr.channel_type(), PhysicalChannelType::QrCodeWithOob);
        assert_eq!(qr.trust_level(), TrustLevel::High);
    }

    #[cfg(feature = "solokey")]
    #[tokio::test]
    async fn hardware_key_variant_reports_not_integrated() {
        use crate::error::GenesisError;
        use crate::physical_channels::solokey::SoloKeyChannel;
        let ch = PhysicalChannel::HardwareKey(SoloKeyChannel::new());
        assert_eq!(ch.channel_type(), PhysicalChannelType::HardwareKey);
        assert_eq!(ch.trust_level(), TrustLevel::Low);
        let err = ch.secure_exchange().await.expect_err("FIDO2 path not wired");
        assert!(matches!(err, GenesisError::SoloKeyNotIntegrated(_)));
    }

    #[cfg(feature = "legacy-bluetooth")]
    #[tokio::test]
    #[allow(deprecated, reason = "legacy btleplug channel remains in API until callers migrate")]
    async fn legacy_bluetooth_channel_errors_and_metadata() {
        use crate::error::GenesisError;
        use crate::physical_channels::PhysicalChannelProvider;
        use crate::physical_channels::bluetooth::BluetoothChannel;

        let ch = BluetoothChannel::new();
        assert_eq!(ch.channel_type(), PhysicalChannelType::Bluetooth);
        assert_eq!(ch.trust_level(), TrustLevel::Medium);

        let prox = ch.verify_proximity().await;
        let exchange = ch.secure_exchange().await;
        assert!(matches!(prox, Err(GenesisError::BluetoothError(_))), "got {prox:?}");
        assert!(matches!(exchange, Err(GenesisError::BluetoothError(_))), "got {exchange:?}");
    }
}
