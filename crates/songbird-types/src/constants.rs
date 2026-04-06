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

/// Default HTTP port for Songbird services
pub const DEFAULT_HTTP_PORT: u16 = 8080;

/// Default HTTPS port for Songbird services
pub const DEFAULT_HTTPS_PORT: u16 = 8443;

/// Default port for Songbird services (alias for HTTP)
pub const DEFAULT_PORT: u16 = DEFAULT_HTTP_PORT;

/// Default discovery service port
pub const DEFAULT_DISCOVERY_PORT: u16 = 8081;

/// Default federation service port
pub const DEFAULT_FEDERATION_PORT: u16 = 8082;

/// Default health check port
pub const DEFAULT_HEALTH_PORT: u16 = 8002;

/// Default dashboard port
pub const DEFAULT_DASHBOARD_PORT: u16 = 8003;

/// Default metrics port
pub const DEFAULT_METRICS_PORT: u16 = 8004;

/// Default orchestrator API port (self-referencing for internal metrics)
pub const DEFAULT_ORCHESTRATOR_PORT: u16 = 8000;

/// Default crypto provider transport port (security provider TCP transport fallback)
pub const DEFAULT_CRYPTO_TRANSPORT_PORT: u16 = 9876;

/// Default security vault port (capability-based security provider, dev only)
pub const DEFAULT_SECURITY_VAULT_PORT: u16 = 8200;

/// Default federation bind port
pub const DEFAULT_FEDERATION_BIND_PORT: u16 = 7000;

/// Localhost IPv4 address
pub const LOCALHOST: &str = "127.0.0.1";

/// Builds a `http://localhost:{port}` development fallback URL.
///
/// Only available in debug builds — production code must resolve endpoints
/// through environment variables or capability-based discovery.
#[cfg(debug_assertions)]
#[must_use]
pub fn dev_localhost_url(port: u16) -> String {
    format!("http://{LOCALHOST}:{port}")
}

/// Production bind address (all interfaces)
pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";

/// Development bind address (localhost only)
pub const DEVELOPMENT_BIND_ADDRESS: &str = "127.0.0.1";

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
// SYSTEM CONSTANTS
// ============================================================================

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
// BACKWARD COMPATIBILITY EXPORTS
// ============================================================================

/// Backward compatibility module for legacy code
pub mod legacy {
    pub use super::*;

    // Additional legacy aliases if needed
    /// Legacy alias for the development bind address (`127.0.0.1`).
    pub const DEFAULT_BIND_ADDRESS: &str = super::DEVELOPMENT_BIND_ADDRESS;
    /// Legacy alias for loopback (`127.0.0.1`).
    pub const DEFAULT_LOCALHOST: &str = super::LOCALHOST;
}

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
        // Value checks would be optimized out by compiler for const values
    }

    #[test]
    fn test_default_retry_attempts() {
        assert_eq!(DEFAULT_RETRY_ATTEMPTS, 3);
        // Value checks would be optimized out by compiler for const values
    }

    #[test]
    fn test_default_port() {
        assert_eq!(DEFAULT_PORT, 8080);
        // Value checks would be optimized out by compiler for const values
    }

    #[test]
    fn test_max_concurrent_connections() {
        assert_eq!(MAX_CONCURRENT_CONNECTIONS, 1000);
        // Value checks would be optimized out by compiler for const values
    }

    #[test]
    fn test_constants_are_reasonable() {
        // All constant values are validated at compile time
        assert_eq!(DEFAULT_TIMEOUT_SECS, 30);
        assert_eq!(DEFAULT_RETRY_ATTEMPTS, 3);
        assert_eq!(DEFAULT_PORT, 8080);
        assert_eq!(MAX_CONCURRENT_CONNECTIONS, 1000);
    }
}
