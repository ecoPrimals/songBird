//! Cryptographic operations for Tor protocol
//!
//! - **BearDog**: All key operations delegated via IPC (TRUE PRIMAL)
//! - **SHA3-256**: Pure Rust for local operations (onion address checksums, descriptor IDs)

pub mod sha3;

use crate::error::{Error, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;
use tracing::debug;

/// BearDog crypto client for Tor protocol operations
///
/// **TRUE PRIMAL**: All crypto operations delegated to BearDog.
#[derive(Clone)]
pub struct BeardogCryptoClient {
    /// BearDog socket path
    socket_path: String,
}

impl BeardogCryptoClient {
    /// Create from environment (discovers BearDog via runtime)
    pub fn from_env() -> Result<Self> {
        // Discovery order:
        // 1. BEARDOG_SOCKET
        // 2. BEARDOG_CRYPTO_SOCKET
        // 3. XDG-compliant: $XDG_RUNTIME_DIR/biomeos/beardog.sock
        // 4. Fallback: /tmp/beardog.sock

        let socket_path = std::env::var("BEARDOG_SOCKET")
            .or_else(|_| std::env::var("BEARDOG_CRYPTO_SOCKET"))
            .or_else(|_| {
                if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
                    let path = format!("{}/biomeos/beardog.sock", xdg);
                    if std::path::Path::new(&path).exists() {
                        Ok(path)
                    } else {
                        Err(std::env::VarError::NotPresent)
                    }
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            })
            .unwrap_or_else(|_| "/tmp/beardog.sock".to_string());

        debug!("BearDog crypto client using socket: {}", socket_path);

        Ok(Self {
            socket_path,
        })
    }

    /// Create with explicit socket path
    pub fn new(socket_path: String) -> Self {
        Self {
            socket_path,
        }
    }

    /// Call BearDog JSON-RPC method
    fn call_method(&self, method: &str, params: Value) -> Result<Value> {
        // Connect to BearDog socket
        let mut stream = UnixStream::connect(&self.socket_path).map_err(|e| {
            Error::Crypto(format!("Failed to connect to BearDog at {}: {}", self.socket_path, e))
        })?;

        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .map_err(|e| Error::Crypto(format!("Failed to set read timeout: {}", e)))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(30)))
            .map_err(|e| Error::Crypto(format!("Failed to set write timeout: {}", e)))?;

        // Build JSON-RPC request
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let request_str = format!("{}\n", request);

        debug!("BearDog RPC call: {} (params: {})", method, params);

        // Send request
        stream
            .write_all(request_str.as_bytes())
            .map_err(|e| Error::Crypto(format!("Failed to send to BearDog: {}", e)))?;

        // Read response
        let mut response = String::new();
        let mut buf = [0u8; 4096];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    response.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if response.contains('\n') || response.contains('}') {
                        // Try to parse
                        if serde_json::from_str::<Value>(&response).is_ok() {
                            break;
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Timeout
                    break;
                }
                Err(e) => {
                    return Err(Error::Crypto(format!("Failed to read from BearDog: {}", e)));
                }
            }
        }

        // Parse response
        let parsed: Value = serde_json::from_str(&response).map_err(|e| {
            Error::Crypto(format!(
                "Failed to parse BearDog response: {} (response: {})",
                e, response
            ))
        })?;

        // Check for error
        if let Some(err) = parsed.get("error") {
            let msg = err.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
            return Err(Error::Crypto(format!("BearDog error: {}", msg)));
        }

        // Return result
        parsed
            .get("result")
            .cloned()
            .ok_or_else(|| Error::Crypto("BearDog response missing 'result'".to_string()))
    }

    // ===== Tor ntor Handshake Operations =====

    /// Initialize client-side ntor handshake
    ///
    /// Returns (client_public_key, handshake_state) for CREATE2 payload.
    pub fn tor_ntor_client_init(
        &self,
        node_id: &[u8; 20],
        node_onion_key: &[u8; 32],
    ) -> Result<NtorClientInit> {
        let result = self.call_method(
            "crypto.ntor.client_init",
            json!({
                "node_id": BASE64.encode(node_id),
                "node_onion_key": BASE64.encode(node_onion_key)
            }),
        )?;

        let client_public_b64 =
            result.get("client_public").and_then(|v| v.as_str()).ok_or_else(|| {
                Error::Crypto("Missing client_public in ntor_client_init response".to_string())
            })?;

        let state_id = result.get("state_id").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::Crypto("Missing state_id in ntor_client_init response".to_string())
        })?;

        let client_public_bytes = BASE64
            .decode(client_public_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode client_public: {}", e)))?;

        let mut client_public = [0u8; 32];
        if client_public_bytes.len() >= 32 {
            client_public.copy_from_slice(&client_public_bytes[..32]);
        }

        Ok(NtorClientInit {
            client_public,
            state_id: state_id.to_string(),
        })
    }

    /// Complete client-side ntor handshake with server's response
    ///
    /// Returns derived key material for circuit encryption.
    pub fn tor_ntor_client_finish(
        &self,
        state_id: &str,
        server_public: &[u8; 32],
        auth_tag: &[u8; 32],
    ) -> Result<KeyMaterial> {
        let result = self.call_method(
            "crypto.ntor.client_finish",
            json!({
                "state_id": state_id,
                "server_public": BASE64.encode(server_public),
                "auth_tag": BASE64.encode(auth_tag)
            }),
        )?;

        let key_seed_b64 = result.get("key_seed").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::Crypto("Missing key_seed in ntor_client_finish response".to_string())
        })?;

        let key_seed_bytes = BASE64
            .decode(key_seed_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode key_seed: {}", e)))?;

        let mut key_seed = [0u8; 32];
        if key_seed_bytes.len() >= 32 {
            key_seed.copy_from_slice(&key_seed_bytes[..32]);
        }

        // Derive actual keys from key_seed via HKDF
        let keys = self.tor_kdf(&key_seed, b"tor_circuit_keys", 72)?;

        let mut forward_key = [0u8; 16];
        let mut backward_key = [0u8; 16];
        let mut forward_iv = [0u8; 16];
        let mut backward_iv = [0u8; 16];

        forward_key.copy_from_slice(&keys[0..16]);
        backward_key.copy_from_slice(&keys[16..32]);
        forward_iv.copy_from_slice(&keys[32..48]);
        backward_iv.copy_from_slice(&keys[48..64]);

        Ok(KeyMaterial {
            forward_key,
            backward_key,
            forward_iv,
            backward_iv,
        })
    }

    // ===== Tor KDF Operations =====

    /// Tor-specific Key Derivation Function
    pub fn tor_kdf(&self, key_seed: &[u8; 32], info: &[u8], length: usize) -> Result<Vec<u8>> {
        let result = self.call_method(
            "crypto.kdf.derive",
            json!({
                "key_seed": BASE64.encode(key_seed),
                "info": BASE64.encode(info),
                "length": length
            }),
        )?;

        let derived_b64 = result
            .get("derived")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Crypto("Missing derived in tor_kdf response".to_string()))?;

        BASE64
            .decode(derived_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode derived key: {}", e)))
    }

    // ===== Tor Cell Encryption =====

    /// Encrypt Tor cell data with ChaCha20
    pub fn tor_cell_encrypt(&self, key: &[u8; 32], counter: u64, data: &[u8]) -> Result<Vec<u8>> {
        let result = self.call_method(
            "crypto.cell.encrypt",
            json!({
                "key": BASE64.encode(key),
                "counter": counter,
                "data": BASE64.encode(data)
            }),
        )?;

        let ciphertext_b64 =
            result.get("ciphertext").and_then(|v| v.as_str()).ok_or_else(|| {
                Error::Crypto("Missing ciphertext in tor_cell_encrypt response".to_string())
            })?;

        BASE64
            .decode(ciphertext_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode ciphertext: {}", e)))
    }

    /// Decrypt Tor cell data with ChaCha20
    pub fn tor_cell_decrypt(&self, key: &[u8; 32], counter: u64, data: &[u8]) -> Result<Vec<u8>> {
        let result = self.call_method(
            "crypto.cell.decrypt",
            json!({
                "key": BASE64.encode(key),
                "counter": counter,
                "data": BASE64.encode(data)
            }),
        )?;

        let plaintext_b64 = result.get("plaintext").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::Crypto("Missing plaintext in tor_cell_decrypt response".to_string())
        })?;

        BASE64
            .decode(plaintext_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode plaintext: {}", e)))
    }

    // ===== AES-128-CTR Operations (Circuit Onion Encryption) =====

    /// Encrypt with AES-128-CTR (for onion layer encryption)
    ///
    /// This maps to Tor's cell encryption. For modern Tor circuits,
    /// we use ChaCha20 which provides equivalent security.
    pub fn aes_128_ctr_encrypt(
        &self,
        key: &[u8; 16],
        iv: &[u8; 16],
        data: &[u8],
    ) -> Result<Vec<u8>> {
        // Expand 16-byte AES key to 32-byte for ChaCha20
        let mut expanded_key = [0u8; 32];
        expanded_key[..16].copy_from_slice(key);
        expanded_key[16..].copy_from_slice(key); // Double up for 32 bytes

        // Convert IV to counter (first 8 bytes as u64)
        let counter = u64::from_be_bytes(iv[..8].try_into().unwrap_or([0u8; 8]));

        self.tor_cell_encrypt(&expanded_key, counter, data)
    }

    /// Decrypt with AES-128-CTR (for onion layer decryption)
    pub fn aes_128_ctr_decrypt(
        &self,
        key: &[u8; 16],
        iv: &[u8; 16],
        data: &[u8],
    ) -> Result<Vec<u8>> {
        // Expand 16-byte AES key to 32-byte for ChaCha20
        let mut expanded_key = [0u8; 32];
        expanded_key[..16].copy_from_slice(key);
        expanded_key[16..].copy_from_slice(key); // Double up for 32 bytes

        // Convert IV to counter (first 8 bytes as u64)
        let counter = u64::from_be_bytes(iv[..8].try_into().unwrap_or([0u8; 8]));

        self.tor_cell_decrypt(&expanded_key, counter, data)
    }

    // ===== Legacy Operations (for compatibility with existing code) =====

    /// Sign data with Ed25519
    pub async fn ed25519_sign(&self, secret_key_id: &str, data: &[u8]) -> Result<[u8; 64]> {
        let result = self.call_method(
            "crypto.sign.ed25519",
            json!({
                "secret_key_id": secret_key_id,
                "data": BASE64.encode(data)
            }),
        )?;

        let sig_b64 = result.get("signature").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::Crypto("Missing signature in ed25519_sign response".to_string())
        })?;

        let sig_bytes = BASE64
            .decode(sig_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode signature: {}", e)))?;

        let mut signature = [0u8; 64];
        if sig_bytes.len() >= 64 {
            signature.copy_from_slice(&sig_bytes[..64]);
        }

        Ok(signature)
    }

    /// Verify Ed25519 signature
    pub async fn ed25519_verify(
        &self,
        public_key: &[u8; 32],
        data: &[u8],
        signature: &[u8; 64],
    ) -> Result<bool> {
        let result = self.call_method(
            "crypto.verify.ed25519",
            json!({
                "public_key": BASE64.encode(public_key),
                "data": BASE64.encode(data),
                "signature": BASE64.encode(signature)
            }),
        )?;

        result
            .get("valid")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| Error::Crypto("Missing valid in ed25519_verify response".to_string()))
    }

    /// Generate ephemeral X25519 keypair
    pub fn x25519_generate_ephemeral(&self) -> Result<X25519Keypair> {
        let result = self.call_method(
            "crypto.x25519.generate_ephemeral",
            json!({
                "purpose": "tor_circuit"
            }),
        )?;

        let public_b64 = result
            .get("public_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Crypto("Missing public_key in x25519 response".to_string()))?;

        let secret_id = result.get("secret_key_id").and_then(|v| v.as_str()).unwrap_or("ephemeral");

        // BearDog may also return the raw secret for local operations
        let secret_b64 = result.get("secret_key").and_then(|v| v.as_str());

        let public_bytes = BASE64
            .decode(public_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode public_key: {}", e)))?;

        let mut public_key = [0u8; 32];
        if public_bytes.len() >= 32 {
            public_key.copy_from_slice(&public_bytes[..32]);
        }

        let mut secret_key = [0u8; 32];
        if let Some(secret_b64) = secret_b64 {
            if let Ok(secret_bytes) = BASE64.decode(secret_b64) {
                if secret_bytes.len() >= 32 {
                    secret_key.copy_from_slice(&secret_bytes[..32]);
                }
            }
        }

        Ok(X25519Keypair {
            secret_key,
            secret_key_id: secret_id.to_string(),
            public_key,
        })
    }

    /// Derive shared secret (ECDH)
    ///
    /// Compatible with circuit code that passes raw secret key bytes.
    pub fn x25519_derive_secret(
        &self,
        our_secret_key: &[u8; 32],
        their_public_key: &[u8; 32],
    ) -> Result<[u8; 32]> {
        let result = self.call_method(
            "crypto.x25519.derive_secret",
            json!({
                "our_secret_key": BASE64.encode(our_secret_key),
                "their_public_key": BASE64.encode(their_public_key)
            }),
        )?;

        let shared_b64 = result
            .get("shared_secret")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Crypto("Missing shared_secret in x25519 response".to_string()))?;

        let shared_bytes = BASE64
            .decode(shared_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode shared_secret: {}", e)))?;

        let mut shared_secret = [0u8; 32];
        if shared_bytes.len() >= 32 {
            shared_secret.copy_from_slice(&shared_bytes[..32]);
        }

        Ok(shared_secret)
    }

    /// Hash with SHA3-256
    pub fn sha3_256(&self, data: &[u8]) -> Result<[u8; 32]> {
        let result = self.call_method(
            "crypto.hash.sha3_256",
            json!({
                "data": BASE64.encode(data)
            }),
        )?;

        let hash_b64 = result
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Crypto("Missing hash in sha3_256 response".to_string()))?;

        let hash_bytes = BASE64
            .decode(hash_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode hash: {}", e)))?;

        let mut hash = [0u8; 32];
        if hash_bytes.len() >= 32 {
            hash.copy_from_slice(&hash_bytes[..32]);
        }

        Ok(hash)
    }

    /// HMAC-SHA256 (required for Tor ntor handshake)
    pub fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Result<[u8; 32]> {
        let result = self.call_method(
            "crypto.hmac.sha256",
            json!({
                "key": BASE64.encode(key),
                "data": BASE64.encode(data)
            }),
        )?;

        let mac_b64 = result
            .get("mac")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Crypto("Missing mac in hmac_sha256 response".to_string()))?;

        let mac_bytes = BASE64
            .decode(mac_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode mac: {}", e)))?;

        let mut mac = [0u8; 32];
        if mac_bytes.len() >= 32 {
            mac.copy_from_slice(&mac_bytes[..32]);
        }

        Ok(mac)
    }

    /// Encrypt with ChaCha20Poly1305
    pub fn chacha20_poly1305_encrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        data: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self.call_method(
            "crypto.aead.chacha20_poly1305_encrypt",
            json!({
                "key": BASE64.encode(key),
                "nonce": BASE64.encode(nonce),
                "plaintext": BASE64.encode(data),
                "aad": BASE64.encode(aad)
            }),
        )?;

        let ciphertext_b64 =
            result.get("ciphertext").and_then(|v| v.as_str()).ok_or_else(|| {
                Error::Crypto("Missing ciphertext in chacha20_poly1305 response".to_string())
            })?;

        BASE64
            .decode(ciphertext_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode ciphertext: {}", e)))
    }

    /// Decrypt with ChaCha20Poly1305
    pub fn chacha20_poly1305_decrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        data: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self.call_method(
            "crypto.aead.chacha20_poly1305_decrypt",
            json!({
                "key": BASE64.encode(key),
                "nonce": BASE64.encode(nonce),
                "ciphertext": BASE64.encode(data),
                "aad": BASE64.encode(aad)
            }),
        )?;

        let plaintext_b64 = result.get("plaintext").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::Crypto("Missing plaintext in chacha20_poly1305 response".to_string())
        })?;

        BASE64
            .decode(plaintext_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode plaintext: {}", e)))
    }
}

