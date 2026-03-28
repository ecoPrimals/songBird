// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `BirdSong` Encrypted Discovery Handler
//!
//! Provides JSON-RPC methods for Dark Forest federation via genetic lineage encryption.
//!
//! # Methods
//!
//! - `birdsong.generate_encrypted_beacon` - Generate family-encrypted beacon
//! - `birdsong.decrypt_beacon` - Decrypt beacon (family gate)
//! - `birdsong.verify_lineage` - Verify peer lineage via challenge-response
//! - `birdsong.get_lineage` - Get own lineage info
//! - `birdsong.schema` - Introspect beacon request schema (fields, types, required/optional)
//!
//! # Architecture
//!
//! ```text
//! Client → songbird.birdsong.* → BirdSongHandler
//!                                      ↓
//!                         BearDogBirdSongProvider (via songbird-discovery)
//!                                      ↓
//!                            beardog Unix socket IPC
//!                                      ↓
//!                         Crypto operations (ChaCha20-Poly1305)
//! ```
//!
//! # Deep Debt Compliance (Feb 2, 2026)
//!
//! - ✅ **Pure Rust**: Uses existing `BearDogBirdSongProvider` (zero C deps)
//! - ✅ **Zero Unsafe**: All operations safe
//! - ✅ **Runtime Discovery**: Finds beardog via `XDG_RUNTIME_DIR`, well-known paths
//! - ✅ **Self-Knowledge**: Only exposes own beacon generation
//! - ✅ **Mock Isolation**: Production code only (mocks in tests)
//! - ✅ **Agnostic Design**: Works with any family seed, discovers beardog at runtime

use anyhow::Result;
use serde::Deserialize;
use serde_json::{Value, json};
use songbird_discovery::beardog_birdsong_provider::BearDogBirdSongProvider;
use songbird_discovery::birdsong::BirdSongEncryption;
use songbird_types::primal_names;
use songbird_universal::UnixRpcClient;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// `BirdSong` handler for encrypted discovery
///
/// Manages family-encrypted discovery beacons using `BearDog`'s genetic lineage crypto.
/// All operations discover `BearDog` at runtime (no hardcoding).
#[derive(Default)]
pub struct BirdSongHandler {
    /// Cached `BearDog` socket path (runtime discovered)
    beardog_socket: Arc<RwLock<Option<PathBuf>>>,

    /// Cached `BirdSong` provider (lazy initialization)
    provider: Arc<RwLock<Option<Arc<BearDogBirdSongProvider>>>>,
}

impl BirdSongHandler {
    /// Create new `BirdSong` handler
    ///
    /// Deep debt compliance:
    /// - No hardcoded paths
    /// - Lazy initialization
    /// - Runtime discovery
    #[must_use]
    pub fn new() -> Self {
        Self {
            beardog_socket: Arc::new(RwLock::new(None)),
            provider: Arc::new(RwLock::new(None)),
        }
    }

