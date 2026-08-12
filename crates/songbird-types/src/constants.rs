// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! 🔧 Songbird Canonical Constants System
//!
//! **SINGLE SOURCE OF TRUTH FOR ALL STATIC CONSTANTS** ✅
//!
//! This module consolidates all static constants from across the Songbird ecosystem.
//! Dynamic/calculated constants remain in `songbird-config` for environment-specific logic.
//!
//! ## Consolidation Summary
//! - **Static constants**: Defined here once
//! - **Dynamic constants**: Calculated in songbird-config based on environment
//! - **Test constants**: Specialized variants in songbird-test-utils
//!
//! ## Usage
//! ```rust
//! use songbird_types::constants::*;
//! ```

use std::time::Duration;

// ============================================================================
// NETWORK CONSTANTS
// ============================================================================

/// Default port for Songbird services (HTTP API fallback).
pub const DEFAULT_PORT: u16 = crate::defaults::ports::DEFAULT_HTTP_PORT;

/// Localhost IPv4 address
pub const LOCALHOST: &str = "127.0.0.1";

/// Localhost hostname — used in service configs and host discovery defaults
pub const LOCALHOST_HOSTNAME: &str = "localhost";

/// IPv6 loopback address
pub const LOCALHOST_IPV6: &str = "::1";

/// IPv6 loopback (bracketed form for socket addresses)
pub const LOCALHOST_IPV6_BRACKETED: &str = "[::1]";

/// Returns `true` when `host` is a recognized loopback / localhost string.
#[must_use]
pub fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "0.0.0.0" | "localhost" | "::1" | "[::1]")
}

/// Builds a `http://localhost:{port}` development fallback URL.
///
/// Only available in debug builds — production code must resolve endpoints
/// through environment variables or capability-based discovery.
#[cfg(debug_assertions)]
#[must_use]
pub fn dev_localhost_url(port: u16) -> String {
    format!("http://{LOCALHOST}:{port}")
}

/// Returns the appropriate HTTP scheme based on `SONGBIRD_TLS_ENABLED`.
///
/// Defaults to `"http"` for LAN mesh communication where TLS is handled
/// at a lower layer (BTSP).
#[must_use]
pub fn http_scheme() -> &'static str {
    if crate::error_helpers::SafeEnv::get_bool("SONGBIRD_TLS_ENABLED", false) {
        "https"
    } else {
        "http"
    }
}

/// Builds a transport-aware URL from host, port, and optional path.
/// Scheme is selected via `http_scheme()` (environment-driven).
#[must_use]
pub fn endpoint_url(host: &str, port: u16, path: &str) -> String {
    let scheme = http_scheme();
    format!("{scheme}://{host}:{port}{path}")
}

/// Production bind address (all interfaces, IPv4)
pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";

/// Production bind address (all interfaces, IPv6 dual-stack)
pub const PRODUCTION_BIND_ADDRESS_IPV6: &str = "[::]";

/// Ephemeral bind address — all interfaces, OS-assigned port.
pub const EPHEMERAL_BIND_ADDR: &str = "0.0.0.0:0";

/// Development bind address (localhost only)
pub const DEVELOPMENT_BIND_ADDRESS: &str = "127.0.0.1";

/// FHS system runtime directory for well-known service paths.
pub const SYSTEM_RUNTIME_DIR: &str = "/var/run";

/// biomeOS system runtime socket directory (FHS-standard).
pub const BIOMEOS_SYSTEM_RUNTIME_DIR: &str = "/var/run/biomeos";

/// Songbird PID/state directory under FHS system runtime (Unix only).
pub const SONGBIRD_SYSTEM_RUNTIME_DIR: &str = "/var/run/songbird";

/// Platform-aware PID/state directory.
///
/// On Unix: `/var/run/songbird`
/// On Windows: `%PROGRAMDATA%\songbird` (falls back to `C:\ProgramData\songbird`)
#[must_use]
pub fn songbird_runtime_dir() -> std::path::PathBuf {
    #[cfg(unix)]
    {
        std::path::PathBuf::from(SONGBIRD_SYSTEM_RUNTIME_DIR)
    }
    #[cfg(not(unix))]
    {
        std::env::var("PROGRAMDATA").map_or_else(
            |_| std::path::PathBuf::from(r"C:\ProgramData\songbird"),
            |pd| std::path::PathBuf::from(pd).join("songbird"),
        )
    }
}

/// User runtime directory prefix (FHS: /run/user/{uid}).
pub const USER_RUNTIME_PREFIX: &str = "/run/user";

/// macOS/iOS shared temporary directory for IPC sockets.
pub const MACOS_SHARED_TMP_DIR: &str = "/var/tmp";

// ============================================================================
// TIMEOUT CONSTANTS
// ============================================================================

/// Default timeout for network operations (in seconds)
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default connection timeout
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Default read timeout
pub const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Default write timeout
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

/// Default request timeout
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Default health check timeout
pub const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

// ============================================================================
// RESOURCE LIMITS
// ============================================================================

/// Default retry attempts for failed operations
pub const DEFAULT_RETRY_ATTEMPTS: u32 = 3;

