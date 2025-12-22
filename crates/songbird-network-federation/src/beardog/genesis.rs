//! Genesis Support for BearDog Integration
//!
//! Types and interfaces for physical genesis bootstrap coordination.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Genesis witness proof for new node births
///
/// Included in BirdSong discovery broadcasts when a new node is announcing
/// its genesis certification. This allows the network to verify the node
/// was properly witnessed during physical bootstrap.
///
/// ## Security Model: "Never Alone in the Dark Forest"
///
/// A new node is vulnerable during genesis. By including multi-primal witness
/// signatures, we ensure:
/// - Physical proximity was verified (SoloKey, QR, Bluetooth)
/// - Multiple primals witnessed the genesis
/// - Cryptographic lineage established from birth
/// - Node has strong trust anchors immediately
///
/// ## Privacy
///
/// The witness proof itself is encrypted in BirdSong broadcasts, so only
/// nodes with lineage can decrypt and verify the genesis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenesisWitnessProof {
    /// Unique ceremony identifier
    pub ceremony_id: String,

    /// New node's identifier
    pub node_id: String,

    /// Physical witness device (SoloKey, etc.)
    pub witness_device_id: String,

    /// Signature from witness device
    pub witness_signature: Vec<u8>,

    /// Physical channel used (HardwareKey, QR, Bluetooth)
    pub physical_channel: PhysicalChannelType,

    /// Primal witness signatures (Songbird, BearDog, etc.)
    pub primal_witnesses: Vec<PrimalWitnessSignature>,

    /// Genesis timestamp
    pub birth_timestamp: DateTime<Utc>,

    /// Trust level achieved during genesis
    pub trust_level: GenesisTrustLevel,
}

/// Physical channel type for genesis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PhysicalChannelType {
    /// FIDO2/WebAuthn hardware security key (highest trust)
    HardwareKey,

    /// QR code + out-of-band verification (high trust)
    QrCode,

    /// Bluetooth LE proximity (medium trust)
    Bluetooth,

    /// Custom channel (extensibility)
    Custom(String),
}

/// Signature from a primal witnessing the genesis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrimalWitnessSignature {
    /// Primal name (e.g., "Songbird", "BearDog")
    pub primal_name: String,

    /// Lineage granted by this primal
    pub lineage_data: Vec<u8>,

    /// Cryptographic signature from this primal
    pub signature: Vec<u8>,

    /// When this primal witnessed the genesis
    pub witness_timestamp: DateTime<Utc>,
}

/// Trust level achieved during genesis
///
/// Variants are ordered from lowest to highest trust.
/// This ordering is used for comparison operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GenesisTrustLevel {
    /// Bluetooth only (basic trust - lowest)
    Basic,

    /// QR + single primal (medium trust)
    Medium,

    /// Hardware key or multiple primals (high trust)
    High,

    /// Hardware key + multiple primals (maximum trust - highest)
    Maximum,
}

impl GenesisWitnessProof {
    /// Verify the genesis proof structure is valid
    ///
    /// Checks:
    /// - Has at least one primal witness
    /// - Timestamps are valid
    /// - Trust level matches physical channel
    ///
    /// Note: This does NOT verify cryptographic signatures.
    /// That requires coordination with BearDog.
    pub fn verify_structure(&self) -> anyhow::Result<()> {
        if self.primal_witnesses.is_empty() {
            anyhow::bail!("Genesis proof must have at least one primal witness");
        }

        let now = Utc::now();
        if self.birth_timestamp > now {
            anyhow::bail!("Genesis timestamp cannot be in the future");
        }

        // Verify trust level is appropriate for physical channel and witness count
        let expected_max_trust = match self.physical_channel {
            PhysicalChannelType::HardwareKey => {
                if self.primal_witnesses.len() >= 2 {
                    GenesisTrustLevel::Maximum
                } else {
                    GenesisTrustLevel::High
                }
            }
            PhysicalChannelType::QrCode => GenesisTrustLevel::Medium,
            PhysicalChannelType::Bluetooth => GenesisTrustLevel::Basic,
            PhysicalChannelType::Custom(_) => GenesisTrustLevel::Basic,
        };

        // Trust level should not exceed what the channel supports
        if self.trust_level > expected_max_trust {
            anyhow::bail!(
                "Trust level {:?} exceeds maximum {:?} for channel {:?} with {} witnesses",
                self.trust_level,
                expected_max_trust,
                self.physical_channel,
                self.primal_witnesses.len()
            );
        }

        Ok(())
    }