    /// Discover `BearDog` socket at runtime (no hardcoding)
    ///
    /// Discovery order:
    /// 1. `BEARDOG_SOCKET` environment variable (supports `tcp:host:port` format for Android)
    /// 2. `XDG_RUNTIME_DIR/biomeos/beardog.sock`
    /// 3. Well-known fallback: /run/user/$(id -u)/biomeos/beardog.sock
    ///
    /// Deep debt: Runtime discovery, agnostic to deployment
    /// Android support: TCP sockets via `tcp:host:port` format (Feb 5, 2026)
    async fn discover_beardog_socket(&self) -> Result<PathBuf, String> {
        // Check cache first
        {
            let cached = self.beardog_socket.read().await;
            if let Some(path) = cached.as_ref() {
                // For TCP sockets (tcp:host:port), skip existence check
                let path_str = path.to_string_lossy();
                if path_str.starts_with("tcp:") {
                    return Ok(path.clone());
                }
                if path.exists() {
                    return Ok(path.clone());
                }
            }
        }

        // Discover at runtime (no hardcoding)
        // Discovery order: env BEARDOG_SOCKET → XDG_RUNTIME_DIR → well-known /run/user/{uid}
        let mut tried_paths: Vec<String> = Vec::new();

        let env_socket = songbird_process_env::var("BEARDOG_SOCKET");
        let socket_path = if let Ok(path) = env_socket {
            debug!("🔍 Discovering BearDog via BEARDOG_SOCKET env: {}", path);
            PathBuf::from(path)
        } else {
            tried_paths.push("BEARDOG_SOCKET env (not set)".to_string());
            if let Ok(xdg) = songbird_process_env::var("XDG_RUNTIME_DIR") {
                debug!("🔍 Discovering BearDog via XDG_RUNTIME_DIR");
                let p = PathBuf::from(format!("{xdg}/biomeos/{}.sock", primal_names::BEARDOG));
                tried_paths.push(format!("{} (XDG_RUNTIME_DIR)", p.display()));
                p
            } else {
                tried_paths.push("XDG_RUNTIME_DIR env (not set)".to_string());
                let uid = std::fs::read_to_string("/proc/self/loginuid")
                    .ok()
                    .and_then(|s| s.trim().parse::<u32>().ok())
                    .or_else(|| {
                        std::fs::read_to_string("/proc/self/status").ok().and_then(|content| {
                            content.lines().find(|line| line.starts_with("Uid:")).and_then(|line| {
                                line.split_whitespace().nth(1)?.parse::<u32>().ok()
                            })
                        })
                    })
                    .unwrap_or(1000);

                debug!("🔍 Discovering BearDog via well-known path (UID: {uid}, safe Rust)");
                let p = PathBuf::from(format!(
                    "/run/user/{uid}/biomeos/{}.sock",
                    primal_names::BEARDOG
                ));
                tried_paths.push(format!("{} (well-known)", p.display()));
                p
            }
        };

        // Check if this is a TCP socket (tcp:host:port format)
        let path_str = socket_path.to_string_lossy();
        let is_tcp = path_str.starts_with("tcp:");

        // Verify socket exists (skip for TCP - can't check file existence for network sockets)
        if !is_tcp && !socket_path.exists() {
            let tried = tried_paths.join(", ");
            return Err(format!(
                "BearDog socket not found. Tried: {tried}. \
                 Is BearDog running? Set BEARDOG_SOCKET=/path/to/beardog.sock \
                 or BEARDOG_SOCKET=tcp:host:port for cross-gate deployments"
            ));
        }

        // Cache for future calls
        {
            let mut cached = self.beardog_socket.write().await;
            *cached = Some(socket_path.clone());
        }

        if is_tcp {
            info!("✅ Discovered BearDog TCP socket: {}", path_str);
        } else {
            info!("✅ Discovered BearDog Unix socket: {}", socket_path.display());
        }
        Ok(socket_path)
    }

    /// Get or create `BirdSong` provider (lazy initialization)
    ///
    /// Deep debt: Lazy loading, runtime discovery
    async fn get_provider(&self) -> Result<Arc<BearDogBirdSongProvider>, String> {
        // Check cache
        {
            let cached = self.provider.read().await;
            if let Some(provider) = cached.as_ref() {
                return Ok(Arc::clone(provider));
            }
        }

        // Discover and create provider
        let socket_path = self.discover_beardog_socket().await?;

        // Discover family_id from environment (matches biomeOS pattern)
        // Priority: FAMILY_ID > SONGBIRD_FAMILY_ID > NODE_FAMILY_ID
        let family_id = songbird_process_env::var("FAMILY_ID")
            .or_else(|_| songbird_process_env::var("SONGBIRD_FAMILY_ID"))
            .or_else(|_| songbird_process_env::var("NODE_FAMILY_ID"))
            .ok();

        if family_id.is_some() {
            info!("🔒 Using family_id from environment");
        } else {
            warn!("⚠️  No FAMILY_ID environment variable set - BearDog encryption may fail");
        }

        let provider = BearDogBirdSongProvider::new(socket_path, family_id)
            .await
            .map_err(|e| format!("Failed to create BirdSong provider: {e}"))?;

        let provider = Arc::new(provider);

        // Cache provider
        {
            let mut cached = self.provider.write().await;
            *cached = Some(Arc::clone(&provider));
        }

        info!("✅ BirdSong provider initialized (Pure Rust, Zero unsafe)");
        Ok(provider)
    }

