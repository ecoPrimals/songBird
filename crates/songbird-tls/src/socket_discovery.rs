//! XDG-compliant socket discovery for TLS layer
//!
//! This module provides functions to discover Unix socket paths for BearDog
//! and Neural API following the XDG Base Directory Specification.
//!
//! ## Discovery Order
//! 1. Explicitly provided path (e.g., from CLI arguments)
//! 2. Environment variable (e.g., `BEARDOG_SOCKET`, `NEURAL_API_SOCKET`)
//! 3. XDG_RUNTIME_DIR (e.g., `/run/user/1000/biomeos/beardog-nat0.sock`)
//! 4. Fallback to `/tmp` (e.g., `/tmp/beardog-nat0.sock`)
//!
//! ## Zero Hardcoding
//! - No hardcoded paths, only fallback defaults.
//! - Uses `FAMILY_ID` for multi-instance support.
//!
//! ## Compatibility
//! This is a duplicate of the socket_discovery module from songbird-http-client,
//! kept separate to avoid circular dependencies between crates.

use std::path::PathBuf;
use tracing::{debug, trace, warn};

/// Discover an XDG-compliant socket path for a given primal.
///
/// Constructs a path like `/run/user/<UID>/biomeos/<primal>-<family_id>.sock`
/// if `XDG_RUNTIME_DIR` and `FAMILY_ID` are set.
fn discover_xdg_socket(primal_name: &str, family_id: &str) -> Option<String> {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(runtime_dir)
            .join("biomeos")
            .join(format!("{}-{}.sock", primal_name, family_id));
        if path.exists() {
            let path_str = path.to_string_lossy().into_owned();
            debug!("Found XDG socket for {}: {}", primal_name, path_str);
            return Some(path_str);
        } else {
            trace!("XDG socket path does not exist: {}", path.display());
        }
    } else {
        trace!("XDG_RUNTIME_DIR not set for {}", primal_name);
    }
    None
}

/// Discover the BearDog socket path.
///
/// Prioritizes:
/// 1. `explicit_path` (from CLI)
/// 2. `BEARDOG_SOCKET` or `BEARDOG_CRYPTO_SOCKET` env vars
/// 3. XDG_RUNTIME_DIR + `FAMILY_ID`
/// 4. `/tmp/beardog-nat0.sock` (fallback)
pub fn discover_beardog_socket(explicit_path: Option<&PathBuf>) -> String {
    if let Some(path) = explicit_path {
        let path_str = path.to_string_lossy().into_owned();
        debug!("Using explicit BearDog socket path: {}", path_str);
        return path_str;
    }

    // Check both BEARDOG_SOCKET and BEARDOG_CRYPTO_SOCKET for TLS compatibility
    if let Ok(env_path) = std::env::var("BEARDOG_SOCKET") {
        if !env_path.is_empty() {
            debug!("Using BEARDOG_SOCKET env var: {}", env_path);
            return env_path;
        }
    }

    if let Ok(env_path) = std::env::var("BEARDOG_CRYPTO_SOCKET") {
        if !env_path.is_empty() {
            debug!("Using BEARDOG_CRYPTO_SOCKET env var: {}", env_path);
            return env_path;
        }
    }

    // Also check SONGBIRD_CRYPTO_SOCKET for backward compatibility
    if let Ok(env_path) = std::env::var("SONGBIRD_CRYPTO_SOCKET") {
        if !env_path.is_empty() {
            debug!("Using SONGBIRD_CRYPTO_SOCKET env var: {}", env_path);
            return env_path;
        }
    }

    if let Ok(family_id) = std::env::var("FAMILY_ID") {
        if let Some(xdg_path) = discover_xdg_socket("beardog", &family_id) {
            return xdg_path;
        }
    }

    let fallback = "/tmp/beardog-nat0.sock".to_string();
    warn!("Falling back to default BearDog socket path: {}", fallback);
    fallback
}

