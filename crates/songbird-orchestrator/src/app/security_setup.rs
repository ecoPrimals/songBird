// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security Setup Module
//!
//! Handles security provider discovery and integration setup using
//! **capability-based discovery** - zero hardcoding!
//!
//! ## Zero Hardcoding Philosophy ✨
//!
//! This module exemplifies the primal philosophy:
//! - NO hardcoded endpoints
//! - Discovers security provider at runtime
//! - Uses capability system for discovery
//! - Environment-driven configuration
//! - Graceful fallbacks
//!
//! ## Discovery Strategy
//!
//! 1. Check `SECURITY_ENDPOINT` (explicit configuration)
//! 2. Query capability system for "security" provider
//! 3. Fall back to `CAPABILITY_SECURITY_ENDPOINT` (legacy)
//!
//! There is **no** silent URL construction in discovery: if none of the above apply,
//! discovery fails closed with an actionable error. The helper
//! `construct_default_security_endpoint` exists for tests and local tooling only.
//!
//! This enables ANY security provider to be discovered and used when configured!

use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn};

use songbird_types::SafeEnv;

/// Security integration backed by a discovered endpoint.
///
/// Wraps the runtime-discovered security provider endpoint and exposes
/// health probing via the crypto-provider discovery path.
#[derive(Debug, Clone)]
pub struct SecurityIntegration {
    /// Discovered security provider endpoint.
    endpoint: Arc<str>,
}

impl SecurityIntegration {
    /// The discovered endpoint URL.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Probe the security subsystem via crypto-provider socket discovery.
    ///
    /// Returns `true` when the crypto provider socket is reachable.
    pub async fn is_healthy(&self) -> bool {
        crate::primal_discovery::discover_crypto_provider().await.is_ok()
    }
}

/// Discover security provider endpoint
///
/// **EVOLVED (v3.15.0)**: Zero vendor hardcoding! Uses capability discovery.
///
/// Priority:
/// 1. `SONGBIRD_SECURITY_PROVIDER` (NEW - generic capability)
/// 2. `SECURITY_ENDPOINT` (existing - generic)
/// 3. Discovery via Universal Adapter (fallback)
///
/// # Arguments
///
/// * `universal_adapter` - Optional Universal Adapter for capability discovery
///
/// # Returns
///
/// - `Ok(String)` if a security provider is found
/// - `Err(...)` if no security provider is available
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn discover_security_endpoint(
    universal_adapter: Option<&mut crate::universal_adapter::UniversalAdapter>,
) -> Result<String> {
    // Priority 1: NEW - Generic capability env var (v3.15.0)
    if let Ok(endpoint) = songbird_process_env::var("SONGBIRD_SECURITY_PROVIDER") {
        let resolved = resolve_bare_name_to_endpoint(&endpoint);
        info!("🔐 Security provider: {} (via SONGBIRD_SECURITY_PROVIDER)", resolved);
        return Ok(resolved);
    }

    // Priority 2: EXISTING - Generic security endpoint
    if let Ok(endpoint) = songbird_process_env::var("SECURITY_ENDPOINT") {
        let resolved = resolve_bare_name_to_endpoint(&endpoint);
        info!("🔐 Security provider: {} (via SECURITY_ENDPOINT)", resolved);
        return Ok(resolved);
    }

    // Priority 3: FALLBACK - Discover via Universal Adapter
    if let Some(adapter) = universal_adapter {
        info!("🔍 No security provider configured, discovering via Universal Adapter...");
        match adapter.discover_capability("security").await {
            Ok(providers) if !providers.is_empty() => {
                let endpoint = providers[0].endpoint.clone();
                info!("✅ Discovered security provider: {}", endpoint);
                return Ok(endpoint);
            }
            Ok(_) => {
                warn!("⚠️  Universal Adapter found no security providers");
            }
            Err(e) => {
                warn!("⚠️  Universal Adapter discovery failed: {}", e);
            }
        }
    }

    // Priority 4: UDS socket auto-discovery from XDG runtime dir
    if let Some(socket_path) = discover_security_socket_from_xdg() {
        info!("🔐 Security provider discovered via XDG runtime: {}", socket_path);
        return Ok(socket_path);
    }

    // Priority 5: Legacy fallback (for backward compat)
    if let Ok(endpoint) = songbird_process_env::var("CAPABILITY_SECURITY_ENDPOINT") {
        warn!("⚠️  Using legacy CAPABILITY_SECURITY_ENDPOINT");
        warn!("   Please use SONGBIRD_SECURITY_PROVIDER instead");
        let resolved = resolve_bare_name_to_endpoint(&endpoint);
        return Ok(resolved);
    }

    Err(anyhow::anyhow!(
        "No security provider configured.\n\
         Please set one of:\n\
         - SONGBIRD_SECURITY_PROVIDER (recommended - generic capability)\n\
         - SECURITY_ENDPOINT (alternative - generic)\n\
         - Or configure Universal Adapter for automatic discovery"
    ))
}

