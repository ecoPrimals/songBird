//! BearDog RPC client for crypto operations
//!
//! Communicates with BearDog via JSON-RPC 2.0 over Unix sockets.

use crate::error::{Error, Result};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{debug, error, info, trace, warn};

/// JSON-RPC request
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

/// JSON-RPC response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    /// Request ID (can be null for notifications per JSON-RPC 2.0 spec)
    id: Option<u64>,
}

/// JSON-RPC error
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    data: Option<Value>,
}

/// BearDog RPC client (routes through Neural API for capability translation)
#[derive(Debug)]
pub struct BearDogClient {
    neural_api_socket: String,
    request_id: std::sync::atomic::AtomicU64,
}

impl BearDogClient {
    /// Create a new BearDog client that routes through Neural API
    /// 
    /// This client uses Neural API's capability translation to convert
    /// semantic capability names (e.g., "crypto.generate_keypair") to
    /// actual provider method names (e.g., "x25519_generate_ephemeral").
    /// 
    /// # Arguments
    /// * `neural_api_socket` - Path to Neural API socket (e.g., "/tmp/neural-api-nat0.sock")
    pub fn new(neural_api_socket: impl Into<String>) -> Self {
        Self {
            neural_api_socket: neural_api_socket.into(),
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }
    
    /// Create from environment variable (fallback to default)
    pub fn from_env() -> Self {
        let socket = std::env::var("NEURAL_API_SOCKET")
            .unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());
        Self::new(socket)
    }

    /// Generate x25519 keypair
    pub async fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        debug!("🔑 Generating x25519 keypair via BearDog");
        
        let result = self.call("crypto.generate_keypair", json!({
            "algorithm": "x25519"
        })).await?;

        let public_key = result["public_key"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing public_key".to_string()))?;
        let private_key = result["secret_key"]  // BearDog returns "secret_key", not "private_key"
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing secret_key in BearDog response".to_string()))?;

        let public_key = BASE64_STANDARD.decode(public_key)
            .map_err(|e| Error::BearDogRpc(format!("Invalid public_key base64: {}", e)))?;
        let private_key = BASE64_STANDARD.decode(private_key)
            .map_err(|e| Error::BearDogRpc(format!("Invalid private_key base64: {}", e)))?;

        Ok((public_key, private_key))
    }

