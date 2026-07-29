// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Enrollment cryptographic utilities — HMAC proofs, hub resolution, security provider RPC.
//!
//! Extracted from `gate_enrollment.rs` for cohesion: these are self-contained
//! utility functions with no dependency on `MeshHandler` state.

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Resolve the `WireGuard` hub endpoint from environment.
///
/// # Errors
///
/// Returns `Err` in production builds when `WG_HUB_ENDPOINT` is not set
/// (cfg `debug_assertions` falls back to development default with a warning).
pub fn resolve_hub_endpoint() -> Result<String, String> {
    match std::env::var("WG_HUB_ENDPOINT") {
        Ok(v) if !v.is_empty() => Ok(v),
        _ => {
            if cfg!(debug_assertions) {
                tracing::warn!("WG_HUB_ENDPOINT not set — using development default");
                Ok("157.230.3.183:51820".into())
            } else {
                Err("WG_HUB_ENDPOINT required in production".into())
            }
        }
    }
}

/// Resolve the `WireGuard` hub public key from environment.
///
/// # Errors
///
/// Returns `Err` in production builds when `WG_HUB_PUBKEY` is not set.
pub fn resolve_hub_pubkey() -> Result<String, String> {
    match std::env::var("WG_HUB_PUBKEY") {
        Ok(v) if !v.is_empty() => Ok(v),
        _ => {
            if cfg!(debug_assertions) {
                tracing::warn!("WG_HUB_PUBKEY not set — using development default");
                Ok("A2fvz3czkqRUuu2mzkSS6IVr/TCQcpsJX9HbDBa1FBc=".into())
            } else {
                Err("WG_HUB_PUBKEY required in production".into())
            }
        }
    }
}

/// Compute HMAC-SHA256 enrollment proof (mirrors bearDog's algorithm).
///
/// Uses HKDF extract+expand from the family seed to derive a per-generation
/// enrollment key, then HMACs the registration payload.
pub fn compute_hub_enrollment_proof(
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

    let mut extract_mac =
        HmacSha256::new_from_slice(family_id.as_bytes()).expect("valid HMAC key length");
    extract_mac.update(family_seed);
    let prk = extract_mac.finalize().into_bytes();

    let info = format!("enrollment-v{generation}");
    let mut expand_mac = HmacSha256::new_from_slice(&prk).expect("valid HMAC key length");
    expand_mac.update(info.as_bytes());
    expand_mac.update(&[1u8]);
    let enrollment_key: [u8; 32] = expand_mac.finalize().into_bytes().into();

    let message = format!("{node_id}|{public_key}|{timestamp}|{generation}");
    let mut proof_mac = HmacSha256::new_from_slice(&enrollment_key).expect("valid HMAC key length");
    proof_mac.update(message.as_bytes());
    let proof_bytes = proof_mac.finalize().into_bytes();

    base64::engine::general_purpose::STANDARD.encode(proof_bytes)
}

/// Call bearDog's security provider via JSON-RPC over platform-native transport.
pub async fn call_security_provider(method: &str, params: Value) -> Result<Value, String> {
    let socket_path = songbird_crypto_provider::socket_discovery::discover_security_socket();

    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    })
    .to_string();

    #[cfg(unix)]
    let stream = {
        if !std::path::Path::new(&socket_path).exists() {
            return Err(format!("bearDog socket not found: {socket_path}"));
        }
        tokio::net::UnixStream::connect(&socket_path)
            .await
            .map_err(|e| format!("connect to bearDog (UDS): {e}"))?
    };

    #[cfg(not(unix))]
    let stream = {
        let addr = if socket_path.contains(':') {
            socket_path.clone()
        } else {
            "127.0.0.1:9100".to_string()
        };
        tokio::net::TcpStream::connect(&addr)
            .await
            .map_err(|e| format!("connect to bearDog (TCP {addr}): {e}"))?
    };

    let (reader, mut writer) = stream.into_split();
    writer.write_all(request.as_bytes()).await.map_err(|e| format!("write to bearDog: {e}"))?;
    writer.write_all(b"\n").await.map_err(|e| format!("write newline: {e}"))?;
    writer.shutdown().await.map_err(|e| format!("shutdown write: {e}"))?;

    let mut response = String::new();
    let mut buf_reader = BufReader::new(reader);
    buf_reader.read_line(&mut response).await.map_err(|e| format!("read from bearDog: {e}"))?;

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

/// Load the family seed, resolving file paths if needed.
///
/// The `FAMILY_SEED` env var may contain a direct value or a file path
/// (e.g. `/etc/membrane/family/family.key`). If it starts with `/` and
/// the file exists, read the file and base64-encode the raw bytes.
pub fn load_family_seed_value() -> Option<String> {
    let raw =
        std::env::var("FAMILY_SEED").or_else(|_| std::env::var("BEARDOG_FAMILY_SEED")).ok()?;

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
pub fn load_family_seed_bytes() -> Option<Vec<u8>> {
    let raw =
        std::env::var("FAMILY_SEED").or_else(|_| std::env::var("BEARDOG_FAMILY_SEED")).ok()?;

    if raw.starts_with('/') {
        std::fs::read(&raw).ok().or_else(|| Some(raw.into_bytes()))
    } else {
        Some(raw.into_bytes())
    }
}

/// Constant-time equality check for tokens.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Validate `WireGuard` public key format (base64-encoded 32 bytes = 44 chars + optional `=`).
pub fn is_valid_wg_pubkey(key: &str) -> bool {
    let len = key.len();
    (43..=44).contains(&len)
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}

/// Validate mesh IP format (4 octets, each valid u8).
pub fn is_valid_mesh_ip(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn constant_time_eq_works() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
    }

    #[test]
    fn valid_wg_pubkey_format() {
        assert!(is_valid_wg_pubkey("A2fvz3czkqRUuu2mzkSS6IVr/TCQcpsJX9HbDBa1FBc="));
        assert!(!is_valid_wg_pubkey("too_short"));
        assert!(!is_valid_wg_pubkey("A2fvz3czkqRUuu2mzkSS6IVr/TCQcpsJX9Hb; rm -rf /"));
        assert!(!is_valid_wg_pubkey(""));
    }

    #[test]
    fn valid_mesh_ip_format() {
        assert!(is_valid_mesh_ip("10.13.37.20"));
        assert!(is_valid_mesh_ip("192.168.1.1"));
        assert!(!is_valid_mesh_ip("10.13.37.999"));
        assert!(!is_valid_mesh_ip("10.13.37"));
        assert!(!is_valid_mesh_ip("; rm -rf /"));
    }
}
