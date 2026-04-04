// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! NFC configuration with security-provider (crypto) socket discovery

use std::path::PathBuf;
use std::time::Duration;

/// NFC protocol configuration
///
/// All crypto operations delegated to the configured security provider — zero hardcoded secrets
#[derive(Debug, Clone)]
pub struct NfcConfig {
    /// Security provider Unix socket path for crypto operations (see discovery order below)
    pub security_provider_socket: PathBuf,

    /// Exchange timeout (including timing protection delays)
    pub exchange_timeout: Duration,

    /// Enable timing protection (constant-time operations)
    pub timing_protection: bool,

    /// Target exchange duration for timing protection
    pub target_exchange_duration: Duration,

    /// Maximum random delay for timing protection
    pub max_random_delay: Duration,

    /// Enable connection validation
    pub validate_connection: bool,
}

impl Default for NfcConfig {
    fn default() -> Self {
        Self {
            // Security provider socket discovered at runtime (no hardcoding)
            security_provider_socket: Self::discover_security_socket(),

            exchange_timeout: Duration::from_secs(30),
            timing_protection: true,
            target_exchange_duration: Duration::from_secs(10),
            max_random_delay: Duration::from_millis(500),
            validate_connection: true,
        }
    }
}

impl NfcConfig {
    /// Create new configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set security provider socket path
    #[must_use]
    pub fn with_security_provider_socket(mut self, socket: PathBuf) -> Self {
        self.security_provider_socket = socket;
        self
    }

    /// Deprecated alias for [`NfcConfig::with_security_provider_socket`].
    #[deprecated(note = "use with_security_provider_socket")]
    #[must_use]
    pub fn with_beardog_socket(self, socket: PathBuf) -> Self {
        self.with_security_provider_socket(socket)
    }

