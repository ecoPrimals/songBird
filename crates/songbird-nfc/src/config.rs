// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! NFC configuration with `BearDog` integration

use std::path::PathBuf;
use std::time::Duration;

/// NFC protocol configuration
///
/// All crypto operations delegated to `BearDog` - zero hardcoded secrets
#[derive(Debug, Clone)]
pub struct NfcConfig {
    /// `BearDog` socket path for crypto operations
    pub beardog_socket: PathBuf,

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
            // BearDog socket discovered at runtime (no hardcoding)
            beardog_socket: Self::discover_beardog_socket(),

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

    /// Set `BearDog` socket path
    #[must_use]
    pub fn with_beardog_socket(mut self, socket: PathBuf) -> Self {
        self.beardog_socket = socket;
        self
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
    fn discover_beardog_socket() -> PathBuf {
        // 1. Capability-based env vars (preferred - primal agnostic)
        for env_var in &[
            "SECURITY_PROVIDER_SOCKET",
            "CRYPTO_PROVIDER_SOCKET",
            "SONGBIRD_SECURITY_PROVIDER",
            "BEARDOG_SOCKET", // backward compatibility
        ] {
            if let Ok(socket) = std::env::var(env_var) {
                return PathBuf::from(socket);
            }
        }

        // 2. XDG runtime directory (capability names first, then provider hints)
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
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
            let fallback_paths = [
                "/tmp/biomeos/security.sock",
                "/tmp/biomeos/crypto.sock",
                "/tmp/biomeos/beardog.sock",
            ];

            for path in fallback_paths {
                let path_buf = PathBuf::from(path);
                if path_buf.exists() {
                    return path_buf;
                }
            }

            PathBuf::from("/tmp/biomeos/security.sock")
        }

        #[cfg(not(unix))]
        {
            PathBuf::from("security.sock")
        }
    }
}
