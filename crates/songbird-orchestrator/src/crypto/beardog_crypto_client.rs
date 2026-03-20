// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `BearDog` Crypto Client for TLS Delegation
//!
//! Provides Pure Rust TLS by delegating ALL crypto operations to `BearDog`.
//! This achieves 100% Pure Rust (TRUE ecoBin) by leveraging `BearDog`'s
//! RustCrypto-based primitives via JSON-RPC over Unix sockets.
//!
//! **Architecture**:
//! - Songbird: TLS protocol logic (Pure Rust state machine)
//! - `BearDog`: ALL crypto operations (Pure Rust `RustCrypto`!)
//! - Result: 100% Pure Rust HTTPS!
//!
//! **Pattern**: Generic `call_beardog_rpc` eliminates per-operation boilerplate.

use anyhow::{Context, Result};
use base64::Engine;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::debug;

// Platform-agnostic IPC transport
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;

/// Standard base64 encoder/decoder
const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

/// Platform-agnostic connection helper
#[cfg(unix)]
async fn connect_platform(path: &str) -> std::io::Result<PlatformStream> {
    PlatformStream::connect(path).await
}

#[cfg(windows)]
async fn connect_platform(address: &str) -> std::io::Result<PlatformStream> {
    PlatformStream::connect(address).await
}

// ============================================================================
// Generic JSON-RPC helper — eliminates all per-operation boilerplate
// ============================================================================

/// Call a `BearDog` crypto RPC method
///
/// Handles connection, JSON-RPC framing, and error extraction.
/// Each crypto function only needs to construct params and decode the result.
async fn call_beardog_rpc(
    socket_path: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let mut stream = connect_platform(socket_path)
        .await
        .context(format!("Failed to connect to BearDog crypto at {socket_path}"))?;

    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    });

    let request_json = serde_json::to_string(&request)?;
    stream.write_all(request_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    let response_str = read_json_rpc_response(&mut stream).await?;

    let response: serde_json::Value = serde_json::from_str(&response_str)
        .context(format!("Failed to parse BearDog {method} response"))?;

    // Check for JSON-RPC error
    if let Some(err) = response.get("error") {
        anyhow::bail!("BearDog RPC error for {method}: {err}");
    }

    response
        .get("result")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Missing 'result' field in BearDog {method} response"))
}

/// Extract a base64-encoded string field from a JSON value and decode it
fn decode_b64_field(value: &serde_json::Value, field: &str, context: &str) -> Result<Vec<u8>> {
    let encoded = value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing '{field}' in {context} response"))?;
    B64.decode(encoded).context(format!("Failed to decode {field} from {context}"))
}

// ============================================================================
// Ed25519 Signing
// ============================================================================

/// Sign a message with Ed25519 via `BearDog`
///
/// # Arguments
/// * `socket_path` - Path to `BearDog`'s crypto Unix socket
/// * `message` - Message bytes to sign
/// * `key_id` - Key identifier (e.g., "`tls_signing_key`")
/// * `purpose` - Purpose for audit logging (e.g., "`tls_handshake`")
///
/// # Errors
///
/// Returns an error if `BearDog` is unavailable or signing fails.
pub async fn sign_ed25519(
    socket_path: &str,
    message: &[u8],
    key_id: &str,
    purpose: &str,
) -> Result<Vec<u8>> {
    debug!("🔐 Signing with Ed25519 via BearDog (key: {key_id})");

    let result = call_beardog_rpc(
        socket_path,
        "crypto.sign_ed25519",
        json!({
            "message": B64.encode(message),
            "key_id": key_id,
            "purpose": purpose,
        }),
    )
    .await?;

    let signature = decode_b64_field(&result, "signature", "Ed25519 sign")?;
    debug!("✅ Ed25519 signature obtained ({} bytes)", signature.len());
    Ok(signature)
}

// ============================================================================
// Ed25519 Verification
// ============================================================================

