// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `BearDog` `BirdSong` Provider - Production & Test Implementations
//!
//! Production implementation connects to `BearDog` via Unix socket JSON-RPC.
//! Test mocks allow testing lineage relay without `BearDog` running.
//!
//! ## Deep Debt Compliance
//!
//! - ✅ Modern async Rust (trait-based, async/await)
//! - ✅ Zero unsafe code
//! - ✅ Runtime discovery (no hardcoded paths)
//! - ✅ Mocks isolated to `#[cfg(any(test, feature = "test-utils"))]`
//! - ✅ Pure Rust (Unix sockets, not HTTP)

use crate::birdsong::{BirdSongCrypto, LineageHint};
use crate::error::Result;
use crate::relay::RelayAuthority;
use crate::types::{MaskingLevel, NodeId, RelayAuthorization};
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{debug, info};

// Imports only used by mock implementations
#[cfg(any(test, feature = "test-utils"))]
use std::collections::HashMap;
#[cfg(any(test, feature = "test-utils"))]
use std::sync::Arc;
#[cfg(any(test, feature = "test-utils"))]
use tokio::sync::RwLock;

/// Production `BearDog` `BirdSong` Provider
///
/// Connects to `BearDog` via Unix socket JSON-RPC to provide lineage-based
/// encryption for relay broadcasts. Only family members with lineage proofs
/// can decrypt messages.
///
/// ## Deep Debt Principles
///
/// - Runtime discovery (socket path via env or discovery)
/// - Zero unsafe code (pure Rust async)
/// - Trait-based (implements `BirdSongCrypto`)
/// - Graceful error handling
pub struct BearDogBirdSongProvider {
    socket_path: PathBuf,
    family_id: Option<String>,
}

impl BearDogBirdSongProvider {
    /// Create new `BearDog` `BirdSong` provider
    ///
    /// # Arguments
    ///
    /// * `socket_path` - `BearDog` Unix socket path (discovered at runtime)
    /// * `family_id` - Optional family ID for validation
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use songbird_lineage_relay::beardog::BearDogBirdSongProvider;
    ///
    /// # async fn example() {
    /// let provider = BearDogBirdSongProvider::new(
    ///     "/tmp/beardog.sock",
    ///     Some("ecoPrimals-family-123".to_string())
    /// );
    /// # }
    /// ```
    #[must_use]
    pub fn new(socket_path: impl Into<PathBuf>, family_id: Option<String>) -> Self {
        let socket_path = socket_path.into();

        info!("🐻 BearDog BirdSong provider created (Unix socket)");
        info!("   Socket: {:?}", socket_path);
        if let Some(ref fam) = family_id {
            info!("   Family ID: {}", fam);
        }

        Self {
            socket_path,
            family_id,
        }
    }

    /// Test-only accessors (see `beardog_tests` module; submodules could read fields inline).
    #[cfg(test)]
    pub(crate) fn test_socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Test-only accessor for configured family id.
    #[cfg(test)]
    pub(crate) fn test_family_id(&self) -> Option<&String> {
        self.family_id.as_ref()
    }

    /// Call `BearDog` JSON-RPC method via Unix socket
    ///
    /// Pure Rust implementation using tokio `UnixStream`.
    async fn call_beardog(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Connect to BearDog Unix socket
        let mut stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to connect to BearDog at {}: {}",
                self.socket_path.display(),
                e
            ))
        })?;

        // Build JSON-RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        // Serialize and send
        let request_bytes = serde_json::to_vec(&request)?;

        stream.write_all(&request_bytes).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to write to BearDog: {e}"
            ))
        })?;
        stream.write_all(b"\n").await.ok(); // Newline delimiter

        // Read response
        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to read from BearDog: {e}"
            ))
        })?;

        // Parse JSON-RPC response
        let response: serde_json::Value = serde_json::from_slice(&response_bytes)?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return Err(crate::error::LineageRelayError::BirdSongError(format!(
                "BearDog RPC error: {error}"
            )));
        }

        // Return result
        response.get("result").cloned().ok_or_else(|| {
            crate::error::LineageRelayError::BirdSongError(
                "No result in BearDog response".to_string(),
            )
        })
    }
}

