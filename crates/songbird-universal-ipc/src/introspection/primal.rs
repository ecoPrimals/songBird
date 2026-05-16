// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `primal.info` and `primal.capabilities` payloads.

use serde_json::Value;
use songbird_types::primal_names;

use super::capability_tokens::SONGBIRD_CAPABILITY_STRINGS;

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

/// Generate `primal.announce` payload (biomeOS v3.57 atomic registration).
///
/// Replaces separate `lifecycle.register` + `capability.register` + `method.register`
/// with a single atomic announcement that includes identity, capabilities, methods,
/// and signal-tier membership.
#[must_use]
pub fn primal_announce() -> Value {
    serde_json::json!({
        "primal": primal_names::SELF_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "domain": "network",
        "license": "AGPL-3.0-or-later",
        "provided_capabilities": SONGBIRD_CAPABILITY_STRINGS,
        "consumed_capabilities": [
            "security",
            "crypto"
        ],
        "signal_tiers": ["tower"],
        "endpoints": {
            "transports": ["unix_socket", "tcp"],
            "protocols": ["json-rpc", "ndjson", "btsp"]
        },
        "methods": super::capability_tokens::callable_methods_list(),
        "status": "ready"
    })
}