/// Maximum number of concurrent connections
pub const MAX_CONCURRENT_CONNECTIONS: usize = 1000;

/// Default buffer size (64KB)
pub const DEFAULT_BUFFER_SIZE: usize = 65536;

/// Maximum request size (10MB)
pub const MAX_REQUEST_SIZE: u64 = 10_485_760;

/// Maximum connections per IP
pub const MAX_CONNECTIONS_PER_IP: u32 = 10;

// ============================================================================
// GAMING CONSTANTS
// ============================================================================

/// Gaming port range start
pub const GAMING_PORT_RANGE_START: u16 = 6112;

/// Gaming port range end
pub const GAMING_PORT_RANGE_END: u16 = 6200;

/// Maximum players per gaming session
pub const MAX_PLAYERS_PER_SESSION: usize = 100;

/// Gaming buffer size (64KB)
pub const GAMING_BUFFER_SIZE: usize = 65536;

// ============================================================================
// HEALTH CHECK CONSTANTS
// ============================================================================

/// Default health check interval
pub const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);

/// Maximum failed health checks before marking unhealthy
pub const MAX_FAILED_HEALTH_CHECKS: u32 = 3;

/// Default health check endpoint path
pub const DEFAULT_HEALTH_ENDPOINT: &str = "/health";

// ============================================================================
// DISCOVERY CONSTANTS
// ============================================================================

/// Default multicast group for ecosystem discovery (IPv4 link-local)
pub const DISCOVERY_MULTICAST_GROUP: &str = "239.255.42.99";

/// Default multicast port for discovery announcements
pub const DISCOVERY_MULTICAST_PORT: u16 = 4242;

/// Default mDNS multicast address (RFC 6762)
pub const MDNS_MULTICAST_GROUP: &str = "224.0.0.251";

/// Default Consul HTTP API port
pub const CONSUL_DEFAULT_PORT: u16 = 8500;

/// Default Consul HTTP API base URL (local agent)
pub const CONSUL_DEFAULT_URL: &str = "http://127.0.0.1:8500";

/// Default mDNS port (IANA)
pub const MDNS_PORT: u16 = 5353;

/// Default broadcast discovery port (UDP peer announcements)
pub const BROADCAST_DISCOVERY_PORT: u16 = 2300;

/// Default public STUN server (primary) — host:port
pub const DEFAULT_STUN_SERVER_1: &str = "stun.nextcloud.com:3478";

/// Default public STUN server (secondary) — host:port
pub const DEFAULT_STUN_SERVER_2: &str = "stun.cloudflare.com:3478";

/// Discovery interval
pub const DISCOVERY_INTERVAL: Duration = Duration::from_secs(30);

/// Service registration TTL
pub const REGISTRATION_TTL: Duration = Duration::from_secs(300);

/// Maximum discovery retries
pub const MAX_DISCOVERY_RETRIES: u32 = 3;

// ============================================================================
// RIBOCIPHER TRANSPORT SIGNAL — Stream 7 convergent standard
// ============================================================================

/// riboCipher transport signal module.
///
/// Modeled on ribosomal codon reading: the accept loop (ribosome) reads
/// signal bytes (codons) and routes deterministically. Three tiers:
///
/// - `CLEAR` (`0xEC`): Ecosystem JSON-RPC, plaintext NDJSON. Default for local IPC.
/// - `MITO` (`0xED`): Mitochondrial obfuscation. Federation inter-gate traffic.
/// - `NUCLEAR` (`0xEE`): Nuclear-sealed envelope. High-security tunnels.
///
/// Deprecation path: WARN (111-112) → ERROR (112) → REJECT (113) → REMOVE (114)
pub mod ribocipher {
    /// Clear ecosystem signal — plaintext JSON-RPC follows.
    pub const CLEAR: u8 = 0xEC;
    /// Mito-obfuscated signal — federation inter-gate traffic.
    pub const MITO: u8 = 0xED;
    /// Nuclear-sealed signal — high-security encrypted tunnel.
    pub const NUCLEAR: u8 = 0xEE;

    /// riboCipher version byte (follows the signal byte).
    pub const VERSION_1: u8 = 0x01;

    /// Full two-byte clear signal prefix: `[0xEC, 0x01]`.
    pub const CLEAR_PREFIX: [u8; 2] = [CLEAR, VERSION_1];
    /// Full two-byte mito signal prefix: `[0xED, 0x01]`.
    pub const MITO_PREFIX: [u8; 2] = [MITO, VERSION_1];
    /// Full two-byte nuclear signal prefix: `[0xEE, 0x01]`.
    pub const NUCLEAR_PREFIX: [u8; 2] = [NUCLEAR, VERSION_1];

    /// Returns `true` if the byte is a recognized riboCipher signal tier.
    #[must_use]
    pub const fn is_signal_byte(b: u8) -> bool {
        matches!(b, CLEAR | MITO | NUCLEAR)
    }

