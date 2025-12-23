//! Physical channel implementations for genesis

use crate::{error::*, types::*};
use async_trait::async_trait;

#[cfg(feature = "solokey")]
pub mod solokey;

#[cfg(feature = "qr")]
pub mod qr_code;

#[cfg(feature = "bluetooth")]
pub mod bluetooth;

#[cfg(feature = "bluetooth-pure")]
pub mod bluetooth_pure;

// Mock implementation for testing
pub mod mock;
pub use mock::MockPhysicalChannel;

/// Physical channel trait for genesis ceremonies
#[async_trait]
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
#[derive(Debug)]
pub enum PhysicalChannel {
    /// Hardware security key (SoloKey, YubiKey, etc.)
    #[cfg(feature = "solokey")]
    HardwareKey(solokey::SoloKeyChannel),

    /// QR code with out-of-band verification
    #[cfg(feature = "qr")]
    QrCode(qr_code::QrCodeChannel),

    /// Bluetooth LE pairing
    #[cfg(feature = "bluetooth")]
    Bluetooth(bluetooth::BluetoothChannel),

    /// Mock channel for testing
    Mock(MockPhysicalChannel),
}

#[async_trait]
impl PhysicalChannelProvider for PhysicalChannel {
    async fn verify_proximity(&self) -> Result<ProximityProof> {
        match self {
            #[cfg(feature = "solokey")]
            Self::HardwareKey(ch) => ch.verify_proximity().await,

            #[cfg(feature = "qr")]
            Self::QrCode(ch) => ch.verify_proximity().await,

            #[cfg(feature = "bluetooth")]
            Self::Bluetooth(ch) => ch.verify_proximity().await,

            Self::Mock(ch) => ch.verify_proximity().await,
        }
    }

    async fn secure_exchange(&self) -> Result<Vec<u8>> {
        match self {
            #[cfg(feature = "solokey")]
            Self::HardwareKey(ch) => ch.secure_exchange().await,

            #[cfg(feature = "qr")]
            Self::QrCode(ch) => ch.secure_exchange().await,

            #[cfg(feature = "bluetooth")]
            Self::Bluetooth(ch) => ch.secure_exchange().await,

            Self::Mock(ch) => ch.secure_exchange().await,
        }
    }

    fn trust_level(&self) -> TrustLevel {
        match self {
            #[cfg(feature = "solokey")]
            Self::HardwareKey(ch) => ch.trust_level(),

            #[cfg(feature = "qr")]
            Self::QrCode(ch) => ch.trust_level(),

            #[cfg(feature = "bluetooth")]
            Self::Bluetooth(ch) => ch.trust_level(),

            Self::Mock(ch) => ch.trust_level(),
        }
    }

    fn channel_type(&self) -> PhysicalChannelType {
        match self {
            #[cfg(feature = "solokey")]
            Self::HardwareKey(ch) => ch.channel_type(),

            #[cfg(feature = "qr")]
            Self::QrCode(ch) => ch.channel_type(),

            #[cfg(feature = "bluetooth")]
            Self::Bluetooth(ch) => ch.channel_type(),

            Self::Mock(ch) => ch.channel_type(),
        }
    }
}