/// Discover the Neural API socket path.
///
/// Prioritizes:
/// 1. `explicit_path` (from CLI)
/// 2. `NEURAL_API_SOCKET` or `NEURALS_SOCKET` env vars
/// 3. XDG_RUNTIME_DIR + `FAMILY_ID`
/// 4. `/tmp/neural-api-nat0.sock` (fallback)
pub fn discover_neural_api_socket(explicit_path: Option<&PathBuf>) -> String {
    if let Some(path) = explicit_path {
        let path_str = path.to_string_lossy().into_owned();
        debug!("Using explicit Neural API socket path: {}", path_str);
        return path_str;
    }

    if let Ok(env_path) = std::env::var("NEURAL_API_SOCKET") {
        if !env_path.is_empty() {
            debug!("Using NEURAL_API_SOCKET env var: {}", env_path);
            return env_path;
        }
    }

    if let Ok(env_path) = std::env::var("NEURALS_SOCKET") {
        if !env_path.is_empty() {
            debug!("Using NEURALS_SOCKET env var: {}", env_path);
            return env_path;
        }
    }

    if let Ok(family_id) = std::env::var("FAMILY_ID") {
        if let Some(xdg_path) = discover_xdg_socket("neural-api", &family_id) {
            return xdg_path;
        }
    }

    let fallback = "/tmp/neural-api-nat0.sock".to_string();
    warn!("Falling back to default Neural API socket path: {}", fallback);
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::Path;

    // Helper to create a dummy socket file for testing XDG_RUNTIME_DIR
    fn create_dummy_socket(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::File::create(path).unwrap();
    }

    #[test]
    fn test_explicit_path_priority() {
        let custom_path = PathBuf::from("/custom/explicit/beardog.sock");
        let discovered = discover_beardog_socket(Some(&custom_path));
        assert_eq!(discovered, "/custom/explicit/beardog.sock");

        let custom_path = PathBuf::from("/custom/explicit/neural.sock");
        let discovered = discover_neural_api_socket(Some(&custom_path));
        assert_eq!(discovered, "/custom/explicit/neural.sock");
    }

    #[test]
    fn test_env_var_priority_beardog() {
        env::remove_var("BEARDOG_SOCKET");
        env::remove_var("BEARDOG_CRYPTO_SOCKET");
        env::remove_var("SONGBIRD_CRYPTO_SOCKET");
        env::remove_var("XDG_RUNTIME_DIR");
        env::remove_var("FAMILY_ID");

        env::set_var("BEARDOG_SOCKET", "/env/beardog.sock");
        let discovered = discover_beardog_socket(None);
        assert_eq!(discovered, "/env/beardog.sock");
        env::remove_var("BEARDOG_SOCKET");

        env::set_var("BEARDOG_CRYPTO_SOCKET", "/env/beardog-crypto.sock");
        let discovered = discover_beardog_socket(None);
        assert_eq!(discovered, "/env/beardog-crypto.sock");
        env::remove_var("BEARDOG_CRYPTO_SOCKET");

        env::set_var("SONGBIRD_CRYPTO_SOCKET", "/env/songbird-crypto.sock");
        let discovered = discover_beardog_socket(None);
        assert_eq!(discovered, "/env/songbird-crypto.sock");
        env::remove_var("SONGBIRD_CRYPTO_SOCKET");
    }

    #[test]
    fn test_env_var_priority_neural() {
        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("NEURALS_SOCKET");
        env::remove_var("XDG_RUNTIME_DIR");
        env::remove_var("FAMILY_ID");

        env::set_var("NEURAL_API_SOCKET", "/env/neural.sock");
        let discovered = discover_neural_api_socket(None);
        assert_eq!(discovered, "/env/neural.sock");
        env::remove_var("NEURAL_API_SOCKET");

        env::set_var("NEURALS_SOCKET", "/env/neurals.sock");
        let discovered = discover_neural_api_socket(None);
        assert_eq!(discovered, "/env/neurals.sock");
        env::remove_var("NEURALS_SOCKET");
    }

    #[test]
    fn test_xdg_path_construction() {
        env::remove_var("BEARDOG_SOCKET");
        env::remove_var("BEARDOG_CRYPTO_SOCKET");
        env::remove_var("SONGBIRD_CRYPTO_SOCKET");
        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("NEURALS_SOCKET");

        env::set_var("XDG_RUNTIME_DIR", "/tmp/test_xdg_runtime_tls");
        env::set_var("FAMILY_ID", "testfam");

        let beardog_xdg_path = PathBuf::from("/tmp/test_xdg_runtime_tls/biomeos/beardog-testfam.sock");
        create_dummy_socket(&beardog_xdg_path);

        let discovered = discover_beardog_socket(None);
        assert_eq!(discovered, beardog_xdg_path.to_string_lossy().into_owned());

        let neural_xdg_path =
            PathBuf::from("/tmp/test_xdg_runtime_tls/biomeos/neural-api-testfam.sock");
        create_dummy_socket(&neural_xdg_path);

        let discovered = discover_neural_api_socket(None);
        assert_eq!(discovered, neural_xdg_path.to_string_lossy().into_owned());

        fs::remove_file(&beardog_xdg_path).unwrap();
        fs::remove_file(&neural_xdg_path).unwrap();
        fs::remove_dir_all("/tmp/test_xdg_runtime_tls/biomeos").unwrap();
        fs::remove_dir_all("/tmp/test_xdg_runtime_tls").unwrap();
        env::remove_var("XDG_RUNTIME_DIR");
        env::remove_var("FAMILY_ID");
    }

    #[test]
    fn test_legacy_fallback() {
        // Ensure no env vars or XDG_RUNTIME_DIR are set
        env::remove_var("BEARDOG_SOCKET");
        env::remove_var("BEARDOG_CRYPTO_SOCKET");
        env::remove_var("SONGBIRD_CRYPTO_SOCKET");
        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("NEURALS_SOCKET");
        env::remove_var("XDG_RUNTIME_DIR");
        env::remove_var("FAMILY_ID");

        let discovered = discover_beardog_socket(None);
        assert_eq!(discovered, "/tmp/beardog-nat0.sock");

        let discovered = discover_neural_api_socket(None);
        assert_eq!(discovered, "/tmp/neural-api-nat0.sock");
    }

    #[test]
    #[ignore] // Run with: cargo test --package songbird-tls test_empty_env_var_ignored -- --ignored --test-threads=1
    fn test_empty_env_var_ignored() {
        // NOTE: This test modifies environment variables and may fail when run in parallel
        // with other tests due to env var sharing. Run with --test-threads=1 for reliable results.
        use std::sync::atomic::{AtomicU32, Ordering};
        static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        // Clear all related env vars first
        env::remove_var("BEARDOG_SOCKET");
        env::remove_var("BEARDOG_CRYPTO_SOCKET");
        env::remove_var("SONGBIRD_CRYPTO_SOCKET");
        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("NEURALS_SOCKET");
        env::remove_var("XDG_RUNTIME_DIR");
        env::remove_var("FAMILY_ID");

        let test_dir = format!("/tmp/test_xdg_runtime_empty_tls_{}", test_id);
        let family_id = format!("testfam_empty_{}", test_id);

        // Set empty BEARDOG_SOCKET (should be ignored)
        env::set_var("BEARDOG_SOCKET", "");
        env::set_var("XDG_RUNTIME_DIR", &test_dir);
        env::set_var("FAMILY_ID", &family_id);

        let beardog_xdg_path =
            PathBuf::from(format!("{}/biomeos/beardog-{}.sock", test_dir, family_id));
        create_dummy_socket(&beardog_xdg_path);

        let discovered = discover_beardog_socket(None);
        assert_eq!(discovered, beardog_xdg_path.to_string_lossy().into_owned());

        fs::remove_file(&beardog_xdg_path).unwrap();
        fs::remove_dir_all(format!("{}/biomeos", test_dir)).unwrap();
        fs::remove_dir_all(&test_dir).unwrap();
        env::remove_var("BEARDOG_SOCKET");
        env::remove_var("XDG_RUNTIME_DIR");
        env::remove_var("FAMILY_ID");
    }
}