    /// Get the number of primal witnesses
    pub fn witness_count(&self) -> usize {
        self.primal_witnesses.len()
    }

    /// Check if a specific primal witnessed this genesis
    pub fn has_primal_witness(&self, primal_name: &str) -> bool {
        self.primal_witnesses.iter().any(|w| w.primal_name == primal_name)
    }

    /// Get the age of this genesis proof
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.birth_timestamp
    }

    /// Check if this is a fresh genesis (less than 1 hour old)
    pub fn is_fresh(&self) -> bool {
        self.age().num_hours() < 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_proof_validation() {
        let proof = GenesisWitnessProof {
            ceremony_id: "test-ceremony-123".to_string(),
            node_id: "test-node-456".to_string(),
            witness_device_id: "solokey-789".to_string(),
            witness_signature: vec![1, 2, 3, 4],
            physical_channel: PhysicalChannelType::HardwareKey,
            primal_witnesses: vec![
                PrimalWitnessSignature {
                    primal_name: "Songbird".to_string(),
                    lineage_data: vec![5, 6, 7, 8],
                    signature: vec![9, 10, 11, 12],
                    witness_timestamp: Utc::now(),
                },
                PrimalWitnessSignature {
                    primal_name: "BearDog".to_string(),
                    lineage_data: vec![13, 14, 15, 16],
                    signature: vec![17, 18, 19, 20],
                    witness_timestamp: Utc::now(),
                },
            ],
            birth_timestamp: Utc::now(),
            trust_level: GenesisTrustLevel::Maximum,
        };

        // Should validate successfully
        assert!(proof.verify_structure().is_ok());
        assert_eq!(proof.witness_count(), 2);
        assert!(proof.has_primal_witness("Songbird"));
        assert!(proof.has_primal_witness("BearDog"));
        assert!(!proof.has_primal_witness("Unknown"));
        assert!(proof.is_fresh());
    }

    #[test]
    fn test_genesis_proof_no_witnesses() {
        let proof = GenesisWitnessProof {
            ceremony_id: "test".to_string(),
            node_id: "test".to_string(),
            witness_device_id: "test".to_string(),
            witness_signature: vec![],
            physical_channel: PhysicalChannelType::HardwareKey,
            primal_witnesses: vec![],
            birth_timestamp: Utc::now(),
            trust_level: GenesisTrustLevel::Maximum,
        };

        // Should fail - no witnesses
        assert!(proof.verify_structure().is_err());
    }

    #[test]
    fn test_trust_level_validation() {
        let mut proof = GenesisWitnessProof {
            ceremony_id: "test".to_string(),
            node_id: "test".to_string(),
            witness_device_id: "test".to_string(),
            witness_signature: vec![],
            physical_channel: PhysicalChannelType::HardwareKey,
            primal_witnesses: vec![PrimalWitnessSignature {
                primal_name: "Test".to_string(),
                lineage_data: vec![],
                signature: vec![],
                witness_timestamp: Utc::now(),
            }],
            birth_timestamp: Utc::now(),
            trust_level: GenesisTrustLevel::Maximum, // Too high! Only 1 witness
        };

        // Should fail - trust level too high (needs 2+ witnesses for Maximum)
        assert!(proof.verify_structure().is_err());

        // Fix trust level to match single witness
        proof.trust_level = GenesisTrustLevel::High;
        assert!(proof.verify_structure().is_ok());

        // Add second witness - now Maximum is OK
        proof.primal_witnesses.push(PrimalWitnessSignature {
            primal_name: "Test2".to_string(),
            lineage_data: vec![],
            signature: vec![],
            witness_timestamp: Utc::now(),
        });
        proof.trust_level = GenesisTrustLevel::Maximum;
        assert!(proof.verify_structure().is_ok());
    }
}
