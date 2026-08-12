// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Autonomous gate enrollment — zero-operator mesh provisioning.
//!
//! `mesh.gate_enroll` orchestrates the full enrollment pipeline for a new gate:
//!
//! 1. Verify physical proof (FIDO2 attestation / beacon proximity / token)
//! 2. Allocate mesh IP from the dynamic pool (.20–.254)
//! 3. Register `WireGuard` peer on the hub
//! 4. Register SSH key via Forgejo API
//! 5. Deliver family seed (encrypted to enrollee's WG public key)
//! 6. Call `mesh.enroll` for BTSP-verified genetic enrollment
//!
//! This runs on the hub gate (golgiBody) and is the WAN-reachable entry point
//! for K-Derm inward trust escalation.

mod forgejo;
mod ip_allocation;
mod proof;
mod seed;
mod types;
mod wireguard;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

pub use types::{
    EnrollPhase, ForgejoProvisionConfig, GateEnrollRequest, GateEnrollResponse, WgProvisionConfig,
};

use serde_json::{Value, json};
use tracing::info;

use super::MeshHandler;
use super::enrollment_crypto::{
    compute_hub_enrollment_proof, load_family_seed_bytes, resolve_hub_endpoint, resolve_hub_pubkey,
};

use forgejo::register_forgejo_key;
use ip_allocation::allocate_mesh_ip;
use proof::verify_physical_proof;
use seed::deliver_family_seed;
use wireguard::register_wg_peer;

impl MeshHandler {
    /// Handle `mesh.gate_enroll` — full autonomous gate enrollment.
    pub async fn handle_gate_enroll(&self, params: Value) -> Result<Value, String> {
        let request: GateEnrollRequest = serde_json::from_value(params)
            .map_err(|e| format!("Invalid gate_enroll params: {e}"))?;

        info!(gate = %request.gate_name, "mesh.gate_enroll: starting autonomous enrollment");

        let mut phases = Vec::new();

        // Phase 1: Verify physical proof
        let proof_phase = verify_physical_proof(&request.physical_proof).await;
        let proof_ok = proof_phase.ok;
        phases.push(proof_phase);

        if !proof_ok {
            return serde_json::to_value(GateEnrollResponse {
                enrolled: false,
                gate_name: request.gate_name,
                mesh_ip: None,
                wg_config: None,
                forgejo_config: None,
                family_seed_encrypted: None,
                reason: Some("Physical proof verification failed".into()),
                phases,
            })
            .map_err(|e| format!("Serialize: {e}"));
        }

        // Phase 2: Allocate mesh IP
        let (ip_phase, allocated_ip) = allocate_mesh_ip(&request.gate_name).await;
        phases.push(ip_phase);

        let Some(mesh_ip) = allocated_ip else {
            return serde_json::to_value(GateEnrollResponse {
                enrolled: false,
                gate_name: request.gate_name,
                mesh_ip: None,
                wg_config: None,
                forgejo_config: None,
                family_seed_encrypted: None,
                reason: Some("IP pool exhausted".into()),
                phases,
            })
            .map_err(|e| format!("Serialize: {e}"));
        };

        // Phase 3: Register WireGuard peer on hub
        let wg_phase = register_wg_peer(&request.gate_name, &request.wg_public_key, &mesh_ip).await;
        phases.push(wg_phase);

        // Phase 4: Register SSH key on Forgejo (if provided)
        let forgejo_config = if let Some(ref ssh_key) = request.ssh_public_key {
            let (fg_phase, registered) = register_forgejo_key(&request.gate_name, ssh_key).await;
            phases.push(fg_phase);
            Some(ForgejoProvisionConfig {
                host: format!("{}.1", ip_allocation::mesh_subnet()),
                port: songbird_process_env::var("FORGEJO_SSH_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(2222),
                org: songbird_process_env::var("FORGEJO_ORG")
                    .unwrap_or_else(|_| "ecoPrimals".into()),
                ssh_key_registered: registered,
            })
        } else {
            phases.push(EnrollPhase {
                name: "forgejo.key".into(),
                ok: true,
                detail: "no SSH key provided — skipped".into(),
            });
            None
        };

        // Phase 5: Encrypt and deliver family seed
        let (seed_phase, encrypted_seed) = deliver_family_seed(&request.wg_public_key).await;
        phases.push(seed_phase);

        // Phase 6: BTSP genetic enrollment via mesh.enroll
        let enroll_phase =
            self.genetic_enroll(&request.gate_name, &request.wg_public_key, &mesh_ip).await;
        let genetically_enrolled = enroll_phase.ok;
        phases.push(enroll_phase);

        let wg_config = Some(WgProvisionConfig {
            hub_endpoint: resolve_hub_endpoint().unwrap_or_default(),
            hub_public_key: resolve_hub_pubkey().unwrap_or_default(),
            assigned_ip: mesh_ip.clone(),
            subnet: format!("{}.0/24", ip_allocation::mesh_subnet()),
            dns: format!("{}.1", ip_allocation::mesh_subnet()),
        });

        let all_pass = phases.iter().all(|p| p.ok);

        info!(
            gate = %request.gate_name,
            mesh_ip = %mesh_ip,
            enrolled = all_pass,
            genetic = genetically_enrolled,
            "mesh.gate_enroll: complete"
        );

        serde_json::to_value(GateEnrollResponse {
            enrolled: all_pass,
            gate_name: request.gate_name,
            mesh_ip: Some(mesh_ip),
            wg_config,
            forgejo_config,
            family_seed_encrypted: encrypted_seed,
            reason: if all_pass {
                None
            } else {
                Some("one or more phases failed — see phases array".into())
            },
            phases,
        })
        .map_err(|e| format!("Serialize: {e}"))
    }

    /// Internal: run BTSP genetic enrollment after provisioning.
    async fn genetic_enroll(
        &self,
        gate_name: &str,
        public_key: &str,
        mesh_ip: &str,
    ) -> EnrollPhase {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());

        let family_seed_bytes = load_family_seed_bytes();

        let Some(seed_bytes) = family_seed_bytes else {
            return EnrollPhase {
                name: "genetic.enroll".into(),
                ok: false,
                detail: "FAMILY_SEED not available on hub — cannot compute enrollment proof".into(),
            };
        };

        let proof = compute_hub_enrollment_proof(&seed_bytes, gate_name, public_key, timestamp, 0);

        let enroll_params = json!({
            "node_id": gate_name,
            "public_key": public_key,
            "timestamp": timestamp,
            "proof": proof,
            "address": format!("{mesh_ip}:7700"),
        });

        match self.handle_enroll(enroll_params).await {
            Ok(result) => {
                let enrolled = result.get("enrolled").and_then(Value::as_bool).unwrap_or(false);
                EnrollPhase {
                    name: "genetic.enroll".into(),
                    ok: enrolled,
                    detail: if enrolled {
                        format!("{gate_name} genetically enrolled into mesh")
                    } else {
                        let reason =
                            result.get("reason").and_then(Value::as_str).unwrap_or("unknown");
                        format!("genetic enrollment failed: {reason}")
                    },
                }
            }
            Err(e) => EnrollPhase {
                name: "genetic.enroll".into(),
                ok: false,
                detail: format!("mesh.enroll call failed: {e}"),
            },
        }
    }
}