#[async_trait]
impl BirdSongCrypto for BearDogBirdSongProvider {
    async fn encrypt_for_lineage(&self, message: &[u8], hint: LineageHint) -> Result<Vec<u8>> {
        debug!("🔒 Encrypting for lineage via BearDog (hint: {:?})", hint);

        // Encode message as base64 for JSON-RPC
        use base64::{Engine as _, engine::general_purpose};
        let plaintext_b64 = general_purpose::STANDARD.encode(message);

        // Build request params
        let params = serde_json::json!({
            "plaintext": plaintext_b64,
            "family_id": self.family_id,
            "lineage_hint": format!("{:?}", hint) // Serialized for BearDog
        });

        // Call birdsong.encrypt
        let result = self.call_beardog("birdsong.encrypt", params).await?;

        // Extract ciphertext
        let ciphertext_b64 = result
            .get("ciphertext")
            .or_else(|| result.get("encrypted")) // v1 compatibility
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::LineageRelayError::BirdSongError(
                    "No ciphertext in BearDog encrypt response".to_string(),
                )
            })?;

        let ciphertext = general_purpose::STANDARD.decode(ciphertext_b64).map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Invalid base64 ciphertext: {e}"
            ))
        })?;

        debug!("✅ Encrypted {} → {} bytes", message.len(), ciphertext.len());
        Ok(ciphertext)
    }

    async fn decrypt_birdsong(&self, encrypted: &[u8], sender: &NodeId) -> Result<Option<Vec<u8>>> {
        debug!("🔓 Decrypting BirdSong from {:?}", sender);

        // Encode ciphertext as base64 for JSON-RPC
        use base64::{Engine as _, engine::general_purpose};
        let ciphertext_b64 = general_purpose::STANDARD.encode(encrypted);

        // Build request params
        let params = serde_json::json!({
            "ciphertext": ciphertext_b64,
            "sender_node_id": sender.0
        });

        // Call birdsong.decrypt
        let Ok(result) = self.call_beardog("birdsong.decrypt", params).await else {
            // Decryption failure might just mean different family (noise)
            debug!("🔇 BearDog decrypt failed - likely different family (noise)");
            return Ok(None);
        };

        // Check success flag
        let success = result.get("success").and_then(serde_json::Value::as_bool).unwrap_or(false);

        if !success {
            debug!("🔇 BearDog decrypt: different family (noise)");
            return Ok(None);
        }

        // Extract plaintext
        let plaintext_b64 = result.get("plaintext").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::LineageRelayError::BirdSongError(
                "No plaintext in BearDog decrypt response".to_string(),
            )
        })?;

        let plaintext = general_purpose::STANDARD.decode(plaintext_b64).map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!("Invalid base64 plaintext: {e}"))
        })?;

        debug!("✅ Decrypted {} bytes from family", plaintext.len());
        Ok(Some(plaintext))
    }
}

// ═══════════════════════════════════════════════════════════════════
// PRODUCTION: BearDog Relay Authority
// Delegates lineage verification to BearDog via Unix socket JSON-RPC.
// Replaces MockRelayAuthority in all production constructors.
// ═══════════════════════════════════════════════════════════════════

/// Production relay authority backed by `BearDog`
///
/// Delegates lineage-based relay authorization to `BearDog` via Unix socket
/// JSON-RPC. No hardcoded lineage graphs -- `BearDog` owns the truth.
///
/// ## Deep Debt Compliance
///
/// - ✅ Real implementation (not a mock)
/// - ✅ Runtime discovery (socket path via env or discovery)
/// - ✅ Zero unsafe code
/// - ✅ Async/await
pub struct BearDogRelayAuthority {
    socket_path: PathBuf,
}

impl BearDogRelayAuthority {
    /// Create new `BearDog` relay authority
    ///
    /// Discovers security provider socket path at runtime:
    /// 1. `SECURITY_PROVIDER_SOCKET` / `CRYPTO_PROVIDER_SOCKET` / `BEARDOG_SOCKET` (see [`Self::discover_socket_path`])
    /// 2. XDG runtime dir capability-named sockets under `biomeos/`
    /// 3. Legacy fallbacks under `/tmp/biomeos/` or `/tmp/`
    pub fn new() -> Self {
        let socket_path = Self::discover_socket_path();
        info!("BearDog relay authority created (socket: {:?})", socket_path);
        Self {
            socket_path,
        }
    }

