// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use serde::{Deserialize, Serialize};

/// Physical proof types for enrollment trust tiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PhysicalProof {
    /// FIDO2/`WebAuthn` attestation (`SoloKey`, `YubiKey`) — strongest tier.
    Fido2 {
        credential_id: String,
        attestation: String,
    },
    /// grapheneGate BLE/NFC proximity proof — medium tier.
    BeaconProximity {
        beacon_id: String,
        challenge_response: String,
    },
    /// Pre-shared enrollment token — weakest autonomous tier.
    Token {
        token: String,
    },
}

/// Request parameters for `mesh.gate_enroll`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateEnrollRequest {
    pub gate_name: String,
    pub wg_public_key: String,
    #[serde(default)]
    pub ssh_public_key: Option<String>,
    pub physical_proof: PhysicalProof,
    #[serde(default)]
    pub composition: Option<String>,
}

/// Response from `mesh.gate_enroll`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateEnrollResponse {
    pub enrolled: bool,
    pub gate_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wg_config: Option<WgProvisionConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forgejo_config: Option<ForgejoProvisionConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_seed_encrypted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub phases: Vec<EnrollPhase>,
}

/// `WireGuard` provisioning details returned to the enrollee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WgProvisionConfig {
    pub hub_endpoint: String,
    pub hub_public_key: String,
    pub assigned_ip: String,
    pub subnet: String,
    pub dns: String,
}

/// Forgejo provisioning details returned to the enrollee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgejoProvisionConfig {
    pub host: String,
    pub port: u16,
    pub org: String,
    pub ssh_key_registered: bool,
}

/// Individual enrollment phase result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollPhase {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}
