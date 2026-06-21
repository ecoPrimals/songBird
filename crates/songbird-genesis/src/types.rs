// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Core types for genesis operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Physical channel type for genesis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalChannelType {
    /// Hardware security key (`SoloKey`, `YubiKey`, etc.)
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
    #[must_use]
    pub const fn trust_level(&self) -> u8 {
        match self {
            Self::HardwareKey => 5,
            Self::Nfc | Self::QrCodeWithOob => 4,
            Self::Bluetooth => 3,
        }
    }

    /// Check if this channel provides hardware attestation
    #[must_use]
    pub const fn has_hardware_attestation(&self) -> bool {
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use chrono::TimeZone;

    #[test]
    fn physical_channel_type_trust_stars_and_attestation() {
        assert_eq!(
            PhysicalChannelType::HardwareKey.trust_level(),
            5,
            "hardware key should map to five stars"
        );
        assert!(PhysicalChannelType::HardwareKey.has_hardware_attestation());

        assert_eq!(PhysicalChannelType::QrCodeWithOob.trust_level(), 4);
        assert_eq!(PhysicalChannelType::Nfc.trust_level(), 4);
        assert!(!PhysicalChannelType::QrCodeWithOob.has_hardware_attestation());
        assert!(PhysicalChannelType::Nfc.has_hardware_attestation());

        assert_eq!(PhysicalChannelType::Bluetooth.trust_level(), 3);
        assert!(!PhysicalChannelType::Bluetooth.has_hardware_attestation());
    }

    #[test]
    fn trust_level_from_u8_star_mapping() {
        assert_eq!(TrustLevel::from(5_u8), TrustLevel::Maximum);
        assert_eq!(TrustLevel::from(4_u8), TrustLevel::High);
        assert_eq!(TrustLevel::from(3_u8), TrustLevel::Medium);
        assert_eq!(TrustLevel::from(2_u8), TrustLevel::Low);
        assert_eq!(TrustLevel::from(0_u8), TrustLevel::Low);
    }

    #[test]
    fn trust_level_ordering() {
        assert!(TrustLevel::Low < TrustLevel::Medium);
        assert!(TrustLevel::Medium < TrustLevel::High);
        assert!(TrustLevel::High < TrustLevel::Maximum);
    }

    #[test]
    fn serde_roundtrip_core_types() {
        let proof = ProximityProof {
            channel_type: PhysicalChannelType::Bluetooth,
            timestamp: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap(),
            proof_data: vec![1, 2, 3],
            attestation: None,
        };
        let json = serde_json::to_string(&proof).expect("serialize proof");
        let back: ProximityProof = serde_json::from_str(&json).expect("deserialize proof");
        assert_eq!(back.channel_type, proof.channel_type);
        assert_eq!(back.proof_data, proof.proof_data);

        let lineage = PrimalLineage {
            primal_name: String::from("songbird"),
            lineage_data: vec![9],
            signature: vec![8],
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&lineage).expect("serialize lineage");
        let back: PrimalLineage = serde_json::from_str(&json).expect("deserialize lineage");
        assert_eq!(back.primal_name, "songbird");
    }

    #[tokio::test(start_paused = true)]
    async fn tokio_virtual_time_advances_for_deterministic_async_delays() {
        let start = tokio::time::Instant::now();
        let delay = std::time::Duration::from_secs(2);
        let sleep = tokio::time::sleep(delay);
        tokio::time::advance(delay).await;
        sleep.await;
        assert!(
            start.elapsed() >= delay,
            "paused clock should advance only via `time::advance` in tests"
        );
    }

    #[test]
    fn physical_channel_type_serde_roundtrip_all_variants() {
        for ch in [
            PhysicalChannelType::HardwareKey,
            PhysicalChannelType::QrCodeWithOob,
            PhysicalChannelType::Bluetooth,
            PhysicalChannelType::Nfc,
        ] {
            let json = serde_json::to_string(&ch).expect("serialize channel type");
            let back: PhysicalChannelType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, ch, "round-trip should preserve {ch:?}");
        }
    }

    #[test]
    fn trust_level_serde_roundtrip_all_variants() {
        for level in [TrustLevel::Low, TrustLevel::Medium, TrustLevel::High, TrustLevel::Maximum] {
            let json = serde_json::to_string(&level).expect("serialize trust");
            let back: TrustLevel = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, level);
        }
    }

    #[test]
    fn genesis_lineage_serde_roundtrip_with_multiple_primals() {
        use std::collections::HashMap;
        use uuid::Uuid;

        let mut primal_lineages = HashMap::new();
        primal_lineages.insert(
            String::from("a"),
            PrimalLineage {
                primal_name: String::from("a"),
                lineage_data: vec![1, 2],
                signature: vec![3],
                timestamp: Utc.with_ymd_and_hms(2025, 6, 1, 12, 0, 0).unwrap(),
            },
        );
        primal_lineages.insert(
            String::from("b"),
            PrimalLineage {
                primal_name: String::from("b"),
                lineage_data: vec![],
                signature: vec![9, 9],
                timestamp: Utc.with_ymd_and_hms(2025, 6, 2, 0, 0, 0).unwrap(),
            },
        );

        let gl = GenesisLineage {
            witness_device_id: String::from("witness-1"),
            primal_lineages,
            birth_timestamp: Utc.with_ymd_and_hms(2025, 6, 3, 0, 0, 0).unwrap(),
            ceremony_id: Uuid::nil(),
        };

        let json = serde_json::to_string(&gl).expect("serialize lineage");
        let back: GenesisLineage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.witness_device_id, "witness-1");
        assert_eq!(back.primal_lineages.len(), 2);
        assert_eq!(back.ceremony_id, Uuid::nil());
        assert!(back.primal_lineages.contains_key("a"));
    }

    #[test]
    fn primal_genesis_request_and_response_serde_roundtrip() {
        let req = PrimalGenesisRequest {
            new_node_id: String::from("child"),
            witness_device_id: String::from("w"),
            witness_pubkey: vec![0xab, 0xcd],
            proximity_proof: ProximityProof {
                channel_type: PhysicalChannelType::Nfc,
                timestamp: Utc.with_ymd_and_hms(2024, 2, 2, 2, 2, 2).unwrap(),
                proof_data: vec![7],
                attestation: Some(vec![8, 8]),
            },
            timestamp: Utc.with_ymd_and_hms(2024, 2, 3, 0, 0, 0).unwrap(),
        };
        let req_json = serde_json::to_string(&req).expect("req serde");
        let req_back: PrimalGenesisRequest = serde_json::from_str(&req_json).expect("req de");
        assert_eq!(req_back.new_node_id, "child");
        assert_eq!(req_back.proximity_proof.attestation, Some(vec![8, 8]));

        let resp = PrimalGenesisResponse {
            primal_name: String::from("songbird"),
            lineage: PrimalLineage {
                primal_name: String::from("songbird"),
                lineage_data: vec![1],
                signature: vec![2],
                timestamp: Utc.with_ymd_and_hms(2024, 3, 3, 3, 3, 3).unwrap(),
            },
            success: false,
            error: Some(String::from("temporary")),
        };
        let resp_json = serde_json::to_string(&resp).expect("resp serde");
        let resp_back: PrimalGenesisResponse = serde_json::from_str(&resp_json).expect("resp de");
        assert!(!resp_back.success);
        assert_eq!(resp_back.error.as_deref(), Some("temporary"));
        assert_eq!(resp_back.lineage.primal_name, "songbird");
    }

    #[test]
    fn proximity_proof_roundtrip_preserves_empty_proof_data() {
        let proof = ProximityProof {
            channel_type: PhysicalChannelType::HardwareKey,
            timestamp: Utc::now(),
            proof_data: vec![],
            attestation: None,
        };
        let json = serde_json::to_string(&proof).expect("serialize");
        let back: ProximityProof = serde_json::from_str(&json).expect("deserialize");
        assert!(back.proof_data.is_empty());
        assert!(back.attestation.is_none());
    }

    #[test]
    fn trust_level_from_u8_maps_star_counts_and_saturates_low() {
        assert_eq!(TrustLevel::from(6_u8), TrustLevel::Low);
        assert_eq!(TrustLevel::from(1_u8), TrustLevel::Low);
        assert_eq!(TrustLevel::from(u8::MAX), TrustLevel::Low);
    }
}
