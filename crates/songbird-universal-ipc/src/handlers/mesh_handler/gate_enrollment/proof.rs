// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::enrollment_crypto::{call_security_provider, constant_time_eq};
use super::types::{EnrollPhase, PhysicalProof};
use serde_json::{Value, json};

pub(super) async fn verify_physical_proof(proof: &PhysicalProof) -> EnrollPhase {
    match proof {
        PhysicalProof::Fido2 {
            credential_id,
            attestation,
        } => {
            let result = call_security_provider(
                "fido2.verify_attestation",
                json!({
                    "credential_id": credential_id,
                    "attestation": attestation,
                    "purpose": "gate_enrollment",
                }),
            )
            .await;

            match result {
                Ok(value) => {
                    let valid = value.get("valid").and_then(Value::as_bool).unwrap_or(false);
                    EnrollPhase {
                        name: "proof.verify".into(),
                        ok: valid,
                        detail: if valid {
                            format!(
                                "FIDO2 attestation verified (credential: {}...)",
                                &credential_id[..8.min(credential_id.len())]
                            )
                        } else {
                            "FIDO2 attestation invalid".into()
                        },
                    }
                }
                Err(e) => EnrollPhase {
                    name: "proof.verify".into(),
                    ok: false,
                    detail: format!("FIDO2 verification unavailable: {e}"),
                },
            }
        }
        PhysicalProof::BeaconProximity {
            beacon_id,
            challenge_response,
        } => {
            let result = call_security_provider(
                "beacon.verify_proximity",
                json!({
                    "beacon_id": beacon_id,
                    "challenge_response": challenge_response,
                }),
            )
            .await;

            match result {
                Ok(value) => {
                    let valid = value.get("valid").and_then(Value::as_bool).unwrap_or(false);
                    EnrollPhase {
                        name: "proof.verify".into(),
                        ok: valid,
                        detail: if valid {
                            format!("beacon proximity verified (beacon: {beacon_id})")
                        } else {
                            "beacon proximity challenge failed".into()
                        },
                    }
                }
                Err(e) => EnrollPhase {
                    name: "proof.verify".into(),
                    ok: false,
                    detail: format!("beacon verification unavailable: {e}"),
                },
            }
        }
        PhysicalProof::Token {
            token,
        } => {
            let expected = songbird_process_env::var("GATE_ENROLLMENT_TOKEN").ok();
            match expected {
                Some(exp) if constant_time_eq(exp.as_bytes(), token.as_bytes()) => EnrollPhase {
                    name: "proof.verify".into(),
                    ok: true,
                    detail: "enrollment token verified".into(),
                },
                Some(_) => EnrollPhase {
                    name: "proof.verify".into(),
                    ok: false,
                    detail: "enrollment token mismatch".into(),
                },
                None => EnrollPhase {
                    name: "proof.verify".into(),
                    ok: false,
                    detail: "GATE_ENROLLMENT_TOKEN not set on hub".into(),
                },
            }
        }
    }
}