    /// Create with explicit socket path
    pub fn with_socket_path(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
        }
    }

    /// Discover security provider socket path at runtime (capability-first)
    ///
    /// ## Resolution Order (capability-first, primal-agnostic)
    ///
    /// 1. `SECURITY_PROVIDER_SOCKET` - Capability-based (preferred)
    /// 2. `CRYPTO_PROVIDER_SOCKET` - Capability-based alternative
    /// 3. `BEARDOG_SOCKET` - Provider-specific (backward compatibility)
    /// 4. XDG: `$XDG_RUNTIME_DIR/biomeos/security.sock` - Capability-named
    /// 5. XDG: `$XDG_RUNTIME_DIR/biomeos/beardog.sock` - Provider hint
    /// 6. Legacy: `/tmp/biomeos/security.sock` - Fallback
    fn discover_socket_path() -> PathBuf {
        // 1. Capability-based env vars (preferred - primal agnostic)
        for env_var in &[
            "SECURITY_PROVIDER_SOCKET",
            "CRYPTO_PROVIDER_SOCKET",
            "BEARDOG_SOCKET", // backward compatibility
        ] {
            if let Ok(path) = songbird_process_env::var(env_var) {
                return PathBuf::from(path);
            }
        }

        // 2. XDG runtime directory (capability names first, then provider hints)
        if let Ok(xdg) = songbird_process_env::var("XDG_RUNTIME_DIR") {
            let biomeos = PathBuf::from(&xdg).join("biomeos");

            // Capability-named sockets only — no primal identities
            for socket_name in &["security.sock", "crypto.sock"] {
                let path = biomeos.join(socket_name);
                if path.exists() {
                    return path;
                }
            }
        }

        // 3. Legacy fallback (capability name preferred)
        let fallback_paths =
            ["/tmp/biomeos/security.sock", "/tmp/biomeos/beardog.sock", "/tmp/security.sock"];

        for path in fallback_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                return path_buf;
            }
        }

        // Final fallback (most common provider)
        PathBuf::from("/tmp/biomeos/security.sock")
    }

    /// Call `BearDog` JSON-RPC method via Unix socket
    async fn call_beardog(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let mut stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to connect to BearDog at {}: {}",
                self.socket_path.display(),
                e
            ))
        })?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let request_bytes = serde_json::to_vec(&request)?;
        stream.write_all(&request_bytes).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to write to BearDog: {e}"
            ))
        })?;
        stream.write_all(b"\n").await.ok();

        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to read from BearDog: {e}"
            ))
        })?;

        let response: serde_json::Value = serde_json::from_slice(&response_bytes)?;

        if let Some(error) = response.get("error") {
            return Err(crate::error::LineageRelayError::BirdSongError(format!(
                "BearDog RPC error: {error}"
            )));
        }

        response.get("result").cloned().ok_or_else(|| {
            crate::error::LineageRelayError::BirdSongError(
                "No result in BearDog response".to_string(),
            )
        })
    }

    /// Parse masking level from `BearDog` response string
    fn parse_masking_level(level: Option<&str>) -> MaskingLevel {
        match level.unwrap_or("full_visibility") {
            "none" => MaskingLevel::None,
            "timing_only" => MaskingLevel::TimingOnly,
            "size_obfuscation" => MaskingLevel::SizeObfuscation,
            "full" => MaskingLevel::Full,
            "masked" => MaskingLevel::Masked,
            "sub_masked" => MaskingLevel::SubMasked,
            _ => MaskingLevel::FullVisibility,
        }
    }
}

