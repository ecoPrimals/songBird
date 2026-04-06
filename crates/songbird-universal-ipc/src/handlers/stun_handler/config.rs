// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! STUN server list resolution from environment and built-in defaults.
//!
//! Centralizes `BIOMEOS_STUN_SERVERS` parsing and the fallback list used by IPC handlers when
//! callers do not supply explicit STUN servers.

use std::sync::LazyLock;

/// Primary default STUN server (first in [`default_stun_servers_fallback`]; used when
/// `BIOMEOS_STUN_SERVERS` is unset and as a last-resort single server).
pub const DEFAULT_PRIMARY_STUN_SERVER: &str = "stun.nextcloud.com:3478";

/// Default STUN servers for IPC handlers, from `BIOMEOS_STUN_SERVERS` or built-in defaults.
///
/// Parsed once per process; empty or whitespace-only env values use the defaults.
pub fn stun_server_list() -> Vec<String> {
    static SERVERS: LazyLock<Vec<String>> = LazyLock::new(|| {
        songbird_process_env::var("BIOMEOS_STUN_SERVERS").map_or_else(
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
        DEFAULT_PRIMARY_STUN_SERVER.to_string(),
        "stun.cloudflare.com:3478".to_string(),
        "stun.sip.us:3478".to_string(),
    ]
}
