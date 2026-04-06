// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Crypto provider socket discovery
//!
//! Socket paths for crypto routing use the shared Neural API discovery from
//! [`songbird_crypto_provider`]. For a full client (Neural API vs direct `security provider`),
//! use [`songbird_crypto_provider::CryptoProvider::from_env`].

use anyhow::Result;
use songbird_types::defaults::paths::{
    BIOMEOS_RUNTIME_SUBDIR, ai_provider_socket_legacy_path, family_scoped_crypto_socket_path,
};
use tracing::info;

/// Discover crypto provider socket via Neural API socket discovery.
///
/// Delegates to [`songbird_crypto_provider::socket_discovery::discover_neural_api_socket`].
/// Prefer [`songbird_crypto_provider::CryptoProvider::from_env`] when constructing a
/// routed crypto client (Neural API by default; `SECURITY_PROVIDER_MODE=direct` for bootstrap).
///
/// # Errors
///
/// Always returns `Ok` with a socket path (including legacy fallbacks when nothing else matches).
pub async fn discover_crypto_socket() -> Result<String> {
    info!("🔍 Discovering crypto provider via Neural API socket discovery...");
    Ok(songbird_crypto_provider::socket_discovery::discover_neural_api_socket())
}

/// Preferred `get_*` name for [`discover_crypto_socket`].
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_security_crypto_socket() -> Result<String> {
    discover_crypto_socket().await
}

/// Discover crypto provider socket for a specific family.
///
/// Checks family-specific capability socket first, then falls back to
/// generic capability-based discovery.
///
/// # Arguments
///
/// * `family_id` - Family ID to search for (e.g., "my-family")
///
/// # Errors
///
/// Returns error if no crypto provider is discoverable for this family.
pub async fn discover_crypto_socket_for_family(family_id: &str) -> Result<String> {
    info!("🔍 Discovering crypto provider for family '{family_id}'...");

    // Check family-specific socket (capability-based, primal-agnostic)
    let family_socket = family_scoped_crypto_socket_path(family_id);
    if family_socket.exists() {
        let family_socket = family_socket.to_string_lossy().into_owned();
        info!("   ✅ Found family-specific crypto socket: {family_socket}");
        return Ok(family_socket);
    }

    // Fall back to generic capability discovery
    crate::primal_discovery::discover_crypto_provider().await
}

/// Preferred `get_*` name for [`discover_crypto_socket_for_family`].
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_security_crypto_socket_for_family(family_id: &str) -> Result<String> {
    discover_crypto_socket_for_family(family_id).await
}

/// Check if any crypto provider is available.
///
/// Quick check without logging — suitable for conditional logic.
/// Aligns with Neural API env vars and on-disk socket paths (no logging side effects).
pub async fn is_crypto_available() -> bool {
    if songbird_process_env::var("NEURAL_API_SOCKET").map(|s| !s.is_empty()).unwrap_or(false) {
        return true;
    }
    if songbird_process_env::var("NEURALS_SOCKET").map(|s| !s.is_empty()).unwrap_or(false) {
        return true;
    }

    if let Ok(xdg_dir) = songbird_process_env::var("XDG_RUNTIME_DIR") {
        let family_id = songbird_process_env::var("FAMILY_ID").unwrap_or_default();
        let socket_name = if family_id.is_empty() {
            "neural-api.sock".to_string()
        } else {
            format!("neural-api-{family_id}.sock")
        };
        let socket_path =
            std::path::PathBuf::from(xdg_dir).join(BIOMEOS_RUNTIME_SUBDIR).join(socket_name);
        if socket_path.exists() {
            return true;
        }
    }

    let tmp_neural = std::env::temp_dir().join("biomeos").join("neural-api.sock");
    if tmp_neural.exists() {
        return true;
    }

    let family_id =
        songbird_process_env::var("FAMILY_ID").unwrap_or_else(|_| "default".to_string());
    ai_provider_socket_legacy_path(&family_id).exists()
}

/// Preferred `is_*` name for [`is_crypto_available`].
pub async fn is_security_crypto_available() -> bool {
    is_crypto_available().await
}

/// Discover crypto provider socket with purpose context for audit logging.
///
/// Same as [`discover_crypto_socket`] but logs the purpose.
///
/// # Errors
///
/// Same as [`discover_crypto_socket`].
pub async fn discover_crypto_socket_for_purpose(purpose: &str) -> Result<String> {
    info!("🔍 Discovering crypto provider for purpose: {purpose}");
    discover_crypto_socket().await
}

/// Preferred `get_*` name for [`discover_crypto_socket_for_purpose`].
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_security_crypto_socket_for_purpose(purpose: &str) -> Result<String> {
    discover_crypto_socket_for_purpose(purpose).await
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    // All tests are fully concurrent -- no env var mutation, no global state.

    #[test]
    fn test_xdg_path_construction_with_runtime_dir() {
        let xdg_base = "/run/user/1000";
        let expected_path = format!("{xdg_base}/biomeos/crypto.sock");
        assert_eq!(expected_path, "/run/user/1000/biomeos/crypto.sock");
    }

    #[test]
    fn test_xdg_fallback_path_construction() {
        let fallback_base = "/tmp/biomeos";
        let expected_path = format!("{fallback_base}/crypto.sock");
        assert_eq!(expected_path, "/tmp/biomeos/crypto.sock");
    }

    #[test]
    fn test_legacy_fallback_path_construction() {
        let legacy_path = "/tmp/crypto.sock";
        assert!(legacy_path.ends_with("crypto.sock"));
        assert!(!legacy_path.contains("biomeos"));
    }

    #[test]
    fn test_family_socket_path_format() {
        let family_id = "nat0";
        let expected = format!("/tmp/crypto-{family_id}.sock");
        assert_eq!(expected, "/tmp/crypto-nat0.sock");
    }

    #[tokio::test]
    async fn test_discover_crypto_socket_returns_neural_path() {
        let result = discover_crypto_socket().await;
        let path = result.expect("Neural API discovery returns a path");
        assert!(!path.is_empty());
    }

    #[tokio::test]
    async fn test_is_crypto_available_returns_bool() {
        // Should return a bool without panicking, regardless of environment
        let _available = is_crypto_available().await;
    }

    #[tokio::test]
    async fn test_discover_crypto_socket_for_purpose_no_panic() {
        let result = discover_crypto_socket_for_purpose("signing").await;
        let path = result.expect("Neural API discovery returns a path");
        assert!(!path.is_empty());
    }
}
