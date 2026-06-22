// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Identity payload and family ID resolution from environment.

use serde_json::Value;
use songbird_types::primal_names;

/// Resolve canonical `BirdSong` / biomeOS `family_id` from environment keys.
///
/// Priority: `SONGBIRD_ORCHESTRATOR_FAMILY_ID` → `BIOMEOS_FAMILY_ID` →
/// `SONGBIRD_FAMILY_ID` → `FAMILY_ID` → `NODE_FAMILY_ID`, then `"default"`.
#[must_use]
pub fn canonical_family_id(env: impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env("SONGBIRD_ORCHESTRATOR_FAMILY_ID")
        .or_else(|_| env("BIOMEOS_FAMILY_ID"))
        .or_else(|_| env("SONGBIRD_FAMILY_ID"))
        .or_else(|_| env("FAMILY_ID"))
        .or_else(|_| env("NODE_FAMILY_ID"))
        .unwrap_or_else(|_| String::from("default"))
}

/// Generate legacy identity response (backward-compat `identity` method).
#[must_use]
pub fn identity(family_id: &str) -> Value {
    serde_json::json!({
        "primal": primal_names::SELF_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "family_id": family_id,
        "capabilities": [
            "ipc.register", "ipc.resolve", "ipc.discover", "ipc.list",
            "http.request", "http.get", "http.post",
            "secure_http",
            "stun.get_public_address", "stun.bind",
            "stun.probe_port_pattern", "stun.detect_nat_type",
            "igd.discover", "igd.map_port", "igd.auto_configure",
            "birdsong.generate_encrypted_beacon", "birdsong.decrypt_beacon",
            "birdsong.verify_lineage", "birdsong.get_lineage",
            "birdsong.advertise", "birdsong.schema",
            "relay.serve", "relay.status", "relay.allocate", "relay.forward",
            "mesh.status", "mesh.find_path", "mesh.peers",
            "mesh.topology", "mesh.auto_discover",
            "punch.request", "punch.coordinate", "punch.status",
            "onion.start", "onion.connect", "onion.address",
            "tor.connect", "tor.circuit.build",
            "discovery.peers",
            "rendezvous.register", "rendezvous.lookup",
            "peer.connect",
            "discover_capabilities"
        ]
    })
}

/// Wire Standard Level 2 `identity.get` response.
///
/// Returns `{primal, version, domain, license}` per Capability Wire Standard v1.0.
#[must_use]
pub fn identity_get() -> Value {
    serde_json::json!({
        "primal": primal_names::SELF_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "domain": "network",
        "license": "AGPL-3.0-or-later"
    })
}