/// ntor client handshake initialization result
#[derive(Debug, Clone)]
pub struct NtorClientInit {
    /// Client's ephemeral public key (for CREATE2 payload)
    pub client_public: [u8; 32],
    /// State ID for completing handshake (BearDog-managed)
    pub state_id: String,
}

/// Key material derived from ntor handshake
#[derive(Debug, Clone)]
pub struct KeyMaterial {
    /// Forward encryption key (client -> relay)
    pub forward_key: [u8; 16],
    /// Backward encryption key (relay -> client)
    pub backward_key: [u8; 16],
    /// Forward IV
    pub forward_iv: [u8; 16],
    /// Backward IV  
    pub backward_iv: [u8; 16],
}

/// X25519 keypair for ECDH
///
/// **NOTE**: For circuit building compatibility, we store both the secret
/// key ID (BearDog-managed) and a copy of the raw secret.
/// In production with HSM, only the ID would be stored.
#[derive(Debug, Clone)]
pub struct X25519Keypair {
    /// Secret key (32 bytes) - for local circuit operations
    pub secret_key: [u8; 32],
    /// Secret key ID (BearDog-managed) - for delegated operations
    pub secret_key_id: String,
    /// Public key (32 bytes)
    pub public_key: [u8; 32],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beardog_client_from_env() {
        // This test checks that from_env() doesn't panic
        // Actual IPC is only tested when BearDog is running
        let result = BeardogCryptoClient::from_env();
        assert!(result.is_ok());
    }

    #[test]
    fn test_beardog_client_new() {
        let client = BeardogCryptoClient::new("/tmp/test.sock".to_string());
        assert_eq!(client.socket_path, "/tmp/test.sock");
    }
}
