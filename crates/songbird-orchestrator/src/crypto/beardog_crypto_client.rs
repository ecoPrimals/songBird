//! BearDog Crypto Client for TLS Delegation
//!
//! Provides Pure Rust TLS by delegating ALL crypto operations to BearDog.
//! This achieves 100% Pure Rust (TRUE ecoBin) by leveraging BearDog's
//! RustCrypto-based primitives via JSON-RPC over Unix sockets.
//!
//! **Architecture**:
//! - Songbird: TLS protocol logic (Pure Rust state machine)
//! - BearDog: ALL crypto operations (Pure Rust RustCrypto!)
//! - Result: 100% Pure Rust HTTPS! 🎉
//!
//! **Pattern**: Proven from `beardog_jwt_client.rs` (production-tested)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// Platform-agnostic IPC transport
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;

/// Platform-agnostic connection helper
#[cfg(unix)]
async fn connect_platform(path: &str) -> std::io::Result<PlatformStream> {
    PlatformStream::connect(path).await
}

#[cfg(windows)]
async fn connect_platform(address: &str) -> std::io::Result<PlatformStream> {
    PlatformStream::connect(address).await
}
use tracing::debug;

// ============================================================================
// Ed25519 Signing
// ============================================================================

#[derive(Debug, Serialize)]
struct Ed25519SignRequest {
    jsonrpc: String,
    method: String,
    params: Ed25519SignParams,
    id: u64,
}

#[derive(Debug, Serialize)]
struct Ed25519SignParams {
    message: String, // base64-encoded
    key_id: String,
    purpose: String,
}