    /// Human-readable tier name for logging.
    #[must_use]
    pub const fn tier_name(b: u8) -> &'static str {
        match b {
            CLEAR => "clear",
            MITO => "mito",
            NUCLEAR => "nuclear",
            _ => "unknown",
        }
    }
}

// ============================================================================
// ENDPOINT CONSTRUCTION
// ============================================================================

/// Standard JSON-RPC endpoint path for inter-primal HTTP communication.
pub const JSONRPC_PATH: &str = "/jsonrpc";

/// Build an HTTP JSON-RPC endpoint URL from a socket address.
///
/// Constructs `http://{ip}:{port}/jsonrpc` from the given address components.
/// Centralizes the ad-hoc URL format that was previously scattered across
/// capability propagation, remote dispatch, and federation modules.
#[must_use]
pub fn jsonrpc_endpoint_url(addr: &std::net::SocketAddr) -> String {
    format!("http://{}:{}{JSONRPC_PATH}", addr.ip(), addr.port())
}

// ============================================================================
// SYSTEM CONSTANTS
// ============================================================================

/// Fallback base directory when `HOME` is unset.
///
/// Used only as a degraded last-resort path prefix for data/cache/config/log
/// directories when no user home directory can be determined. Production
/// deployments should always have `HOME` set.
pub const HOME_FALLBACK_DIR: &str = "/tmp";

/// Default configuration file path
pub const DEFAULT_CONFIG_PATH: &str = "songbird.toml";

/// Default log level
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Default cache TTL
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);

/// Metrics collection interval
pub const METRICS_COLLECTION_INTERVAL: Duration = Duration::from_secs(60);

// ============================================================================
// ENVIRONMENT NAMES
// ============================================================================

/// Development environment name
pub const ENV_DEVELOPMENT: &str = "development";

/// Testing environment name
pub const ENV_TESTING: &str = "testing";

/// Staging environment name
pub const ENV_STAGING: &str = "staging";

/// Production environment name
pub const ENV_PRODUCTION: &str = "production";

// ============================================================================
// TESTS
// ============================================================================

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_timeout_secs() {
        assert_eq!(DEFAULT_TIMEOUT_SECS, 30);
    }

    #[test]
    fn test_default_retry_attempts() {
        assert_eq!(DEFAULT_RETRY_ATTEMPTS, 3);
    }

    #[test]
    fn test_default_port() {
        assert_eq!(DEFAULT_PORT, 8080);
    }

    #[test]
    fn test_max_concurrent_connections() {
        assert_eq!(MAX_CONCURRENT_CONNECTIONS, 1000);
    }

    #[test]
    fn test_constants_are_reasonable() {
        assert_eq!(DEFAULT_TIMEOUT_SECS, 30);
        assert_eq!(DEFAULT_RETRY_ATTEMPTS, 3);
        assert_eq!(DEFAULT_PORT, 8080);
        assert_eq!(MAX_CONCURRENT_CONNECTIONS, 1000);
    }

    mod ribocipher_tests {
        use super::super::ribocipher;

        #[test]
        fn signal_bytes_are_distinct() {
            assert_ne!(ribocipher::CLEAR, ribocipher::MITO);
            assert_ne!(ribocipher::MITO, ribocipher::NUCLEAR);
            assert_ne!(ribocipher::CLEAR, ribocipher::NUCLEAR);
        }

        #[test]
        fn signal_byte_values() {
            assert_eq!(ribocipher::CLEAR, 0xEC);
            assert_eq!(ribocipher::MITO, 0xED);
            assert_eq!(ribocipher::NUCLEAR, 0xEE);
        }

        #[test]
        fn is_signal_byte_recognizes_all_tiers() {
            assert!(ribocipher::is_signal_byte(0xEC));
            assert!(ribocipher::is_signal_byte(0xED));
            assert!(ribocipher::is_signal_byte(0xEE));
        }

        #[test]
        fn is_signal_byte_rejects_non_signals() {
            assert!(!ribocipher::is_signal_byte(0x00));
            assert!(!ribocipher::is_signal_byte(0x16)); // TLS
            assert!(!ribocipher::is_signal_byte(b'{'));
            assert!(!ribocipher::is_signal_byte(0xEB));
            assert!(!ribocipher::is_signal_byte(0xEF));
            assert!(!ribocipher::is_signal_byte(0xFF));
        }

        #[test]
        fn tier_names() {
            assert_eq!(ribocipher::tier_name(0xEC), "clear");
            assert_eq!(ribocipher::tier_name(0xED), "mito");
            assert_eq!(ribocipher::tier_name(0xEE), "nuclear");
            assert_eq!(ribocipher::tier_name(0x00), "unknown");
        }

        #[test]
        fn prefixes_match_signal_plus_version() {
            assert_eq!(ribocipher::CLEAR_PREFIX, [0xEC, 0x01]);
            assert_eq!(ribocipher::MITO_PREFIX, [0xED, 0x01]);
            assert_eq!(ribocipher::NUCLEAR_PREFIX, [0xEE, 0x01]);
        }

        #[test]
        fn version_byte_is_one() {
            assert_eq!(ribocipher::VERSION_1, 0x01);
        }
    }
}
