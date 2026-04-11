// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `security provider` JWT Client for Neural API
//!
//! Provides orchestrator-managed JWT secret provisioning from `security provider` to primals.
//! This is proper separation of concerns - the orchestrator handles integration,
//! primals just receive configuration.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// Platform-agnostic IPC transport
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;

/// Platform-agnostic connection helper
#[cfg(unix)]
async fn connect_platform(path: &str) -> std::io::Result<PlatformStream> {
    PlatformStream::connect(path).await
}

#[cfg(windows)]
async fn connect_platform(address: &str) -> std::io::Result<PlatformStream> {
    PlatformStream::connect(address).await
}
use tracing::{info, warn};

/// JWT `purpose` parameter for storage capability provider authentication (`crypto.generate.jwt_secret`).
pub const STORAGE_PROVIDER_AUTHENTICATION_PURPOSE: &str = "storage_provider_authentication";

/// Legacy JWT `purpose` id used by some deployments (prefer [`STORAGE_PROVIDER_AUTHENTICATION_PURPOSE`]).
#[deprecated(note = "use STORAGE_PROVIDER_AUTHENTICATION_PURPOSE (capability-based naming)")]
#[allow(dead_code, reason = "public wire-compat constant for legacy integrations")]
pub const NESTGATE_AUTHENTICATION_PURPOSE: &str = "nestgate_authentication";

/// Request for JWT secret generation
#[derive(Debug, Serialize)]
struct JwtSecretRequest {
    jsonrpc: String,
    method: String,
    params: JwtSecretParams,
    id: u64,
}

#[derive(Debug, Serialize)]
struct JwtSecretParams {
    purpose: String,
    strength: String,
}

/// Response from `security provider` JWT secret generation (success case)
#[derive(Debug, Deserialize)]
struct JwtSecretResponse {
    #[allow(
        dead_code,
        reason = "deserialized from JSON-RPC envelope; protocol field not read by client"
    )]
    jsonrpc: String,
    result: Option<JwtSecretResult>,
    error: Option<JsonRpcError>,
    #[allow(
        dead_code,
        reason = "deserialized from JSON-RPC envelope; correlation ID not read by client"
    )]
    id: u64,
}

/// JSON-RPC error object
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct JwtSecretResult {
    secret: String,
    #[allow(dead_code, reason = "deserialized from provider response; logged for audit trail")]
    purpose: String,
    strength: String,
    byte_length: usize,
    #[serde(default)]
    #[allow(
        dead_code,
        reason = "deserialized from provider response; available for key rotation metrics"
    )]
    encoded_length: usize,
    #[serde(default)]
    algorithm: String,
}

/// Fetch JWT secret from the security provider via JSON-RPC over Unix socket
///
/// # Arguments
/// * `socket_path` - Path to the security provider's Unix socket
/// * `purpose` - Purpose of the JWT secret (typically [`STORAGE_PROVIDER_AUTHENTICATION_PURPOSE`]; legacy peers may still expect the `nestgate_authentication` purpose string)
///
/// # Returns
/// * `Ok(String)` - Base64-encoded JWT secret (512 bits / 88 characters)
/// * `Err` - If the security provider is unavailable or request fails
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn fetch_jwt_secret_from_security_provider(
    socket_path: &str,
    purpose: &str,
) -> Result<String> {
    info!("🔐 Fetching JWT secret from security provider at: {}", socket_path);
    info!("   Purpose: {}", purpose);

    // Connect to security provider (platform-agnostic)
    let mut stream = connect_platform(socket_path)
        .await
        .context(format!("Failed to connect to security provider at {socket_path}"))?;

    // Create JSON-RPC request
    let request = JwtSecretRequest {
        jsonrpc: "2.0".to_string(),
        method: "crypto.generate.jwt_secret".to_string(),
        params: JwtSecretParams {
            purpose: purpose.to_string(),
            strength: "high".to_string(), // 512 bits, production-ready
        },
        id: 1,
    };

    // Serialize request
    let request_json =
        serde_json::to_string(&request).context("Failed to serialize JWT request")?;

    info!("   📤 Sending JSON-RPC request...");

    // Send request (JSON-RPC over Unix socket)
    stream
        .write_all(request_json.as_bytes())
        .await
        .context("Failed to write to security provider socket")?;

    stream.write_all(b"\n").await.context("Failed to write newline to security provider socket")?;

    // Read response
    let mut response_buffer = Vec::new();
    let mut read_buffer = [0u8; 4096];

    loop {
        match stream.read(&mut read_buffer).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                response_buffer.extend_from_slice(&read_buffer[..n]);
                // Check if we have a complete JSON response
                if response_buffer.contains(&b'\n') {
                    break;
                }
            }
            Err(e) => return Err(e).context("Failed to read from security provider socket"),
        }
    }

    let response_str = String::from_utf8(response_buffer)
        .context("security provider response was not valid UTF-8")?;

    info!("   📥 Received response from security provider");

    // Parse JSON-RPC response (may be success or error)
    let response: JwtSecretResponse = serde_json::from_str(response_str.trim())
        .context("Failed to parse security provider JWT response")?;

    // Check for JSON-RPC error
    if let Some(err) = response.error {
        anyhow::bail!("security provider returned error [{}]: {}", err.code, err.message);
    }

    // Extract secret from result
    let result = response.result.ok_or_else(|| {
        anyhow::anyhow!("security provider response missing both result and error")
    })?;

    let secret = result.secret;

    info!("✅ JWT secret obtained from security provider");
    info!("   Length: {} characters", secret.len());
    info!("   Strength: {} ({} bytes)", result.strength, result.byte_length);
    info!("   Algorithm: {}", result.algorithm);

    Ok(secret)
}