impl Default for BearDogRelayAuthority {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RelayAuthority for BearDogRelayAuthority {
    async fn authorize_relay(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<RelayAuthorization> {
        debug!("Authorizing relay: {} -> {} via BearDog", relay_node.0, requester.0);

        let params = serde_json::json!({
            "relay_node": relay_node.0,
            "requester": requester.0
        });

        match self.call_beardog("lineage.authorize_relay", params).await {
            Ok(result) => {
                let authorized =
                    result.get("authorized").and_then(serde_json::Value::as_bool).unwrap_or(false);

                let masking_level =
                    Self::parse_masking_level(result.get("masking_level").and_then(|v| v.as_str()));

                let ttl =
                    result.get("ttl_seconds").and_then(serde_json::Value::as_u64).unwrap_or(300);

                let audit_token = result
                    .get("audit_token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("beardog_auth")
                    .to_string();

                Ok(RelayAuthorization {
                    relay_node: relay_node.clone(),
                    requester: requester.clone(),
                    authorized,
                    masking_level,
                    ttl_seconds: ttl,
                    issued_at: SystemTime::now(),
                    audit_token,
                })
            }
            Err(e) => {
                // BearDog unavailable — deny by default (fail-secure)
                debug!("BearDog unavailable for relay auth, denying: {}", e);
                Ok(RelayAuthorization {
                    relay_node: relay_node.clone(),
                    requester: requester.clone(),
                    authorized: false,
                    masking_level: MaskingLevel::FullVisibility,
                    ttl_seconds: 0_u64,
                    issued_at: SystemTime::now(),
                    audit_token: "beardog_unavailable_deny".to_string(),
                })
            }
        }
    }

    async fn determine_masking(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<MaskingLevel> {
        let params = serde_json::json!({
            "relay_node": relay_node.0,
            "requester": requester.0
        });

        self.call_beardog("lineage.determine_masking", params).await.map_or_else(
            |_| {
                // BearDog unavailable — no masking (fail-secure: full visibility)
                Ok(MaskingLevel::FullVisibility)
            },
            |result| {
                let level =
                    Self::parse_masking_level(result.get("masking_level").and_then(|v| v.as_str()));
                Ok(level)
            },
        )
    }
}

// ═══════════════════════════════════════════════════════════════════
// TEST MOCKS - Gated behind cfg(test) or feature = "test-utils"
//
// Unit tests get these via #[cfg(test)]; integration tests
// enable the `test-utils` feature in dev-dependencies.
// ═══════════════════════════════════════════════════════════════════

#[cfg(any(test, feature = "test-utils"))]
/// In-memory lineage graph for tests and the `test-utils` feature (replaces `BearDog` in CI).
pub struct MockLineageProvider {
    /// Lineage graph: `node_id` → `parent_id`
    lineages: Arc<RwLock<HashMap<String, String>>>,
    /// Descendants: `ancestor_id` → list of `descendant_ids`
    descendants: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

#[cfg(any(test, feature = "test-utils"))]
impl MockLineageProvider {
    /// Create new mock lineage provider
    #[must_use]
    pub fn new() -> Self {
        Self {
            lineages: Arc::new(RwLock::new(HashMap::new())),
            descendants: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add lineage relationship (for testing)
    pub async fn add_lineage(&self, child: &str, parent: &str) {
        self.lineages.write().await.insert(child.to_string(), parent.to_string());

        // Add to descendants
        let mut descendants = self.descendants.write().await;
        descendants.entry(parent.to_string()).or_insert_with(Vec::new).push(child.to_string());
    }

    /// Check if node2 is an ancestor of node1
    pub async fn is_ancestor(&self, node: &str, potential_ancestor: &str) -> bool {
        let lineages = self.lineages.read().await;
        let mut current = node.to_string();

        // Walk up the lineage chain
        while let Some(parent) = lineages.get(&current) {
            if parent == potential_ancestor {
                return true;
            }
            current = parent.clone();
        }

        false
    }

    /// Check if node2 is a descendant of node1
    pub async fn is_descendant(&self, node: &str, potential_descendant: &str) -> bool {
        self.is_ancestor(potential_descendant, node).await
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl Default for MockLineageProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock `BirdSong` crypto (for testing and integration tests)
#[cfg(any(test, feature = "test-utils"))]
pub struct MockBirdSongCrypto {
    lineage_provider: Arc<MockLineageProvider>,
    my_id: String,
}

#[cfg(any(test, feature = "test-utils"))]
impl MockBirdSongCrypto {
    /// Create new mock crypto
    #[must_use]
    pub const fn new(lineage_provider: Arc<MockLineageProvider>, my_id: String) -> Self {
        Self {
            lineage_provider,
            my_id,
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
#[async_trait]
impl BirdSongCrypto for MockBirdSongCrypto {
    async fn encrypt_for_lineage(&self, message: &[u8], _hint: LineageHint) -> Result<Vec<u8>> {
        // Mock: prepend "LINEAGE:" to indicate lineage-encrypted
        let mut encrypted = b"LINEAGE:".to_vec();
        encrypted.extend_from_slice(message);
        Ok(encrypted)
    }

    async fn decrypt_birdsong(&self, encrypted: &[u8], sender: &NodeId) -> Result<Option<Vec<u8>>> {
        // Check if we're in sender's lineage
        let can_decrypt = self.lineage_provider.is_ancestor(&self.my_id, &sender.0).await
            || self.lineage_provider.is_descendant(&self.my_id, &sender.0).await;

        if can_decrypt && encrypted.starts_with(b"LINEAGE:") {
            Ok(Some(encrypted[8..].to_vec()))
        } else {
            Ok(None)
        }
    }
}

/// Mock relay authority (for testing and integration tests)
#[cfg(any(test, feature = "test-utils"))]
pub struct MockRelayAuthority {
    lineage_provider: Arc<MockLineageProvider>,
}

#[cfg(any(test, feature = "test-utils"))]
impl MockRelayAuthority {
    /// Create new mock relay authority
    #[must_use]
    pub const fn new(lineage_provider: Arc<MockLineageProvider>) -> Self {
        Self {
            lineage_provider,
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
#[async_trait]
impl RelayAuthority for MockRelayAuthority {
    async fn authorize_relay(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<RelayAuthorization> {
        // Check if relay_node is ancestor of requester
        let authorized = self.lineage_provider.is_ancestor(&requester.0, &relay_node.0).await;

        Ok(RelayAuthorization {
            relay_node: relay_node.clone(),
            requester: requester.clone(),
            authorized,
            masking_level: if authorized {
                MaskingLevel::Masked
            } else {
                MaskingLevel::FullVisibility
            },
            ttl_seconds: 300_u64,
            issued_at: SystemTime::now(),
            audit_token: format!("mock_token_{}", uuid::Uuid::new_v4()),
        })
    }

    async fn determine_masking(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<MaskingLevel> {
        // Simple masking: masked for descendants
        let is_ancestor = self.lineage_provider.is_ancestor(&requester.0, &relay_node.0).await;

        Ok(if is_ancestor {
            MaskingLevel::Masked
        } else {
            MaskingLevel::FullVisibility
        })
    }
}