    /// Set exchange timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.exchange_timeout = timeout;
        self
    }

    /// Enable/disable timing protection
    #[must_use]
    pub const fn with_timing_protection(mut self, enabled: bool) -> Self {
        self.timing_protection = enabled;
        self
    }

    /// Discover security/crypto provider socket at runtime (capability-first)
    ///
    /// ## Resolution Order (capability-first, primal-agnostic)
    ///
    /// 1. `SECURITY_PROVIDER_SOCKET` - Capability-based (preferred for NFC)
    /// 2. `CRYPTO_PROVIDER_SOCKET` - Capability-based alternative
    /// 3. `SONGBIRD_SECURITY_PROVIDER` - Legacy capability-based
    /// 4. `BEARDOG_SOCKET` - Provider-specific (backward compatibility)
    /// 5. XDG: `$XDG_RUNTIME_DIR/biomeos/security.sock` - Capability-named
    /// 6. XDG: `$XDG_RUNTIME_DIR/biomeos/crypto.sock` - Capability-named
    /// 7. XDG: `$XDG_RUNTIME_DIR/biomeos/beardog.sock` - Provider hint
    /// 8. Legacy: `/tmp/biomeos/security.sock` - Fallback
    fn discover_security_socket() -> PathBuf {
        // 1. Capability-based env vars (preferred - primal agnostic)
        for env_var in &[
            "SECURITY_PROVIDER_SOCKET",
            "CRYPTO_PROVIDER_SOCKET",
            "SONGBIRD_SECURITY_PROVIDER",
            "BEARDOG_SOCKET", // backward compatibility
        ] {
            if let Ok(socket) = songbird_process_env::var(env_var) {
                return PathBuf::from(socket);
            }
        }

        // 2. XDG runtime directory (capability names first, then provider hints)
        if let Ok(xdg_runtime) = songbird_process_env::var("XDG_RUNTIME_DIR") {
            let biomeos = PathBuf::from(&xdg_runtime).join("biomeos");

            // Capability-named sockets only — no primal identities
            for socket_name in &["security.sock", "crypto.sock"] {
                let socket = biomeos.join(socket_name);
                if socket.exists() {
                    return socket;
                }
            }
        }

        // 3. Fallback (platform-specific, capability name preferred)
        #[cfg(unix)]
        {
            use songbird_types::defaults::paths::{
                biomeos_socket_dir_tmp, security_socket_default_path,
            };

            let fallback_paths = [
                security_socket_default_path(),
                biomeos_socket_dir_tmp().join("crypto.sock"),
                biomeos_socket_dir_tmp().join("beardog.sock"),
            ];

            for path in &fallback_paths {
                if path.exists() {
                    return path.clone();
                }
            }

            security_socket_default_path()
        }

        #[cfg(not(unix))]
        {
            PathBuf::from("security.sock")
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::Duration;

    /// Serialize overlay mutations for `discover_security_socket` resolution order tests.
    static ENV_OVERLAY_LOCK: Mutex<()> = Mutex::new(());

    fn clear_socket_overlay_keys() {
        for key in [
            "SECURITY_PROVIDER_SOCKET",
            "CRYPTO_PROVIDER_SOCKET",
            "SONGBIRD_SECURITY_PROVIDER",
            "BEARDOG_SOCKET",
        ] {
            songbird_process_env::remove_var(key);
        }
    }

    #[test]
    fn new_matches_default() {
        let a = NfcConfig::new();
        let b = NfcConfig::default();
        assert_eq!(a.exchange_timeout, b.exchange_timeout, "new() should mirror default()");
        assert_eq!(a.timing_protection, b.timing_protection);
        assert_eq!(a.validate_connection, b.validate_connection);
    }

    #[test]
    fn builder_methods_override_fields() {
        let socket = PathBuf::from("/tmp/custom-security.sock");
        let cfg = NfcConfig::default()
            .with_security_provider_socket(socket.clone())
            .with_timeout(Duration::from_secs(60))
            .with_timing_protection(false)
            .with_timing_protection(true);

        assert_eq!(
            cfg.security_provider_socket, socket,
            "with_security_provider_socket should stick"
        );
        assert_eq!(cfg.exchange_timeout, Duration::from_secs(60));
        assert!(cfg.timing_protection, "last with_timing_protection wins");
    }

    #[test]
    fn with_timeout_is_const_path() {
        let cfg = NfcConfig::default().with_timeout(Duration::from_nanos(1));
        assert_eq!(cfg.exchange_timeout, Duration::from_nanos(1));
    }

    #[test]
    fn discover_security_socket_prefers_security_provider_overlay() {
        let _g = ENV_OVERLAY_LOCK.lock().expect("env overlay lock");
        clear_socket_overlay_keys();
        songbird_process_env::set_var("SECURITY_PROVIDER_SOCKET", "/tmp/overlay-security.sock");
        songbird_process_env::set_var("CRYPTO_PROVIDER_SOCKET", "/tmp/overlay-crypto.sock");
        let cfg = NfcConfig::default();
        assert_eq!(
            cfg.security_provider_socket,
            PathBuf::from("/tmp/overlay-security.sock"),
            "SECURITY_PROVIDER_SOCKET should win over CRYPTO_PROVIDER_SOCKET"
        );
        clear_socket_overlay_keys();
    }

    #[test]
    fn discover_security_socket_falls_through_to_crypto_when_security_unset() {
        let _g = ENV_OVERLAY_LOCK.lock().expect("env overlay lock");
        clear_socket_overlay_keys();
        songbird_process_env::set_var("CRYPTO_PROVIDER_SOCKET", "/tmp/only-crypto.sock");
        let cfg = NfcConfig::default();
        assert_eq!(cfg.security_provider_socket, PathBuf::from("/tmp/only-crypto.sock"));
        clear_socket_overlay_keys();
    }

    #[test]
    fn timing_protection_fields_are_independent() {
        let cfg =
            NfcConfig::default().with_timing_protection(false).with_timeout(Duration::from_secs(5));
        assert!(!cfg.timing_protection);
        assert_eq!(cfg.exchange_timeout, Duration::from_secs(5));
        assert!(
            cfg.target_exchange_duration > Duration::ZERO,
            "default target exchange duration should remain positive"
        );
    }

    #[test]
    fn validate_connection_defaults_true() {
        assert!(NfcConfig::default().validate_connection);
    }

    #[test]
    fn with_timeout_can_be_zero() {
        let cfg = NfcConfig::default().with_timeout(Duration::from_secs(0));
        assert_eq!(cfg.exchange_timeout, Duration::ZERO);
    }

    #[test]
    fn discover_security_socket_prefers_songbird_security_provider_over_beardog() {
        let _g = ENV_OVERLAY_LOCK.lock().expect("env overlay lock");
        clear_socket_overlay_keys();
        songbird_process_env::set_var("SONGBIRD_SECURITY_PROVIDER", "/tmp/songbird.sock");
        songbird_process_env::set_var("BEARDOG_SOCKET", "/tmp/beardog.sock");
        let cfg = NfcConfig::default();
        assert_eq!(cfg.security_provider_socket, PathBuf::from("/tmp/songbird.sock"));
        clear_socket_overlay_keys();
    }

    #[test]
    fn explicit_builder_socket_skips_discovery_order() {
        let explicit = PathBuf::from("/tmp/explicit-only.sock");
        let cfg = NfcConfig::default().with_security_provider_socket(explicit.clone());
        assert_eq!(cfg.security_provider_socket, explicit);
    }
}
