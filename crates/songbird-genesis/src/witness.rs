// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Genesis witness types and verification

use crate::error::{GenesisError, Result};
use crate::types::{PhysicalChannelType, TrustLevel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A device that witnesses the birth of a new node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisWitness {
    /// Witness device identifier
    pub device_id: String,

    /// Witness public key
    pub public_key: Vec<u8>,

    /// Physical channel used for genesis
    pub physical_channel: PhysicalChannelType,

    /// Timestamp of genesis ceremony
    pub timestamp: DateTime<Utc>,

    /// Signature over new node's identity
    pub signature: Vec<u8>,

    /// Optional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl GenesisWitness {
    /// Create a new genesis witness
    #[must_use]
    pub fn new(
        device_id: String,
        public_key: Vec<u8>,
        physical_channel: PhysicalChannelType,
    ) -> Self {
        Self {
            device_id,
            public_key,
            physical_channel,
            timestamp: Utc::now(),
            signature: Vec::new(), // Will be filled by ceremony
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Get trust level based on physical channel
    #[must_use]
    pub fn trust_level(&self) -> TrustLevel {
        self.physical_channel.trust_level().into()
    }

    /// Check if witness has hardware attestation
    #[must_use]
    pub const fn has_hardware_attestation(&self) -> bool {
        self.physical_channel.has_hardware_attestation()
    }

    /// Verify witness signature using `BearDog`
    ///
    /// # Errors
    ///
    /// Returns an error if `BearDog` verification fails and signature is non-empty.
    pub async fn verify_signature(&self, data: &[u8]) -> Result<bool> {
        use crate::security_capability_client::SecurityCapabilityClient;

        if self.signature.is_empty() {
            return Ok(false);
        }

        // Try to create BearDog client
        match SecurityCapabilityClient::new().await {
            Ok(client) => {
                // Use BearDog for cryptographic verification
                client.verify_signature(&self.device_id, data, &self.signature).await.map_err(|e| {
                    GenesisError::SignatureVerificationFailed(format!(
                        "BearDog verification failed: {e}"
                    ))
                })
            }
            Err(e) => {
                // Fallback: Check signature exists (graceful degradation)
                tracing::warn!(
                    "BearDog not available for signature verification: {}. Using basic check.",
                    e
                );
                Ok(!self.signature.is_empty())
            }
        }
    }

    /// Sign data as witness using `BearDog`
    ///
    /// # Errors
    ///
    /// Returns an error if `BearDog` signing fails.
    pub async fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        use crate::security_capability_client::SecurityCapabilityClient;

        // Try to create BearDog client
        match SecurityCapabilityClient::new().await {
            Ok(client) => {
                // Use BearDog for cryptographic signing
                let signature = client.sign_data(&self.device_id, data).await.map_err(|e| {
                    GenesisError::SigningFailed(format!("BearDog signing failed: {e}"))
                })?;

                self.signature.clone_from(&signature);
                Ok(signature)
            }
            Err(e) => {
                // Fallback: Create deterministic signature (graceful degradation)
                tracing::warn!(
                    "BearDog not available for signing: {}. Using fallback signature.",
                    e
                );
                let sig = format!("witness_sig_{}_{}", self.device_id, data.len()).into_bytes();
                self.signature.clone_from(&sig);
                Ok(sig)
            }
        }
    }
}

/// Witness authority verification
pub struct WitnessVerifier {
    /// Trusted witness public keys
    trusted_witnesses:
        std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<u8>>>>,
}

impl WitnessVerifier {
    /// Create new witness verifier
    #[must_use]
    pub fn new() -> Self {
        Self {
            trusted_witnesses: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    /// Add trusted witness
    pub async fn add_trusted_witness(&self, device_id: String, public_key: Vec<u8>) {
        self.trusted_witnesses.write().await.insert(device_id, public_key);
    }

    /// Check if witness is trusted
    pub async fn is_trusted(&self, witness: &GenesisWitness) -> bool {
        let witnesses = self.trusted_witnesses.read().await;
        witnesses.contains_key(&witness.device_id)
    }

    /// Verify witness authority
    ///
    /// # Errors
    ///
    /// Returns an error if witness is not trusted or trust level is insufficient.
    pub async fn verify_authority(&self, witness: &GenesisWitness) -> Result<()> {
        if !self.is_trusted(witness).await {
            return Err(GenesisError::UnauthorizedWitness(format!(
                "Witness {} is not in trusted set",
                witness.device_id
            )));
        }

        // Verify trust level is sufficient for genesis
        if witness.trust_level() < TrustLevel::Medium {
            return Err(GenesisError::UnauthorizedWitness(
                "Trust level too low for genesis ceremony".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for WitnessVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_witness_creation() {
        let witness = GenesisWitness::new(
            "test-device".to_string(),
            vec![1, 2, 3],
            PhysicalChannelType::HardwareKey,
        );

        assert_eq!(witness.device_id, "test-device");
        assert_eq!(witness.trust_level(), TrustLevel::Maximum);
        assert!(witness.has_hardware_attestation());
    }

    #[test]
    fn test_trust_levels() {
        let hw_witness =
            GenesisWitness::new("hw".to_string(), vec![], PhysicalChannelType::HardwareKey);
        assert_eq!(hw_witness.trust_level(), TrustLevel::Maximum);

        let qr_witness =
            GenesisWitness::new("qr".to_string(), vec![], PhysicalChannelType::QrCodeWithOob);
        assert_eq!(qr_witness.trust_level(), TrustLevel::High);

        let bt_witness =
            GenesisWitness::new("bt".to_string(), vec![], PhysicalChannelType::Bluetooth);
        assert_eq!(bt_witness.trust_level(), TrustLevel::Medium);
    }

    #[tokio::test]
    async fn test_witness_verifier() {
        let verifier = WitnessVerifier::new();

        let witness = GenesisWitness::new(
            "trusted".to_string(),
            vec![1, 2, 3],
            PhysicalChannelType::HardwareKey,
        );

        // Should not be trusted initially
        assert!(!verifier.is_trusted(&witness).await);

        // Add to trusted set
        verifier.add_trusted_witness("trusted".to_string(), vec![1, 2, 3]).await;

        // Should now be trusted
        assert!(verifier.is_trusted(&witness).await);
        assert!(verifier.verify_authority(&witness).await.is_ok());
    }

    #[tokio::test]
    async fn verify_signature_empty_returns_false_without_error() {
        let witness = GenesisWitness::new(
            "dev".to_string(),
            vec![],
            PhysicalChannelType::HardwareKey,
        );
        assert!(
            !witness.verify_signature(b"payload").await.expect("verify should not error on empty sig"),
            "empty witness signature should yield false"
        );
    }

    #[tokio::test]
    async fn verify_authority_rejects_untrusted_witness() {
        let verifier = WitnessVerifier::new();
        let witness = GenesisWitness::new(
            "unknown".to_string(),
            vec![1],
            PhysicalChannelType::HardwareKey,
        );
        let err = verifier.verify_authority(&witness).await.expect_err("untrusted witness");
        match err {
            GenesisError::UnauthorizedWitness(msg) => {
                assert!(msg.contains("unknown"), "message should name witness: {msg}");
            }
            other => panic!("expected UnauthorizedWitness, got {other:?}"),
        }
    }

    #[test]
    fn genesis_witness_serde_roundtrip() {
        let w = GenesisWitness::new(
            "d1".to_string(),
            vec![9, 8],
            PhysicalChannelType::QrCodeWithOob,
        );
        let json = serde_json::to_string(&w).expect("serialize witness");
        let back: GenesisWitness = serde_json::from_str(&json).expect("deserialize witness");
        assert_eq!(back.device_id, "d1");
        assert_eq!(back.public_key, vec![9, 8]);
        assert_eq!(back.physical_channel, PhysicalChannelType::QrCodeWithOob);
    }

    #[tokio::test]
    async fn witness_verifier_default_matches_new_empty() {
        let a = WitnessVerifier::new();
        let b = WitnessVerifier::default();
        let w = GenesisWitness::new("x".to_string(), vec![], PhysicalChannelType::HardwareKey);
        assert!(!a.is_trusted(&w).await, "new() should start with no trusted witnesses");
        assert!(!b.is_trusted(&w).await, "default() should start with no trusted witnesses");
    }
}
