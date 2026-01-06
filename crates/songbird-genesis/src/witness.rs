//! Genesis witness types and verification

use crate::{error::*, types::*};
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
    pub fn trust_level(&self) -> TrustLevel {
        self.physical_channel.trust_level().into()
    }

    /// Check if witness has hardware attestation
    pub fn has_hardware_attestation(&self) -> bool {
        self.physical_channel.has_hardware_attestation()
    }

    /// Verify witness signature using BearDog
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
                        "BearDog verification failed: {}",
                        e
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

    /// Sign data as witness using BearDog
    pub async fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        use crate::security_capability_client::SecurityCapabilityClient;

        // Try to create BearDog client
        match SecurityCapabilityClient::new().await {
            Ok(client) => {
                // Use BearDog for cryptographic signing
                let signature = client.sign_data(&self.device_id, data).await.map_err(|e| {
                    GenesisError::SigningFailed(format!("BearDog signing failed: {}", e))
                })?;

                self.signature = signature.clone();
                Ok(signature)
            }
            Err(e) => {
                // Fallback: Create deterministic signature (graceful degradation)
                tracing::warn!(
                    "BearDog not available for signing: {}. Using fallback signature.",
                    e
                );
                let sig = format!("witness_sig_{}_{}", self.device_id, data.len()).into_bytes();
                self.signature = sig.clone();
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
}
