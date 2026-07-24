// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `primal.info` and `primal.capabilities` payloads.

use serde_json::Value;
use songbird_types::primal_names;

/// Generate primal info (self-knowledge only)
#[must_use]
pub fn primal_info() -> Value {
    serde_json::json!({
        "name": primal_names::SELF_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Network Orchestration & Discovery Primal",
        "capabilities": [
            "discovery", "stun", "mdns", "http", "ipc",
            "rendezvous", "peer", "birdsong", "igd",
            "relay", "mesh", "punch", "onion", "tor"
        ],
        "role": "network_orchestrator",
        "discovery_methods": [
            "mdns", "stun", "udp_broadcast", "tcp_direct",
            "birdsong_encrypted", "ssdp", "nat_pmp"
        ],
        "endpoints": {
            "primary": "runtime_discovered",
            "protocols": ["unix_socket", "tcp"]
        },
        "security": {
            "birdsong": "genetic_lineage_encryption",
            "family_only": true
        }
    })
}

/// Generate detailed capability descriptions
#[must_use]
pub fn primal_capabilities() -> Value {
    serde_json::json!({
        "capabilities": [
            {
                "name": "discovery",
                "operations": ["peers", "mdns", "broadcast", "scan"],
                "description": "Service discovery and peer finding",
                "protocols": ["mdns", "udp_multicast"]
            },
            {
                "name": "stun",
                "operations": ["get_public_address", "bind", "serve", "stop", "status", "probe_port_pattern", "detect_nat_type"],
                "description": "NAT traversal via STUN with port pattern probing",
                "rfc": "RFC 5389"
            },
            {
                "name": "igd",
                "operations": ["discover", "map_port", "unmap_port", "status", "external_ip", "auto_configure"],
                "description": "Router port forwarding via UPnP IGD + NAT-PMP",
                "rfcs": ["RFC 6970", "RFC 6886"]
            },
            {
                "name": "http",
                "operations": ["request", "get", "post"],
                "description": "HTTP/HTTPS client with TLS 1.3",
                "features": ["redirect_following", "adaptive_user_agent", "tls_1_3"]
            },
            {
                "name": "ipc",
                "operations": ["register", "resolve", "discover", "list"],
                "description": "Inter-primal communication registry",
                "transport": "unix_socket"
            },
            {
                "name": "rendezvous",
                "operations": ["register", "lookup"],
                "description": "Rendezvous protocol for peer coordination",
                "protocol": "http_based"
            },
            {
                "name": "peer",
                "operations": ["connect"],
                "description": "Direct peer-to-peer connection establishment",
                "transport": "udp"
            },
            {
                "name": "birdsong",
                "operations": ["generate_encrypted_beacon", "decrypt_beacon", "verify_lineage", "get_lineage", "advertise"],
                "description": "Dark Forest encrypted discovery (genetic lineage, family-only)",
                "security": "genetic_lineage",
                "encryption": "chacha20_poly1305",
                "provider": "security"
            },
            {
                "name": "relay",
                "operations": ["serve", "stop", "status", "allocate"],
                "description": "Lineage-gated relay for symmetric NAT traversal",
                "authorization": "genetic_lineage"
            },
            {
                "name": "mesh",
                "operations": ["init", "status", "find_path", "announce", "peers", "health_check", "auto_discover"],
                "description": "Distributed relay mesh for cross-NAT connectivity with auto-discovery"
            },
            {
                "name": "punch",
                "operations": ["request", "coordinate", "status"],
                "description": "UDP hole punching with relay-assisted coordinated punch"
            },
            {
                "name": "onion",
                "operations": ["start", "stop", "status", "connect", "address"],
                "description": "Sovereign .onion service for NAT traversal",
                "encryption": "x25519_chacha20poly1305",
                "provider": "security"
            },
            {
                "name": "tor",
                "operations": ["status", "connect", "service.start", "service.stop",
                    "consensus.fetch", "circuit.build", "circuit.close"],
                "description": "Pure Rust Tor protocol for symmetric NAT traversal",
                "implementation": "from_scratch",
                "provider": "security"
            }
        ]
    })
}

