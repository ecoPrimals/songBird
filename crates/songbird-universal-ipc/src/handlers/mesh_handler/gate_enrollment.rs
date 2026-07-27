// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Autonomous gate enrollment — zero-operator mesh provisioning.
//!
//! `mesh.gate_enroll` orchestrates the full enrollment pipeline for a new gate:
//!
//! 1. Verify physical proof (FIDO2 attestation / beacon proximity / token)
//! 2. Allocate mesh IP from the dynamic pool (.20–.254)
//! 3. Register WireGuard peer on the hub
//! 4. Register SSH key via Forgejo API
//! 5. Deliver family seed (encrypted to enrollee's WG public key)
//! 6. Call `mesh.enroll` for BTSP-verified genetic enrollment
//!
//! This runs on the hub gate (golgiBody) and is the WAN-reachable entry point
//! for K-Derm inward trust escalation.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{info, warn};

use super::MeshHandler;

/// IP pool range for dynamic gate allocation.
const POOL_START: u8 = 20;
const POOL_END: u8 = 254;
const MESH_SUBNET: &str = "10.13.37";

/// Physical proof types for enrollment trust tiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PhysicalProof {
    /// FIDO2/WebAuthn attestation (SoloKey, YubiKey) — strongest tier.
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
    Token { token: String },
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

/// WireGuard provisioning details returned to the enrollee.
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

