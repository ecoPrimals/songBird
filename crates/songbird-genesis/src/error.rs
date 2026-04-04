// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Error types for genesis operations

use thiserror::Error;

/// Result type for genesis operations
pub type Result<T> = std::result::Result<T, GenesisError>;

/// Errors that can occur during genesis operations
#[derive(Debug, Error)]
pub enum GenesisError {
    /// Physical proximity verification failed
    #[error("Physical proximity verification failed: {0}")]
    ProximityVerificationFailed(String),

    /// Witness signature invalid
    #[error("Witness signature invalid: {0}")]
    InvalidWitnessSignature(String),

    /// Physical channel error
    #[error("Physical channel error: {0}")]
    PhysicalChannelError(String),

    /// Coordination error (when using primal coordination)
    #[error("Coordination failed: {0}")]
    CoordinationFailed(String),

    /// Genesis ceremony timeout
    #[error("Genesis ceremony timed out after {0}s")]
    CeremonyTimeout(u64),

    /// Witness not authorized
    #[error("Witness not authorized: {0}")]
    UnauthorizedWitness(String),

    /// Invalid genesis certificate
    #[error("Invalid genesis certificate: {0}")]
    InvalidCertificate(String),

    /// Lineage establishment failed
    #[error("Lineage establishment failed: {0}")]
    LineageFailed(String),

    /// Cryptographic signature verification failed
    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    /// Cryptographic signing failed
    #[error("Signing failed: {0}")]
    SigningFailed(String),

    /// Hardware key error (`SoloKey`, `YubiKey`, etc.)
    #[error("Hardware key error: {0}")]
    HardwareKeyError(String),

    /// FIDO2 / CTAP2 hardware key path not integrated (the `solokey` feature compiles the channel only).
    #[error("SoloKey / FIDO2 not integrated: {0}")]
    SoloKeyNotIntegrated(String),

    /// QR code error
    #[error("QR code error: {0}")]
    QrCodeError(String),

    /// Bluetooth error
    #[error("Bluetooth error: {0}")]
    BluetoothError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),

    /// Optional feature or dependency not enabled
    #[error("Feature unavailable: {0}")]
    FeatureUnavailable(String),
}

impl From<anyhow::Error> for GenesisError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<String> for GenesisError {
    fn from(msg: String) -> Self {
        Self::Other(msg)
    }
}

impl From<&str> for GenesisError {
    fn from(msg: &str) -> Self {
        Self::Other(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::GenesisError;
    use std::error::Error as _;

    #[test]
    fn display_and_source_for_variants() {
        let cases: Vec<(GenesisError, &str)> = vec![
            (
                GenesisError::ProximityVerificationFailed("x".into()),
                "Physical proximity verification failed: x",
            ),
            (GenesisError::InvalidWitnessSignature("sig".into()), "Witness signature invalid: sig"),
            (GenesisError::PhysicalChannelError("ch".into()), "Physical channel error: ch"),
            (GenesisError::CoordinationFailed("coord".into()), "Coordination failed: coord"),
            (GenesisError::CeremonyTimeout(60), "Genesis ceremony timed out after 60s"),
            (GenesisError::UnauthorizedWitness("u".into()), "Witness not authorized: u"),
            (GenesisError::InvalidCertificate("c".into()), "Invalid genesis certificate: c"),
            (GenesisError::LineageFailed("l".into()), "Lineage establishment failed: l"),
            (
                GenesisError::SignatureVerificationFailed("v".into()),
                "Signature verification failed: v",
            ),
            (GenesisError::SigningFailed("s".into()), "Signing failed: s"),
            (GenesisError::HardwareKeyError("h".into()), "Hardware key error: h"),
            (
                GenesisError::SoloKeyNotIntegrated("detail".into()),
                "SoloKey / FIDO2 not integrated: detail",
            ),
            (GenesisError::QrCodeError("q".into()), "QR code error: q"),
            (GenesisError::BluetoothError("b".into()), "Bluetooth error: b"),
            (GenesisError::FeatureUnavailable("f".into()), "Feature unavailable: f"),
        ];

        for (err, want_prefix) in cases {
            assert_eq!(err.to_string(), want_prefix, "Display should match for {err:?}");
        }

        let serde_err = serde_json::from_str::<i32>("not-json").expect_err("invalid json");
        let wrapped: GenesisError = serde_err.into();
        assert!(wrapped.to_string().contains("Serialization error"));
        assert!(wrapped.source().is_some(), "serde error should set source");

        let io_err = std::io::Error::other("disk");
        let wrapped: GenesisError = io_err.into();
        assert!(wrapped.to_string().starts_with("IO error:"));

        let anyhow_err = anyhow::anyhow!("boom");
        let from_any: GenesisError = anyhow_err.into();
        assert_eq!(from_any.to_string(), "boom");

        let from_string: GenesisError = GenesisError::from("hello".to_string());
        assert_eq!(from_string.to_string(), "hello");

        let from_str: GenesisError = GenesisError::from("slice");
        assert_eq!(from_str.to_string(), "slice");

        let other = GenesisError::Other("custom failure".to_string());
        assert_eq!(other.to_string(), "custom failure");
    }

    #[test]
    fn ceremony_timeout_zero_seconds_display() {
        let err = GenesisError::CeremonyTimeout(0);
        assert_eq!(err.to_string(), "Genesis ceremony timed out after 0s");
    }

    #[test]
    fn ceremony_timeout_large_u64_display() {
        let err = GenesisError::CeremonyTimeout(u64::MAX);
        assert!(err.to_string().contains("18446744073709551615"));
    }

    #[test]
    fn feature_unavailable_allows_empty_detail() {
        let err = GenesisError::FeatureUnavailable(String::new());
        assert_eq!(err.to_string(), "Feature unavailable: ");
    }
}
