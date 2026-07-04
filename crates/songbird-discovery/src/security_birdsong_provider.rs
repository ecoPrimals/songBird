// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security-provider `BirdSong` encryption
//!
//! Implements family-based encryption via the security provider JSON-RPC API (consumed as
//! [`crate::birdsong::BirdSongEncryption::Security`]).
//! This provider connects to the crypto provider's encryption API to encrypt/decrypt discovery
//! packets based on genetic family lineage.

use anyhow::Result;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use songbird_universal::UnixRpcClient;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, trace, warn};

/// `security provider` encryption request (for JSON-RPC birdsong.encrypt method)
#[derive(Debug, Clone, Serialize)]
struct SecurityProviderEncryptRequest {
    /// Plaintext data to encrypt (base64 encoded automatically by serde)
    #[serde(with = "base64_serde")]
    plaintext: Vec<u8>,

    /// Optional family ID (uses node's family if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    family_id: Option<String>,
}

/// Base64 serialization helper (matching the provider's format)
mod base64_serde {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(data))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        STANDARD.decode(s).map_err(serde::de::Error::custom)
    }
}

/// Security-provider encryption response (adaptive format)
///
/// Handles both v1 ("encrypted") and v2 ("ciphertext") field names
/// for backward compatibility and graceful API evolution.
#[derive(Debug, Clone, Deserialize)]
struct SecurityProviderEncryptResponse {
    /// Encrypted data (deserialized from base64 automatically)
    /// Supports both "ciphertext" (v2) and "encrypted" (v1) field names
    #[serde(alias = "encrypted")] // v1 compatibility
    #[serde(with = "base64_serde")]
    ciphertext: Vec<u8>,

    /// Family ID used for encryption
    family_id: String,
}

/// Security-provider decryption request
#[derive(Debug, Clone, Serialize)]
struct SecurityProviderDecryptRequest {
    /// Ciphertext to decrypt (base64 encoded automatically)
    #[serde(with = "base64_serde")]
    ciphertext: Vec<u8>,

    /// Family ID for decryption (required by the provider)
    #[serde(skip_serializing_if = "Option::is_none")]
    family_id: Option<String>,
}

/// Security-provider decryption response
#[derive(Debug, Clone, Deserialize)]
struct SecurityProviderDecryptResponse {
    /// Decrypted plaintext (deserialized from base64 automatically)
    #[serde(with = "base64_serde")]
    plaintext: Vec<u8>,

    /// Family ID that encrypted this
    family_id: String,

    /// Whether we can decrypt (same family)
    success: bool,
}

/// Connection type for `security provider` (Unix socket or TCP)
#[derive(Debug, Clone)]
#[allow(
    dead_code,
    reason = "constructed by connect() and pattern-matched in route(); callers wired incrementally"
)]
enum SecurityConnection {
    /// Unix socket path
    Unix(PathBuf),
    /// TCP address (host, port)
    Tcp(String, u16),
}

/// Security-provider `BirdSong` encryption
///
/// Connects to the provider's JSON-RPC interface to provide family-based encryption
/// for discovery packets. Only peers from the same genetic family can
/// decrypt each other's packets.
///
/// **Pure Rust Implementation**: Supports both Unix sockets and TCP connections
/// for cross-platform compatibility (Android uses TCP due to Unix socket restrictions).
///
/// ## Connection Formats
///
/// - Unix socket: `/path/to/security.sock` (or legacy provider-specific path)
/// - TCP socket: `tcp:host:port` (e.g., `tcp:127.0.0.1:9900`)
pub struct SecurityBirdSongProvider {
    /// Connection type (Unix socket or TCP)
    #[allow(
        dead_code,
        reason = "stored for reconnection logic; read when connection routing is wired"
    )]
    connection: SecurityConnection,

    /// Legacy socket path field name (for backward compatibility)
    #[allow(
        dead_code,
        reason = "kept for backward-compat introspection; superseded by `connection`"
    )]
    socket_path: PathBuf,

    /// JSON-RPC client for provider communication (Pure Rust!)
    /// For Unix sockets only - TCP uses direct connection per request
    client: Option<UnixRpcClient>,

    /// TCP endpoint for direct connections (Android support)
    tcp_endpoint: Option<(String, u16)>,

    /// Our family ID (cached from identity query)
    family_id: Option<String>,

    /// Provider availability
    available: bool,
}