/// Generate secure random JWT secret as fallback
///
/// This is used when `security provider` is unavailable. Still cryptographically secure,
/// but `security provider` is preferred for consistency across NUCLEUS.
///
/// # Arguments
/// * `bytes` - Number of random bytes to generate (default: 64 for 512 bits)
///
/// # Returns
/// * Base64-encoded random secret
/// # Errors
///
/// Returns an error if the operation fails.
pub fn generate_secure_random_jwt(bytes: usize) -> Result<String> {
    use rand::RngCore;

    warn!("⚠️ Generating fallback JWT secret (security provider unavailable)");
    warn!("   This is cryptographically secure but not coordinated with NUCLEUS");

    let mut rng = rand::thread_rng();
    let mut secret_bytes = vec![0u8; bytes];
    rng.fill_bytes(&mut secret_bytes);

    use base64::Engine;
    let secret = base64::engine::general_purpose::STANDARD.encode(&secret_bytes);

    info!("✅ Secure random JWT secret generated");
    info!("   Length: {} characters ({} bytes, {} bits)", secret.len(), bytes, bytes * 8);

    Ok(secret)
}

/// Provision JWT secret for a primal
///
/// Tries `security provider` first (preferred), falls back to secure random if unavailable.
///
/// # Arguments
/// * `security_provider_socket` - Optional path to the security provider socket
/// * `purpose` - Purpose of the JWT secret
///
/// # Returns
/// * JWT secret (base64-encoded, 512 bits minimum)
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn provision_jwt_secret(
    security_provider_socket: Option<&str>,
    purpose: &str,
) -> Result<String> {
    // Try security provider first (preferred)
    if let Some(socket_path) = security_provider_socket {
        match fetch_jwt_secret_from_security_provider(socket_path, purpose).await {
            Ok(secret) => {
                info!("✅ Using security provider-provided JWT secret (preferred)");
                return Ok(secret);
            }
            Err(e) => {
                warn!("⚠️ security provider JWT fetch failed: {}", e);
                warn!("   Falling back to secure random generation...");
            }
        }
    } else {
        warn!("⚠️ No security provider socket provided, using secure random JWT");
    }

    // Secure fallback: generate cryptographically strong random
    generate_secure_random_jwt(64) // 64 bytes = 512 bits
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use base64::Engine;

    #[test]
    fn test_generate_secure_random_jwt() {
        let secret = generate_secure_random_jwt(64).unwrap();

        // Should be base64-encoded (64 bytes → ~88 characters)
        assert!(secret.len() >= 85 && secret.len() <= 90);

        // Should be different each time
        let secret2 = generate_secure_random_jwt(64).unwrap();
        assert_ne!(secret, secret2);
    }

    #[test]
    fn generate_secure_random_jwt_16_bytes_decodes() {
        let secret = generate_secure_random_jwt(16).unwrap();
        let raw = base64::engine::general_purpose::STANDARD.decode(secret.as_bytes()).unwrap();
        assert_eq!(raw.len(), 16);
    }

    #[test]
    fn generate_secure_random_jwt_zero_bytes_empty_payload() {
        let secret = generate_secure_random_jwt(0).unwrap();
        assert!(
            base64::engine::general_purpose::STANDARD.decode(secret.as_bytes()).unwrap().is_empty()
        );
    }

    #[test]
    fn jwt_secret_response_parses_success_payload() {
        let json = r#"{"jsonrpc":"2.0","result":{"secret":"c2VjcmV0","purpose":"p","strength":"high","byte_length":64,"algorithm":"HS512"},"error":null,"id":1}"#;
        let r: JwtSecretResponse = serde_json::from_str(json).unwrap();
        assert!(r.error.is_none());
        let res = r.result.unwrap();
        assert_eq!(res.secret, "c2VjcmV0");
        assert_eq!(res.strength, "high");
        assert_eq!(res.byte_length, 64);
    }

    #[test]
    fn jwt_secret_response_parses_jsonrpc_error() {
        let json = r#"{"jsonrpc":"2.0","result":null,"error":{"code":-1,"message":"fail"},"id":1}"#;
        let r: JwtSecretResponse = serde_json::from_str(json).unwrap();
        assert!(r.result.is_none());
        assert_eq!(r.error.as_ref().unwrap().code, -1);
    }

    #[test]
    fn jwt_secret_request_serializes_expected_method() {
        let req = JwtSecretRequest {
            jsonrpc: "2.0".to_string(),
            method: "crypto.generate.jwt_secret".to_string(),
            params: JwtSecretParams {
                purpose: STORAGE_PROVIDER_AUTHENTICATION_PURPOSE.to_string(),
                strength: "high".to_string(),
            },
            id: 42,
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["method"], "crypto.generate.jwt_secret");
        assert_eq!(v["params"]["purpose"], STORAGE_PROVIDER_AUTHENTICATION_PURPOSE);
    }

    #[tokio::test]
    async fn test_provision_jwt_secret_fallback() {
        // No security provider available, should fall back to secure random
        let secret = provision_jwt_secret(None, "test_purpose").await.unwrap();

        assert!(secret.len() >= 85);
        assert!(!secret.is_empty());
    }

    #[tokio::test]
    async fn provision_jwt_with_socket_path_falls_back_when_unavailable() {
        let secret =
            provision_jwt_secret(Some("/nonexistent/security-provider.sock"), "p").await.unwrap();
        assert!(secret.len() >= 32);
    }
}