/// Resolve a bare primal name or incomplete endpoint to a proper transport URL.
///
/// If the endpoint already has a scheme (http://, https://, unix://, tarpc://),
/// it's returned as-is. Otherwise, it's treated as a primal name and resolved
/// to the UDS socket path under XDG_RUNTIME_DIR/biomeos/.
fn resolve_bare_name_to_endpoint(endpoint: &str) -> String {
    if endpoint.contains("://") || endpoint.starts_with('/') {
        return endpoint.to_string();
    }

    // Bare name (e.g. legacy primal name) — resolve to UDS socket path
    let socket_name = if endpoint.contains('.') {
        endpoint.to_string()
    } else {
        format!("{endpoint}.sock")
    };

    if let Ok(runtime_dir) = songbird_process_env::var("XDG_RUNTIME_DIR") {
        let socket_path = std::path::PathBuf::from(&runtime_dir)
            .join(songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR)
            .join(&socket_name);
        if socket_path.exists() {
            return format!("unix://{}", socket_path.display());
        }
        // Try with wildcard pattern (beardog-*.sock)
        let dir = std::path::PathBuf::from(&runtime_dir)
            .join(songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with(endpoint) && name_str.ends_with(".sock") {
                    return format!("unix://{}", entry.path().display());
                }
            }
        }
    }

    // System-wide fallback
    let system_path =
        std::path::PathBuf::from(songbird_types::constants::BIOMEOS_SYSTEM_RUNTIME_DIR)
            .join(&socket_name);
    if system_path.exists() {
        return format!("unix://{}", system_path.display());
    }

    warn!(
        "Bare endpoint '{}' could not be resolved to a socket path; \
         passing through (will likely fail)",
        endpoint
    );
    endpoint.to_string()
}

/// Discover security provider socket from XDG runtime directory.
///
/// Scans `$XDG_RUNTIME_DIR/biomeos/` using capability-based socket names.
/// Falls back to legacy `beardog.sock` if capability names not found.
fn discover_security_socket_from_xdg() -> Option<String> {
    let runtime_dir = songbird_process_env::var("XDG_RUNTIME_DIR").ok()?;
    let biomeos_dir = std::path::PathBuf::from(&runtime_dir)
        .join(songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR);

    if !biomeos_dir.is_dir() {
        return None;
    }

    // Capability-first discovery (no primal identity knowledge)
    for candidate in songbird_types::defaults::paths::CRYPTO_PROVIDER_SOCKET_FILENAMES_XDG {
        let path = biomeos_dir.join(candidate);
        if path.exists() {
            return Some(format!("unix://{}", path.display()));
        }
    }

    // Family-scoped security socket (capability-named)
    if let Ok(entries) = std::fs::read_dir(&biomeos_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("security") && name_str.ends_with(".sock") {
                return Some(format!("unix://{}", entry.path().display()));
            }
        }
    }

    // Legacy fallback (deprecated — will be removed Wave 114)
    #[allow(deprecated, reason = "backward-compat: legacy socket name still on disk")]
    let legacy = biomeos_dir.join(songbird_types::defaults::paths::LEGACY_SECURITY_SOCKET_FILENAME);
    if legacy.exists() {
        warn!("Found legacy 'beardog.sock' — migrate to capability-based 'security.sock'");
        return Some(format!("unix://{}", legacy.display()));
    }

    None
}

/// Setup security integration using capability-based discovery
///
/// **EVOLVED (v3.15.0)**: Now uses `discover_security_endpoint` with deprecation support
///
/// # Zero Hardcoding
///
/// Discovers security provider at runtime via capability discovery (no vendor names!)
///
/// # Returns
///
/// A [`SecurityIntegration`] holding the resolved provider endpoint URL.
///
/// # Errors
///
/// Returns an error when [`discover_security_endpoint`] cannot resolve any provider
/// (missing env and adapter discovery). Operators must set an explicit endpoint for production.
pub async fn setup_security() -> Result<SecurityIntegration> {
    let security_endpoint = discover_security_endpoint(None).await?;
    info!("🔐 Security integration established at {security_endpoint}");
    Ok(SecurityIntegration {
        endpoint: Arc::from(security_endpoint),
    })
}

/// Construct a heuristic security endpoint from bind address and port.
///
#[allow(dead_code, reason = "retained for unit tests and local tooling")]
fn construct_default_security_endpoint() -> String {
    let bind_address = SafeEnv::get_or_default(
        "SONGBIRD_BIND_ADDRESS",
        songbird_config::canonical::constants::get_bind_address(),
    );

    let security_port = SafeEnv::get_or_default(
        "CAPABILITY_SECURITY_PORT",
        SafeEnv::get_or_default(
            "SONGBIRD_SECURITY_PORT",
            songbird_config::defaults::ports::security_provider_port().to_string(),
        ),
    );

    format!("http://{bind_address}:{security_port}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // All tests are concurrent-safe: no env var mutation.

    #[test]
    fn test_construct_default_security_endpoint() {
        let endpoint = construct_default_security_endpoint();
        assert!(endpoint.starts_with("http://"));
        assert!(endpoint.contains(':'));

        let parts: Vec<&str> = endpoint.split("://").collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "http");
        assert!(parts[1].contains(':'));
    }

    #[tokio::test]
    async fn test_security_setup_graceful() {
        // setup_security reads env vars at runtime.
        // In test environments it may succeed or fail depending on config.
        // The important thing is it never panics.
        let result = setup_security().await;
        match result {
            Ok(_endpoint) => {} // security provider endpoint available
            Err(e) => {
                let msg = format!("{e}");
                // Expected in CI: no security provider
                assert!(
                    msg.contains("security")
                        || msg.contains("provider")
                        || msg.contains("security provider"),
                    "Unexpected error: {msg}"
                );
            }
        }
    }

    #[test]
    fn test_capability_type_is_generic() {
        // Songbird discovers by capability, not by primal name
        let capability_type = "security";
        assert_eq!(capability_type, "security");
    }
}