impl MeshHandler {
    /// Handle `mesh.gate_enroll` — full autonomous gate enrollment.
    pub async fn handle_gate_enroll(&self, params: Value) -> Result<Value, String> {
        let request: GateEnrollRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid gate_enroll params: {e}"))?;

        info!(gate = %request.gate_name, "mesh.gate_enroll: starting autonomous enrollment");

        let mut phases = Vec::new();

        // Phase 1: Verify physical proof
        let proof_phase = verify_physical_proof(&request.physical_proof).await;
        let proof_ok = proof_phase.ok;
        phases.push(proof_phase);

        if !proof_ok {
            return Ok(serde_json::to_value(GateEnrollResponse {
                enrolled: false,
                gate_name: request.gate_name,
                mesh_ip: None,
                wg_config: None,
                forgejo_config: None,
                family_seed_encrypted: None,
                reason: Some("Physical proof verification failed".into()),
                phases,
            })
            .map_err(|e| format!("Serialize: {e}"))?);
        }

        // Phase 2: Allocate mesh IP
        let (ip_phase, allocated_ip) = allocate_mesh_ip(&request.gate_name).await;
        phases.push(ip_phase);

        let Some(mesh_ip) = allocated_ip else {
            return Ok(serde_json::to_value(GateEnrollResponse {
                enrolled: false,
                gate_name: request.gate_name,
                mesh_ip: None,
                wg_config: None,
                forgejo_config: None,
                family_seed_encrypted: None,
                reason: Some("IP pool exhausted".into()),
                phases,
            })
            .map_err(|e| format!("Serialize: {e}"))?);
        };

        // Phase 3: Register WireGuard peer on hub
        let wg_phase =
            register_wg_peer(&request.gate_name, &request.wg_public_key, &mesh_ip).await;
        phases.push(wg_phase);

        // Phase 4: Register SSH key on Forgejo (if provided)
        let forgejo_config = if let Some(ref ssh_key) = request.ssh_public_key {
            let (fg_phase, registered) =
                register_forgejo_key(&request.gate_name, ssh_key).await;
            phases.push(fg_phase);
            Some(ForgejoProvisionConfig {
                host: "10.13.37.1".into(),
                port: 2222,
                org: "ecoPrimals".into(),
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
        let (seed_phase, encrypted_seed) =
            deliver_family_seed(&request.wg_public_key).await;
        phases.push(seed_phase);

        // Phase 6: BTSP genetic enrollment via mesh.enroll
        let enroll_phase = self
            .genetic_enroll(&request.gate_name, &request.wg_public_key, &mesh_ip)
            .await;
        let genetically_enrolled = enroll_phase.ok;
        phases.push(enroll_phase);

        let wg_config = Some(WgProvisionConfig {
            hub_endpoint: resolve_hub_endpoint(),
            hub_public_key: resolve_hub_pubkey(),
            assigned_ip: mesh_ip.clone(),
            subnet: format!("{MESH_SUBNET}.0/24"),
            dns: format!("{MESH_SUBNET}.1"),
        });

        let all_pass = phases.iter().all(|p| p.ok);

        info!(
            gate = %request.gate_name,
            mesh_ip = %mesh_ip,
            enrolled = all_pass,
            genetic = genetically_enrolled,
            "mesh.gate_enroll: complete"
        );

        Ok(serde_json::to_value(GateEnrollResponse {
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
        .map_err(|e| format!("Serialize: {e}"))?)
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

        let proof = compute_hub_enrollment_proof(
            &seed_bytes,
            gate_name,
            public_key,
            timestamp,
            0,
        );

        let enroll_params = json!({
            "node_id": gate_name,
            "public_key": public_key,
            "timestamp": timestamp,
            "proof": proof,
            "address": format!("{mesh_ip}:7700"),
        });

        match self.handle_enroll(enroll_params).await {
            Ok(result) => {
                let enrolled = result
                    .get("enrolled")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                EnrollPhase {
                    name: "genetic.enroll".into(),
                    ok: enrolled,
                    detail: if enrolled {
                        format!("{gate_name} genetically enrolled into mesh")
                    } else {
                        let reason = result
                            .get("reason")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
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

// ── Phase implementations ──────────────────────────────────────────

async fn verify_physical_proof(proof: &PhysicalProof) -> EnrollPhase {
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
                    let valid = value
                        .get("valid")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    EnrollPhase {
                        name: "proof.verify".into(),
                        ok: valid,
                        detail: if valid {
                            format!("FIDO2 attestation verified (credential: {}...)", &credential_id[..8.min(credential_id.len())])
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
                    let valid = value
                        .get("valid")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
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
        PhysicalProof::Token { token } => {
            let expected = std::env::var("GATE_ENROLLMENT_TOKEN").ok();
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

/// Allocate a mesh IP from the dynamic pool.
///
/// Checks which IPs in the .20–.254 range are already allocated by querying
/// `wg show wg0 allowed-ips` on the local hub.
async fn allocate_mesh_ip(gate_name: &str) -> (EnrollPhase, Option<String>) {
    let used_ips = match tokio::process::Command::new("wg")
        .args(["show", "wg0", "allowed-ips"])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_used_ips(&stdout)
        }
        Ok(_) | Err(_) => {
            return (
                EnrollPhase {
                    name: "ip.allocate".into(),
                    ok: false,
                    detail: "cannot query wg0 allowed-ips — is WireGuard running?".into(),
                },
                None,
            );
        }
    };

    for octet in POOL_START..=POOL_END {
        let candidate = format!("{MESH_SUBNET}.{octet}");
        if !used_ips.contains(&candidate) {
            info!(gate = %gate_name, ip = %candidate, "ip.allocate: assigned from pool");
            return (
                EnrollPhase {
                    name: "ip.allocate".into(),
                    ok: true,
                    detail: format!("{candidate} allocated for {gate_name} (pool .{POOL_START}–.{POOL_END})"),
                },
                Some(candidate),
            );
        }
    }

    warn!("ip.allocate: pool exhausted (.{POOL_START}–.{POOL_END})");
    (
        EnrollPhase {
            name: "ip.allocate".into(),
            ok: false,
            detail: format!("pool exhausted — all .{POOL_START}–.{POOL_END} allocated"),
        },
        None,
    )
}

/// Parse IPs from `wg show wg0 allowed-ips` output.
fn parse_used_ips(wg_output: &str) -> Vec<String> {
    wg_output
        .lines()
        .flat_map(|line| {
            line.split_whitespace().skip(1).filter_map(|cidr| {
                cidr.split('/').next().map(String::from)
            })
        })
        .collect()
}

/// Register a WireGuard peer on the local hub.
async fn register_wg_peer(gate_name: &str, wg_pubkey: &str, mesh_ip: &str) -> EnrollPhase {
    let cmd = format!(
        "wg set wg0 peer {wg_pubkey} allowed-ips {mesh_ip}/32 && wg-quick save wg0"
    );

    match tokio::process::Command::new("sh")
        .args(["-c", &cmd])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            info!(gate = %gate_name, ip = %mesh_ip, "wg.peer: registered on hub");
            EnrollPhase {
                name: "wg.peer".into(),
                ok: true,
                detail: format!("peer {gate_name} ({mesh_ip}) registered on hub wg0"),
            }
        }
        Ok(output) => EnrollPhase {
            name: "wg.peer".into(),
            ok: false,
            detail: format!(
                "wg set failed (exit {}): {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        },
        Err(e) => EnrollPhase {
            name: "wg.peer".into(),
            ok: false,
            detail: format!("wg command failed: {e}"),
        },
    }
}

/// Register an SSH public key on Forgejo via its REST API.
///
/// Uses `curl` to POST to the Forgejo API, avoiding additional HTTP client
/// dependencies. Requires `FORGEJO_API_TOKEN` and optionally `FORGEJO_API_URL`.
async fn register_forgejo_key(gate_name: &str, ssh_pubkey: &str) -> (EnrollPhase, bool) {
    let forgejo_url = std::env::var("FORGEJO_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000/api/v1".into());
    let forgejo_token = std::env::var("FORGEJO_API_TOKEN");

    let Ok(token) = forgejo_token else {
        return (
            EnrollPhase {
                name: "forgejo.key".into(),
                ok: false,
                detail: "FORGEJO_API_TOKEN not set — cannot register SSH key".into(),
            },
            false,
        );
    };

    let url = format!("{forgejo_url}/user/keys");
    let body = json!({
        "title": format!("{gate_name}-deploy"),
        "key": ssh_pubkey,
        "read_only": false,
    });

    match tokio::process::Command::new("curl")
        .args([
            "-s",
            "-o", "/dev/stdout",
            "-w", "\n%{http_code}",
            "-X", "POST",
            "-H", &format!("Authorization: token {token}"),
            "-H", "Content-Type: application/json",
            "-d", &body.to_string(),
            &url,
        ])
        .output()
        .await
    {
        Ok(output) => {
            let raw = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = raw.trim().rsplitn(2, '\n').collect();
            let status_code: u16 = lines.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let response_body = lines.get(1).unwrap_or(&"");

            if (200..300).contains(&status_code) {
                info!(gate = %gate_name, "forgejo.key: SSH key registered");
                (
                    EnrollPhase {
                        name: "forgejo.key".into(),
                        ok: true,
                        detail: format!("SSH key registered as {gate_name}-deploy"),
                    },
                    true,
                )
            } else if status_code == 422 || response_body.contains("already") {
                (
                    EnrollPhase {
                        name: "forgejo.key".into(),
                        ok: true,
                        detail: format!("SSH key already registered (HTTP {status_code})"),
                    },
                    true,
                )
            } else {
                (
                    EnrollPhase {
                        name: "forgejo.key".into(),
                        ok: false,
                        detail: format!("Forgejo API error (HTTP {status_code}): {response_body}"),
                    },
                    false,
                )
            }
        }
        Err(e) => (
            EnrollPhase {
                name: "forgejo.key".into(),
                ok: false,
                detail: format!("curl failed: {e}"),
            },
            false,
        ),
    }
}

/// Load the family seed, resolving file paths if needed.
///
/// The `FAMILY_SEED` env var may contain a direct value or a file path
/// (e.g. `/etc/membrane/family/family.key`). If it starts with `/` and
/// the file exists, read the file and base64-encode the raw bytes.
fn load_family_seed_value() -> Option<String> {
    let raw = std::env::var("FAMILY_SEED")
        .or_else(|_| std::env::var("BEARDOG_FAMILY_SEED"))
        .ok()?;

    if raw.starts_with('/') {
        match std::fs::read(&raw) {
            Ok(bytes) => {
                use base64::Engine;
                Some(base64::engine::general_purpose::STANDARD.encode(&bytes))
            }
            Err(_) => Some(raw),
        }
    } else {
        Some(raw)
    }
}

/// Load the family seed as raw bytes for HMAC computation.
fn load_family_seed_bytes() -> Option<Vec<u8>> {
    let raw = std::env::var("FAMILY_SEED")
        .or_else(|_| std::env::var("BEARDOG_FAMILY_SEED"))
        .ok()?;

    if raw.starts_with('/') {
        std::fs::read(&raw).ok().or_else(|| Some(raw.into_bytes()))
    } else {
        Some(raw.into_bytes())
    }
}

/// Deliver the family seed encrypted to the enrollee's WireGuard public key.
///
/// Uses bearDog's `crypto.encrypt` to wrap the seed before transit.
async fn deliver_family_seed(wg_pubkey: &str) -> (EnrollPhase, Option<String>) {
    let family_seed = load_family_seed_value();

    let Some(seed) = family_seed else {
        return (
            EnrollPhase {
                name: "seed.deliver".into(),
                ok: false,
                detail: "FAMILY_SEED not available on hub".into(),
            },
            None,
        );
    };

    match call_security_provider(
        "crypto.encrypt",
        json!({
            "plaintext": seed,
            "recipient_key": wg_pubkey,
            "purpose": "gate_enrollment_seed_delivery",
        }),
    )
    .await
    {
        Ok(result) => {
            let ciphertext = result
                .get("ciphertext")
                .and_then(Value::as_str)
                .map(String::from);

            match ciphertext {
                Some(ct) => (
                    EnrollPhase {
                        name: "seed.deliver".into(),
                        ok: true,
                        detail: "family seed encrypted for enrollee".into(),
                    },
                    Some(ct),
                ),
                None => (
                    EnrollPhase {
                        name: "seed.deliver".into(),
                        ok: false,
                        detail: "crypto.encrypt returned no ciphertext".into(),
                    },
                    None,
                ),
            }
        }
        Err(e) => (
            EnrollPhase {
                name: "seed.deliver".into(),
                ok: false,
                detail: format!("bearDog crypto.encrypt unavailable: {e}"),
            },
            None,
        ),
    }
}

// ── Helpers ────────────────────────────────────────────────────────

/// Call bearDog's security provider via JSON-RPC over UDS.
async fn call_security_provider(method: &str, params: Value) -> Result<Value, String> {
    let socket_path =
        songbird_crypto_provider::socket_discovery::discover_security_socket();

    if !std::path::Path::new(&socket_path).exists() {
        return Err(format!("bearDog socket not found: {socket_path}"));
    }

    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    })
    .to_string();

    let stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .map_err(|e| format!("connect to bearDog: {e}"))?;

    let (reader, mut writer) = stream.into_split();
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    writer
        .write_all(request.as_bytes())
        .await
        .map_err(|e| format!("write to bearDog: {e}"))?;
    writer
        .write_all(b"\n")
        .await
        .map_err(|e| format!("write newline: {e}"))?;
    writer
        .shutdown()
        .await
        .map_err(|e| format!("shutdown write: {e}"))?;

    let mut response = String::new();
    let mut buf_reader = BufReader::new(reader);
    buf_reader
        .read_line(&mut response)
        .await
        .map_err(|e| format!("read from bearDog: {e}"))?;

    let parsed: Value =
        serde_json::from_str(&response).map_err(|e| format!("parse bearDog response: {e}"))?;

    if let Some(result) = parsed.get("result") {
        Ok(result.clone())
    } else if let Some(error) = parsed.get("error") {
        Err(format!(
            "bearDog error: {}",
            error.get("message").and_then(Value::as_str).unwrap_or("unknown")
        ))
    } else {
        Ok(parsed)
    }
}

fn resolve_hub_endpoint() -> String {
    std::env::var("WG_HUB_ENDPOINT")
        .unwrap_or_else(|_| "157.230.3.183:51820".into())
}

fn resolve_hub_pubkey() -> String {
    std::env::var("WG_HUB_PUBKEY")
        .unwrap_or_else(|_| "A2fvz3czkqRUuu2mzkSS6IVr/TCQcpsJX9HbDBa1FBc=".into())
}

/// Compute HMAC-SHA256 enrollment proof (mirrors bearDog's algorithm).
fn compute_hub_enrollment_proof(
    family_seed: &[u8],
    node_id: &str,
    public_key: &str,
    timestamp: u64,
    generation: u32,
) -> String {
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let family_id = std::env::var("FAMILY_ID").unwrap_or_else(|_| "default".into());

    // HKDF extract
    let mut extract_mac =
        HmacSha256::new_from_slice(family_id.as_bytes()).expect("HMAC key init");
    extract_mac.update(family_seed);
    let prk = extract_mac.finalize().into_bytes();

    // HKDF expand
    let info = format!("enrollment-v{generation}");
    let mut expand_mac = HmacSha256::new_from_slice(&prk).expect("HMAC key init");
    expand_mac.update(info.as_bytes());
    expand_mac.update(&[1u8]);
    let enrollment_key: [u8; 32] = expand_mac.finalize().into_bytes().into();

    // HMAC proof
    let message = format!("{node_id}|{public_key}|{timestamp}|{generation}");
    let mut proof_mac = HmacSha256::new_from_slice(&enrollment_key).expect("HMAC key init");
    proof_mac.update(message.as_bytes());
    let proof_bytes = proof_mac.finalize().into_bytes();

    base64::engine::general_purpose::STANDARD.encode(proof_bytes)
}

/// Constant-time equality check for tokens.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn constant_time_eq_works() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
    }

    #[test]
    fn hub_enrollment_proof_is_deterministic() {
        let p1 = compute_hub_enrollment_proof(b"seed", "gate1", "key1", 1000, 0);
        let p2 = compute_hub_enrollment_proof(b"seed", "gate1", "key1", 1000, 0);
        assert_eq!(p1, p2);
    }

    #[test]
    fn hub_enrollment_proof_varies_with_input() {
        let p1 = compute_hub_enrollment_proof(b"seed", "gate1", "key1", 1000, 0);
        let p2 = compute_hub_enrollment_proof(b"seed", "gate2", "key1", 1000, 0);
        assert_ne!(p1, p2);
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
}
