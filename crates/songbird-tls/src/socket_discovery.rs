//! XDG-compliant socket discovery for TLS layer
//!
//! This module provides functions to discover Unix socket paths for `BearDog`
//! and Neural API following the XDG Base Directory Specification.
//!
//! ## Discovery Order
//! 1. Explicitly provided path (e.g., from CLI arguments)
//! 2. Environment variable (e.g., `BEARDOG_SOCKET`, `NEURAL_API_SOCKET`)
//! 3. `XDG_RUNTIME_DIR` (e.g., `/run/user/1000/biomeos/beardog-nat0.sock`)
//! 4. Fallback to `/tmp` (e.g., `/tmp/beardog-nat0.sock`)
//!
//! ## Zero Hardcoding
//! - No hardcoded paths, only fallback defaults.
//! - Uses `FAMILY_ID` for multi-instance support.
//!
//! ## Concurrency
//! - Thread-safe: Uses dependency injection for env var reading
//! - No global mutable state in tests
//! - Fully concurrent test execution (no `#[ignore]` needed)
//!
//! ## Compatibility
//! This is a duplicate of the `socket_discovery` module from songbird-http-client,
//! kept separate to avoid circular dependencies between crates.

use std::path::PathBuf;
use tracing::{debug, trace, warn};

/// Trait for reading environment variables (dependency injection for testing)
pub trait EnvReader: Send + Sync {
    /// Read an environment variable
    ///
    /// # Errors
    ///
    /// Returns `VarError` if the variable is not set or contains invalid Unicode.
    fn var(&self, key: &str) -> Result<String, std::env::VarError>;
}

/// Real environment variable reader (production)
#[derive(Debug, Clone, Copy)]
pub struct SystemEnv;

impl EnvReader for SystemEnv {
    fn var(&self, key: &str) -> Result<String, std::env::VarError> {
        std::env::var(key)
    }
}

/// Mock environment variable reader for testing (thread-safe, no global state)
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockEnv {
    vars: std::collections::HashMap<String, String>,
}

#[cfg(test)]
impl MockEnv {
    pub fn new() -> Self {
        Self {
            vars: std::collections::HashMap::new(),
        }
    }

