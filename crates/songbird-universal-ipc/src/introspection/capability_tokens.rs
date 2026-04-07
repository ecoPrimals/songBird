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

/// Mapping from NEST capability tokens to their primary callable JSON-RPC methods.
///
/// Returned by `capabilities.methods` so callers know which method to invoke
/// for each advertised capability.
pub const CAPABILITY_METHOD_MAP: &[(&str, &[&str])] = &[
    ("network.discovery", &["discovery.peers", "discovery.announce", "discovery.list_peers"]),
    ("network.federation", &["songbird.federation.peers", "songbird.federation.status"]),
    ("network.relay", &["relay.serve", "relay.stop", "relay.status", "relay.allocate"]),
    (
        "network.stun",
        &["stun.serve", "stun.stop", "stun.status", "stun.get_public_address", "stun.bind"],
    ),
    (
        "network.igd",
        &["igd.discover", "igd.map_port", "igd.unmap_port", "igd.status", "igd.external_ip"],
    ),
    ("network.quic", &["health.readiness"]),
    ("network.tls", &["http.request", "http.get", "http.post"]),
    ("network.tor", &["tor.status", "tor.connect", "tor.circuit.build"]),
    (
        "network.onion",
        &["onion.start", "onion.stop", "onion.status", "onion.connect", "onion.address"],
    ),
    (
        "ipc.jsonrpc",
        &["rpc.methods", "rpc.discover", "ipc.register", "ipc.resolve", "ipc.discover"],
    ),
    ("ipc.tarpc", &["rpc.methods"]),
    ("crypto.delegate", &["health.readiness"]),
    ("nfc.genesis", &["health.readiness"]),
    ("bluetooth.pair", &["health.readiness"]),
];

/// JSON object mapping capability tokens to their callable methods (`capabilities.methods`).
#[must_use]
pub fn capabilities_methods() -> Value {
    let map: serde_json::Map<String, Value> = CAPABILITY_METHOD_MAP
        .iter()
        .map(|(token, methods)| {
            (
                (*token).to_string(),
                Value::Array(methods.iter().map(|m| Value::String((*m).to_string())).collect()),
            )
        })
        .collect();
    Value::Object(map)
}
