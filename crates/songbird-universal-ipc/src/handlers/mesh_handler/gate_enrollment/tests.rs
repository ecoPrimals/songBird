// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::ip_allocation::parse_used_ips;
use super::types::{
    EnrollPhase, ForgejoProvisionConfig, GateEnrollRequest, GateEnrollResponse, PhysicalProof,
    WgProvisionConfig,
};

#[test]
fn parse_used_ips_from_wg_output() {
    let output = "\
QxYl...= 10.13.37.1/32
Ab3f...= 10.13.37.2/32 192.168.4.0/22
Kz9p...= 10.13.37.7/32
";
    let ips = parse_used_ips(output);
    assert!(ips.contains(&"10.13.37.1".to_string()));
    assert!(ips.contains(&"10.13.37.2".to_string()));
    assert!(ips.contains(&"10.13.37.7".to_string()));
    assert!(ips.contains(&"192.168.4.0".to_string()));
}

#[test]
fn parse_used_ips_empty() {
    assert!(parse_used_ips("").is_empty());
}

#[test]
fn gate_enroll_request_deserializes() {
    let json = r#"{
            "gate_name": "testGate",
            "wg_public_key": "abc123",
            "physical_proof": {
                "type": "token",
                "token": "secret"
            }
        }"#;
    let req: GateEnrollRequest = serde_json::from_str(json).expect("should deserialize");
    assert_eq!(req.gate_name, "testGate");
    assert!(matches!(req.physical_proof, PhysicalProof::Token { .. }));
}

#[test]
fn gate_enroll_request_with_fido2() {
    let json = r#"{
            "gate_name": "remoteGate",
            "wg_public_key": "xyz789",
            "ssh_public_key": "ssh-ed25519 AAAA...",
            "physical_proof": {
                "type": "fido2",
                "credential_id": "cred123",
                "attestation": "att456"
            },
            "composition": "full"
        }"#;
    let req: GateEnrollRequest = serde_json::from_str(json).expect("should deserialize");
    assert_eq!(req.composition.as_deref(), Some("full"));
    assert!(matches!(req.physical_proof, PhysicalProof::Fido2 { .. }));
}

#[test]
fn gate_enroll_response_serializes() {
    let resp = GateEnrollResponse {
        enrolled: true,
        gate_name: "testGate".into(),
        mesh_ip: Some("10.13.37.20".into()),
        wg_config: Some(WgProvisionConfig {
            hub_endpoint: "157.230.3.183:51820".into(),
            hub_public_key: "A2fvz3c...".into(),
            assigned_ip: "10.13.37.20".into(),
            subnet: "10.13.37.0/24".into(),
            dns: "10.13.37.1".into(),
        }),
        forgejo_config: None,
        family_seed_encrypted: Some("encrypted_blob".into()),
        reason: None,
        phases: vec![EnrollPhase {
            name: "proof.verify".into(),
            ok: true,
            detail: "token verified".into(),
        }],
    };
    let json = serde_json::to_string(&resp).expect("should serialize");
    assert!(json.contains("testGate"));
    assert!(json.contains("10.13.37.20"));
}