    pub fn set(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
impl EnvReader for MockEnv {
    fn var(&self, key: &str) -> Result<String, std::env::VarError> {
        self.vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    }
}

/// Discover an XDG-compliant socket path for a given primal (legacy family-ID support).
///
/// Constructs a path like `/run/user/<UID>/biomeos/<primal>-<family_id>.sock`
/// if `XDG_RUNTIME_DIR` and `FAMILY_ID` are set.
///
/// NOTE: Capability-first discovery is now preferred. This function is retained
/// for backward compatibility with deployments using family-ID in socket names.
#[allow(dead_code)] // Retained for backward compatibility, may be used in legacy paths
fn discover_xdg_socket_with_env(
    primal_name: &str,
    family_id: &str,
    env: &impl EnvReader,
) -> Option<String> {
    if let Ok(runtime_dir) = env.var("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(runtime_dir)
            .join("biomeos")
            .join(format!("{primal_name}-{family_id}.sock"));
        if path.exists() {
            let path_str = path.to_string_lossy().into_owned();
            debug!("Found XDG socket for {}: {}", primal_name, path_str);
            return Some(path_str);
        }
        trace!("XDG socket path does not exist: {}", path.display());
    } else {
        trace!("XDG_RUNTIME_DIR not set for {}", primal_name);
    }
    None
}

/// Internal: Discover security/crypto provider socket path (capability-first)
///
/// ## Resolution Order (capability-first, primal-agnostic)
///
/// 1. `explicit_path` (from CLI)
/// 2. Capability-based env vars: `CRYPTO_PROVIDER_SOCKET`, `SECURITY_PROVIDER_SOCKET`
/// 3. Legacy provider-specific: `BEARDOG_SOCKET`, `BEARDOG_CRYPTO_SOCKET`
/// 4. Legacy Songbird: `SONGBIRD_CRYPTO_SOCKET`, `SONGBIRD_SECURITY_PROVIDER`
/// 5. XDG: `$XDG_RUNTIME_DIR/biomeos/{socket}.sock` (capability names first)
/// 6. UID fallback: `/run/user/$UID/biomeos/security.sock`
/// 7. Legacy: `/tmp/biomeos/security.sock`
fn discover_beardog_socket_with_env(
    explicit_path: Option<&PathBuf>,
    env: &impl EnvReader,
) -> String {
    if let Some(path) = explicit_path {
        let path_str = path.to_string_lossy().into_owned();
        debug!("Using explicit security provider socket path: {}", path_str);
        return path_str;
    }

    // 1. Capability-based env vars (preferred - primal agnostic)
    let capability_env_vars = ["CRYPTO_PROVIDER_SOCKET", "SECURITY_PROVIDER_SOCKET"];

    for env_var in capability_env_vars {
        if let Ok(env_path) = env.var(env_var) {
            if !env_path.is_empty() {
                debug!("Using {} env var (capability-based): {}", env_var, env_path);
                return env_path;
            }
        }
    }

    // 2. Legacy provider-specific env vars (backward compatibility)
    let legacy_env_vars = [
        "BEARDOG_SOCKET",
        "BEARDOG_CRYPTO_SOCKET",
        "SONGBIRD_CRYPTO_SOCKET",
        "SONGBIRD_SECURITY_PROVIDER",
    ];

    for env_var in legacy_env_vars {
        if let Ok(env_path) = env.var(env_var) {
            if !env_path.is_empty() {
                debug!("Using {} env var (legacy): {}", env_var, env_path);
                return env_path;
            }
        }
    }

    // 3. XDG discovery (capability names first, then provider hints)
    if let Ok(runtime_dir) = env.var("XDG_RUNTIME_DIR") {
        let biomeos = PathBuf::from(&runtime_dir).join("biomeos");

        // Capability-named sockets first
        for socket_name in &["crypto.sock", "security.sock", "beardog.sock"] {
            let xdg_path = biomeos.join(socket_name);
            if xdg_path.exists() {
                let path_str = xdg_path.to_string_lossy().into_owned();
                debug!("Found biomeOS standard socket: {}", path_str);
                return path_str;
            }
        }
    }

    // 4. UID-based fallback (without XDG_RUNTIME_DIR, capability names first)
    if let Ok(uid) = env.var("UID") {
        for socket_name in &["security.sock", "crypto.sock", "beardog.sock"] {
            let uid_path = PathBuf::from(format!("/run/user/{uid}/biomeos/{socket_name}"));
            if uid_path.exists() {
                let path_str = uid_path.to_string_lossy().into_owned();
                debug!("Found UID-based socket: {}", path_str);
                return path_str;
            }
        }
    }

    // 5. Legacy /tmp fallback (capability name preferred)
    let fallback_paths = [
        "/tmp/biomeos/security.sock",
        "/tmp/biomeos/crypto.sock",
        "/tmp/biomeos/beardog.sock",
        "/tmp/beardog.sock",
    ];

    for path in fallback_paths {
        if PathBuf::from(path).exists() {
            debug!("Found legacy fallback socket: {}", path);
            return path.to_string();
        }
    }

    // Final fallback (capability name)
    let fallback = "/tmp/biomeos/security.sock".to_string();
    warn!("Falling back to default socket path: {}", fallback);
    fallback
}

/// Discover security/crypto provider socket path (capability-first, production API).
///
/// ## Resolution Order (capability-first, primal-agnostic)
///
/// 1. `explicit_path` (from CLI)
/// 2. Capability-based: `CRYPTO_PROVIDER_SOCKET`, `SECURITY_PROVIDER_SOCKET`
/// 3. Legacy: `BEARDOG_SOCKET`, `BEARDOG_CRYPTO_SOCKET`, `SONGBIRD_*` env vars
/// 4. XDG: `$XDG_RUNTIME_DIR/biomeos/{capability}.sock` (capability names first)
/// 5. UID: `/run/user/$UID/biomeos/security.sock`
/// 6. Legacy: `/tmp/biomeos/security.sock` (fallback)
#[must_use]
pub fn discover_beardog_socket(explicit_path: Option<&PathBuf>) -> String {
    discover_beardog_socket_with_env(explicit_path, &SystemEnv)
}

/// Internal: Discover AI/Neural API socket path (capability-first)
///
/// ## Resolution Order (capability-first, primal-agnostic)
///
/// 1. `explicit_path` (from CLI)
/// 2. Capability-based: `AI_PROVIDER_SOCKET`, `NEURAL_API_SOCKET`
/// 3. Legacy: `NEURALS_SOCKET`
/// 4. XDG: `$XDG_RUNTIME_DIR/biomeos/{capability}.sock` (ai.sock first)
/// 5. UID: `/run/user/$UID/biomeos/ai.sock`
/// 6. Legacy: `/tmp/biomeos/ai.sock` (fallback)
fn discover_neural_api_socket_with_env(
    explicit_path: Option<&PathBuf>,
    env: &impl EnvReader,
) -> String {
    if let Some(path) = explicit_path {
        let path_str = path.to_string_lossy().into_owned();
        debug!("Using explicit Neural API socket path: {}", path_str);
        return path_str;
    }

    // 1. Capability-based env vars first (AI capability)
    let capability_env_vars = ["AI_PROVIDER_SOCKET", "NEURAL_API_SOCKET", "NEURALS_SOCKET"];

    for env_var in capability_env_vars {
        if let Ok(env_path) = env.var(env_var) {
            if !env_path.is_empty() {
                debug!("Using {} env var: {}", env_var, env_path);
                return env_path;
            }
        }
    }

    // 2. XDG discovery (capability names first)
    if let Ok(runtime_dir) = env.var("XDG_RUNTIME_DIR") {
        let biomeos = PathBuf::from(&runtime_dir).join("biomeos");

        // Capability-named sockets first, then provider hints
        for socket_name in &["ai.sock", "neural-api.sock", "squirrel.sock"] {
            let xdg_path = biomeos.join(socket_name);
            if xdg_path.exists() {
                let path_str = xdg_path.to_string_lossy().into_owned();
                debug!("Found biomeOS AI socket: {}", path_str);
                return path_str;
            }
        }
    }

    // 3. UID-based fallback (capability names first)
    if let Ok(uid) = env.var("UID") {
        for socket_name in &["ai.sock", "neural-api.sock", "squirrel.sock"] {
            let uid_path = PathBuf::from(format!("/run/user/{uid}/biomeos/{socket_name}"));
            if uid_path.exists() {
                let path_str = uid_path.to_string_lossy().into_owned();
                debug!("Found UID-based AI socket: {}", path_str);
                return path_str;
            }
        }
    }

    // 4. Legacy fallback (capability name preferred)
    let fallback_paths =
        ["/tmp/biomeos/ai.sock", "/tmp/biomeos/neural-api.sock", "/tmp/biomeos/squirrel.sock"];

    for path in fallback_paths {
        if PathBuf::from(path).exists() {
            return path.to_string();
        }
    }

    let fallback = "/tmp/biomeos/ai.sock".to_string();
    warn!("Falling back to legacy Neural API socket path: {}", fallback);
    fallback
}

/// Discover the Neural API socket path (production API).
///
/// Prioritizes:
/// 1. `explicit_path` (from CLI)
/// 2. `NEURAL_API_SOCKET` or `NEURALS_SOCKET` env vars
/// 3. `$XDG_RUNTIME_DIR/biomeos/neural-api-{family_id}.sock` (if `FAMILY_ID` set)
/// 4. `$XDG_RUNTIME_DIR/biomeos/beardog.sock` (biomeOS standard)
/// 5. `/run/user/$UID/biomeos/beardog.sock` (UID fallback)
/// 6. `/tmp/beardog.sock` (legacy fallback)
#[must_use]
pub fn discover_neural_api_socket(explicit_path: Option<&PathBuf>) -> String {
    discover_neural_api_socket_with_env(explicit_path, &SystemEnv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    // Helper to create a dummy socket file for testing XDG_RUNTIME_DIR
    fn create_dummy_socket(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::File::create(path).unwrap();
    }

    // ============================================================================
    // ✅ ALL TESTS BELOW RUN CONCURRENTLY!
    // ============================================================================
    //
    // These tests use dependency injection (MockEnv) instead of modifying global
    // environment variables, making them fully thread-safe and concurrent.
    //
    // This is the idiomatic Rust approach: no shared mutable state, no #[ignore],
    // no sleep(), no serial execution. Just pure concurrent correctness.
    // ============================================================================

    #[test]
    fn test_explicit_path_priority() {
        let env = MockEnv::new(); // Empty env, explicit path takes priority

        let custom_path = PathBuf::from("/custom/explicit/beardog.sock");
        let discovered = discover_beardog_socket_with_env(Some(&custom_path), &env);
        assert_eq!(discovered, "/custom/explicit/beardog.sock");

        let custom_path = PathBuf::from("/custom/explicit/neural.sock");
        let discovered = discover_neural_api_socket_with_env(Some(&custom_path), &env);
        assert_eq!(discovered, "/custom/explicit/neural.sock");
    }

    #[test]
    fn test_env_var_priority_beardog() {
        // Test BEARDOG_SOCKET priority
        let env = MockEnv::new().set("BEARDOG_SOCKET", "/env/beardog.sock");
        let discovered = discover_beardog_socket_with_env(None, &env);
        assert_eq!(discovered, "/env/beardog.sock");

        // Test BEARDOG_CRYPTO_SOCKET priority (when BEARDOG_SOCKET not set)
        let env = MockEnv::new().set("BEARDOG_CRYPTO_SOCKET", "/env/beardog-crypto.sock");
        let discovered = discover_beardog_socket_with_env(None, &env);
        assert_eq!(discovered, "/env/beardog-crypto.sock");

        // Test SONGBIRD_CRYPTO_SOCKET priority (when others not set)
        let env = MockEnv::new().set("SONGBIRD_CRYPTO_SOCKET", "/env/songbird-crypto.sock");
        let discovered = discover_beardog_socket_with_env(None, &env);
        assert_eq!(discovered, "/env/songbird-crypto.sock");

        // Test priority order: BEARDOG_SOCKET > BEARDOG_CRYPTO_SOCKET
        let env = MockEnv::new()
            .set("BEARDOG_SOCKET", "/env/beardog.sock")
            .set("BEARDOG_CRYPTO_SOCKET", "/env/beardog-crypto.sock");
        let discovered = discover_beardog_socket_with_env(None, &env);
        assert_eq!(discovered, "/env/beardog.sock");
    }

    #[test]
    fn test_env_var_priority_neural() {
        // Test NEURAL_API_SOCKET priority
        let env = MockEnv::new().set("NEURAL_API_SOCKET", "/env/neural.sock");
        let discovered = discover_neural_api_socket_with_env(None, &env);
        assert_eq!(discovered, "/env/neural.sock");

        // Test NEURALS_SOCKET priority (when NEURAL_API_SOCKET not set)
        let env = MockEnv::new().set("NEURALS_SOCKET", "/env/neurals.sock");
        let discovered = discover_neural_api_socket_with_env(None, &env);
        assert_eq!(discovered, "/env/neurals.sock");

        // Test priority order: NEURAL_API_SOCKET > NEURALS_SOCKET
        let env = MockEnv::new()
            .set("NEURAL_API_SOCKET", "/env/neural.sock")
            .set("NEURALS_SOCKET", "/env/neurals.sock");
        let discovered = discover_neural_api_socket_with_env(None, &env);
        assert_eq!(discovered, "/env/neural.sock");
    }

    #[test]
    fn test_xdg_path_construction() {
        // Thread-safe test using MockEnv (no global state modification)
        use std::sync::atomic::{AtomicU32, Ordering};
        static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);

        let test_dir = format!("/tmp/test_xdg_runtime_tls_{}", test_id);

        let env = MockEnv::new().set("XDG_RUNTIME_DIR", &test_dir);

        // Capability-first: crypto.sock and ai.sock are preferred over provider names
        let crypto_path = PathBuf::from(format!("{}/biomeos/crypto.sock", test_dir));
        create_dummy_socket(&crypto_path);

        let discovered = discover_beardog_socket_with_env(None, &env);
        assert_eq!(discovered, crypto_path.to_string_lossy().into_owned());

        let ai_path = PathBuf::from(format!("{}/biomeos/ai.sock", test_dir));
        create_dummy_socket(&ai_path);

        let discovered = discover_neural_api_socket_with_env(None, &env);
        assert_eq!(discovered, ai_path.to_string_lossy().into_owned());

        // Cleanup
        fs::remove_file(&crypto_path).unwrap();
        fs::remove_file(&ai_path).unwrap();
        fs::remove_dir_all(format!("{}/biomeos", test_dir)).unwrap();
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_legacy_fallback() {
        // Thread-safe test using MockEnv with empty environment
        let env = MockEnv::new(); // No env vars set

        // Capability-first: Falls back to security.sock (capability name preferred)
        let discovered = discover_beardog_socket_with_env(None, &env);
        assert_eq!(discovered, "/tmp/biomeos/security.sock");

        // AI capability: Falls back to ai.sock (capability name preferred)
        let discovered = discover_neural_api_socket_with_env(None, &env);
        assert_eq!(discovered, "/tmp/biomeos/ai.sock");
    }

    #[test]
    fn test_security_provider_env_var() {
        // SONGBIRD_SECURITY_PROVIDER takes priority after BEARDOG_* vars
        let env =
            MockEnv::new().set("SONGBIRD_SECURITY_PROVIDER", "/run/user/1000/biomeos/beardog.sock");
        let discovered = discover_beardog_socket_with_env(None, &env);
        assert_eq!(discovered, "/run/user/1000/biomeos/beardog.sock");
    }

    #[test]
    fn test_xdg_standard_without_family_id() {
        // biomeOS standard: $XDG_RUNTIME_DIR/biomeos/beardog.sock (no family ID needed)
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let test_id = COUNTER.fetch_add(1, Ordering::SeqCst);

        let test_dir = format!("/tmp/test_xdg_standard_{}", test_id);
        let env = MockEnv::new().set("XDG_RUNTIME_DIR", &test_dir);

        let xdg_path = PathBuf::from(format!("{}/biomeos/beardog.sock", test_dir));
        create_dummy_socket(&xdg_path);

        let discovered = discover_beardog_socket_with_env(None, &env);
        assert_eq!(discovered, xdg_path.to_string_lossy().into_owned());

        // Cleanup
        fs::remove_file(&xdg_path).unwrap();
        fs::remove_dir_all(format!("{}/biomeos", test_dir)).unwrap();
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_empty_env_var_ignored() {
        // ✅ NO MORE #[ignore]! This test now runs concurrently!
        // Thread-safe test using MockEnv - no global state modification
        use std::sync::atomic::{AtomicU32, Ordering};
        static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);

        let test_dir = format!("/tmp/test_xdg_runtime_empty_tls_{}", test_id);

        // Empty env vars should be ignored, XDG capability-named socket should be used
        let env = MockEnv::new()
            .set("BEARDOG_SOCKET", "") // Empty - should be ignored
            .set("CRYPTO_PROVIDER_SOCKET", "") // Empty - should be ignored
            .set("XDG_RUNTIME_DIR", &test_dir);

        // Capability-first: security.sock is preferred
        let security_path = PathBuf::from(format!("{}/biomeos/security.sock", test_dir));
        create_dummy_socket(&security_path);

        let discovered = discover_beardog_socket_with_env(None, &env);
        assert_eq!(discovered, security_path.to_string_lossy().into_owned());

        // Cleanup
        fs::remove_file(&security_path).unwrap();
        fs::remove_dir_all(format!("{}/biomeos", test_dir)).unwrap();
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_concurrent_discovery() {
        // ✅ NEW TEST: Demonstrates true concurrent execution!
        // Multiple threads discovering sockets simultaneously with no race conditions
        use std::thread;

        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let env =
                        MockEnv::new().set("BEARDOG_SOCKET", format!("/env/beardog-{}.sock", i));
                    let discovered = discover_beardog_socket_with_env(None, &env);
                    assert_eq!(discovered, format!("/env/beardog-{}.sock", i));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