/// Verify an Ed25519 signature via `BearDog`
///
/// # Arguments
/// * `socket_path` - Path to `BearDog`'s crypto Unix socket
/// * `message` - Message bytes that were signed
/// * `signature` - Signature bytes to verify
/// * `public_key` - Public key bytes
///
/// # Errors
///
/// Returns an error if `BearDog` is unavailable or verification fails.
pub async fn verify_ed25519(
    socket_path: &str,
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> Result<bool> {
    debug!("🔍 Verifying Ed25519 signature via BearDog");

    let result = call_beardog_rpc(
        socket_path,
        "crypto.verify_ed25519",
        json!({
            "message": B64.encode(message),
            "signature": B64.encode(signature),
            "public_key": B64.encode(public_key),
        }),
    )
    .await?;

    let valid = result
        .get("valid")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow::anyhow!("Missing 'valid' in Ed25519 verify response"))?;

    debug!("✅ Ed25519 verification result: {valid}");
    Ok(valid)
}

// ============================================================================
// X25519 Key Exchange
// ============================================================================

/// Generate ephemeral X25519 key pair via `BearDog`
///
/// **`BearDog`'s Stateless Design**: Returns actual key bytes, not IDs!
/// The caller is responsible for managing the secret key securely.
///
/// # Arguments
/// * `socket_path` - Path to `BearDog`'s crypto Unix socket
/// * `purpose` - Purpose for audit logging (e.g., "`tls_key_exchange`")
///
/// # Errors
///
/// Returns an error if `BearDog` is unavailable or generation fails.
pub async fn x25519_generate_ephemeral(
    socket_path: &str,
    purpose: &str,
) -> Result<(Vec<u8>, Vec<u8>)> {
    debug!("🔑 Generating X25519 ephemeral key pair via BearDog");

    let result = call_beardog_rpc(
        socket_path,
        "crypto.x25519_generate_ephemeral",
        json!({ "purpose": purpose }),
    )
    .await?;

    let public_key = decode_b64_field(&result, "public_key", "X25519 generate")?;
    let secret_key = decode_b64_field(&result, "secret_key", "X25519 generate")?;

    debug!("✅ X25519 ephemeral key pair generated (stateless, {} bytes each)", public_key.len());
    Ok((public_key, secret_key))
}

/// Derive X25519 shared secret via `BearDog`
///
/// **`BearDog`'s Stateless Design**: Pass actual key bytes, not IDs!
///
/// # Arguments
/// * `socket_path` - Path to `BearDog`'s crypto Unix socket
/// * `our_secret_key` - Our secret key bytes (from `x25519_generate_ephemeral`)
/// * `their_public_key` - Their public key bytes
///
/// # Errors
///
/// Returns an error if `BearDog` is unavailable or derivation fails.
pub async fn x25519_derive_secret(
    socket_path: &str,
    our_secret_key: &[u8],
    their_public_key: &[u8],
) -> Result<Vec<u8>> {
    debug!("🤝 Deriving X25519 shared secret via BearDog");

    let result = call_beardog_rpc(
        socket_path,
        "crypto.x25519_derive_secret",
        json!({
            "our_secret": B64.encode(our_secret_key),
            "their_public": B64.encode(their_public_key),
        }),
    )
    .await?;

    let shared_secret = decode_b64_field(&result, "shared_secret", "X25519 derive")?;
    debug!("✅ X25519 shared secret derived ({} bytes)", shared_secret.len());
    Ok(shared_secret)
}

// ============================================================================
// ChaCha20-Poly1305 AEAD
// ============================================================================

/// Encrypt with ChaCha20-Poly1305 AEAD via `BearDog`
///
/// **`BearDog`'s Secure Design**: Generates nonce automatically!
/// This ensures cryptographically secure, non-repeating nonces.
///
/// # Arguments
/// * `socket_path` - Path to `BearDog`'s crypto Unix socket
/// * `plaintext` - Plaintext bytes to encrypt
/// * `key` - Encryption key (32 bytes)
/// * `aad` - Additional authenticated data (optional)
///
/// # Errors
///
/// Returns an error if `BearDog` is unavailable or encryption fails.
pub async fn chacha20_poly1305_encrypt(
    socket_path: &str,
    plaintext: &[u8],
    key: &[u8],
    aad: Option<&[u8]>,
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    debug!("🔒 Encrypting with ChaCha20-Poly1305 via BearDog ({} bytes)", plaintext.len());

    let result = call_beardog_rpc(
        socket_path,
        "crypto.chacha20_poly1305_encrypt",
        json!({
            "plaintext": B64.encode(plaintext),
            "key": B64.encode(key),
            "nonce": "",
            "aad": aad.map(|a| B64.encode(a)).unwrap_or_default(),
        }),
    )
    .await?;

    let ciphertext = decode_b64_field(&result, "ciphertext", "ChaCha20 encrypt")?;
    let nonce = decode_b64_field(&result, "nonce", "ChaCha20 encrypt")?;
    let tag = decode_b64_field(&result, "tag", "ChaCha20 encrypt")?;

    debug!(
        "✅ ChaCha20-Poly1305 encryption complete ({} bytes ct, {} bytes nonce, {} bytes tag)",
        ciphertext.len(),
        nonce.len(),
        tag.len()
    );
    Ok((ciphertext, nonce, tag))
}

