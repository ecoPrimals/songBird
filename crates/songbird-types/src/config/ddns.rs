// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Dynamic DNS update configuration (H2-15).
//!
//! When a node's public IP changes (detected via STUN), the DDNS monitor
//! calls the configured provider to publish the new address so that peers
//! can reach this node by hostname rather than by raw IP.
//!
//! ## Provider Model
//!
//! [`DdnsProvider`] is an async trait abstracting the DNS update call.
//! Implementations exist for:
//!
//! - [`NoopDdnsProvider`] — used in tests and when DDNS is disabled.
//! - Production providers (e.g. RFC 2136 `nsupdate`, Cloudflare, Route 53)
//!   are wired via feature flags or runtime config — not shipped in this
//!   module to keep `songbird-types` dependency-free.
//!
//! ## Configuration
//!
//! | Env Var                     | Default          | Description                        |
//! |-----------------------------|------------------|------------------------------------|
//! | `SONGBIRD_DDNS_ENABLED`     | (unset = false)  | Enable DDNS updates on IP change   |
//! | `SONGBIRD_DDNS_PROVIDER`    | `noop`           | Provider name (`noop`, `rfc2136`, `cloudflare`, …) |
//! | `SONGBIRD_DDNS_HOSTNAME`    | —                | FQDN to update                     |
//! | `SONGBIRD_DDNS_TTL`         | `60`             | DNS record TTL in seconds          |
//! | `SONGBIRD_DDNS_ZONE`        | —                | DNS zone (for RFC 2136)            |
//! | `SONGBIRD_DDNS_SERVER`      | —                | DNS server address (for RFC 2136)  |
//! | `SONGBIRD_DDNS_KEY_NAME`    | —                | TSIG key name (for RFC 2136)       |
//! | `SONGBIRD_DDNS_KEY_SECRET`  | —                | TSIG key secret (for RFC 2136)     |

use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::IpAddr;

/// DDNS configuration (H2-15).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdnsConfig {
    /// Whether DDNS updates are enabled.
    pub enabled: bool,

    /// Provider identifier (`noop`, `rfc2136`, `cloudflare`, etc.).
    pub provider: String,

    /// Fully-qualified domain name to update.
    pub hostname: Option<String>,

    /// DNS record TTL in seconds.
    pub ttl: u32,

    /// DNS zone (provider-specific; used by RFC 2136).
    pub zone: Option<String>,

    /// DNS server address (provider-specific; used by RFC 2136).
    pub server: Option<String>,

    /// TSIG key name (provider-specific; used by RFC 2136).
    pub key_name: Option<String>,
}

impl Default for DdnsConfig {
    fn default() -> Self {
        Self {
            enabled: songbird_process_env::var("SONGBIRD_DDNS_ENABLED").is_ok(),
            provider: songbird_process_env::var("SONGBIRD_DDNS_PROVIDER")
                .unwrap_or_else(|_| "noop".to_string()),
            hostname: songbird_process_env::var("SONGBIRD_DDNS_HOSTNAME").ok(),
            ttl: songbird_process_env::var("SONGBIRD_DDNS_TTL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            zone: songbird_process_env::var("SONGBIRD_DDNS_ZONE").ok(),
            server: songbird_process_env::var("SONGBIRD_DDNS_SERVER").ok(),
            key_name: songbird_process_env::var("SONGBIRD_DDNS_KEY_NAME").ok(),
        }
    }
}

/// Result of a DDNS update attempt.
#[derive(Debug, Clone)]
pub enum DdnsUpdateResult {
    /// Record updated successfully.
    Updated {
        /// The new IP address that was published.
        new_ip: IpAddr,
    },
    /// No update needed (IP unchanged).
    Unchanged,
    /// Provider is disabled or not configured.
    Disabled,
}

/// Error from a DDNS update attempt.
#[derive(Debug, Clone)]
pub enum DdnsError {
    /// Provider returned an error.
    ProviderError(String),
    /// Configuration is incomplete (e.g. missing hostname).
    ConfigError(String),
    /// Network/IO error communicating with the DNS server.
    NetworkError(String),
}

impl fmt::Display for DdnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderError(msg) => write!(f, "DDNS provider error: {msg}"),
            Self::ConfigError(msg) => write!(f, "DDNS config error: {msg}"),
            Self::NetworkError(msg) => write!(f, "DDNS network error: {msg}"),
        }
    }
}

impl std::error::Error for DdnsError {}

/// Trait for pluggable DDNS providers (H2-15).
///
/// Implementations must be `Send + Sync` for use in async runtimes.
pub trait DdnsProvider: Send + Sync {
    /// Publish a new A/AAAA record for the configured hostname.
    ///
    /// # Errors
    ///
    /// Returns [`DdnsError`] if the update fails.
    fn update(
        &self,
        config: &DdnsConfig,
        new_ip: IpAddr,
    ) -> impl std::future::Future<Output = Result<DdnsUpdateResult, DdnsError>> + Send;
}

/// No-op provider for tests and disabled configurations.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopDdnsProvider;

impl DdnsProvider for NoopDdnsProvider {
    async fn update(
        &self,
        _config: &DdnsConfig,
        _new_ip: IpAddr,
    ) -> Result<DdnsUpdateResult, DdnsError> {
        Ok(DdnsUpdateResult::Disabled)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn default_config_disabled() {
        let config = DdnsConfig {
            enabled: false,
            provider: "noop".to_string(),
            hostname: None,
            ttl: 60,
            zone: None,
            server: None,
            key_name: None,
        };
        assert!(!config.enabled);
        assert_eq!(config.provider, "noop");
        assert_eq!(config.ttl, 60);
    }

    #[tokio::test]
    async fn noop_provider_returns_disabled() {
        let provider = NoopDdnsProvider;
        let config = DdnsConfig {
            enabled: false,
            provider: "noop".to_string(),
            hostname: None,
            ttl: 60,
            zone: None,
            server: None,
            key_name: None,
        };
        let result = provider
            .update(&config, IpAddr::V4(Ipv4Addr::LOCALHOST))
            .await
            .expect("noop should never fail");
        assert!(matches!(result, DdnsUpdateResult::Disabled));
    }

    #[test]
    fn ddns_error_display() {
        let e = DdnsError::ProviderError("timeout".into());
        assert!(e.to_string().contains("timeout"));
        let e = DdnsError::ConfigError("missing hostname".into());
        assert!(e.to_string().contains("hostname"));
        let e = DdnsError::NetworkError("connection refused".into());
        assert!(e.to_string().contains("refused"));
    }

    #[test]
    fn ddns_update_result_debug() {
        let r = DdnsUpdateResult::Updated {
            new_ip: IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
        };
        assert!(!format!("{r:?}").is_empty());
        let r = DdnsUpdateResult::Unchanged;
        assert!(!format!("{r:?}").is_empty());
    }
}
