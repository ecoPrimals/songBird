// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration for hole punch attempts and STUN server resolution.

use std::sync::LazyLock;
use std::time::Duration;

/// Public STUN servers for discovery, from `SONGBIRD_STUN_SERVERS`, `BIOMEOS_STUN_SERVERS`, or built-in defaults.
///
/// Parsed once per process; empty or whitespace-only env values use the defaults.
/// `SONGBIRD_STUN_SERVERS` takes precedence when both comma lists are set.
fn stun_server_list() -> Vec<String> {
    static SERVERS: LazyLock<Vec<String>> = LazyLock::new(|| {
        songbird_process_env::var("SONGBIRD_STUN_SERVERS")
            .or_else(|_| songbird_process_env::var("BIOMEOS_STUN_SERVERS"))
            .map_or_else(
                |_| default_stun_servers_fallback(),
                |servers| {
                    let parsed: Vec<String> = servers
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(std::string::ToString::to_string)
                        .collect();
                    if parsed.is_empty() {
                        default_stun_servers_fallback()
                    } else {
                        parsed
                    }
                },
            )
    });
    SERVERS.clone()
}

pub fn default_stun_servers_fallback() -> Vec<String> {
    let port = songbird_types::defaults::ports::DEFAULT_STUN_PORT;
    vec![
        format!("stun.nextcloud.com:{port}"),
        format!("stun.cloudflare.com:{port}"),
        format!("stun.sip.us:{port}"),
    ]
}

/// Configuration for hole punch attempts
///
/// ## STUN Server Configuration
///
/// STUN servers are resolved in this order:
/// 1. Explicitly configured via `stun_servers` field
/// 2. Environment variable `SONGBIRD_STUN_SERVERS` or `BIOMEOS_STUN_SERVERS` (comma-separated; Songbird name preferred)
/// 3. Self-hosted via `BIOMEOS_STUN_SERVER` environment variable
/// 4. Public STUN servers (default fallback)
///
/// For sovereign operation, configure self-hosted STUN:
/// ```bash
/// export BIOMEOS_STUN_SERVER="my-stun.local:3478"
/// ```
#[derive(Debug, Clone)]
pub struct HolePunchConfig {
    /// Number of simultaneous punch attempts
    pub max_attempts: u32,
    /// Timeout for each attempt
    pub attempt_timeout: Duration,
    /// Delay between punch packets
    pub packet_interval: Duration,
    /// Total timeout for punch coordination
    pub total_timeout: Duration,
    /// STUN servers to use for address discovery (resolved from env or defaults)
    pub stun_servers: Vec<String>,
    /// Timeout waiting for punch ack from peer
    pub ack_timeout: Duration,
}

impl Default for HolePunchConfig {
    fn default() -> Self {
        Self {
            max_attempts: 20,
            attempt_timeout: Duration::from_millis(500),
            packet_interval: Duration::from_millis(50),
            total_timeout: Duration::from_secs(10),
            stun_servers: Self::resolve_stun_servers(),
            ack_timeout: Duration::from_secs(5),
        }
    }
}

impl HolePunchConfig {
    /// Overrides the resolved STUN server list used for discovery and punching.
    #[must_use]
    pub fn with_stun_servers(mut self, servers: Vec<String>) -> Self {
        self.stun_servers = servers;
        self
    }

    /// Resolve STUN servers from environment or defaults
    ///
    /// Resolution order:
    /// 1. `BIOMEOS_STUN_SERVER` (single self-hosted)
    /// 2. `SONGBIRD_STUN_SERVERS` or `BIOMEOS_STUN_SERVERS` (comma-separated; Songbird name preferred)
    /// 3. Default public servers
    fn resolve_stun_servers() -> Vec<String> {
        let mut servers = Vec::new();

        // 1. Self-hosted first (highest priority, maximum sovereignty)
        if let Ok(self_hosted) = songbird_process_env::var("BIOMEOS_STUN_SERVER") {
            servers.push(self_hosted);
        }

        // 2. Custom servers from env (`SONGBIRD_STUN_SERVERS` preferred over legacy `BIOMEOS_STUN_SERVERS`)
        if let Ok(custom) = songbird_process_env::var("SONGBIRD_STUN_SERVERS")
            .or_else(|_| songbird_process_env::var("BIOMEOS_STUN_SERVERS"))
        {
            servers
                .extend(custom.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
        }

        // 3. Public fallback (only if no custom servers)
        if servers.is_empty() && songbird_process_env::var("BIOMEOS_NO_PUBLIC_STUN").is_err() {
            servers.extend(Self::default_public_stun_servers());
        }

        servers
    }

    /// Default public STUN servers (fallback only)
    fn default_public_stun_servers() -> Vec<String> {
        stun_server_list()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn default_stun_servers_fallback_returns_three_servers() {
        let servers = default_stun_servers_fallback();
        assert_eq!(servers.len(), 3);
        assert!(servers.iter().all(|s| s.contains(":3478")));
    }

    #[test]
    fn hole_punch_config_default_values() {
        songbird_process_env::remove_var("SONGBIRD_STUN_SERVERS");
        songbird_process_env::remove_var("BIOMEOS_STUN_SERVERS");
        songbird_process_env::remove_var("BIOMEOS_STUN_SERVER");

        let config = HolePunchConfig::default();
        assert_eq!(config.max_attempts, 20);
        assert_eq!(config.attempt_timeout, Duration::from_millis(500));
        assert_eq!(config.packet_interval, Duration::from_millis(50));
        assert_eq!(config.total_timeout, Duration::from_secs(10));
        assert_eq!(config.ack_timeout, Duration::from_secs(5));
        assert!(!config.stun_servers.is_empty());
    }

    #[test]
    fn with_stun_servers_overrides_defaults() {
        let config =
            HolePunchConfig::default().with_stun_servers(vec!["my-stun.local:3478".to_string()]);
        assert_eq!(config.stun_servers.len(), 1);
        assert_eq!(config.stun_servers[0], "my-stun.local:3478");
    }

    #[test]
    fn hole_punch_config_clone_preserves_fields() {
        let original = HolePunchConfig {
            max_attempts: 5,
            attempt_timeout: Duration::from_millis(100),
            packet_interval: Duration::from_millis(25),
            total_timeout: Duration::from_secs(3),
            stun_servers: vec!["test:1234".to_string()],
            ack_timeout: Duration::from_secs(2),
        };
        let cloned = original.clone();
        assert_eq!(original.max_attempts, cloned.max_attempts);
        assert_eq!(cloned.max_attempts, 5);
        assert_eq!(cloned.stun_servers, vec!["test:1234"]);
        assert_eq!(cloned.ack_timeout, Duration::from_secs(2));
    }

    #[test]
    fn hole_punch_config_debug_output() {
        let config = HolePunchConfig::default();
        let dbg = format!("{config:?}");
        assert!(dbg.contains("max_attempts"));
        assert!(dbg.contains("stun_servers"));
    }
}
