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
    vec![
        "stun.nextcloud.com:3478".to_string(),
        "stun.cloudflare.com:3478".to_string(),
        "stun.sip.us:3478".to_string(),
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
