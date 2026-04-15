// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Network defaults: multicast groups, discovery ports, and STUN endpoints.
//!
//! Environment overrides follow the same pattern as `songbird_config::defaults::ports`
//! helpers (parse env, then fall back to a default constant).
//!
//! **Note:** `SONGBIRD_DISCOVERY_PORT` in `songbird-config` selects the HTTP discovery *service*
//! port. Multicast announcement uses `SONGBIRD_DISCOVERY_MULTICAST_PORT`.

use crate::constants::{
    BROADCAST_DISCOVERY_PORT, DEFAULT_STUN_SERVER_1, DEFAULT_STUN_SERVER_2,
    DISCOVERY_MULTICAST_GROUP, DISCOVERY_MULTICAST_PORT, MDNS_MULTICAST_GROUP, MDNS_PORT,
};

/// Default CORS origin for development when `SONGBIRD_CORS_ORIGINS` is unset.
///
/// Production deployments should always set `SONGBIRD_CORS_ORIGINS` explicitly.
pub const DEFAULT_CORS_ORIGIN: &str = "http://localhost:3000";

/// Resolve CORS origins from environment, falling back to [`DEFAULT_CORS_ORIGIN`].
///
/// `SONGBIRD_CORS_ORIGINS` accepts a comma-separated list of origins.
#[must_use]
pub fn cors_origins() -> Vec<String> {
    songbird_process_env::var("SONGBIRD_CORS_ORIGINS").map_or_else(
        |_| vec![DEFAULT_CORS_ORIGIN.to_string()],
        |v| v.split(',').map(|s| s.trim().to_string()).collect(),
    )
}

/// Ecosystem discovery multicast group (IPv4), overridable via `SONGBIRD_DISCOVERY_MULTICAST_GROUP`.
#[must_use]
pub fn discovery_multicast_group() -> String {
    songbird_process_env::var("SONGBIRD_DISCOVERY_MULTICAST_GROUP")
        .unwrap_or_else(|_| DISCOVERY_MULTICAST_GROUP.to_string())
}

/// Multicast UDP port for ecosystem discovery announcements (default [`DISCOVERY_MULTICAST_PORT`]).
///
/// # Environment variable
///
/// `SONGBIRD_DISCOVERY_MULTICAST_PORT` — not `SONGBIRD_DISCOVERY_PORT` (that is the HTTP discovery
/// service port in `songbird_config::defaults::ports::discovery_port`).
#[must_use]
pub fn discovery_port() -> u16 {
    songbird_process_env::var("SONGBIRD_DISCOVERY_MULTICAST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DISCOVERY_MULTICAST_PORT)
}

/// `host:port` string for ecosystem discovery multicast (observability / status display).
#[must_use]
pub fn ecosystem_discovery_multicast_addr() -> String {
    format!("{}:{}", discovery_multicast_group(), discovery_port())
}

/// mDNS multicast group (RFC 6762), overridable via `SONGBIRD_MDNS_MULTICAST_GROUP`.
#[must_use]
pub fn mdns_multicast_group() -> String {
    songbird_process_env::var("SONGBIRD_MDNS_MULTICAST_GROUP")
        .unwrap_or_else(|_| MDNS_MULTICAST_GROUP.to_string())
}

/// mDNS UDP port, overridable via `SONGBIRD_MDNS_PORT`.
#[must_use]
pub fn mdns_port() -> u16 {
    songbird_process_env::var("SONGBIRD_MDNS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(MDNS_PORT)
}

/// UDP port for broadcast-style peer discovery, overridable via `SONGBIRD_BROADCAST_DISCOVERY_PORT`.
#[must_use]
pub fn broadcast_discovery_port() -> u16 {
    songbird_process_env::var("SONGBIRD_BROADCAST_DISCOVERY_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(BROADCAST_DISCOVERY_PORT)
}

/// Default STUN server 1 (host:port), overridable via `SONGBIRD_STUN_SERVER_1`.
#[must_use]
pub fn stun_server_1() -> String {
    songbird_process_env::var("SONGBIRD_STUN_SERVER_1")
        .unwrap_or_else(|_| DEFAULT_STUN_SERVER_1.to_string())
}

/// Default STUN server 2 (host:port), overridable via `SONGBIRD_STUN_SERVER_2`.
#[must_use]
pub fn stun_server_2() -> String {
    songbird_process_env::var("SONGBIRD_STUN_SERVER_2")
        .unwrap_or_else(|_| DEFAULT_STUN_SERVER_2.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn cors_origins_default_is_localhost_3000() {
        songbird_process_env::remove_var("SONGBIRD_CORS_ORIGINS");
        let origins = cors_origins();
        assert_eq!(origins, vec![DEFAULT_CORS_ORIGIN]);
    }

    #[tokio::test]
    async fn cors_origins_parses_comma_list() {
        songbird_process_env::set_var("SONGBIRD_CORS_ORIGINS", "https://a.io, https://b.io");
        let origins = cors_origins();
        songbird_process_env::remove_var("SONGBIRD_CORS_ORIGINS");
        assert_eq!(origins, vec!["https://a.io", "https://b.io"]);
    }
}