/// Generate `primal.announce` payload (biomeOS v3.69+ Neural API wire schema).
///
/// Replaces separate `lifecycle.register` + `capability.register` + `method.register`
/// with a single atomic announcement that includes identity, capabilities, methods,
/// signal-tier membership, cost hints, and latency estimates.
///
/// `socket_path` is the full UDS path this instance is listening on
/// (e.g. `$XDG_RUNTIME_DIR/biomeos/songbird-ecoPrimal.sock`).
/// Routing-domain capabilities for Neural API weight seeding.
///
/// These align with `cost_hints` and `latency_estimates` keys so biomeOS can
/// correctly attach weights per capability domain.
const ROUTING_CAPABILITIES: &[&str] = &["relay", "communication", "presence"];

#[must_use]
pub fn primal_announce_with_socket(socket_path: &str) -> Value {
    serde_json::json!({
        "primal": primal_names::SELF_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "domain": "network",
        "license": "AGPL-3.0-or-later",
        "capabilities": ROUTING_CAPABILITIES,
        "consumed_capabilities": [
            "security",
            "crypto"
        ],
        "socket": socket_path,
        "signal_tiers": ["tower"],
        "cost_hints": {
            "relay": 15.0,
            "communication": 10.0,
            "presence": 5.0
        },
        "latency_estimates": {
            "relay": 20,
            "communication": 10,
            "presence": 5
        },
        "endpoints": {
            "transports": ["unix_socket", "tcp"],
            "protocols": ["json-rpc", "ndjson", "btsp"]
        },
        "methods": super::capability_tokens::callable_methods_list(),
        "status": "ready"
    })
}

/// Generate `primal.announce` payload with socket path resolved from environment.
///
/// Reads `XDG_RUNTIME_DIR` to construct the socket path. Falls back to
/// `{temp_dir}/biomeos/songbird.sock` if XDG is unavailable.
#[must_use]
pub fn primal_announce() -> Value {
    let socket_path = resolve_self_socket_path();
    primal_announce_with_socket(&socket_path)
}

/// Resolve this instance's UDS path from the environment.
fn resolve_self_socket_path() -> String {
    use songbird_types::primal_names::{BIOMEOS_DIR, DEFAULT_FAMILY_ID, SELF_NAME};

    let family_id = songbird_process_env::var("FAMILY_ID")
        .or_else(|_| songbird_process_env::var("BIOMEOS_FAMILY_ID"))
        .or_else(|_| songbird_process_env::var("SONGBIRD_FAMILY_ID"))
        .unwrap_or_else(|_| String::from(DEFAULT_FAMILY_ID));

    let sock_name = if family_id == "default" || family_id.is_empty() {
        format!("{SELF_NAME}.sock")
    } else {
        format!("{SELF_NAME}-{family_id}.sock")
    };

    if let Ok(xdg) = songbird_process_env::var("XDG_RUNTIME_DIR") {
        return format!("{xdg}/{BIOMEOS_DIR}/{sock_name}");
    }

    let tmp = std::env::temp_dir();
    format!("{}/{BIOMEOS_DIR}/{sock_name}", tmp.display())
}

/// `btsp.capabilities` — advertise supported BTSP transport security features.
#[must_use]
pub fn btsp_capabilities() -> Value {
    serde_json::json!({
        "protocol": "btsp",
        "version": "1.0",
        "ciphers": ["chacha20-poly1305"],
        "kdf": "hkdf-sha256",
        "handshake": "challenge-response",
        "features": [
            "encrypted-framing",
            "null-cipher-fallback",
            "bonding-policy-enforcement",
            "ndjson-wire",
            "binary-wire"
        ]
    })
}