#[derive(Debug, Deserialize)]
struct Ed25519SignResponse {
    jsonrpc: String,
    result: Ed25519SignResult,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct Ed25519SignResult {
    signature: String, // base64-encoded
}

/// Sign a message with Ed25519 via BearDog
///
/// # Arguments
/// * `socket_path` - Path to BearDog's crypto Unix socket
/// * `message` - Message bytes to sign
/// * `key_id` - Key identifier (e.g., "tls_signing_key")
/// * `purpose` - Purpose for audit logging (e.g., "tls_handshake")
///
/// # Returns
/// * `Ok(Vec<u8>)` - Signature bytes
/// * `Err` - If BearDog is unavailable or signing fails
pub async fn sign_ed25519(
    socket_path: &str,
    message: &[u8],
    key_id: &str,
    purpose: &str,
) -> Result<Vec<u8>> {
    debug!("🔐 Signing with Ed25519 via BearDog (key: {})", key_id);

    // Connect to BearDog
    let mut stream = connect_platform(socket_path)
        .await
        .context(format!("Failed to connect to BearDog crypto at {}", socket_path))?;

    // Encode message to base64
    use base64::Engine;
    let message_b64 = base64::engine::general_purpose::STANDARD.encode(message);

    // Create JSON-RPC request
    let request = Ed25519SignRequest {
        jsonrpc: "2.0".to_string(),
        method: "crypto.sign_ed25519".to_string(),
        params: Ed25519SignParams {
            message: message_b64,
            key_id: key_id.to_string(),
            purpose: purpose.to_string(),
        },
        id: 1,
    };

    // Send request
    let request_json = serde_json::to_string(&request)?;
    stream.write_all(request_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    // Read response
    let response_str = read_json_rpc_response(&mut stream).await?;

    // Parse response
    let response: Ed25519SignResponse = serde_json::from_str(&response_str)
        .context("Failed to parse BearDog Ed25519 sign response")?;

    // Decode signature from base64
    let signature = base64::engine::general_purpose::STANDARD
        .decode(&response.result.signature)
        .context("Failed to decode Ed25519 signature")?;

    debug!("✅ Ed25519 signature obtained ({} bytes)", signature.len());

    Ok(signature)
}

// ============================================================================
// Ed25519 Verification
// ============================================================================

#[derive(Debug, Serialize)]
struct Ed25519VerifyRequest {
    jsonrpc: String,
    method: String,
    params: Ed25519VerifyParams,
    id: u64,
}

#[derive(Debug, Serialize)]
struct Ed25519VerifyParams {
    message: String,    // base64-encoded
    signature: String,  // base64-encoded
    public_key: String, // base64-encoded
}

#[derive(Debug, Deserialize)]
struct Ed25519VerifyResponse {
    jsonrpc: String,
    result: Ed25519VerifyResult,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct Ed25519VerifyResult {
    valid: bool,
}

/// Verify an Ed25519 signature via BearDog
///
/// # Arguments
/// * `socket_path` - Path to BearDog's crypto Unix socket
/// * `message` - Message bytes that were signed
/// * `signature` - Signature bytes to verify
/// * `public_key` - Public key bytes
///
/// # Returns
/// * `Ok(true)` - Signature is valid
/// * `Ok(false)` - Signature is invalid
/// * `Err` - If BearDog is unavailable or verification fails
pub async fn verify_ed25519(
    socket_path: &str,
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> Result<bool> {
    debug!("🔍 Verifying Ed25519 signature via BearDog");

    // Connect to BearDog
    let mut stream = connect_platform(socket_path)
        .await
        .context(format!("Failed to connect to BearDog crypto at {}", socket_path))?;

    // Encode to base64
    use base64::Engine;
    let encoder = base64::engine::general_purpose::STANDARD;
    let message_b64 = encoder.encode(message);
    let signature_b64 = encoder.encode(signature);
    let public_key_b64 = encoder.encode(public_key);

    // Create JSON-RPC request
    let request = Ed25519VerifyRequest {
        jsonrpc: "2.0".to_string(),
        method: "crypto.verify_ed25519".to_string(),
        params: Ed25519VerifyParams {
            message: message_b64,
            signature: signature_b64,
            public_key: public_key_b64,
        },
        id: 2,
    };

    // Send request
    let request_json = serde_json::to_string(&request)?;
    stream.write_all(request_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    // Read response
    let response_str = read_json_rpc_response(&mut stream).await?;

    // Parse response
    let response: Ed25519VerifyResponse = serde_json::from_str(&response_str)
        .context("Failed to parse BearDog Ed25519 verify response")?;

    debug!("✅ Ed25519 verification result: {}", response.result.valid);

    Ok(response.result.valid)
}

// ============================================================================
// X25519 Key Exchange
// ============================================================================

#[derive(Debug, Serialize)]
struct X25519GenerateRequest {
    jsonrpc: String,
    method: String,
    params: X25519GenerateParams,
    id: u64,
}

#[derive(Debug, Serialize)]
struct X25519GenerateParams {
    purpose: String,
}

#[derive(Debug, Deserialize)]
struct X25519GenerateResponse {
    jsonrpc: String,
    result: X25519GenerateResult,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct X25519GenerateResult {
    public_key: String, // base64-encoded
    secret_key: String, // base64-encoded (BearDog is stateless!)
}

/// Generate ephemeral X25519 key pair via BearDog
///
/// **BearDog's Stateless Design**: Returns actual key bytes, not IDs!
/// The caller is responsible for managing the secret key securely.
///
/// # Arguments
/// * `socket_path` - Path to BearDog's crypto Unix socket
/// * `purpose` - Purpose for audit logging (e.g., "tls_key_exchange")
///
/// # Returns
/// * `Ok((public_key, secret_key))` - Both keys as bytes (32 bytes each)
/// * `Err` - If BearDog is unavailable or generation fails
pub async fn x25519_generate_ephemeral(
    socket_path: &str,
    purpose: &str,
) -> Result<(Vec<u8>, Vec<u8>)> {
    debug!("🔑 Generating X25519 ephemeral key pair via BearDog");

    // Connect to BearDog
    let mut stream = connect_platform(socket_path)
        .await
        .context(format!("Failed to connect to BearDog crypto at {}", socket_path))?;

    // Create JSON-RPC request
    let request = X25519GenerateRequest {
        jsonrpc: "2.0".to_string(),
        method: "crypto.x25519_generate_ephemeral".to_string(),
        params: X25519GenerateParams {
            purpose: purpose.to_string(),
        },
        id: 3,
    };

    // Send request
    let request_json = serde_json::to_string(&request)?;
    stream.write_all(request_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    // Read response
    let response_str = read_json_rpc_response(&mut stream).await?;

    // Parse response
    let response: X25519GenerateResponse = serde_json::from_str(&response_str)
        .context("Failed to parse BearDog X25519 generate response")?;

    // Decode both keys from base64
    use base64::Engine;
    let public_key = base64::engine::general_purpose::STANDARD
        .decode(&response.result.public_key)
        .context("Failed to decode X25519 public key")?;

    let secret_key = base64::engine::general_purpose::STANDARD
        .decode(&response.result.secret_key)
        .context("Failed to decode X25519 secret key")?;

    debug!("✅ X25519 ephemeral key pair generated (stateless, {} bytes each)", public_key.len());

    Ok((public_key, secret_key))
}

#[derive(Debug, Serialize)]
struct X25519DeriveRequest {
    jsonrpc: String,
    method: String,
    params: X25519DeriveParams,
    id: u64,
}

#[derive(Debug, Serialize)]
struct X25519DeriveParams {
    our_secret: String,   // base64-encoded secret key bytes (stateless!)
    their_public: String, // base64-encoded public key
}

#[derive(Debug, Deserialize)]
struct X25519DeriveResponse {
    jsonrpc: String,
    result: X25519DeriveResult,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct X25519DeriveResult {
    shared_secret: String, // base64-encoded
}

/// Derive X25519 shared secret via BearDog
///
/// **BearDog's Stateless Design**: Pass actual key bytes, not IDs!
///
/// # Arguments
/// * `socket_path` - Path to BearDog's crypto Unix socket
/// * `our_secret_key` - Our secret key bytes (from generate_ephemeral)
/// * `their_public_key` - Their public key bytes
///
/// # Returns
/// * `Ok(shared_secret)` - Shared secret bytes (32 bytes)
/// * `Err` - If BearDog is unavailable or derivation fails
pub async fn x25519_derive_secret(
    socket_path: &str,
    our_secret_key: &[u8],
    their_public_key: &[u8],
) -> Result<Vec<u8>> {
    debug!("🤝 Deriving X25519 shared secret via BearDog");

    // Connect to BearDog
    let mut stream = connect_platform(socket_path)
        .await
        .context(format!("Failed to connect to BearDog crypto at {}", socket_path))?;

    // Encode both keys to base64
    use base64::Engine;
    let our_secret_b64 = base64::engine::general_purpose::STANDARD.encode(our_secret_key);
    let their_public_b64 = base64::engine::general_purpose::STANDARD.encode(their_public_key);

    // Create JSON-RPC request
    let request = X25519DeriveRequest {
        jsonrpc: "2.0".to_string(),
        method: "crypto.x25519_derive_secret".to_string(),
        params: X25519DeriveParams {
            our_secret: our_secret_b64,
            their_public: their_public_b64,
        },
        id: 4,
    };

    // Send request
    let request_json = serde_json::to_string(&request)?;
    stream.write_all(request_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    // Read response
    let response_str = read_json_rpc_response(&mut stream).await?;

    // Parse response
    let response: X25519DeriveResponse = serde_json::from_str(&response_str)
        .context("Failed to parse BearDog X25519 derive response")?;

    // Decode shared secret from base64
    let shared_secret = base64::engine::general_purpose::STANDARD
        .decode(&response.result.shared_secret)
        .context("Failed to decode X25519 shared secret")?;

    debug!("✅ X25519 shared secret derived ({} bytes)", shared_secret.len());

    Ok(shared_secret)
}

// ============================================================================
// ChaCha20-Poly1305 AEAD
// ============================================================================

#[derive(Debug, Serialize)]
struct ChaChaEncryptRequest {
    jsonrpc: String,
    method: String,
    params: ChaChaEncryptParams,
    id: u64,
}

#[derive(Debug, Serialize)]
struct ChaChaEncryptParams {
    plaintext: String, // base64-encoded
    key: String,       // base64-encoded
    nonce: String,     // base64-encoded
    aad: String,       // base64-encoded (additional authenticated data)
}

#[derive(Debug, Deserialize)]
struct ChaChaEncryptResponse {
    jsonrpc: String,
    result: ChaChaEncryptResult,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct ChaChaEncryptResult {
    ciphertext: String, // base64-encoded
    nonce: String,      // base64-encoded (BearDog generates!)
    tag: String,        // base64-encoded auth tag
}

/// Encrypt with ChaCha20-Poly1305 AEAD via BearDog
///
/// **BearDog's Secure Design**: Generates nonce automatically!
/// This ensures cryptographically secure, non-repeating nonces.
///
/// # Arguments
/// * `socket_path` - Path to BearDog's crypto Unix socket
/// * `plaintext` - Plaintext bytes to encrypt
/// * `key` - Encryption key (32 bytes)
/// * `aad` - Additional authenticated data (optional)
///
/// # Returns
/// * `Ok((ciphertext, nonce, tag))` - Ciphertext, nonce (12 bytes), and auth tag (16 bytes)
/// * `Err` - If BearDog is unavailable or encryption fails
pub async fn chacha20_poly1305_encrypt(
    socket_path: &str,
    plaintext: &[u8],
    key: &[u8],
    aad: Option<&[u8]>,
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    debug!("🔒 Encrypting with ChaCha20-Poly1305 via BearDog ({} bytes)", plaintext.len());

    // Connect to BearDog
    let mut stream = connect_platform(socket_path)
        .await
        .context(format!("Failed to connect to BearDog crypto at {}", socket_path))?;

    // Encode to base64
    use base64::Engine;
    let encoder = base64::engine::general_purpose::STANDARD;
    let plaintext_b64 = encoder.encode(plaintext);
    let key_b64 = encoder.encode(key);
    let aad_b64 = aad.map(|a| encoder.encode(a)).unwrap_or_default();

    // Create JSON-RPC request (no nonce - BearDog generates it!)
    let request = ChaChaEncryptRequest {
        jsonrpc: "2.0".to_string(),
        method: "crypto.chacha20_poly1305_encrypt".to_string(),
        params: ChaChaEncryptParams {
            plaintext: plaintext_b64,
            key: key_b64,
            nonce: String::new(), // Empty - BearDog will generate
            aad: aad_b64,
        },
        id: 5,
    };

    // Send request
    let request_json = serde_json::to_string(&request)?;
    stream.write_all(request_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    // Read response
    let response_str = read_json_rpc_response(&mut stream).await?;

    // Parse response
    let response: ChaChaEncryptResponse = serde_json::from_str(&response_str)
        .context("Failed to parse BearDog ChaCha20-Poly1305 encrypt response")?;

    // Decode all parts from base64
    let ciphertext = encoder
        .decode(&response.result.ciphertext)
        .context("Failed to decode ChaCha20-Poly1305 ciphertext")?;

    let nonce = encoder
        .decode(&response.result.nonce)
        .context("Failed to decode ChaCha20-Poly1305 nonce")?;

    let tag =
        encoder.decode(&response.result.tag).context("Failed to decode ChaCha20-Poly1305 tag")?;

    debug!("✅ ChaCha20-Poly1305 encryption complete ({} bytes ciphertext, {} bytes nonce, {} bytes tag)", 
           ciphertext.len(), nonce.len(), tag.len());

    Ok((ciphertext, nonce, tag))
}

#[derive(Debug, Serialize)]
struct ChaChaDecryptRequest {
    jsonrpc: String,
    method: String,
    params: ChaChaDecryptParams,
    id: u64,
}

#[derive(Debug, Serialize)]
struct ChaChaDecryptParams {
    ciphertext: String, // base64-encoded
    key: String,        // base64-encoded
    nonce: String,      // base64-encoded
    tag: String,        // base64-encoded auth tag
    aad: String,        // base64-encoded (optional)
}

#[derive(Debug, Deserialize)]
struct ChaChaDecryptResponse {
    jsonrpc: String,
    result: ChaChaDecryptResult,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct ChaChaDecryptResult {
    plaintext: String, // base64-encoded
}

/// Decrypt with ChaCha20-Poly1305 AEAD via BearDog
///
/// # Arguments
/// * `socket_path` - Path to BearDog's crypto Unix socket
/// * `ciphertext` - Ciphertext bytes to decrypt
/// * `key` - Decryption key (32 bytes)
/// * `nonce` - Nonce (12 bytes, from encryption)
/// * `tag` - Authentication tag (16 bytes, from encryption)
/// * `aad` - Additional authenticated data (must match encryption AAD)
///
/// # Returns
/// * `Ok(plaintext)` - Plaintext bytes
/// * `Err` - If BearDog is unavailable, decryption fails, or auth tag is invalid
pub async fn chacha20_poly1305_decrypt(
    socket_path: &str,
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
    tag: &[u8],
    aad: Option<&[u8]>,
) -> Result<Vec<u8>> {
    debug!("🔓 Decrypting with ChaCha20-Poly1305 via BearDog ({} bytes)", ciphertext.len());

    // Connect to BearDog
    let mut stream = connect_platform(socket_path)
        .await
        .context(format!("Failed to connect to BearDog crypto at {}", socket_path))?;

    // Encode to base64
    use base64::Engine;
    let encoder = base64::engine::general_purpose::STANDARD;
    let ciphertext_b64 = encoder.encode(ciphertext);
    let key_b64 = encoder.encode(key);
    let nonce_b64 = encoder.encode(nonce);
    let tag_b64 = encoder.encode(tag);
    let aad_b64 = aad.map(|a| encoder.encode(a)).unwrap_or_default();

    // Create JSON-RPC request
    let request = ChaChaDecryptRequest {
        jsonrpc: "2.0".to_string(),
        method: "crypto.chacha20_poly1305_decrypt".to_string(),
        params: ChaChaDecryptParams {
            ciphertext: ciphertext_b64,
            key: key_b64,
            nonce: nonce_b64,
            tag: tag_b64,
            aad: aad_b64,
        },
        id: 6,
    };

    // Send request
    let request_json = serde_json::to_string(&request)?;
    stream.write_all(request_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    // Read response
    let response_str = read_json_rpc_response(&mut stream).await?;

    // Parse response
    let response: ChaChaDecryptResponse = serde_json::from_str(&response_str)
        .context("Failed to parse BearDog ChaCha20-Poly1305 decrypt response")?;

    // Decode plaintext from base64
    let plaintext = encoder
        .decode(&response.result.plaintext)
        .context("Failed to decode ChaCha20-Poly1305 plaintext")?;

    debug!("✅ ChaCha20-Poly1305 decryption complete ({} bytes)", plaintext.len());

    Ok(plaintext)
}

// ============================================================================
// Blake3 Hashing
// ============================================================================

#[derive(Debug, Serialize)]
struct Blake3HashRequest {
    jsonrpc: String,
    method: String,
    params: Blake3HashParams,
    id: u64,
}

#[derive(Debug, Serialize)]
struct Blake3HashParams {
    data: String, // base64-encoded
}

#[derive(Debug, Deserialize)]
struct Blake3HashResponse {
    jsonrpc: String,
    result: Blake3HashResult,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct Blake3HashResult {
    hash: String, // base64-encoded
}

/// Hash data with Blake3 via BearDog
///
/// # Arguments
/// * `socket_path` - Path to BearDog's crypto Unix socket
/// * `data` - Data bytes to hash
///
/// # Returns
/// * `Ok(hash)` - Blake3 hash bytes (32 bytes)
/// * `Err` - If BearDog is unavailable or hashing fails
pub async fn blake3_hash(socket_path: &str, data: &[u8]) -> Result<Vec<u8>> {
    debug!("# Hashing with Blake3 via BearDog ({} bytes)", data.len());

    // Connect to BearDog
    let mut stream = connect_platform(socket_path)
        .await
        .context(format!("Failed to connect to BearDog crypto at {}", socket_path))?;

    // Encode to base64
    use base64::Engine;
    let data_b64 = base64::engine::general_purpose::STANDARD.encode(data);

    // Create JSON-RPC request
    let request = Blake3HashRequest {
        jsonrpc: "2.0".to_string(),
        method: "crypto.blake3_hash".to_string(),
        params: Blake3HashParams {
            data: data_b64,
        },
        id: 7,
    };

    // Send request
    let request_json = serde_json::to_string(&request)?;
    stream.write_all(request_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    // Read response
    let response_str = read_json_rpc_response(&mut stream).await?;

    // Parse response
    let response: Blake3HashResponse = serde_json::from_str(&response_str)
        .context("Failed to parse BearDog Blake3 hash response")?;

    // Decode hash from base64
    let hash = base64::engine::general_purpose::STANDARD
        .decode(&response.result.hash)
        .context("Failed to decode Blake3 hash")?;

    debug!("✅ Blake3 hash complete ({} bytes)", hash.len());

    Ok(hash)
}

// ============================================================================
// HMAC-SHA256
// ============================================================================

#[derive(Debug, Serialize)]
struct HmacSha256Request {
    jsonrpc: String,
    method: String,
    params: HmacSha256Params,
    id: u64,
}

#[derive(Debug, Serialize)]
struct HmacSha256Params {
    key: String,  // base64-encoded
    data: String, // base64-encoded
}

#[derive(Debug, Deserialize)]
struct HmacSha256Response {
    jsonrpc: String,
    result: HmacSha256Result,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct HmacSha256Result {
    mac: String, // base64-encoded (BearDog uses "mac" not "hmac")
}

/// Compute HMAC-SHA256 via BearDog
///
/// # Arguments
/// * `socket_path` - Path to BearDog's crypto Unix socket
/// * `key` - HMAC key bytes
/// * `data` - Data bytes to MAC
///
/// # Returns
/// * `Ok(hmac)` - HMAC-SHA256 bytes (32 bytes)
/// * `Err` - If BearDog is unavailable or HMAC computation fails
pub async fn hmac_sha256(socket_path: &str, key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    debug!("🔏 Computing HMAC-SHA256 via BearDog ({} bytes)", data.len());

    // Connect to BearDog
    let mut stream = connect_platform(socket_path)
        .await
        .context(format!("Failed to connect to BearDog crypto at {}", socket_path))?;

    // Encode to base64
    use base64::Engine;
    let encoder = base64::engine::general_purpose::STANDARD;
    let key_b64 = encoder.encode(key);
    let data_b64 = encoder.encode(data);

    // Create JSON-RPC request
    let request = HmacSha256Request {
        jsonrpc: "2.0".to_string(),
        method: "crypto.hmac_sha256".to_string(),
        params: HmacSha256Params {
            key: key_b64,
            data: data_b64,
        },
        id: 8,
    };

    // Send request
    let request_json = serde_json::to_string(&request)?;
    stream.write_all(request_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    // Read response
    let response_str = read_json_rpc_response(&mut stream).await?;

    // Parse response
    let response: HmacSha256Response = serde_json::from_str(&response_str)
        .context("Failed to parse BearDog HMAC-SHA256 response")?;

    // Decode HMAC from base64
    let hmac = encoder.decode(&response.result.mac).context("Failed to decode HMAC-SHA256")?;

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

    let response_str =
        String::from_utf8(response_buffer).context("BearDog response was not valid UTF-8")?;

    Ok(response_str)
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
        let (public_key, secret_key_id) =
            x25519_generate_ephemeral(&socket, purpose).await.unwrap();
        assert_eq!(public_key.len(), 32); // X25519 public key is 32 bytes
        assert!(!secret_key_id.is_empty());

        // Derive shared secret (using our own public key for testing)
        let shared_secret = x25519_derive_secret(&socket, &secret_key_id, &public_key).await;
        assert!(shared_secret.is_ok());
        assert_eq!(shared_secret.unwrap().len(), 32); // X25519 shared secret is 32 bytes
    }

    #[tokio::test]
    #[ignore]
    async fn test_chacha20_poly1305_round_trip() {
        let socket = super::super::discovery::get_beardog_crypto_socket()
            .await
            .expect("BearDog crypto socket not found - is BearDog running?");
        let plaintext = b"Secret message!";
        let key = [0u8; 32]; // 32-byte key
        let aad = b"additional data";

        // Encrypt (BearDog generates nonce!)
        let (ciphertext, nonce, tag) =
            chacha20_poly1305_encrypt(&socket, plaintext, &key, Some(aad)).await.unwrap();

        assert_eq!(ciphertext.len(), plaintext.len()); // Ciphertext same size as plaintext
        assert_eq!(nonce.len(), 12); // Nonce is 12 bytes
        assert_eq!(tag.len(), 16); // Auth tag is 16 bytes

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
        let data = b"Data to hash";

        let hash = blake3_hash(&socket, data).await.unwrap();
        assert_eq!(hash.len(), 32); // Blake3 hash is 32 bytes
    }

    #[tokio::test]
    #[ignore]
    async fn test_hmac_sha256() {
        let socket = super::super::discovery::get_beardog_crypto_socket()
            .await
            .expect("BearDog crypto socket not found - is BearDog running?");
        let key = b"secret key";
        let data = b"data to authenticate";

        let hmac = hmac_sha256(&socket, key, data).await.unwrap();
        assert_eq!(hmac.len(), 32); // SHA-256 HMAC is 32 bytes
    }
}
