// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability tokens, callable method catalog, and Wire Standard envelope
//! for NEST / inter-primal discovery.

use serde_json::{Value, json};
use songbird_types::primal_names;

/// Canonical capability tokens (NEST / inter-primal discovery).
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

/// Every callable JSON-RPC method Songbird's dispatch accepts.
///
/// Built from [`CAPABILITY_METHOD_MAP`] plus meta/introspection methods that are
/// not part of any capability group. This is the `methods` field required by
/// the Capability Wire Standard Level 2 envelope.
const CALLABLE_METHODS: &[&str] = &[
    // ── Meta / introspection ──
    "health.liveness",
    "health.readiness",
    "health.check",
    "capabilities.list",
    "capabilities.methods",
    "identity",
    "identity.get",
    "primal.info",
    "primal.capabilities",
    "rpc.methods",
    "rpc.discover",
    "discover_capabilities",
    // ── IPC registry ──
    "ipc.register",
    "ipc.resolve",
    "ipc.discover",
    "ipc.list",
    // ── Capability resolution ──
    "capability.resolve",
    // ── Lifecycle / composition ──
    "lifecycle.composition",
    "lifecycle.validate_consumed",
    // ── Inference (canonical namespace) ──
    "inference.infer",
    "inference.status",
    "inference.list",
    "inference.load",
    // ── HTTP/HTTPS ──
    "http.request",
    "http.get",
    "http.post",
    // ── STUN / NAT traversal ──
    "stun.serve",
    "stun.stop",
    "stun.status",
    "stun.get_public_address",
    "stun.bind",
    "stun.probe_port_pattern",
    "stun.detect_nat_type",
    // ── IGD ──
    "igd.discover",
    "igd.map_port",
    "igd.unmap_port",
    "igd.status",
    "igd.external_ip",
    "igd.auto_configure",
    // ── Relay ──
    "relay.serve",
    "relay.stop",
    "relay.status",
    "relay.allocate",
    // ── Discovery / rendezvous / peers ──
    "discovery.peers",
    "discovery.announce",
    "rendezvous.register",
    "rendezvous.lookup",
    "peer.connect",
    // ── BirdSong encrypted discovery ──
    "birdsong.generate_encrypted_beacon",
    "birdsong.decrypt_beacon",
    "birdsong.verify_lineage",
    "birdsong.get_lineage",
    "birdsong.advertise",
    "birdsong.schema",
    // ── Mesh networking ──
    "mesh.init",
    "mesh.status",
    "mesh.find_path",
    "mesh.announce",
    "mesh.peers",
    "mesh.topology",
    "mesh.health_check",
    "mesh.auto_discover",
    // ── Hole punching ──
    "punch.request",
    "punch.coordinate",
    "punch.status",
    // ── Sovereign onion ──
    "onion.start",
    "onion.stop",
    "onion.status",
    "onion.connect",
    "onion.address",
    // ── Federation ──
    "federation.peers",
    "federation.status",
    // ── Tor ──
    "tor.status",
    "tor.connect",
    "tor.service.start",
    "tor.service.stop",
    "tor.consensus.fetch",
    "tor.circuit.build",
    "tor.circuit.close",
];

/// Wire Standard Level 3 envelope for `capabilities.list`.
///
/// Returns `{primal, version, methods, provided_capabilities, consumed_capabilities}`
/// per Capability Wire Standard v1.0 Level 3.
#[must_use]
pub fn capabilities_list() -> Value {
    let methods: Vec<Value> =
        CALLABLE_METHODS.iter().map(|s| Value::String((*s).to_string())).collect();

    let provided: Vec<Value> = CAPABILITY_METHOD_MAP
        .iter()
        .map(|(token, group_methods)| {
            let (domain, _) = token.split_once('.').unwrap_or((token, ""));
            json!({
                "type": domain,
                "methods": group_methods,
                "version": env!("CARGO_PKG_VERSION"),
                "description": format!("{token} capability group")
            })
        })
        .collect();

    json!({
        "primal": primal_names::SELF_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "methods": methods,
        "provided_capabilities": provided,
        "consumed_capabilities": CONSUMED_CAPABILITIES,
        "protocol": "jsonrpc-2.0",
        "transport": ["uds", "tcp"]
    })
}

/// Capabilities Songbird consumes from other primals at runtime.
///
/// These are discovered via capability-based discovery, never hardcoded endpoints.
/// Wire Standard Level 3 requires declaring these so biomeOS can validate
/// composition completeness.
pub const CONSUMED_CAPABILITIES: &[&str] = &[
    "crypto.sign",
    "crypto.encrypt_chacha20_poly1305",
    "crypto.decrypt_chacha20_poly1305",
    "crypto.generate_keypair",
    "crypto.ecdh_derive",
    "crypto.sha256",
    "crypto.hkdf_extract",
    "crypto.hkdf_expand",
    "security.tls",
    "storage.put",
    "storage.get",
    "storage.consent.store",
];

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
        &[
            "rpc.methods",
            "rpc.discover",
            "ipc.register",
            "ipc.resolve",
            "ipc.discover",
            "capability.resolve",
            "lifecycle.composition",
            "lifecycle.validate_consumed",
        ],
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
