//! Physical channel implementations for genesis

use crate::error::Result;
use crate::types::{PhysicalChannelType, ProximityProof, TrustLevel};
use async_trait::async_trait;

#[cfg(feature = "solokey")]
pub mod solokey;

#[cfg(feature = "qr")]
pub mod qr_code;

#[cfg(feature = "bluetooth")]
pub mod bluetooth;

#[cfg(feature = "pure-bluetooth")]
pub mod bluetooth_pure;

// Mock implementation - TEST ONLY
#[cfg(test)]
pub mod mock;
#[cfg(test)]
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
pub enum PhysicalChannel {
    /// Hardware security key (`SoloKey`, `YubiKey`, etc.)
    #[cfg(feature = "solokey")]
    HardwareKey(solokey::SoloKeyChannel),

    /// QR code with out-of-band verification
    #[cfg(feature = "qr")]
    QrCode(qr_code::QrCodeChannel),

    /// Bluetooth LE pairing (legacy btleplug)
    #[cfg(feature = "bluetooth")]
    Bluetooth(bluetooth::BluetoothChannel),

    /// Pure Rust Bluetooth LE pairing
    #[cfg(feature = "pure-bluetooth")]
    BluetoothPure(bluetooth_pure::PureRustBluetoothChannel),

    /// Mock channel for testing
    #[cfg(test)]
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

            #[cfg(feature = "pure-bluetooth")]
            Self::BluetoothPure(ch) => ch.verify_proximity().await,

            #[cfg(test)]
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

            #[cfg(feature = "pure-bluetooth")]
            Self::BluetoothPure(ch) => ch.secure_exchange().await,

            #[cfg(test)]
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

            #[cfg(feature = "pure-bluetooth")]
            Self::BluetoothPure(ch) => ch.trust_level(),

            #[cfg(test)]
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

            #[cfg(feature = "pure-bluetooth")]
            Self::BluetoothPure(ch) => ch.channel_type(),

            #[cfg(test)]
            Self::Mock(ch) => ch.channel_type(),
        }
    }
}