    /// Perform ECDH key exchange
    pub async fn ecdh_derive(&self, private_key: &[u8], public_key: &[u8]) -> Result<Vec<u8>> {
        debug!("🔐 Performing ECDH via BearDog");
        
        let result = self.call("crypto.ecdh_derive", json!({
            "private_key": BASE64_STANDARD.encode(private_key),
            "public_key": BASE64_STANDARD.encode(public_key)
        })).await?;

        let shared_secret = result["shared_secret"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing shared_secret".to_string()))?;

        BASE64_STANDARD.decode(shared_secret)
            .map_err(|e| Error::BearDogRpc(format!("Invalid shared_secret base64: {}", e)))
    }

    /// Derive TLS handshake traffic secrets (for encrypting handshake messages)
    /// 
    /// RFC 8446 Section 7.1: Handshake traffic secrets are derived using:
    /// - ECDH shared secret
    /// - Client random
    /// - Server random  
    /// - Transcript hash of ClientHello + ServerHello
    /// 
    /// These keys are used to encrypt/decrypt handshake messages AFTER ServerHello:
    /// - EncryptedExtensions
    /// - Certificate
    /// - CertificateVerify
    /// - Finished
    pub async fn tls_derive_handshake_secrets(
        &self,
        shared_secret: &[u8],
        client_random: &[u8],
        server_random: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<TlsSecrets> {
        info!("🔑 Calling tls_derive_handshake_secrets via Neural API (RFC 8446 Section 7.1)");
        debug!("  → pre_master_secret: {} bytes", shared_secret.len());
        debug!("  → client_random: {} bytes", client_random.len());
        debug!("  → server_random: {} bytes", server_random.len());
        debug!("  → transcript_hash: {} bytes (SHA-256 of ClientHello + ServerHello)", transcript_hash.len());
        debug!("  → cipher_suite: 0x{:04x}", cipher_suite);
        trace!("  → transcript_hash (hex): {}", hex::encode(transcript_hash));
        
        let result = self.call("tls.derive_handshake_secrets", json!({
            "pre_master_secret": BASE64_STANDARD.encode(shared_secret),
            "client_random": BASE64_STANDARD.encode(client_random),
            "server_random": BASE64_STANDARD.encode(server_random),
            "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
            "cipher_suite": cipher_suite
        })).await.map_err(|e| {
            error!("❌ tls_derive_handshake_secrets RPC call failed: {}", e);
            e
        })?;

        debug!("✅ Got response from tls_derive_handshake_secrets");
        trace!("  Response JSON: {}", serde_json::to_string_pretty(&result).unwrap_or_else(|_| "unable to serialize".to_string()));

        debug!("📋 Parsing handshake traffic keys from response...");
        
        let client_write_key = BASE64_STANDARD.decode(
            result["client_write_key"].as_str()
                .ok_or_else(|| Error::BearDogRpc("Missing client_write_key in response".to_string()))?
        ).map_err(|e| Error::BearDogRpc(format!("Invalid client_write_key base64: {}", e)))?;
        debug!("  ✅ client_handshake_key: {} bytes", client_write_key.len());
        
        let server_write_key = BASE64_STANDARD.decode(
            result["server_write_key"].as_str()
                .ok_or_else(|| Error::BearDogRpc("Missing server_write_key in response".to_string()))?
        ).map_err(|e| Error::BearDogRpc(format!("Invalid server_write_key base64: {}", e)))?;
        debug!("  ✅ server_handshake_key: {} bytes", server_write_key.len());
        
        let client_write_iv = BASE64_STANDARD.decode(
            result["client_write_iv"].as_str()
                .ok_or_else(|| Error::BearDogRpc("Missing client_write_iv in response".to_string()))?
        ).map_err(|e| Error::BearDogRpc(format!("Invalid client_write_iv base64: {}", e)))?;
        debug!("  ✅ client_handshake_iv: {} bytes", client_write_iv.len());
        
        let server_write_iv = BASE64_STANDARD.decode(
            result["server_write_iv"].as_str()
                .ok_or_else(|| Error::BearDogRpc("Missing server_write_iv in response".to_string()))?
        ).map_err(|e| Error::BearDogRpc(format!("Invalid server_write_iv base64: {}", e)))?;
        debug!("  ✅ server_handshake_iv: {} bytes", server_write_iv.len());

        // NEW: Parse traffic secrets (needed for Finished message - RFC 8446 Section 4.4.4)
        let client_handshake_secret = BASE64_STANDARD.decode(
            result["client_handshake_secret"].as_str()
                .ok_or_else(|| Error::BearDogRpc("Missing client_handshake_secret in response".to_string()))?
        ).map_err(|e| Error::BearDogRpc(format!("Invalid client_handshake_secret base64: {}", e)))?;
        debug!("  ✅ client_handshake_secret: {} bytes", client_handshake_secret.len());
        
        let server_handshake_secret = BASE64_STANDARD.decode(
            result["server_handshake_secret"].as_str()
                .ok_or_else(|| Error::BearDogRpc("Missing server_handshake_secret in response".to_string()))?
        ).map_err(|e| Error::BearDogRpc(format!("Invalid server_handshake_secret base64: {}", e)))?;
        debug!("  ✅ server_handshake_secret: {} bytes", server_handshake_secret.len());

        // HEX DUMPS for derived keys (cross-verify with RFC 8448 or server expectations)
        info!("🔍 DERIVED HANDSHAKE KEYS - FULL HEX DUMPS:");
        info!("   client_write_key: {}", hex::encode(&client_write_key));
        info!("   server_write_key: {}", hex::encode(&server_write_key));
        info!("   client_write_iv: {}", hex::encode(&client_write_iv));
        info!("   server_write_iv: {}", hex::encode(&server_write_iv));
        info!("   client_handshake_secret: {}", hex::encode(&client_handshake_secret));
        info!("   server_handshake_secret: {}", hex::encode(&server_handshake_secret));

        info!("✅ Handshake traffic secrets derived successfully (RFC 8446 Section 7.1 compliant)");
        
        Ok(TlsSecrets {
            client_write_key,
            server_write_key,
            client_write_iv,
            server_write_iv,
            client_handshake_secret,
            server_handshake_secret,
        })
    }

    /// Derive TLS application traffic secrets (for encrypting HTTP data)
    /// 
    /// This implements the TLS 1.3 key schedule to derive application traffic keys
    /// from the handshake secret. These keys are used for HTTP data encryption/decryption.
    /// 
    /// RFC 8446 Section 7.1: After the handshake completes, derive master secret and
    /// then derive application traffic secrets for encrypting application data.
    /// 
    /// # Arguments
    /// 
    /// * `shared_secret` - ECDH shared secret (pre-master secret)
    /// * `client_random` - Client random (32 bytes)
    /// * `server_random` - Server random (32 bytes)
    /// * `transcript_hash` - SHA-256 hash of all handshake messages (ClientHello...server Finished)
    /// 
    /// # RFC 8446 Compliance
    /// 
    /// The transcript hash is REQUIRED for correct TLS 1.3 key derivation:
    /// ```text
    /// application_traffic_secret = HKDF-Expand-Label(
    ///     master_secret,
    ///     "c ap traffic" | "s ap traffic",
    ///     Transcript-Hash(ClientHello...server Finished),  // ← REQUIRED!
    ///     Hash.length
    /// )
    /// ```
    pub async fn tls_derive_application_secrets(
        &self,
        shared_secret: &[u8],
        client_random: &[u8],
        server_random: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<TlsSecrets> {
        info!("🔑 Calling tls_derive_application_secrets via Neural API (RFC 8446 compliant)");
        debug!("  → pre_master_secret: {} bytes", shared_secret.len());
        debug!("  → client_random: {} bytes", client_random.len());
        debug!("  → server_random: {} bytes", server_random.len());
        debug!("  → transcript_hash: {} bytes (SHA-256 of all handshake messages)", transcript_hash.len());
        debug!("  → cipher_suite: 0x{:04x}", cipher_suite);
        trace!("  → transcript_hash (hex): {}", hex::encode(transcript_hash));
        
        let result = self.call("tls.derive_application_secrets", json!({
            "pre_master_secret": BASE64_STANDARD.encode(shared_secret),
            "client_random": BASE64_STANDARD.encode(client_random),
            "server_random": BASE64_STANDARD.encode(server_random),
            "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
            "cipher_suite": cipher_suite as u64
        })).await.map_err(|e| {
            error!("❌ tls_derive_application_secrets RPC call failed: {}", e);
            e
        })?;

        debug!("✅ Got response from tls_derive_application_secrets");
        trace!("  Response JSON: {}", serde_json::to_string_pretty(&result).unwrap_or_else(|_| "unable to serialize".to_string()));

        debug!("📋 Parsing application traffic keys from response...");
        
        let client_write_key = BASE64_STANDARD.decode(
            result["client_write_key"].as_str()
                .ok_or_else(|| Error::BearDogRpc("Missing client_write_key in response".to_string()))?
        ).map_err(|e| Error::BearDogRpc(format!("Invalid client_write_key base64: {}", e)))?;
        debug!("  ✅ client_write_key: {} bytes", client_write_key.len());
        
        let server_write_key = BASE64_STANDARD.decode(
            result["server_write_key"].as_str()
                .ok_or_else(|| Error::BearDogRpc("Missing server_write_key in response".to_string()))?
        ).map_err(|e| Error::BearDogRpc(format!("Invalid server_write_key base64: {}", e)))?;
        debug!("  ✅ server_write_key: {} bytes", server_write_key.len());
        
        let client_write_iv = BASE64_STANDARD.decode(
            result["client_write_iv"].as_str()
                .ok_or_else(|| Error::BearDogRpc("Missing client_write_iv in response".to_string()))?
        ).map_err(|e| Error::BearDogRpc(format!("Invalid client_write_iv base64: {}", e)))?;
        debug!("  ✅ client_write_iv: {} bytes", client_write_iv.len());
        
        let server_write_iv = BASE64_STANDARD.decode(
            result["server_write_iv"].as_str()
                .ok_or_else(|| Error::BearDogRpc("Missing server_write_iv in response".to_string()))?
        ).map_err(|e| Error::BearDogRpc(format!("Invalid server_write_iv base64: {}", e)))?;
        debug!("  ✅ server_write_iv: {} bytes", server_write_iv.len());

        // DIAGNOSTIC: Show key derivation details for "invisible 0.5%" investigation
        info!("════════════════════════════════════════════════════════════");
        info!("🔬 BEARDOG APPLICATION KEY DERIVATION (DIAGNOSTIC)");
        info!("════════════════════════════════════════════════════════════");
        info!("BearDog RPC call: tls.derive_application_secrets");
        info!("");
        info!("Inputs (sent to BearDog):");
        info!("  pre_master_secret: {} bytes", shared_secret.len());
        info!("    {}", hex::encode(shared_secret));
        info!("  client_random: {} bytes", client_random.len());
        info!("    {}", hex::encode(client_random));
        info!("  server_random: {} bytes", server_random.len());
        info!("    {}", hex::encode(server_random));
        info!("  transcript_hash: {} bytes (SHA-256)", transcript_hash.len());
        info!("    {}", hex::encode(transcript_hash));
        info!("  cipher_suite: 0x{:04x}", cipher_suite);
        info!("");
        info!("⚠️  CRITICAL: BearDog should derive:");
        info!("  1. Master Secret (from pre_master_secret + transcript_hash)");
        info!("  2. CLIENT_TRAFFIC_SECRET_0 (HKDF-Expand-Label)");
        info!("  3. SERVER_TRAFFIC_SECRET_0 (HKDF-Expand-Label)");
        info!("  4. client_write_key (HKDF-Expand-Label from CLIENT_TRAFFIC_SECRET_0)");
        info!("  5. client_write_iv (HKDF-Expand-Label from CLIENT_TRAFFIC_SECRET_0)");
        info!("");
        info!("Outputs (received from BearDog):");
        info!("  client_write_key: {} bytes", client_write_key.len());
        info!("    {}", hex::encode(&client_write_key));
        info!("  server_write_key: {} bytes", server_write_key.len());
        info!("    {}", hex::encode(&server_write_key));
        info!("  client_write_iv: {} bytes", client_write_iv.len());
        info!("    {}", hex::encode(&client_write_iv));
        info!("  server_write_iv: {} bytes", server_write_iv.len());
        info!("    {}", hex::encode(&server_write_iv));
        info!("");
        info!("🧪 RFC 8448 Test Vector Comparison:");
        info!("  If CLIENT_TRAFFIC_SECRET_0 = 48d566dbe8bb07d33ab06fc01a71a8fe...");
        info!("  Then client_write_key should = 02ba47f1a767ba883ee776e329080865");
        info!("  And client_write_iv should = 0393d92b4ff5ee2768bd4f4a");
        info!("  (Compare with your output above!)");
        info!("════════════════════════════════════════════════════════════");

        info!("🎉 Application traffic keys successfully derived and parsed");
        
        Ok(TlsSecrets {
            client_write_key,
            server_write_key,
            client_write_iv,
            server_write_iv,
            // Note: Application secrets don't need handshake secrets (those were for Finished message)
            client_handshake_secret: vec![],
            server_handshake_secret: vec![],
        })
    }

    /// Compute TLS 1.3 Finished message verify_data (RFC 8446 Section 4.4.4)
    /// 
    /// The Finished message authenticates the entire handshake using HMAC:
    /// ```text
    /// verify_data = HMAC(finished_key, Transcript-Hash(Handshake Context))
    /// ```
    /// 
    /// Where `finished_key` is derived from the handshake traffic secret:
    /// ```text
    /// finished_key = HKDF-Expand-Label(BaseKey, "finished", "", Hash.length)
    /// ```
    /// 
    /// # Arguments
    /// 
    /// * `transcript_hash` - SHA-256 hash of all handshake messages up to and including server Finished
    /// * `cipher_suite` - Negotiated TLS 1.3 cipher suite (0x1301, 0x1302, or 0x1303)
    /// 
    /// # Returns
    /// 
    /// 32-byte verify_data for SHA-256-based cipher suites (0x1301, 0x1303) or
    /// 48-byte verify_data for SHA-384-based cipher suites (0x1302)
    pub async fn tls_compute_finished_verify_data(
        &self,
        client_handshake_traffic_secret: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<Vec<u8>> {
        info!("🔐 Computing TLS 1.3 Finished verify_data (RFC 8446 Section 4.4.4)");
        debug!("  → client_handshake_traffic_secret: {} bytes", client_handshake_traffic_secret.len());
        debug!("  → transcript_hash: {} bytes", transcript_hash.len());
        debug!("  → cipher_suite: 0x{:04x}", cipher_suite);
        trace!("  → transcript_hash (hex): {}", hex::encode(transcript_hash));
        
        let result = self.call("tls.compute_finished_verify_data", json!({
            "base_key": BASE64_STANDARD.encode(client_handshake_traffic_secret),
            "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
            "cipher_suite": format!("0x{:04x}", cipher_suite)
        })).await.map_err(|e| {
            error!("❌ tls_compute_finished_verify_data RPC call failed: {}", e);
            e
        })?;

        let verify_data = result["verify_data"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing verify_data in response".to_string()))?;

        let decoded = BASE64_STANDARD.decode(verify_data)
            .map_err(|e| Error::BearDogRpc(format!("Invalid verify_data base64: {}", e)))?;
        
        info!("✅ Finished verify_data computed: {} bytes", decoded.len());
        debug!("   Verify data (hex): {}", hex::encode(&decoded));
        Ok(decoded)
    }

    /// Legacy alias for backwards compatibility
    /// DEPRECATED: Use tls_derive_application_secrets instead
    /// 
    /// # Note
    /// 
    /// This method creates an empty transcript hash for backwards compatibility.
    /// For RFC 8446 compliance, use `tls_derive_application_secrets` with proper transcript hash.
    #[deprecated(since = "5.6.0", note = "Use tls_derive_application_secrets with transcript_hash parameter")]
    pub async fn tls_derive_secrets(
        &self,
        shared_secret: &[u8],
        client_random: &[u8],
        server_random: &[u8],
    ) -> Result<TlsSecrets> {
        // For backwards compatibility, create empty transcript hash (NOT RFC 8446 compliant!)
        warn!("Using deprecated tls_derive_secrets without transcript hash - not RFC 8446 compliant!");
        let empty_transcript_hash = vec![0u8; 32]; // Placeholder
        let default_cipher_suite = 0x1303; // ChaCha20-Poly1305 (backward compat)
        self.tls_derive_application_secrets(shared_secret, client_random, server_random, &empty_transcript_hash, default_cipher_suite).await
    }

    /// Encrypt data with ChaCha20-Poly1305
    pub async fn encrypt(&self, key: &[u8], nonce: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        trace!("🔐 Encrypting {} bytes via BearDog (key={} bytes, nonce={} bytes, aad={} bytes)", 
               plaintext.len(), key.len(), nonce.len(), aad.len());
        
        let result = self.call("crypto.encrypt", json!({
            "algorithm": "chacha20-poly1305",
            "key": BASE64_STANDARD.encode(key),
            "nonce": BASE64_STANDARD.encode(nonce),
            "plaintext": BASE64_STANDARD.encode(plaintext),
            "aad": BASE64_STANDARD.encode(aad)
        })).await.map_err(|e| {
            error!("❌ crypto.encrypt RPC call failed: {}", e);
            e
        })?;

        let ciphertext = result["ciphertext"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing ciphertext in response".to_string()))?;

        let decoded = BASE64_STANDARD.decode(ciphertext)
            .map_err(|e| Error::BearDogRpc(format!("Invalid ciphertext base64: {}", e)))?;
        
        trace!("✅ Encrypted: {} bytes plaintext → {} bytes ciphertext", plaintext.len(), decoded.len());
        Ok(decoded)
    }

    /// Encrypt data with AES-128-GCM (for TLS_AES_128_GCM_SHA256 cipher suite)
    /// 
    /// # Arguments
    /// 
    /// * `key` - 16-byte AES-128 key
    /// * `nonce` - 12-byte nonce (IV for GCM mode)
    /// * `plaintext` - Data to encrypt
    /// * `aad` - Additional Authenticated Data (TLS record header)
    pub async fn encrypt_aes_128_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        trace!("🔐 Encrypting {} bytes with AES-128-GCM via BearDog", plaintext.len());
        
        // Validate lengths
        if key.len() != 16 {
            error!("❌ Wrong key size: {} bytes (need 16 for AES-128)", key.len());
            return Err(Error::BearDogRpc(format!("AES-128-GCM requires 16-byte key, got {}", key.len())));
        }
        if nonce.len() != 12 {
            error!("❌ Wrong nonce size: {} bytes (need 12 for GCM)", nonce.len());
            return Err(Error::BearDogRpc(format!("GCM nonce must be 12 bytes, got {}", nonce.len())));
        }
        
        let result = self.call("crypto.encrypt_aes_128_gcm", json!({
            "key": BASE64_STANDARD.encode(key),
            "nonce": BASE64_STANDARD.encode(nonce),
            "plaintext": BASE64_STANDARD.encode(plaintext),
            "aad": BASE64_STANDARD.encode(aad)
        })).await.map_err(|e| {
            error!("❌ crypto.encrypt_aes_128_gcm RPC call failed: {}", e);
            e
        })?;

        let ciphertext = result["ciphertext"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing ciphertext in response".to_string()))?;

        let decoded = BASE64_STANDARD.decode(ciphertext)
            .map_err(|e| Error::BearDogRpc(format!("Invalid ciphertext base64: {}", e)))?;
        
        trace!("✅ Encrypted: {} bytes plaintext → {} bytes ciphertext+tag", plaintext.len(), decoded.len());
        Ok(decoded)
    }

    /// Encrypt data with AES-256-GCM (for TLS_AES_256_GCM_SHA384 cipher suite)
    /// 
    /// # Arguments
    /// 
    /// * `key` - 32-byte AES-256 key
    /// * `nonce` - 12-byte nonce (IV for GCM mode)
    /// * `plaintext` - Data to encrypt
    /// * `aad` - Additional Authenticated Data (TLS record header)
    pub async fn encrypt_aes_256_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        trace!("🔐 Encrypting {} bytes with AES-256-GCM via BearDog", plaintext.len());
        
        // Validate lengths
        if key.len() != 32 {
            error!("❌ Wrong key size: {} bytes (need 32 for AES-256)", key.len());
            return Err(Error::BearDogRpc(format!("AES-256-GCM requires 32-byte key, got {}", key.len())));
        }
        if nonce.len() != 12 {
            error!("❌ Wrong nonce size: {} bytes (need 12 for GCM)", nonce.len());
            return Err(Error::BearDogRpc(format!("GCM nonce must be 12 bytes, got {}", nonce.len())));
        }
        
        let result = self.call("crypto.encrypt_aes_256_gcm", json!({
            "key": BASE64_STANDARD.encode(key),
            "nonce": BASE64_STANDARD.encode(nonce),
            "plaintext": BASE64_STANDARD.encode(plaintext),
            "aad": BASE64_STANDARD.encode(aad)
        })).await.map_err(|e| {
            error!("❌ crypto.encrypt_aes_256_gcm RPC call failed: {}", e);
            e
        })?;

        let ciphertext = result["ciphertext"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing ciphertext in response".to_string()))?;

        let decoded = BASE64_STANDARD.decode(ciphertext)
            .map_err(|e| Error::BearDogRpc(format!("Invalid ciphertext base64: {}", e)))?;
        
        trace!("✅ Encrypted: {} bytes plaintext → {} bytes ciphertext+tag", plaintext.len(), decoded.len());
        Ok(decoded)
    }

    /// Decrypt data with ChaCha20-Poly1305 (TLS 1.3 cipher suite 0x1303)
    /// 
    /// NOTE: ChaCha20-Poly1305 RPC expects SEPARATE ciphertext and tag parameters!
    /// This is different from AES-GCM which expects them combined.
    pub async fn decrypt(&self, key: &[u8], nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        info!("🔓 BearDog crypto.decrypt call (COMPREHENSIVE DEBUG):");
        info!("   Total ciphertext+tag: {} bytes", ciphertext.len());
        info!("   Key: {} bytes", key.len());
        info!("   Nonce: {} bytes", nonce.len());
        info!("   AAD: {} bytes", aad.len());
        debug!("Decrypt parameters:");
        debug!("  Key (first 16 bytes): {:02x?}", &key[..std::cmp::min(16, key.len())]);
        debug!("  Nonce (full): {:02x?}", nonce);
        debug!("  AAD (full): {:02x?}", aad);
        debug!("  Ciphertext+Tag (first 32 bytes): {:02x?}", &ciphertext[..std::cmp::min(32, ciphertext.len())]);
        debug!("  Ciphertext+Tag (last 16 bytes): {:02x?}", &ciphertext[ciphertext.len().saturating_sub(16)..]);
        
        // ChaCha20-Poly1305 AEAD: Last 16 bytes are the authentication tag
        if ciphertext.len() < 16 {
            error!("❌ Ciphertext too short: {} bytes (need at least 16 for tag)", ciphertext.len());
            return Err(Error::BearDogRpc("Ciphertext too short for ChaCha20-Poly1305 (need at least 16 bytes for tag)".to_string()));
        }
        
        let (actual_ciphertext, tag) = ciphertext.split_at(ciphertext.len() - 16);
        info!("📊 Splitting ciphertext+tag:");
        info!("   Ciphertext: {} bytes", actual_ciphertext.len());
        info!("   Tag: 16 bytes");
        
        // HEX DUMPS for deep debugging
        info!("🔍 FULL HEX DUMPS (for BearDog cross-verification):");
        info!("   Key (32 bytes): {}", hex::encode(key));
        info!("   Nonce (12 bytes): {}", hex::encode(nonce));
        info!("   Ciphertext ({} bytes): {}", actual_ciphertext.len(), hex::encode(actual_ciphertext));
        info!("   Tag (16 bytes): {}", hex::encode(tag));
        info!("   AAD (5 bytes): {}", hex::encode(aad));
        
        info!("📞 Calling BearDog RPC: crypto.decrypt");
        debug!("RPC payload:");
        debug!("  algorithm: chacha20-poly1305");
        debug!("  key: {} bytes (base64)", key.len());
        debug!("  nonce: {} bytes (base64)", nonce.len());
        debug!("  ciphertext: {} bytes (base64)", actual_ciphertext.len());
        debug!("  tag: 16 bytes (base64)");
        debug!("  aad: {} bytes (base64)", aad.len());
        
        let result = self.call("crypto.decrypt", json!({
            "algorithm": "chacha20-poly1305",
            "key": BASE64_STANDARD.encode(key),
            "nonce": BASE64_STANDARD.encode(nonce),
            "ciphertext": BASE64_STANDARD.encode(actual_ciphertext),
            "tag": BASE64_STANDARD.encode(tag),
            "aad": BASE64_STANDARD.encode(aad)
        })).await.map_err(|e| {
            error!("❌ BearDog crypto.decrypt RPC call FAILED!");
            error!("   Error: {}", e);
            error!("");
            error!("   📊 Context:");
            error!("     • Ciphertext: {} bytes", actual_ciphertext.len());
            error!("     • Tag: 16 bytes");
            error!("     • Key: {} bytes", key.len());
            error!("     • Nonce: {} bytes", nonce.len());
            error!("     • AAD: {} bytes", aad.len());
            error!("");
            error!("   🔍 This is likely an AEAD authentication failure!");
            error!("   Possible causes:");
            error!("     1. Key mismatch (derived incorrectly)");
            error!("     2. Nonce mismatch (sequence number or IV wrong)");
            error!("     3. AAD mismatch (TLS record header wrong)");
            error!("     4. Tag corruption (network issue)");
            error!("     5. Ciphertext corruption (network issue)");
            e
        })?;

        let plaintext = result["plaintext"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing plaintext in response".to_string()))?;

        let decoded = BASE64_STANDARD.decode(plaintext)
            .map_err(|e| Error::BearDogRpc(format!("Invalid plaintext base64: {}", e)))?;
        
        info!("✅ BearDog crypto.decrypt SUCCESS!");
        info!("   Ciphertext: {} bytes → Plaintext: {} bytes", ciphertext.len(), decoded.len());
        debug!("Plaintext (first 32 bytes): {:02x?}", &decoded[..std::cmp::min(32, decoded.len())]);
        Ok(decoded)
    }

    /// Decrypt data with AES-128-GCM (for TLS_AES_128_GCM_SHA256 cipher suite)
    /// 
    /// # Arguments
    /// 
    /// * `key` - 16-byte AES-128 key
    /// * `nonce` - 12-byte nonce (IV for GCM mode)
    /// * `ciphertext` - Encrypted data + 16-byte authentication tag
    /// * `aad` - Additional Authenticated Data (TLS record header)
    pub async fn decrypt_aes_128_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        info!("🔓 BearDog crypto.aes128_gcm_decrypt call (COMPREHENSIVE DEBUG):");
        info!("   Total ciphertext+tag: {} bytes", ciphertext.len());
        info!("   Key: {} bytes (expect 16 for AES-128)", key.len());
        info!("   Nonce: {} bytes (expect 12 for GCM)", nonce.len());
        info!("   AAD: {} bytes", aad.len());
        
        // Validate lengths
        if ciphertext.len() < 16 {
            error!("❌ Ciphertext too short: {} bytes (need at least 16 for GCM tag)", ciphertext.len());
            return Err(Error::BearDogRpc("Ciphertext too short for AES-128-GCM".to_string()));
        }
        if key.len() != 16 {
            error!("❌ Wrong key size: {} bytes (need 16 for AES-128)", key.len());
            return Err(Error::BearDogRpc(format!("AES-128-GCM requires 16-byte key, got {}", key.len())));
        }
        if nonce.len() != 12 {
            error!("❌ Wrong nonce size: {} bytes (need 12 for GCM)", nonce.len());
            return Err(Error::BearDogRpc(format!("GCM nonce must be 12 bytes, got {}", nonce.len())));
        }
        
        // CRITICAL FIX: Do NOT split ciphertext and tag!
        // BearDog's aes-gcm crate expects the FULL ciphertext with tag appended
        info!("🔧 CRITICAL FIX: Passing FULL ciphertext (NOT splitting tag)");
        info!("   Per BearDog: RustCrypto aes-gcm expects [encrypted_data] + [16-byte tag]");
        info!("   Total bytes being passed: {} (includes tag)", ciphertext.len());
        
        // HEX DUMPS for verification
        debug!("🔍 Full parameters (for BearDog cross-verification):");
        debug!("   Key (16 bytes): {}", hex::encode(key));
        debug!("   Nonce (12 bytes): {}", hex::encode(nonce));
        debug!("   Ciphertext+Tag ({} bytes): {}", ciphertext.len(), hex::encode(ciphertext));
        debug!("   AAD (5 bytes): {}", hex::encode(aad));
        
        let result = self.call("crypto.decrypt_aes_128_gcm", json!({
            "key": BASE64_STANDARD.encode(key),
            "nonce": BASE64_STANDARD.encode(nonce),
            "ciphertext": BASE64_STANDARD.encode(ciphertext),  // FULL ciphertext WITH tag!
            "aad": BASE64_STANDARD.encode(aad)
        })).await.map_err(|e| {
            error!("❌ BearDog crypto.aes128_gcm_decrypt RPC call FAILED!");
            error!("   Error: {}", e);
            error!("   Ciphertext+Tag: {} bytes", ciphertext.len());
            error!("   Key: {} bytes", key.len());
            error!("   Nonce: {} bytes", nonce.len());
            error!("   AAD: {} bytes", aad.len());
            e
        })?;

        let plaintext = result["plaintext"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing plaintext in response".to_string()))?;

        let decoded = BASE64_STANDARD.decode(plaintext)
            .map_err(|e| Error::BearDogRpc(format!("Invalid plaintext base64: {}", e)))?;
        
        info!("✅ BearDog crypto.aes128_gcm_decrypt SUCCESS!");
        info!("   Ciphertext+Tag: {} bytes → Plaintext: {} bytes", ciphertext.len(), decoded.len());
        Ok(decoded)
    }

    /// Decrypt data with AES-256-GCM (for TLS_AES_256_GCM_SHA384 cipher suite)
    /// 
    /// # Arguments
    /// 
    /// * `key` - 32-byte AES-256 key
    /// * `nonce` - 12-byte nonce (IV for GCM mode)
    /// * `ciphertext` - Encrypted data + 16-byte authentication tag
    /// * `aad` - Additional Authenticated Data (TLS record header)
    pub async fn decrypt_aes_256_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        info!("🔓 BearDog crypto.aes256_gcm_decrypt call (COMPREHENSIVE DEBUG):");
        info!("   Total ciphertext+tag: {} bytes", ciphertext.len());
        info!("   Key: {} bytes (expect 32 for AES-256)", key.len());
        info!("   Nonce: {} bytes (expect 12 for GCM)", nonce.len());
        info!("   AAD: {} bytes", aad.len());
        
        // Validate lengths
        if ciphertext.len() < 16 {
            error!("❌ Ciphertext too short: {} bytes (need at least 16 for GCM tag)", ciphertext.len());
            return Err(Error::BearDogRpc("Ciphertext too short for AES-256-GCM".to_string()));
        }
        if key.len() != 32 {
            error!("❌ Wrong key size: {} bytes (need 32 for AES-256)", key.len());
            return Err(Error::BearDogRpc(format!("AES-256-GCM requires 32-byte key, got {}", key.len())));
        }
        if nonce.len() != 12 {
            error!("❌ Wrong nonce size: {} bytes (need 12 for GCM)", nonce.len());
            return Err(Error::BearDogRpc(format!("GCM nonce must be 12 bytes, got {}", nonce.len())));
        }
        
        // CRITICAL FIX: Do NOT split ciphertext and tag!
        // BearDog's aes-gcm crate expects the FULL ciphertext with tag appended
        info!("🔧 CRITICAL FIX: Passing FULL ciphertext (NOT splitting tag)");
        info!("   Per BearDog: RustCrypto aes-gcm expects [encrypted_data] + [16-byte tag]");
        info!("   Total bytes being passed: {} (includes tag)", ciphertext.len());
        
        let result = self.call("crypto.decrypt_aes_256_gcm", json!({
            "key": BASE64_STANDARD.encode(key),
            "nonce": BASE64_STANDARD.encode(nonce),
            "ciphertext": BASE64_STANDARD.encode(ciphertext),  // FULL ciphertext WITH tag!
            "aad": BASE64_STANDARD.encode(aad)
        })).await.map_err(|e| {
            error!("❌ BearDog crypto.aes256_gcm_decrypt RPC call FAILED!");
            error!("   Error: {}", e);
            e
        })?;

        let plaintext = result["plaintext"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing plaintext in response".to_string()))?;

        let decoded = BASE64_STANDARD.decode(plaintext)
            .map_err(|e| Error::BearDogRpc(format!("Invalid plaintext base64: {}", e)))?;
        
        info!("✅ BearDog crypto.aes256_gcm_decrypt SUCCESS!");
        info!("   Ciphertext+Tag: {} bytes → Plaintext: {} bytes", ciphertext.len(), decoded.len());
        Ok(decoded)
    }

    /// Call BearDog capability via Neural API translation
    /// 
    /// This method sends a `capability.call` request to Neural API, which:
    /// 1. Translates the semantic capability (e.g., "crypto.generate_keypair")
    /// 2. Looks up the actual provider (BearDog)
    /// 3. Routes the call to the actual method (e.g., "x25519_generate_ephemeral")
    /// 4. Returns the result transparently
    async fn call(&self, capability: &str, args: Value) -> Result<Value> {
        let id = self.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        // Build capability.call request for Neural API
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "capability.call".to_string(),
            params: json!({
                "capability": capability,
                "args": args
            }),
            id,
        };

        trace!("→ Neural API capability.call: {} (id={})", capability, id);

        // Connect to Neural API
        let mut stream = UnixStream::connect(&self.neural_api_socket)
            .await
            .map_err(|e| Error::BearDogRpc(format!(
                "Failed to connect to Neural API at {}: {}", 
                self.neural_api_socket, e
            )))?;

        // Send request
        let request_json = serde_json::to_string(&request)?;
        stream.write_all(request_json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;
        
        // Shutdown write to signal we're done
        stream.shutdown().await?;

        // Read response with JSON-aware reading (Neural API keeps socket open)
        use tokio::time::{timeout, Duration};
        let mut buffer = Vec::new();
        let mut temp_buf = [0u8; 4096];
        let read_timeout = Duration::from_millis(100);
        
        loop {
            match timeout(read_timeout, stream.read(&mut temp_buf)).await {
                Ok(Ok(0)) => break, // EOF
                Ok(Ok(n)) => {
                    buffer.extend_from_slice(&temp_buf[..n]);
                    // Check for complete JSON
                    if let Ok(s) = std::str::from_utf8(&buffer) {
                        if serde_json::from_str::<Value>(s).is_ok() {
                            break; // Complete JSON received!
                        }
                    }
                }
                Ok(Err(e)) => return Err(Error::BearDogRpc(format!("Socket read error: {}", e))),
                Err(_) => {
                    // Timeout - check if we have valid JSON
                    if !buffer.is_empty() {
                        if let Ok(s) = std::str::from_utf8(&buffer) {
                            if serde_json::from_str::<Value>(s).is_ok() {
                                break;
                            }
                        }
                    }
                    return Err(Error::BearDogRpc("Timeout reading from Neural API".to_string()));
                }
            }
        }

        // Log raw response for debugging
        if let Ok(response_str) = std::str::from_utf8(&buffer) {
            trace!("← Raw Neural API response ({} bytes): {}", buffer.len(), 
                   if response_str.len() > 500 { 
                       format!("{}... (truncated)", &response_str[..500])
                   } else {
                       response_str.to_string()
                   });
        }

        let response: JsonRpcResponse = serde_json::from_slice(&buffer)
            .map_err(|e| {
                error!("❌ Failed to parse Neural API response for {}: {}", capability, e);
                if let Ok(response_str) = std::str::from_utf8(&buffer) {
                    error!("   Raw response: {}", response_str);
                }
                Error::BearDogRpc(format!("Failed to parse Neural API response: {}", e))
            })?;

        let id_str = response.id.map(|id| id.to_string()).unwrap_or_else(|| "null".to_string());
        trace!("← Neural API result for {} (id={})", capability, id_str);

        // Check for errors
        if let Some(error) = response.error {
            error!("❌ Neural API error for {}: {} (code: {})", capability, error.message, error.code);
            return Err(Error::BearDogRpc(format!(
                "Neural API error for {}: {} (code: {})", 
                capability, error.message, error.code
            )));
        }

        debug!("✅ Neural API call successful: {}", capability);
        response.result.ok_or_else(|| {
            error!("❌ Missing result in Neural API response for {}", capability);
            Error::BearDogRpc("Missing result in response".to_string())
        })
    }
}

/// TLS session secrets
/// 
/// These are the keys and IVs used for TLS record encryption/decryption.
/// In TLS 1.3, there are separate keys for:
/// - Handshake traffic (for encrypting handshake messages)
/// - Application traffic (for encrypting HTTP data)
/// 
/// Songbird derives application traffic keys for HTTP data encryption.
#[derive(Debug, Clone)]
pub struct TlsSecrets {
    pub client_write_key: Vec<u8>,
    pub server_write_key: Vec<u8>,
    pub client_write_iv: Vec<u8>,
    pub server_write_iv: Vec<u8>,
    // NEW: Traffic secrets (needed for RFC 8446 Section 4.4.4 Finished message)
    pub client_handshake_secret: Vec<u8>,
    pub server_handshake_secret: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beardog_client_creation() {
        let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
        assert_eq!(client.neural_api_socket, "/tmp/neural-api-nat0.sock");
    }

    #[test]
    fn test_tls_secrets_clone() {
        let secrets = TlsSecrets {
            client_write_key: vec![1, 2, 3],
            server_write_key: vec![4, 5, 6],
            client_write_iv: vec![7, 8, 9],
            server_write_iv: vec![10, 11, 12],
            client_handshake_secret: vec![13, 14, 15],
            server_handshake_secret: vec![16, 17, 18],
        };
        
        let cloned = secrets.clone();
        assert_eq!(secrets.client_write_key, cloned.client_write_key);
        assert_eq!(secrets.server_write_key, cloned.server_write_key);
        assert_eq!(secrets.client_write_iv, cloned.client_write_iv);
        assert_eq!(secrets.server_write_iv, cloned.server_write_iv);
    }

    #[test]
    fn test_request_id_increment() {
        let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
        let id1 = client.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let id2 = client.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(id2, id1 + 1);
    }
    
    #[test]
    fn test_from_env_default() {
        // When NEURAL_API_SOCKET is not set, should use default
        std::env::remove_var("NEURAL_API_SOCKET");
        let client = BearDogClient::from_env();
        assert_eq!(client.neural_api_socket, "/tmp/neural-api-nat0.sock");
    }
    
    #[test]
    fn test_from_env_custom() {
        // When NEURAL_API_SOCKET is set, should use it
        std::env::set_var("NEURAL_API_SOCKET", "/custom/path.sock");
        let client = BearDogClient::from_env();
        assert_eq!(client.neural_api_socket, "/custom/path.sock");
        std::env::remove_var("NEURAL_API_SOCKET");
    }

    // ====================================================================
    // JSON-RPC RESPONSE PARSING TESTS (Unit Tests)
    // ====================================================================

    #[test]
    fn test_jsonrpc_response_with_numeric_id() {
        let json = r#"{
            "jsonrpc": "2.0",
            "result": {"key": "value"},
            "id": 42
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(42));
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_jsonrpc_response_with_null_id() {
        // This is the FIX! Null IDs are valid per JSON-RPC 2.0 spec
        let json = r#"{
            "jsonrpc": "2.0",
            "result": {"key": "value"},
            "id": null
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, None); // ✅ Now handles null!
        assert!(response.result.is_some());
    }

    #[test]
    fn test_jsonrpc_response_with_error() {
        let json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": -32600,
                "message": "Invalid Request"
            },
            "id": null
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.error.is_some());
        assert!(response.result.is_none());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
    }

    #[test]
    fn test_jsonrpc_response_tls_secrets() {
        // Realistic response from BearDog tls.derive_application_secrets
        let json = r#"{
            "jsonrpc": "2.0",
            "result": {
                "client_write_key": "u1HnZw8Q7wtXXPc9axju3uehJhY6xPzFiIGcvcwEmm0=",
                "server_write_key": "OYSAPFlf/NAvJTpBtx45lnsFtRu3VEOK5tO/EK3kbx8=",
                "client_write_iv": "rkCk3xt3l2SBFeNu",
                "server_write_iv": "otHQEpR5P+EVqd9V",
                "algorithm": "HKDF-SHA256",
                "rfc": "RFC 8446 Section 7.1"
            },
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        
        // Verify all fields present
        assert!(result["client_write_key"].is_string());
        assert!(result["server_write_key"].is_string());
        assert!(result["client_write_iv"].is_string());
        assert!(result["server_write_iv"].is_string());
        assert_eq!(result["algorithm"].as_str().unwrap(), "HKDF-SHA256");
    }

    #[test]
    fn test_tls_secrets_field_sizes() {
        let secrets = TlsSecrets {
            client_write_key: vec![0u8; 32], // ChaCha20 key size
            server_write_key: vec![0u8; 32],
            client_write_iv: vec![0u8; 12],   // ChaCha20 nonce size
            server_write_iv: vec![0u8; 12],
            client_handshake_secret: vec![0u8; 32],
            server_handshake_secret: vec![0u8; 32],
        };
        
        assert_eq!(secrets.client_write_key.len(), 32);
        assert_eq!(secrets.server_write_key.len(), 32);
        assert_eq!(secrets.client_write_iv.len(), 12);
        assert_eq!(secrets.server_write_iv.len(), 12);
    }

    // ====================================================================
    // CHAOS TESTS - Malformed Responses
    // ====================================================================

    #[test]
    fn test_chaos_malformed_json() {
        let json = r#"{"jsonrpc": "2.0", "result": {broken json"#;
        let result: std::result::Result<JsonRpcResponse, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_chaos_missing_jsonrpc_field() {
        let json = r#"{"result": {"key": "value"}, "id": 1}"#;
        let result: std::result::Result<JsonRpcResponse, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_chaos_wrong_jsonrpc_version() {
        let json = r#"{
            "jsonrpc": "1.0",
            "result": {"key": "value"},
            "id": 1
        }"#;
        
        // Should still parse, just has wrong version
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.jsonrpc, "1.0"); // Not 2.0, but parses
    }

    #[test]
    fn test_chaos_both_result_and_error() {
        // Invalid per JSON-RPC spec, but should parse
        let json = r#"{
            "jsonrpc": "2.0",
            "result": {"key": "value"},
            "error": {"code": -32000, "message": "Error"},
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_some());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_chaos_missing_both_result_and_error() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_none());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_chaos_huge_id() {
        let json = r#"{
            "jsonrpc": "2.0",
            "result": {"key": "value"},
            "id": 18446744073709551615
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, Some(u64::MAX));
    }

    #[test]
    fn test_chaos_negative_id() {
        // Negative IDs are invalid, should fail to parse as u64
        let json = r#"{
            "jsonrpc": "2.0",
            "result": {"key": "value"},
            "id": -1
        }"#;
        
        let result: std::result::Result<JsonRpcResponse, _> = serde_json::from_str(json);
        assert!(result.is_err()); // Can't parse negative as u64
    }

    #[test]
    fn test_chaos_string_id() {
        // String IDs are valid per JSON-RPC 2.0, but we only support u64
        let json = r#"{
            "jsonrpc": "2.0",
            "result": {"key": "value"},
            "id": "abc123"
        }"#;
        
        let result: std::result::Result<JsonRpcResponse, _> = serde_json::from_str(json);
        assert!(result.is_err()); // We don't support string IDs
    }