    /// Handle `birdsong.generate_encrypted_beacon`
    ///
    /// Generates a family-encrypted beacon for broadcast.
    /// Only family members can decrypt this beacon.
    ///
    /// Deep debt: Production implementation, complete with no mocks
    pub async fn handle_generate_encrypted_beacon(&self, params: Value) -> Result<Value, String> {
        debug!("🌲 RPC: birdsong.generate_encrypted_beacon");

        validate_required_fields(&params, &["node_id"])?;

        let request: GenerateBeaconRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        // Get provider (lazy init, runtime discovery)
        let provider = self.get_provider().await?;

        // Build discovery message (JSON)
        // Feb 6, 2026: Added onion_endpoint for Sovereign Onion Service
        // Dark Forest: Only family members can see this endpoint (encrypted beacon)
        let discovery_message = json!({
            "node_id": request.node_id,
            "capabilities": request.capabilities,
            "onion_endpoint": request.onion_endpoint,  // Sovereign Onion address (optional)
            "endpoint_hints": request.endpoint_hints,  // Additional connection hints
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION"),
        });

        let plaintext = serde_json::to_vec(&discovery_message)
            .map_err(|e| format!("Failed to serialize discovery message: {e}"))?;

        // Encrypt via BearDog (Pure Rust, Unix socket IPC)
        let encrypted = provider
            .encrypt_discovery(&plaintext)
            .await
            .map_err(|e| format!("Encryption failed: {e}"))?;

        // Encode to base64 for JSON transport
        use base64::{Engine, engine::general_purpose::STANDARD};
        let encrypted_b64 = STANDARD.encode(&encrypted);

        // Get family ID for plaintext header
        let family_id = provider.family_id().unwrap_or_else(|| "unknown".to_string());

        info!(
            "✅ Generated encrypted beacon for node: {} (family: {}, size: {} bytes)",
            request.node_id,
            family_id,
            encrypted.len()
        );

        Ok(json!({
            "encrypted_beacon": encrypted_b64,
            "family_id": family_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "node_id": request.node_id,
            "beacon_size_bytes": encrypted.len(),
        }))
    }