impl SecurityBirdSongProvider {
    /// Create new security-provider `BirdSong` adapter (async factory method)
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Crypto provider socket path. Supports:
    ///   - Unix socket: `/tmp/security.sock` (or capability-discovered path)
    ///   - TCP socket: `tcp:host:port` (e.g., `tcp:127.0.0.1:9900`)
    /// * `family_id` - Optional family ID (will query the provider if not provided)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use songbird_discovery::security_birdsong_provider::SecurityBirdSongProvider;
    ///
    /// # async fn example() {
    /// // Unix socket
    /// let provider = SecurityBirdSongProvider::new(
    ///     "/tmp/security.sock",
    ///     Some(String::from("ecoPrimals-family-123"))
    /// ).await.unwrap();
    ///
    /// // TCP socket (Android)
    /// let provider = SecurityBirdSongProvider::new(
    ///     "tcp:127.0.0.1:9900",
    ///     Some(String::from("ecoPrimals-family-123"))
    /// ).await.unwrap();
    /// # }
    /// ```
    pub async fn new(socket_path: impl Into<PathBuf>, family_id: Option<String>) -> Result<Self> {
        let socket_path = socket_path.into();
        let path_str = socket_path.to_string_lossy().to_string();

        // Check if this is a TCP connection (tcp:host:port format)
        if let Some(addr) = path_str.strip_prefix("tcp:") {
            // Remove "tcp:" prefix
            let parts: Vec<&str> = addr.rsplitn(2, ':').collect();

            if parts.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid TCP address format: {path_str}. Expected tcp:host:port"
                ));
            }

            let port: u16 = parts[0]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid port in TCP address: {}", parts[0]))?;
            let host = parts[1].to_string();

            info!("🎵 Security-provider BirdSong adapter created (TCP: {}:{})", host, port);
            if let Some(ref fam) = family_id {
                info!("   Family ID: {}", fam);
            }

            return Ok(Self {
                connection: SecurityConnection::Tcp(host.clone(), port),
                socket_path,
                client: None,
                tcp_endpoint: Some((host, port)),
                family_id,
                available: true,
            });
        }

        // Unix socket connection
        let client = UnixRpcClient::new(&socket_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to connect to security provider at {}: {e}",
                socket_path.display()
            )
        })?;

        info!("🎵 Security-provider BirdSong adapter created (Pure Rust Unix socket!)");
        info!("   Socket: {:?}", socket_path);
        if let Some(ref fam) = family_id {
            info!("   Family ID: {}", fam);
        }

        Ok(Self {
            connection: SecurityConnection::Unix(socket_path.clone()),
            socket_path,
            client: Some(client),
            tcp_endpoint: None,
            family_id,
            available: true,
        })
    }

    /// Make a JSON-RPC call via TCP
    ///
    /// Used for Android deployments where Unix sockets are restricted.
    async fn tcp_call<P, R>(&self, method: &str, params: P) -> Result<R, String>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let (host, port) = self
            .tcp_endpoint
            .as_ref()
            .ok_or_else(|| String::from("TCP endpoint not configured"))?;

        trace!("TCP JSON-RPC call to {}:{} method: {}", host, port, method);

        // Build JSON-RPC request
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let request_bytes = serde_json::to_vec(&request)
            .map_err(|e| format!("Failed to serialize request: {e}"))?;

        // Connect via TCP
        let addr = format!("{host}:{port}");
        let mut stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| format!("Failed to connect to security provider at {addr}: {e}"))?;

        // Send request
        stream
            .write_all(&request_bytes)
            .await
            .map_err(|e| format!("Failed to write request: {e}"))?;
        stream.write_all(b"\n").await.map_err(|e| format!("Failed to write delimiter: {e}"))?;
        stream.shutdown().await.map_err(|e| format!("Failed to shutdown write: {e}"))?;

        // Read response
        let mut response_bytes = Vec::new();
        stream
            .read_to_end(&mut response_bytes)
            .await
            .map_err(|e| format!("Failed to read response: {e}"))?;

        trace!("TCP received {} bytes", response_bytes.len());

        // Parse JSON-RPC response
        #[derive(Deserialize)]
        struct JsonRpcResponse<T> {
            result: Option<T>,
            error: Option<JsonRpcError>,
        }

        #[derive(Deserialize)]
        struct JsonRpcError {
            code: i32,
            message: String,
        }

        let response: JsonRpcResponse<R> = serde_json::from_slice(&response_bytes)
            .map_err(|e| format!("Failed to parse response: {e}"))?;

        if let Some(error) = response.error {
            return Err(format!("JSON-RPC error {}: {}", error.code, error.message));
        }

        response.result.ok_or_else(|| String::from("Missing result in response"))
    }

    /// Async health check for security-provider availability
    ///
    /// Uses JSON-RPC `health` method for pure Rust health checking.
    pub async fn check_health(&self) -> bool {
        #[derive(Deserialize)]
        struct HealthResponse {
            status: String,
        }

        let result: Result<HealthResponse, _> = if self.tcp_endpoint.is_some() {
            self.tcp_call("health", json!({})).await.map_err(|e| anyhow::anyhow!(e))
        } else if let Some(ref client) = self.client {
            client.call_no_params::<HealthResponse>("health").await
        } else {
            return false;
        };

        match result {
            Ok(response) => {
                let is_ok = response.status == "healthy";
                if is_ok {
                    info!("✅ Security provider health check passed (Pure Rust RPC!)");
                } else {
                    warn!(
                        "⚠️  Security provider health check failed: status = {}",
                        response.status
                    );
                }
                is_ok
            }
            Err(e) => {
                warn!("⚠️  Security provider health check error: {}", e);
                false
            }
        }
    }

    /// Generic RPC call dispatcher: tries the given method over TCP or Unix.
    async fn rpc_call<P: Serialize + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: &P,
    ) -> Result<R, String> {
        if self.tcp_endpoint.is_some() {
            self.tcp_call(method, params).await
        } else if let Some(ref client) = self.client {
            client.call(method, params).await.map_err(|e| format!("{method} JSON-RPC failed: {e}"))
        } else {
            Err(String::from("No security provider connection available"))
        }
    }

    // ─── Legacy (nuclear/lineage tier) encrypt/decrypt ───────────────────

    /// Encrypt using `birdsong.encrypt` (lineage/nuclear tier).
    ///
    /// This is the legacy path that uses the family's lineage seed. For Dark
    /// Forest beacons, prefer [`encrypt_beacon_tier`] which uses beacon-tier
    /// credentials per `DARK_FOREST_BEACON_GENETICS_STANDARD.md`.
    async fn encrypt_internal(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        use songbird_types::defaults::beacon::LEGACY_BIRDSONG_ENCRYPT_METHOD;

        let request = SecurityProviderEncryptRequest {
            plaintext: plaintext.to_vec(),
            family_id: self.family_id.clone(),
        };

        debug!("Lineage-tier encryption via {LEGACY_BIRDSONG_ENCRYPT_METHOD}");
        trace!("   Plaintext size: {} bytes", plaintext.len());

        let resp: SecurityProviderEncryptResponse =
            self.rpc_call(LEGACY_BIRDSONG_ENCRYPT_METHOD, &request).await?;

        debug!(
            "Encrypted {} -> {} bytes (family: {})",
            plaintext.len(),
            resp.ciphertext.len(),
            resp.family_id
        );
        Ok(resp.ciphertext)
    }

    /// Decrypt using `birdsong.decrypt` (lineage/nuclear tier).
    async fn decrypt_internal(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>, String> {
        use songbird_types::defaults::beacon::LEGACY_BIRDSONG_DECRYPT_METHOD;

        let request = SecurityProviderDecryptRequest {
            ciphertext: ciphertext.to_vec(),
            family_id: self.family_id.clone(),
        };

        debug!("Lineage-tier decryption via {LEGACY_BIRDSONG_DECRYPT_METHOD}");

        let resp: SecurityProviderDecryptResponse =
            match self.rpc_call(LEGACY_BIRDSONG_DECRYPT_METHOD, &request).await {
                Ok(r) => r,
                Err(e) => {
                    debug!("Decrypt RPC error (likely different family): {e}");
                    return Err(format!("{LEGACY_BIRDSONG_DECRYPT_METHOD} failed: {e}"));
                }
            };

        if !resp.success {
            debug!("Noise: different family ({})", resp.family_id);
            return Ok(None);
        }

        debug!(
            "Decrypted {} -> {} bytes (family: {})",
            ciphertext.len(),
            resp.plaintext.len(),
            resp.family_id
        );
        Ok(Some(resp.plaintext))
    }

    // ─── Beacon (mitochondrial) tier encrypt/decrypt ─────────────────────
    //
    // Per DARK_FOREST_BEACON_GENETICS_STANDARD.md, Dark Forest beacons MUST
    // use beacon-tier credentials (mitochondrial) and never nuclear/lineage
    // material. These methods call `beacon.encrypt` / `beacon.decrypt` RPCs
    // which operate on the beacon seed, not the lineage seed.
    //
    // Falls back to legacy `birdsong.*` methods when the security provider
    // doesn't yet support `beacon.*` RPCs (graceful degradation).

    /// Encrypt using `beacon.encrypt` (mitochondrial/beacon tier).
    ///
    /// Falls back to `birdsong.encrypt` if the security provider returns
    /// `Method not found` for the beacon-tier method.
    async fn encrypt_beacon_tier(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        use songbird_types::defaults::beacon::BEACON_ENCRYPT_METHOD;

        let request = SecurityProviderEncryptRequest {
            plaintext: plaintext.to_vec(),
            family_id: self.family_id.clone(),
        };

        debug!("Beacon-tier encryption via {BEACON_ENCRYPT_METHOD}");

        match self
            .rpc_call::<_, SecurityProviderEncryptResponse>(BEACON_ENCRYPT_METHOD, &request)
            .await
        {
            Ok(resp) => {
                debug!(
                    "Beacon-tier encrypted {} -> {} bytes",
                    plaintext.len(),
                    resp.ciphertext.len()
                );
                Ok(resp.ciphertext)
            }
            Err(e) if e.contains("Method not found") || e.contains("-32601") => {
                debug!("{BEACON_ENCRYPT_METHOD} not supported — falling back to lineage-tier");
                self.encrypt_internal(plaintext).await
            }
            Err(e) => Err(e),
        }
    }

    /// Decrypt using `beacon.decrypt` (mitochondrial/beacon tier).
    ///
    /// Falls back to `birdsong.decrypt` if the security provider returns
    /// `Method not found` for the beacon-tier method.
    async fn decrypt_beacon_tier(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>, String> {
        use songbird_types::defaults::beacon::BEACON_DECRYPT_METHOD;

        let request = SecurityProviderDecryptRequest {
            ciphertext: ciphertext.to_vec(),
            family_id: self.family_id.clone(),
        };

        debug!("Beacon-tier decryption via {BEACON_DECRYPT_METHOD}");

        match self
            .rpc_call::<_, SecurityProviderDecryptResponse>(BEACON_DECRYPT_METHOD, &request)
            .await
        {
            Ok(resp) => {
                if !resp.success {
                    debug!("Beacon-tier noise: different beacon family");
                    return Ok(None);
                }
                Ok(Some(resp.plaintext))
            }
            Err(e) if e.contains("Method not found") || e.contains("-32601") => {
                debug!("{BEACON_DECRYPT_METHOD} not supported — falling back to lineage-tier");
                self.decrypt_internal(ciphertext).await
            }
            Err(e) => {
                debug!("Beacon-tier decrypt error: {e}");
                Err(e)
            }
        }
    }

    /// Query `beacon.get_id` for our public beacon identifier.
    async fn query_beacon_id(&self) -> Result<Option<Vec<u8>>, String> {
        use songbird_types::defaults::beacon::BEACON_GET_ID_METHOD;

        #[derive(Deserialize)]
        struct BeaconIdResponse {
            #[serde(default)]
            #[serde(with = "optional_base64_serde")]
            beacon_id: Option<Vec<u8>>,
        }

        debug!("Querying {BEACON_GET_ID_METHOD}");

        match self
            .rpc_call::<_, BeaconIdResponse>(BEACON_GET_ID_METHOD, &serde_json::Value::Null)
            .await
        {
            Ok(resp) => Ok(resp.beacon_id),
            Err(e) if e.contains("Method not found") || e.contains("-32601") => {
                debug!("{BEACON_GET_ID_METHOD} not supported — beacon genetics not available");
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Human-readable provider label (logging).
    #[must_use]
    pub fn provider_name(&self) -> String {
        String::from("Security provider")
    }

    /// Cached lineage family id, if known.
    #[must_use]
    pub fn family_id(&self) -> Option<String> {
        self.family_id.clone()
    }

    /// Whether the security adapter considers the remote provider usable.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Encrypt discovery packet for same-family peers (lineage tier).
    pub async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        if !self.available {
            return Err(anyhow::anyhow!("Security provider not available"));
        }
        self.encrypt_internal(plaintext).await.map_err(|e| anyhow::anyhow!(e))
    }

    /// Decrypt discovery packet from same-family peer.
    pub async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
        if !self.available {
            return Err(anyhow::anyhow!("Security provider not available"));
        }
        self.decrypt_internal(ciphertext).await.map_err(|e| anyhow::anyhow!(e))
    }

    /// Encrypt payload for Dark Forest beacon (beacon tier, with random nonce).
    pub async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        if !self.available {
            return Err(anyhow::anyhow!("Security provider not available"));
        }
        let encrypted = self.encrypt_beacon_tier(payload).await.map_err(|e| anyhow::anyhow!(e))?;
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce)
            .map_err(|e| anyhow::anyhow!("Failed to generate random nonce: {e}"))?;
        Ok((encrypted, nonce))
    }

    /// Try decrypting a Dark Forest beacon using beacon-tier credentials.
    pub async fn try_decrypt_beacon(
        &self,
        encrypted: &[u8],
        _nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        if !self.available {
            return Err(anyhow::anyhow!("Security provider not available"));
        }
        self.decrypt_beacon_tier(encrypted).await.map_err(|e| anyhow::anyhow!(e))
    }

    /// Query public beacon id from the security provider, if supported.
    pub async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        if !self.available {
            return Err(anyhow::anyhow!("Security provider not available"));
        }
        self.query_beacon_id().await.map_err(|e| anyhow::anyhow!(e))
    }

    /// Known beacon ids from meetings (not tracked by this adapter).
    pub async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        let _ = self;
        Ok(Vec::new())
    }

    /// Whether Dark Forest beacon support appears active (beacon id present).
    pub async fn supports_dark_forest(&self) -> bool {
        self.get_beacon_id().await.ok().flatten().is_some()
    }
}

/// Optional base64 deserialization helper for beacon ID responses.
mod optional_base64_serde {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) if !s.is_empty() => {
                STANDARD.decode(s).map(Some).map_err(serde::de::Error::custom)
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
#[path = "security_birdsong_provider_tests.rs"]
mod tests;