    #[test]
    fn test_chaos_empty_result() {
        let json = r#"{
            "jsonrpc": "2.0",
            "result": {},
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result.is_object());
        assert_eq!(result.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_chaos_null_result() {
        let json = r#"{
            "jsonrpc": "2.0",
            "result": null,
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        // With skip_serializing_if, null is treated as None
        assert!(response.result.is_none());
    }

    #[test]
    fn test_chaos_array_result() {
        let json = r#"{
            "jsonrpc": "2.0",
            "result": [1, 2, 3],
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_chaos_extra_fields() {
        let json = r#"{
            "jsonrpc": "2.0",
            "result": {"key": "value"},
            "id": 1,
            "extra": "ignored",
            "another": 123
        }"#;
        
        // Should parse fine, extra fields ignored
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_some());
    }

    #[test]
    fn test_chaos_very_large_response() {
        // 10KB response
        let large_value = "x".repeat(10000);
        let json = format!(r#"{{
            "jsonrpc": "2.0",
            "result": {{"data": "{}"}},
            "id": 1
        }}"#, large_value);
        
        let response: JsonRpcResponse = serde_json::from_str(&json).unwrap();
        assert!(response.result.is_some());
    }

    #[test]
    fn test_chaos_deeply_nested_result() {
        let json = r#"{
            "jsonrpc": "2.0",
            "result": {
                "level1": {
                    "level2": {
                        "level3": {
                            "level4": {
                                "level5": {"data": "deep"}
                            }
                        }
                    }
                }
            },
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_some());
    }

    // ====================================================================
    // FAULT INJECTION TESTS
    // ====================================================================

    #[test]
    fn test_fault_error_code_parse_error() {
        let json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": -32700,
                "message": "Parse error"
            },
            "id": null
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let error = response.error.unwrap();
        assert_eq!(error.code, -32700);
        assert_eq!(error.message, "Parse error");
    }

    #[test]
    fn test_fault_error_code_invalid_request() {
        let json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": -32600,
                "message": "Invalid Request"
            },
            "id": null
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
    }

    #[test]
    fn test_fault_error_code_method_not_found() {
        let json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": -32601,
                "message": "Method not found"
            },
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
    }

    #[test]
    fn test_fault_error_code_invalid_params() {
        let json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": -32602,
                "message": "Invalid params"
            },
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
    }

    #[test]
    fn test_fault_error_code_internal_error() {
        let json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": -32603,
                "message": "Internal error"
            },
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let error = response.error.unwrap();
        assert_eq!(error.code, -32603);
    }

    #[test]
    fn test_fault_error_with_data() {
        let json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": -32000,
                "message": "Server error",
                "data": {"detail": "Additional error information"}
            },
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let error = response.error.unwrap();
        assert_eq!(error.code, -32000);
        assert!(error.data.is_some());
    }

    #[test]
    fn test_fault_missing_required_field() {
        // Missing client_write_key should cause parsing to fail
        let json = r#"{
            "server_write_key": "OYSAPFlf/NAvJTpBtx45lnsFtRu3VEOK5tO/EK3kbx8=",
            "client_write_iv": "rkCk3xt3l2SBFeNu",
            "server_write_iv": "otHQEpR5P+EVqd9V"
        }"#;
        
        let result = serde_json::from_str::<Value>(json);
        assert!(result.is_ok());
        let value = result.unwrap();
        
        // Should be missing client_write_key
        assert!(value.get("client_write_key").is_none());
    }

    #[test]
    fn test_fault_invalid_base64() {
        let json = r#"{
            "client_write_key": "not-valid-base64!!!",
            "server_write_key": "also-invalid",
            "client_write_iv": "bad",
            "server_write_iv": "worse"
        }"#;
        
        // JSON parsing succeeds, but base64 decoding will fail
        let value: Value = serde_json::from_str(json).unwrap();
        assert!(value["client_write_key"].is_string());
        
        // Base64 decode will fail
        use base64::Engine;
        use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
        let result = BASE64_STANDARD.decode(value["client_write_key"].as_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_fault_unicode_in_error_message() {
        let json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": -32000,
                "message": "错误: Invalid 参数 🔥"
            },
            "id": 1
        }"#;
        
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let error = response.error.unwrap();
        assert!(error.message.contains("错误"));
        assert!(error.message.contains("🔥"));
    }

    #[test]
    fn test_fault_zero_length_keys() {
        let secrets = TlsSecrets {
            client_write_key: vec![],
            server_write_key: vec![],
            client_write_iv: vec![],
            server_write_iv: vec![],
            client_handshake_secret: vec![],
            server_handshake_secret: vec![],
        };
        
        assert_eq!(secrets.client_write_key.len(), 0);
        assert_eq!(secrets.server_write_key.len(), 0);
    }

    #[test]
    fn test_fault_mismatched_key_sizes() {
        // Keys should be 32 bytes, but we allow any size
        let secrets = TlsSecrets {
            client_write_key: vec![0u8; 16], // Wrong size!
            server_write_key: vec![0u8; 64], // Wrong size!
            client_write_iv: vec![0u8; 8],   // Wrong size!
            server_write_iv: vec![0u8; 24],  // Wrong size!
            client_handshake_secret: vec![0u8; 48], // Wrong size!
            server_handshake_secret: vec![0u8; 20], // Wrong size!
        };
        
        // Should still work, validation happens at crypto layer
        assert_eq!(secrets.client_write_key.len(), 16);
        assert_eq!(secrets.server_write_key.len(), 64);
    }
}