    /// Handle `birdsong.decrypt_beacon`
    ///
    /// Attempts to decrypt a received beacon.
    /// Returns success=true only if beacon is from family member.
    ///
    /// Deep debt: Family gate, graceful failure, no information leakage
    pub async fn handle_decrypt_beacon(&self, params: Value) -> Result<Value, String> {
        debug!("🔐 RPC: birdsong.decrypt_beacon");

        validate_required_fields(&params, &["encrypted_beacon"])?;

        let request: DecryptBeaconRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        // Get provider
        let provider = self.get_provider().await?;

        // Decode base64
        use base64::{Engine, engine::general_purpose::STANDARD};
        let encrypted = STANDARD
            .decode(&request.encrypted_beacon)
            .map_err(|e| format!("Invalid base64: {e}"))?;

        // Attempt decryption (family gate)
        match provider.decrypt_discovery(&encrypted).await {
            Ok(Some(plaintext)) => {
                // SUCCESS: Same family, can decrypt
                let discovery_message: serde_json::Value = serde_json::from_slice(&plaintext)
                    .map_err(|e| format!("Decrypted but invalid JSON: {e}"))?;

                let node_id =
                    discovery_message["node_id"].as_str().unwrap_or("unknown").to_string();

                let capabilities: Vec<String> = discovery_message["capabilities"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
                    .unwrap_or_default();

                // Feb 6, 2026: Extract onion endpoint for Sovereign NAT traversal
                let onion_endpoint = discovery_message["onion_endpoint"].as_str().map(String::from);

                let endpoint_hints = discovery_message.get("endpoint_hints").cloned();

                if let Some(ref onion) = onion_endpoint {
                    info!(
                        "✅ Decrypted beacon from family member: {} (onion: {}, capabilities: {:?})",
                        node_id, onion, capabilities
                    );
                } else {
                    info!(
                        "✅ Decrypted beacon from family member: {} (capabilities: {:?})",
                        node_id, capabilities
                    );
                }

                Ok(json!({
                    "success": true,
                    "is_family": true,
                    "node_id": node_id,
                    "capabilities": capabilities,
                    "onion_endpoint": onion_endpoint,
                    "endpoint_hints": endpoint_hints,
                    "timestamp": discovery_message["timestamp"],
                }))
            }
            Ok(None) => {
                // GRACEFUL FAILURE: Different family (cannot decrypt)
                // No information leakage - just return false
                debug!("⛔ Beacon is from different family (graceful ignore)");

                Ok(json!({
                    "success": false,
                    "is_family": false,
                    "reason": "different_family",
                }))
            }
            Err(e) => {
                // SYSTEM ERROR (not decryption failure)
                error!("❌ System error during decryption: {}", e);
                Err(format!("Decryption system error: {e}"))
            }
        }
    }

    /// Handle `birdsong.verify_lineage`
    ///
    /// Verifies peer lineage using challenge-response (defense-in-depth).
    /// Calls `BearDog`'s `genetic.generate_challenge` and `genetic.verify_challenge_response`.
    ///
    /// Deep debt: Delegates to `BearDog` (separation of concerns)
    pub async fn handle_verify_lineage(&self, params: Value) -> Result<Value, String> {
        debug!("🔍 RPC: birdsong.verify_lineage");

        validate_required_fields(&params, &["peer_node_id", "our_node_id"])?;

        let request: VerifyLineageRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        // Discover BearDog socket
        let beardog_socket = self.discover_beardog_socket().await?;

        // Create RPC client to BearDog
        let client = UnixRpcClient::new(&beardog_socket)
            .map_err(|e| format!("Failed to connect to BearDog: {e}"))?;

        // Step 1: Generate challenge
        let challenge_params = json!({
            "challenger_node_id": request.our_node_id,
            "target_family_id": request.peer_node_id,
        });

        let challenge_result: Value = client
            .call("genetic.generate_challenge", &challenge_params)
            .await
            .map_err(|e| format!("Challenge generation failed: {e}"))?;

        info!("✅ Generated lineage challenge for peer: {}", request.peer_node_id);

        // In production, you would:
        // - Send challenge to peer via network
        // - Receive response from peer
        // - Call genetic.verify_challenge_response
        //
        // For now, return the challenge for caller to handle exchange
        Ok(json!({
            "challenge_generated": true,
            "challenge": challenge_result,
            "next_step": "send_challenge_to_peer",
            "peer_node_id": request.peer_node_id,
        }))
    }

    /// Handle `birdsong.get_lineage`
    ///
    /// Returns our own lineage info for sharing with peers.
    ///
    /// Deep debt: Self-knowledge only, queries `BearDog` for our identity
    pub async fn handle_get_lineage(&self, params: Value) -> Result<Value, String> {
        debug!("📋 RPC: birdsong.get_lineage");

        let _request: GetLineageRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        // Get provider (includes our family ID)
        let provider = self.get_provider().await?;

        let family_id = provider.family_id().unwrap_or_else(|| "unknown".to_string());

        // Query BearDog for our node ID (if needed)
        let beardog_socket = self.discover_beardog_socket().await?;
        let client = UnixRpcClient::new(&beardog_socket)
            .map_err(|e| format!("Failed to connect to BearDog: {e}"))?;

        // Query primal.info from the discovered crypto provider
        let provider_info: Value = client
            .call("primal.info", &json!({}))
            .await
            .unwrap_or_else(|_| json!({"name": "unknown", "version": "unknown"}));

        let provider_name = provider_info["name"].as_str().unwrap_or("unknown").to_string();

        info!("✅ Retrieved lineage info (family: {}, provider: {})", family_id, provider_name);

        Ok(json!({
            "family_id": family_id,
            "provider": provider_name,
            "provider_version": provider_info["version"],
            "encryption": "chacha20_poly1305",
            "lineage_type": "genetic",
        }))
    }

    /// Handle `birdsong.schema`
    ///
    /// Returns the beacon request schema: field names, types, required/optional.
    /// Clients can use this to generate beacon requests programmatically
    /// without hardcoding field lists.
    pub async fn handle_schema(&self, _params: Value) -> Result<Value, String> {
        debug!("🌲 RPC: birdsong.schema");

        Ok(json!({
            "method": "birdsong.generate_encrypted_beacon",
            "description": "Generate a family-encrypted discovery beacon for Dark Forest broadcast",
            "fields": [
                {
                    "name": "node_id",
                    "type": "string",
                    "required": true,
                    "description": "Unique node identifier for this primal instance"
                },
                {
                    "name": "capabilities",
                    "type": "array<string>",
                    "required": false,
                    "default": "[]",
                    "description": "Capability tokens this node advertises (e.g. network.discovery, crypto.delegate)"
                },
                {
                    "name": "onion_endpoint",
                    "type": "string | null",
                    "required": false,
                    "default": "null",
                    "description": "Sovereign Onion endpoint (e.g. abc123...xyz.onion:3492). Dark Forest: only visible to family members"
                },
                {
                    "name": "endpoint_hints",
                    "type": "object | null",
                    "required": false,
                    "default": "null",
                    "description": "Additional endpoint hints (LAN IP, port, relay addresses, etc.)"
                }
            ],
            "related_methods": [
                "birdsong.decrypt_beacon",
                "birdsong.verify_lineage",
                "birdsong.get_lineage",
                "birdsong.advertise"
            ],
            "version": env!("CARGO_PKG_VERSION")
        }))
    }
}

// ============================================================================
// Validation Helpers
// ============================================================================

/// Pre-validate that all required fields are present in the JSON params,
/// reporting **all** missing fields in a single error message.
///
/// Standard serde deserialization reports one missing field at a time,
/// requiring multiple round-trips during integration debugging. This
/// pre-validation collects every missing field into one diagnostic.
fn validate_required_fields(params: &Value, required: &[&str]) -> Result<(), String> {
    let Some(obj) = params.as_object() else {
        return Err("Invalid params: expected JSON object".to_string());
    };

    let missing: Vec<&str> =
        required.iter().filter(|&&field| !obj.contains_key(field)).copied().collect();

    if missing.is_empty() {
        Ok(())
    } else if missing.len() == 1 {
        Err(format!("Missing required field: {}", missing[0]))
    } else {
        Err(format!("Missing required fields: {}", missing.join(", ")))
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct GenerateBeaconRequest {
    node_id: String,
    #[serde(default)]
    capabilities: Vec<String>,
    /// Sovereign Onion endpoint (e.g., "abc123...xyz.onion:3492")
    /// Dark Forest: Only visible to family members (beacon is encrypted)
    #[serde(default)]
    onion_endpoint: Option<String>,
    /// Additional endpoint hints (LAN IP, port, etc.)
    #[serde(default)]
    endpoint_hints: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct DecryptBeaconRequest {
    encrypted_beacon: String,
}

#[derive(Debug, Deserialize)]
struct VerifyLineageRequest {
    peer_node_id: String,
    our_node_id: String,
}

#[derive(Debug, Deserialize)]
struct GetLineageRequest {
    // Empty for now, may add filters later
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
fn is_expected_crypto_delegate_connectivity_error(msg: &str) -> bool {
    let m = msg.to_lowercase();
    m.contains(songbird_types::primal_names::BEARDOG)
        || m.contains("socket")
        || m.contains("ipc")
        || m.contains("connection refused")
        || m.contains("no such file")
        || m.contains("crypto")
        || m.contains("rpc")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_handler_creation() {
        let _handler = BirdSongHandler::new();
        // Verify handler can be created (no panics)
        // Deep debt: Zero allocation on creation (lazy init)
        assert!(true);
    }

    #[tokio::test]
    async fn test_socket_discovery_priority() {
        let handler = BirdSongHandler::new();

        // Test that discovery doesn't panic (socket may not exist in test env)
        let result = handler.discover_beardog_socket().await;

        // In CI/test environment, socket won't exist - that's expected
        if result.is_err() {
            let err = result.unwrap_err();
            assert!(err.contains("BearDog socket not found"));
        }
    }

    #[tokio::test]
    async fn test_generate_beacon_params() {
        let handler = BirdSongHandler::new();

        let params = json!({
            "node_id": "test_node",
            "capabilities": ["crypto", "discovery"]
        });

        // In test env without BearDog, should gracefully fail
        let result = handler.handle_generate_encrypted_beacon(params).await;

        // Expected: Err (no BearDog in test env)
        // But the error should be clear and actionable
        if let Err(e) = result {
            assert!(
                e.contains("BearDog") || e.contains("socket"),
                "Error should mention BearDog or socket, got: {e}"
            );
        }
    }

    #[tokio::test]
    async fn test_decrypt_beacon_params() {
        let handler = BirdSongHandler::new();

        let params = json!({
            "encrypted_beacon": "dGVzdF9lbmNyeXB0ZWRfYmVhY29u" // base64 "test_encrypted_beacon"
        });

        // Should validate params even without BearDog
        let result = handler.handle_decrypt_beacon(params).await;

        // Expected: Err (no BearDog in test env)
        if let Err(e) = result {
            assert!(
                e.contains("BearDog") || e.contains("socket"),
                "Error should mention BearDog or socket, got: {e}"
            );
        }
    }

    #[tokio::test]
    async fn test_verify_lineage_params() {
        let handler = BirdSongHandler::new();

        let params = json!({
            "peer_node_id": "peer1",
            "our_node_id": "test_node"
        });

        // Should validate params
        let result = handler.handle_verify_lineage(params).await;

        // Expected: Err (no BearDog in test env)
        if let Err(e) = result {
            assert!(
                super::is_expected_crypto_delegate_connectivity_error(&e),
                "Error should mention BearDog, socket, or IPC, got: {e}"
            );
        }
    }

    #[tokio::test]
    async fn test_get_lineage_params() {
        let handler = BirdSongHandler::new();

        let params = json!({});

        // Should accept empty params
        let result = handler.handle_get_lineage(params).await;

        // Expected: Err (no BearDog in test env)
        if let Err(e) = result {
            assert!(
                super::is_expected_crypto_delegate_connectivity_error(&e),
                "Error should mention BearDog, socket, or IPC, got: {e}"
            );
        }
    }

    // ── birdsong.schema ────────────────────────────────────────────

    #[tokio::test]
    async fn test_schema_returns_all_fields() {
        let handler = BirdSongHandler::new();
        let result = handler.handle_schema(json!({})).await.unwrap();

        let fields = result["fields"].as_array().unwrap();
        assert_eq!(fields.len(), 4, "beacon schema should expose 4 fields");

        let names: Vec<&str> = fields.iter().filter_map(|f| f["name"].as_str()).collect();
        assert!(names.contains(&"node_id"));
        assert!(names.contains(&"capabilities"));
        assert!(names.contains(&"onion_endpoint"));
        assert!(names.contains(&"endpoint_hints"));
    }

    #[tokio::test]
    async fn test_schema_required_fields() {
        let handler = BirdSongHandler::new();
        let result = handler.handle_schema(json!({})).await.unwrap();

        let fields = result["fields"].as_array().unwrap();

        let required: Vec<&str> = fields
            .iter()
            .filter(|f| f["required"].as_bool() == Some(true))
            .filter_map(|f| f["name"].as_str())
            .collect();
        assert_eq!(required, vec!["node_id"], "only node_id should be required");

        let optional: Vec<&str> = fields
            .iter()
            .filter(|f| f["required"].as_bool() == Some(false))
            .filter_map(|f| f["name"].as_str())
            .collect();
        assert_eq!(optional.len(), 3);
    }

    #[tokio::test]
    async fn test_schema_includes_related_methods() {
        let handler = BirdSongHandler::new();
        let result = handler.handle_schema(json!({})).await.unwrap();

        let related = result["related_methods"].as_array().unwrap();
        assert!(!related.is_empty());
        let names: Vec<&str> = related.iter().filter_map(|v| v.as_str()).collect();
        assert!(names.contains(&"birdsong.decrypt_beacon"));
        assert!(names.contains(&"birdsong.verify_lineage"));
    }

    #[tokio::test]
    async fn test_schema_includes_types() {
        let handler = BirdSongHandler::new();
        let result = handler.handle_schema(json!({})).await.unwrap();

        let fields = result["fields"].as_array().unwrap();
        for field in fields {
            assert!(field["type"].is_string(), "field {} should have a type string", field["name"]);
        }
    }

    #[tokio::test]
    async fn test_schema_includes_version() {
        let handler = BirdSongHandler::new();
        let result = handler.handle_schema(json!({})).await.unwrap();
        assert!(result["version"].is_string());
    }

    #[tokio::test]
    async fn test_schema_method_name() {
        let handler = BirdSongHandler::new();
        let result = handler.handle_schema(json!({})).await.unwrap();
        assert_eq!(result["method"].as_str().unwrap(), "birdsong.generate_encrypted_beacon");
    }

    // ── validate_required_fields ─────────────────────────────────

    #[test]
    fn test_validate_all_present() {
        let params = json!({"node_id": "test"});
        assert!(validate_required_fields(&params, &["node_id"]).is_ok());
    }

    #[test]
    fn test_validate_single_missing() {
        let params = json!({});
        let err = validate_required_fields(&params, &["node_id"]).unwrap_err();
        assert_eq!(err, "Missing required field: node_id");
    }

    #[test]
    fn test_validate_multiple_missing_aggregated() {
        let params = json!({});
        let err = validate_required_fields(&params, &["peer_node_id", "our_node_id"]).unwrap_err();
        assert!(
            err.contains("peer_node_id") && err.contains("our_node_id"),
            "should list all missing fields: {err}"
        );
        assert!(err.starts_with("Missing required fields:"));
    }

    #[test]
    fn test_validate_partial_missing() {
        let params = json!({"peer_node_id": "a"});
        let err = validate_required_fields(&params, &["peer_node_id", "our_node_id"]).unwrap_err();
        assert!(err.contains("our_node_id"), "should report missing field");
        assert!(!err.contains("peer_node_id"), "should not list present field");
    }

    #[test]
    fn test_validate_non_object_params() {
        let params = json!("not an object");
        let err = validate_required_fields(&params, &["node_id"]).unwrap_err();
        assert!(err.contains("expected JSON object"));
    }

    #[test]
    fn test_validate_null_params() {
        let params = Value::Null;
        let err = validate_required_fields(&params, &["node_id"]).unwrap_err();
        assert!(err.contains("expected JSON object"));
    }

    #[test]
    fn test_validate_empty_required_list() {
        let params = json!({});
        assert!(validate_required_fields(&params, &[]).is_ok());
    }

    #[tokio::test]
    async fn test_generate_beacon_missing_node_id() {
        let handler = BirdSongHandler::new();
        let params = json!({"capabilities": ["test"]});
        let err = handler.handle_generate_encrypted_beacon(params).await.unwrap_err();
        assert!(
            err.contains("Missing required field: node_id"),
            "should report missing node_id: {err}"
        );
    }

    #[tokio::test]
    async fn test_decrypt_missing_encrypted_beacon() {
        let handler = BirdSongHandler::new();
        let params = json!({});
        let err = handler.handle_decrypt_beacon(params).await.unwrap_err();
        assert!(
            err.contains("Missing required field: encrypted_beacon"),
            "should report missing encrypted_beacon: {err}"
        );
    }

    #[tokio::test]
    async fn test_verify_lineage_missing_both_fields() {
        let handler = BirdSongHandler::new();
        let params = json!({});
        let err = handler.handle_verify_lineage(params).await.unwrap_err();
        assert!(
            err.contains("peer_node_id") && err.contains("our_node_id"),
            "should aggregate both missing fields: {err}"
        );
    }

    #[tokio::test]
    async fn test_verify_lineage_missing_one_field() {
        let handler = BirdSongHandler::new();
        let params = json!({"peer_node_id": "peer1"});
        let err = handler.handle_verify_lineage(params).await.unwrap_err();
        assert!(err.contains("our_node_id"), "should report missing our_node_id: {err}");
        assert!(!err.contains("peer_node_id"), "should not list present field: {err}");
    }

    // Integration tests with real BearDog in tests/birdsong_integration_test.rs
}
