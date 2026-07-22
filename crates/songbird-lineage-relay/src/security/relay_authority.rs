// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::Result;
use crate::types::{MaskingLevel, NodeId, RelayAuthorization};
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info};

use songbird_types::IpcStream;

// ═══════════════════════════════════════════════════════════════════
// PRODUCTION: Security-provider relay authority
// Delegates lineage verification via Unix socket JSON-RPC.
// Replaces MockRelayAuthority in all production constructors.
// ═══════════════════════════════════════════════════════════════════

/// Production relay authority backed by the security provider
///
/// Delegates lineage-based relay authorization via Unix socket
/// JSON-RPC. No hardcoded lineage graphs — the security provider owns policy.
///
/// ## Deep Debt Compliance
///
/// - ✅ Real implementation (not a mock)
/// - ✅ Runtime discovery (socket path via env or discovery)
/// - ✅ Zero unsafe code
/// - ✅ Async/await
#[derive(Clone, Debug)]
pub struct SecurityRelayAuthority {
    socket_path: PathBuf,
}

impl SecurityRelayAuthority {
    /// Create a new relay authority using the discovered security-provider socket
    ///
    /// Discovers security provider socket path at runtime:
    /// 1. Capability socket env vars (see `Self::discover_socket_path`)
    /// 2. XDG runtime dir capability-named sockets under `biomeos/`
    /// 3. Legacy fallbacks under `/tmp/biomeos/` or `/tmp/`
    pub fn new() -> Self {
        let socket_path = Self::discover_socket_path();
        info!("Security-provider relay authority created (socket: {:?})", socket_path);
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
    /// ## Resolution Order (capability-first, identity-agnostic)
    ///
    /// 1. `SECURITY_PROVIDER_SOCKET` - Capability-based (preferred)
    /// 2. `CRYPTO_PROVIDER_SOCKET` - Capability-based alternative
    /// 3. Legacy security socket env — backward compatibility
    /// 4. XDG: `$XDG_RUNTIME_DIR/biomeos/security.sock` - Capability-named
    /// 5. XDG: `$XDG_RUNTIME_DIR/biomeos/` legacy filename — optional hint on some installs
    /// 6. Legacy: `/tmp/biomeos/security.sock` - Fallback
    fn discover_socket_path() -> PathBuf {
        // 1. Capability-based env vars (preferred — identity-agnostic)
        for env_var in &["SECURITY_PROVIDER_SOCKET", "CRYPTO_PROVIDER_SOCKET", "SECURITY_SOCKET"] {
            if let Ok(path) = songbird_process_env::var(env_var) {
                return PathBuf::from(path);
            }
        }
        // Legacy fallback
        if let Ok(path) = songbird_process_env::var("BEARDOG_SOCKET") {
            tracing::warn!("BEARDOG_SOCKET is deprecated — migrate to SECURITY_PROVIDER_SOCKET");
            return PathBuf::from(path);
        }

        // 2. XDG runtime directory (capability-named sockets first)
        if let Ok(xdg) = songbird_process_env::var("XDG_RUNTIME_DIR") {
            let biomeos =
                PathBuf::from(&xdg).join(songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR);

            // Capability-named sockets only
            for socket_name in &["security.sock", "crypto.sock"] {
                let path = biomeos.join(socket_name);
                if path.exists() {
                    return path;
                }
            }
        }

        // 3. Legacy fallback (capability name preferred)
        #[allow(deprecated, reason = "intentional backward-compat fallback path")]
        use songbird_types::defaults::paths::{
            LEGACY_SECURITY_SOCKET_FILENAME, biomeos_socket_dir_tmp, security_socket_default_path,
            tmp_flat_security_sock_path,
        };

        let b = biomeos_socket_dir_tmp();
        #[allow(deprecated, reason = "intentional backward-compat fallback path")]
        let legacy_name = LEGACY_SECURITY_SOCKET_FILENAME;
        let fallback_paths =
            [security_socket_default_path(), b.join(legacy_name), tmp_flat_security_sock_path()];

        for path in &fallback_paths {
            if path.exists() {
                return path.clone();
            }
        }

        // Final fallback (most common provider)
        security_socket_default_path()
    }

    async fn connect_ipc_relay(path: &std::path::Path) -> Result<IpcStream> {
        let path_str = path.to_string_lossy();
        IpcStream::connect(&path_str).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to connect to security provider at {}: {e}",
                path.display(),
            ))
        })
    }

    /// Call security-provider JSON-RPC method via IPC.
    async fn call_security_rpc(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let mut stream = Self::connect_ipc_relay(&self.socket_path).await?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let request_bytes = serde_json::to_vec(&request)?;
        stream.write_all(&request_bytes).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to write to security provider: {e}"
            ))
        })?;
        stream.write_all(b"\n").await.ok();

        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to read from security provider: {e}"
            ))
        })?;

        let response: serde_json::Value = serde_json::from_slice(&response_bytes)?;

        if let Some(error) = response.get("error") {
            return Err(crate::error::LineageRelayError::BirdSongError(format!(
                "Security provider RPC error: {error}"
            )));
        }

        response.get("result").cloned().ok_or_else(|| {
            crate::error::LineageRelayError::BirdSongError(String::from(
                "No result in security provider response",
            ))
        })
    }

    /// Parse masking level from security-provider response string
    #[cfg(test)]
    pub(crate) fn parse_masking_level_for_test(level: Option<&str>) -> MaskingLevel {
        Self::parse_masking_level(level)
    }

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

impl Default for SecurityRelayAuthority {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityRelayAuthority {
    /// Authorize relay service for requester (security-provider policy).
    pub async fn authorize_relay(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<RelayAuthorization> {
        debug!("Authorizing relay: {} -> {} via security provider", relay_node.0, requester.0);

        let params = serde_json::json!({
            "relay_node": relay_node.0,
            "requester": requester.0
        });

        match self.call_security_rpc("lineage.authorize_relay", params).await {
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
                    .unwrap_or("security_provider_auth")
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
                // Security provider unavailable — deny by default (fail-secure)
                debug!("Security provider unavailable for relay auth, denying: {}", e);
                Ok(RelayAuthorization {
                    relay_node: relay_node.clone(),
                    requester: requester.clone(),
                    authorized: false,
                    masking_level: MaskingLevel::FullVisibility,
                    ttl_seconds: 0_u64,
                    issued_at: SystemTime::now(),
                    audit_token: String::from("security_provider_unavailable_deny"),
                })
            }
        }
    }

    /// Resolve masking tier for the given relay relationship (security-provider policy).
    pub async fn determine_masking(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<MaskingLevel> {
        let params = serde_json::json!({
            "relay_node": relay_node.0,
            "requester": requester.0
        });

        self.call_security_rpc("lineage.determine_masking", params).await.map_or_else(
            |_| {
                // Security provider unavailable — no masking (fail-secure: full visibility)
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
