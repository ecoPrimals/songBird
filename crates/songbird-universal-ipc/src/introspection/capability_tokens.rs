// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Flat capability token list for NEST / inter-primal discovery.

use serde_json::Value;

/// Canonical capability tokens for [`capabilities_list`] (NEST / inter-primal discovery).
///
/// Kept as a single source of truth for `capabilities.list` JSON-RPC and gateways.
pub const SONGBIRD_CAPABILITY_STRINGS: &[&str] = &[
    "network.discovery",
    "network.federation",
    "network.relay",
    "network.stun",
    "network.igd",
    "network.quic",
    "network.tls",
    "network.tor",
    "network.onion",
    "ipc.jsonrpc",
    "ipc.tarpc",
    "crypto.delegate",
    "nfc.genesis",
    "bluetooth.pair",
];

/// Flat capability string list for `capabilities.list` (JSON array result).
#[must_use]
pub fn capabilities_list() -> Value {
    serde_json::Value::Array(
        SONGBIRD_CAPABILITY_STRINGS
            .iter()
            .map(|s| serde_json::Value::String((*s).to_string()))
            .collect(),
    )
}