/// Decrypt with ChaCha20-Poly1305 AEAD via `BearDog`
///
/// # Arguments
/// * `socket_path` - Path to `BearDog`'s crypto Unix socket
/// * `ciphertext` - Ciphertext bytes to decrypt
/// * `key` - Decryption key (32 bytes)
/// * `nonce` - Nonce (12 bytes, from encryption)
/// * `tag` - Authentication tag (16 bytes, from encryption)
/// * `aad` - Additional authenticated data (must match encryption AAD)
///
/// # Errors
///
/// Returns an error if `BearDog` is unavailable, decryption fails, or auth tag is invalid.
pub async fn chacha20_poly1305_decrypt(
    socket_path: &str,
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
    tag: &[u8],
    aad: Option<&[u8]>,
) -> Result<Vec<u8>> {
    debug!("🔓 Decrypting with ChaCha20-Poly1305 via BearDog ({} bytes)", ciphertext.len());

    let result = call_beardog_rpc(
        socket_path,
        "crypto.chacha20_poly1305_decrypt",
        json!({
            "ciphertext": B64.encode(ciphertext),
            "key": B64.encode(key),
            "nonce": B64.encode(nonce),
            "tag": B64.encode(tag),
            "aad": aad.map(|a| B64.encode(a)).unwrap_or_default(),
        }),
    )
    .await?;

    let plaintext = decode_b64_field(&result, "plaintext", "ChaCha20 decrypt")?;
    debug!("✅ ChaCha20-Poly1305 decryption complete ({} bytes)", plaintext.len());
    Ok(plaintext)
}

// ============================================================================
// Blake3 Hashing
// ============================================================================

/// Hash data with Blake3 via `BearDog`
///
/// # Arguments
/// * `socket_path` - Path to `BearDog`'s crypto Unix socket
/// * `data` - Data bytes to hash
///
/// # Errors
///
/// Returns an error if `BearDog` is unavailable or hashing fails.
pub async fn blake3_hash(socket_path: &str, data: &[u8]) -> Result<Vec<u8>> {
    debug!("# Hashing with Blake3 via BearDog ({} bytes)", data.len());

    let result =
        call_beardog_rpc(socket_path, "crypto.blake3_hash", json!({ "data": B64.encode(data) }))
            .await?;

    let hash = decode_b64_field(&result, "hash", "Blake3 hash")?;
    debug!("✅ Blake3 hash complete ({} bytes)", hash.len());
    Ok(hash)
}

// ============================================================================
// HMAC-SHA256
// ============================================================================

