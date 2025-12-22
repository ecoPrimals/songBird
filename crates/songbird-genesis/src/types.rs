//! Core types for genesis operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Physical channel type for genesis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalChannelType {
    /// Hardware security key (SoloKey, YubiKey, etc.)
    /// Trust level: ⭐⭐⭐⭐⭐ (Highest)
    HardwareKey,

    /// QR code with out-of-band verification
    /// Trust level: ⭐⭐⭐⭐ (High)
    QrCodeWithOob,

    /// Bluetooth Low Energy pairing
    /// Trust level: ⭐⭐⭐ (Medium-High)
    Bluetooth,

    /// NFC tap
    /// Trust level: ⭐⭐⭐⭐ (High)
    Nfc,
}

impl PhysicalChannelType {
    /// Get trust level for this channel (0-5 stars)
    pub fn trust_level(&self) -> u8 {
        match self {
            Self::HardwareKey => 5,
            Self::Nfc => 4,
            Self::QrCodeWithOob => 4,
            Self::Bluetooth => 3,
        }
    }

    /// Check if this channel provides hardware attestation
    pub fn has_hardware_attestation(&self) -> bool {
        matches!(self, Self::HardwareKey | Self::Nfc)
    }
}

/// Trust level for genesis ceremony
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Low trust (not recommended for genesis)
    Low,

    /// Medium trust (acceptable with additional verification)
    Medium,

    /// High trust (recommended for genesis)
    High,

    /// Maximum trust (hardware-backed)
    Maximum,
}

impl From<u8> for TrustLevel {
    fn from(level: u8) -> Self {
        match level {
            5 => Self::Maximum,
            4 => Self::High,
            3 => Self::Medium,
            _ => Self::Low,
        }
    }
}

/// Proximity proof from physical channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProximityProof {
    /// Channel used
    pub channel_type: PhysicalChannelType,

    /// Timestamp of proximity verification
    pub timestamp: DateTime<Utc>,

    /// Channel-specific proof data
    pub proof_data: Vec<u8>,

    /// Optional attestation (for hardware keys)
    pub attestation: Option<Vec<u8>>,
}

/// Genesis lineage from a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalLineage {
    /// Which primal provided this lineage
    pub primal_name: String,

    /// Primal-specific lineage data
    pub lineage_data: Vec<u8>,

    /// Signature over lineage
    pub signature: Vec<u8>,

    /// When this lineage was established
    pub timestamp: DateTime<Utc>,
}

/// Complete genesis lineage from all primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisLineage {
    /// Genesis witness who initiated ceremony
    pub witness_device_id: String,

    /// Lineages from all coordinating primals
    pub primal_lineages: HashMap<String, PrimalLineage>,

    /// When genesis ceremony occurred
    pub birth_timestamp: DateTime<Utc>,

    /// Unique genesis ceremony ID
    pub ceremony_id: uuid::Uuid,
}

/// Request to coordinate genesis with a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalGenesisRequest {
    /// New node identifier
    pub new_node_id: String,

    /// Genesis witness information
    pub witness_device_id: String,

    /// Witness public key
    pub witness_pubkey: Vec<u8>,

    /// Physical channel proof
    pub proximity_proof: ProximityProof,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Response from primal genesis coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalGenesisResponse {
    /// Primal name
    pub primal_name: String,

    /// Established lineage
    pub lineage: PrimalLineage,

    /// Success indicator
    pub success: bool,

    /// Optional error message
    pub error: Option<String>,
}
