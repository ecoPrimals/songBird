// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! CLI arguments for the TURN relay server (`songbird relay` subcommand).

use clap::Parser;

/// TURN relay server arguments (RFC 5766 sovereign VPS relay).
///
/// Start a standalone relay server that Songbird clients can allocate
/// through for NAT traversal when direct and STUN-assisted connectivity
/// fail.
///
/// Credentials are loaded from `--credentials-file` (one `username:hex_key`
/// per line) or from `SONGBIRD_RELAY_CREDENTIALS` (same format, newline-separated).
#[derive(Parser, Debug)]
pub struct RelayArgs {
    /// Bind address for the relay UDP listener.
    #[arg(long, default_value = "0.0.0.0", env = "SONGBIRD_RELAY_BIND")]
    pub bind: String,

    /// UDP port for the relay listener.
    #[arg(long, default_value_t = 3478, env = "SONGBIRD_RELAY_PORT")]
    pub port: u16,

    /// Path to credentials file (one `username:hex_key` per line).
    /// If not provided, checks `SONGBIRD_RELAY_CREDENTIALS` env var.
    /// If neither is set, starts with an empty credential store (testing only).
    #[arg(long, env = "SONGBIRD_RELAY_CREDENTIALS_FILE")]
    pub credentials_file: Option<String>,
}