/// Compute HMAC-SHA256 via `BearDog`
///
/// # Arguments
/// * `socket_path` - Path to `BearDog`'s crypto Unix socket
/// * `key` - HMAC key bytes
/// * `data` - Data bytes to MAC
///
/// # Errors
///
/// Returns an error if `BearDog` is unavailable or HMAC computation fails.
pub async fn hmac_sha256(socket_path: &str, key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    debug!("🔏 Computing HMAC-SHA256 via BearDog ({} bytes)", data.len());

    let result = call_beardog_rpc(
        socket_path,
        "crypto.hmac_sha256",
        json!({
            "key": B64.encode(key),
            "data": B64.encode(data),
        }),
    )
    .await?;

    // BearDog uses "mac" not "hmac" in the response
    let hmac = decode_b64_field(&result, "mac", "HMAC-SHA256")?;
    debug!("✅ HMAC-SHA256 complete ({} bytes)", hmac.len());
    Ok(hmac)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Read a JSON-RPC response from a stream (platform-agnostic)
///
/// Reads until newline, handles buffering and EOF correctly.
async fn read_json_rpc_response(stream: &mut PlatformStream) -> Result<String> {
    let mut response_buffer = Vec::new();
    let mut read_buffer = [0u8; 4096];

    loop {
        match stream.read(&mut read_buffer).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                response_buffer.extend_from_slice(&read_buffer[..n]);
                // Check for complete JSON response (newline-delimited)
                if response_buffer.contains(&b'\n') {
                    break;
                }
            }
            Err(e) => return Err(e).context("Failed to read from BearDog socket"),
        }
    }

    String::from_utf8(response_buffer).context("BearDog response was not valid UTF-8")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: These tests require a running BearDog instance with crypto API
    // They are marked #[ignore] to avoid CI failures
    // Run with: cargo test --package songbird-orchestrator -- crypto::beardog_crypto_client --ignored

    #[tokio::test]
    #[ignore]
    async fn test_sign_ed25519() {
        let socket = super::super::discovery::get_beardog_crypto_socket()
            .await
            .expect("BearDog crypto socket not found - is BearDog running?");
        let message = b"Hello, World!";
        let key_id = "test_key";
        let purpose = "test";

        let signature = sign_ed25519(&socket, message, key_id, purpose).await;
        assert!(signature.is_ok());
        assert_eq!(signature.unwrap().len(), 64); // Ed25519 signature is 64 bytes
    }

    #[tokio::test]
    #[ignore]
    async fn test_x25519_key_exchange() {
        let socket = super::super::discovery::get_beardog_crypto_socket()
            .await
            .expect("BearDog crypto socket not found - is BearDog running?");
        let purpose = "test_key_exchange";

        // Generate ephemeral key pair
        let (public_key, secret_key) = x25519_generate_ephemeral(&socket, purpose).await.unwrap();
        assert_eq!(public_key.len(), 32); // X25519 public key is 32 bytes
        assert!(!secret_key.is_empty());

        // Derive shared secret (using our own public key for testing)
        let shared_secret = x25519_derive_secret(&socket, &secret_key, &public_key).await;
        assert!(shared_secret.is_ok());
        assert_eq!(shared_secret.unwrap().len(), 32); // X25519 shared secret is 32 bytes
    }

    #[tokio::test]
    #[ignore]
    async fn test_chacha20_poly1305_round_trip() {
        let socket = super::super::discovery::get_beardog_crypto_socket()
            .await
            .expect("BearDog crypto socket not found - is BearDog running?");
        let key = [42u8; 32]; // Test key
        let plaintext = b"Hello, encrypted world!";
        let aad = b"additional data";

        // Encrypt
        let (ciphertext, nonce, tag) =
            chacha20_poly1305_encrypt(&socket, plaintext, &key, Some(aad)).await.unwrap();

        assert!(!ciphertext.is_empty());
        assert_eq!(nonce.len(), 12); // ChaCha20-Poly1305 nonce is 12 bytes
        assert_eq!(tag.len(), 16); // Poly1305 tag is 16 bytes

        // Decrypt
        let decrypted =
            chacha20_poly1305_decrypt(&socket, &ciphertext, &key, &nonce, &tag, Some(aad))
                .await
                .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    #[ignore]
    async fn test_blake3_hash() {
        let socket = super::super::discovery::get_beardog_crypto_socket()
            .await
            .expect("BearDog crypto socket not found - is BearDog running?");
        let data = b"Hello, world!";

        let hash = blake3_hash(&socket, data).await.unwrap();
        assert_eq!(hash.len(), 32); // Blake3 hash is always 32 bytes
    }

    #[tokio::test]
    #[ignore]
    async fn test_hmac_sha256() {
        let socket = super::super::discovery::get_beardog_crypto_socket()
            .await
            .expect("BearDog crypto socket not found - is BearDog running?");
        let key = b"test_key_material";
        let data = b"test data";

        let mac = hmac_sha256(&socket, key, data).await.unwrap();
        assert_eq!(mac.len(), 32); // HMAC-SHA256 is always 32 bytes
    }

    #[test]
    fn test_b64_constant() {
        // Verify the B64 constant works correctly
        let data = b"Hello, World!";
        let encoded = B64.encode(data);
        let decoded = B64.decode(&encoded).unwrap();
        assert_eq!(&decoded, data);
    }
}
