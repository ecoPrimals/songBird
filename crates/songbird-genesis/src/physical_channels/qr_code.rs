// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QR code with out-of-band verification
//!
//! When the **`qr`** feature is enabled, optional dependencies (`qrcode`, `image`, `rqrr`) are
//! linked for future generation and scanning. Until that pipeline is wired, operations return
//! [`GenesisError::QrCodeError`].
//!
//! Without **`qr`**, [`PhysicalChannelProvider`] methods return
//! [`GenesisError::FeatureUnavailable`].

use crate::error::{GenesisError, Result};
use crate::types::{PhysicalChannelType, ProximityProof, TrustLevel};

use super::PhysicalChannelProvider;

/// QR code channel with out-of-band verification
#[derive(Debug)]
pub struct QrCodeChannel;

impl QrCodeChannel {
    /// Create new QR code channel
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for QrCodeChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalChannelProvider for QrCodeChannel {
    async fn verify_proximity(&self) -> Result<ProximityProof> {
        #[cfg(not(feature = "qr"))]
        {
            return Err(GenesisError::FeatureUnavailable(
                "QR code support requires the 'qr' feature".to_string(),
            ));
        }
        #[cfg(feature = "qr")]
        {
            Err(GenesisError::QrCodeError(
                "QR proximity verification is not yet implemented; enable `qr` and wire generation (qrcode), capture (image), and decode (rqrr)".to_string(),
            ))
        }
    }

    async fn secure_exchange(&self) -> Result<Vec<u8>> {
        #[cfg(not(feature = "qr"))]
        {
            return Err(GenesisError::FeatureUnavailable(
                "QR code support requires the 'qr' feature".to_string(),
            ));
        }
        #[cfg(feature = "qr")]
        {
            Err(GenesisError::QrCodeError(
                "QR secure exchange is not yet implemented; complete OOB scan validation then derive credentials".to_string(),
            ))
        }
    }

    fn trust_level(&self) -> TrustLevel {
        TrustLevel::High
    }

    fn channel_type(&self) -> PhysicalChannelType {
        PhysicalChannelType::QrCodeWithOob
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::QrCodeChannel;
    use crate::error::GenesisError;
    use crate::physical_channels::PhysicalChannelProvider;
    use crate::types::{PhysicalChannelType, TrustLevel};

    #[test]
    fn qr_code_channel_constructors_and_metadata() {
        let ch = QrCodeChannel::new();
        assert_eq!(ch.channel_type(), PhysicalChannelType::QrCodeWithOob);
        assert_eq!(ch.trust_level(), TrustLevel::High);
    }

    #[tokio::test]
    async fn qr_code_channel_proximity_and_exchange_match_feature_flags() {
        let ch = QrCodeChannel::new();
        let prox = ch.verify_proximity().await;
        let exchange = ch.secure_exchange().await;

        #[cfg(feature = "qr")]
        {
            assert!(matches!(prox, Err(GenesisError::QrCodeError(_))), "got {prox:?}");
            assert!(matches!(exchange, Err(GenesisError::QrCodeError(_))), "got {exchange:?}");
        }
        #[cfg(not(feature = "qr"))]
        {
            assert!(matches!(prox, Err(GenesisError::FeatureUnavailable(_))), "got {prox:?}");
            assert!(
                matches!(exchange, Err(GenesisError::FeatureUnavailable(_))),
                "got {exchange:?}"
            );
        }
    }
}
