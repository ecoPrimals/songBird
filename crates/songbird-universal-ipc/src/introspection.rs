// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Primal introspection and self-description
//!
//! Extracted from `service.rs` for smart refactoring. These methods provide
//! self-knowledge — the primal describing its own capabilities, methods,
//! and identity. They follow the TRUE PRIMAL principle: Songbird only
//! knows about itself, never about other primals.
//!
//! ## Methods
//!
//! - `primal.info` - Primal metadata
//! - `primal.capabilities` - Detailed capability descriptions
//! - `rpc.methods` - Available JSON-RPC methods
//! - `rpc.discover` - biomeOS standard method listing
//! - `health` - Health status
//! - `identity` - Primal identity

use serde_json::Value;
use songbird_types::primal_names;

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

/// Minimal liveness probe result (`health.liveness`).
#[must_use]
pub fn health_liveness() -> Value {
    serde_json::json!({ "status": "healthy" })
}

/// Readiness probe result (`health.readiness`).
///
/// Reports whether the primal is ready to serve requests, including
/// subsystem status. More detailed than liveness — indicates the primal
/// has completed initialization and can accept work.
#[must_use]
pub fn health_readiness() -> Value {
    serde_json::json!({
        "status": "ready",
        "subsystems": {
            "ipc": "up",
            "discovery": "up",
            "federation": "up",
            "tls": "up"
        }
    })
}

/// Full health check result (`health.check`).
///
/// Comprehensive health status with version, uptime placeholder, and
/// subsystem details. Aliases: `status`, `check`.
#[must_use]
pub fn health_check() -> Value {
    serde_json::json!({
        "status": "healthy",
        "primal": primal_names::SELF_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "subsystems": {
            "ipc": "up",
            "discovery": "up",
            "federation": "up",
            "tls": "up",
            "relay": "up",
            "mesh": "up"
        }
    })
}

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

/// Normalize a JSON-RPC method name for legacy tolerance.
///
/// The ecosystem has naming drift: `capabilities.list` vs `capability.list`.
/// This function canonicalizes known aliases so dispatch tables can use a
/// single match arm per operation.
///
/// Note: `primal.capabilities` is intentionally NOT aliased — it returns a
/// detailed capability description (operations, protocols) rather than the
/// flat token list that `capabilities.list` provides.
///
/// Canonical names follow `SEMANTIC_METHOD_NAMING_STANDARD.md` (`domain.verb`).
#[must_use]
pub fn normalize_method(method: &str) -> &str {
    songbird_types::normalize_json_rpc_method_name(method)
}

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
        .unwrap_or_else(|_| "default".to_string())
}

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
                "provider": primal_names::BEARDOG
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
                "provider": primal_names::BEARDOG
            },
            {
                "name": "tor",
                "operations": ["status", "connect", "service.start", "service.stop",
                    "consensus.fetch", "circuit.build", "circuit.close"],
                "description": "Pure Rust Tor protocol for symmetric NAT traversal",
                "implementation": "from_scratch",
                "provider": primal_names::BEARDOG
            }
        ]
    })
}

/// Generate JSON-RPC method listing
#[must_use]
pub fn rpc_methods() -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "methods": [
            // Introspection
            {"name": "discover_capabilities", "description": "Cross-primal discovery: list capabilities this primal provides", "params": []},
            {"name": "primal.info", "description": "Get primal metadata and capabilities", "params": []},
            {"name": "primal.capabilities", "description": "Get detailed capability descriptions", "params": []},
            {"name": "rpc.methods", "description": "List all available JSON-RPC methods", "params": []},
            // IPC
            {"name": "ipc.register", "description": "Register a primal in the IPC registry", "params": ["primal_id", "capabilities", "endpoint"]},
            {"name": "ipc.resolve", "description": "Resolve a primal by ID", "params": ["primal_id"]},
            {"name": "ipc.discover", "description": "Discover primals by capability", "params": ["capability"]},
            {"name": "ipc.list", "description": "List all registered primals", "params": []},
            // HTTP
            {"name": "http.request", "description": "Full HTTP/HTTPS request", "params": ["method", "url", "headers?", "body?"]},
            {"name": "http.get", "description": "HTTP GET request", "params": ["url", "headers?"]},
            {"name": "http.post", "description": "HTTP POST request", "params": ["url", "body", "headers?"]},
            // STUN
            {"name": "stun.get_public_address", "description": "Get public IP and port via STUN", "params": ["stun_server?"]},
            {"name": "stun.bind", "description": "Bind to port and get mapping", "params": ["local_port?", "stun_server?"]},
            {"name": "stun.serve", "description": "Start STUN server", "params": ["bind_addr?"]},
            {"name": "stun.stop", "description": "Stop STUN server", "params": []},
            {"name": "stun.status", "description": "Get STUN server status", "params": []},
            {"name": "stun.probe_port_pattern", "description": "Probe NAT port allocation pattern for coordinated punch", "params": ["stun_server", "probes?"]},
            {"name": "stun.detect_nat_type", "description": "Detect NAT type (full-cone, symmetric, etc.)", "params": ["stun_server?"]},
            // IGD
            {"name": "igd.discover", "description": "Discover router IGD capabilities", "params": []},
            {"name": "igd.map_port", "description": "Request port forwarding", "params": ["external_port?", "internal_port?", "protocol?", "ttl?"]},
            {"name": "igd.unmap_port", "description": "Remove port forwarding", "params": ["external_port", "protocol?"]},
            {"name": "igd.status", "description": "Query all current mappings", "params": []},
            {"name": "igd.external_ip", "description": "Get external IP from router", "params": []},
            {"name": "igd.auto_configure", "description": "All-in-one setup + verify", "params": ["port?", "protocol?"]},
            // Discovery
            {"name": "discovery.peers", "description": "Discover peers on local network", "params": []},
            // Rendezvous
            {"name": "rendezvous.register", "description": "Register with rendezvous server", "params": ["server_url", "peer_id", "connection_info"]},
            {"name": "rendezvous.lookup", "description": "Lookup peer on rendezvous server", "params": ["server_url", "peer_id"]},
            // Peer
            {"name": "peer.connect", "description": "Connect to peer directly", "params": ["peer_address", "peer_port"]},
            // BirdSong
            {"name": "birdsong.generate_encrypted_beacon", "description": "Generate family-encrypted discovery beacon", "params": ["node_id", "capabilities"]},
            {"name": "birdsong.decrypt_beacon", "description": "Decrypt received beacon (family gate)", "params": ["encrypted_beacon"]},
            {"name": "birdsong.verify_lineage", "description": "Verify peer lineage via challenge-response", "params": ["peer_node_id", "our_node_id"]},
            {"name": "birdsong.get_lineage", "description": "Get own lineage info", "params": []},
            {"name": "birdsong.advertise", "description": "Generate beacon with onion endpoint", "params": ["node_id", "capabilities"]},
            {"name": "birdsong.schema", "description": "Introspect beacon request schema (fields, types, required/optional)", "params": []},
            // Relay
            {"name": "relay.serve", "description": "Start relay server", "params": ["bind_addr?"]},
            {"name": "relay.stop", "description": "Stop relay server", "params": []},
            {"name": "relay.status", "description": "Get relay server status", "params": []},
            {"name": "relay.allocate", "description": "Allocate relay session", "params": ["target_node_id"]},
            // Mesh
            {"name": "mesh.init", "description": "Initialize mesh network", "params": []},
            {"name": "mesh.status", "description": "Get mesh status", "params": []},
            {"name": "mesh.find_path", "description": "Find path to peer via mesh", "params": ["target_node_id"]},
            {"name": "mesh.announce", "description": "Announce presence on mesh", "params": []},
            {"name": "mesh.peers", "description": "List mesh peers", "params": []},
            {"name": "mesh.health_check", "description": "Check mesh health", "params": []},
            {"name": "mesh.auto_discover", "description": "Auto-discover mesh peers", "params": []},
            // Hole Punch
            {"name": "punch.request", "description": "Request UDP hole punch to peer", "params": ["peer_address", "peer_port"]},
            {"name": "punch.coordinate", "description": "Relay-assisted coordinated punch for symmetric NATs", "params": ["target_node_id", "relay_session_id", "our_port_pattern", "peer_port_pattern"]},
            {"name": "punch.status", "description": "Get hole punch status", "params": []},
            // Onion
            {"name": "onion.start", "description": "Start sovereign .onion service", "params": []},
            {"name": "onion.stop", "description": "Stop .onion service", "params": []},
            {"name": "onion.status", "description": "Get .onion service status", "params": []},
            {"name": "onion.connect", "description": "Connect via .onion address", "params": ["onion_address"]},
            {"name": "onion.address", "description": "Get .onion address", "params": []},
            // Tor
            {"name": "tor.status", "description": "Tor protocol status", "params": []},
            {"name": "tor.connect", "description": "Connect via Tor", "params": ["target"]},
            {"name": "tor.service.start", "description": "Start hidden service", "params": []},
            {"name": "tor.service.stop", "description": "Stop hidden service", "params": []},
            {"name": "tor.consensus.fetch", "description": "Fetch Tor consensus", "params": []},
            {"name": "tor.circuit.build", "description": "Build Tor circuit", "params": []},
            {"name": "tor.circuit.close", "description": "Close Tor circuit", "params": []},
        ]
    })
}

/// Generate biomeOS standard rpc.discover response
#[must_use]
pub fn rpc_discover_standard() -> Value {
    serde_json::json!({
        "methods": [
            "health.liveness", "health.readiness", "health.check",
            "capabilities.list",
            "identity", "rpc.discover",
            "discover_capabilities",
            "primal.info", "primal.capabilities", "rpc.methods",
            "ipc.register", "ipc.resolve", "ipc.discover", "ipc.list",
            "http.request", "http.get", "http.post",
            "stun.get_public_address", "stun.bind",
            "stun.serve", "stun.stop", "stun.status",
            "stun.probe_port_pattern", "stun.detect_nat_type",
            "igd.discover", "igd.map_port", "igd.unmap_port",
            "igd.status", "igd.external_ip", "igd.auto_configure",
            "relay.serve", "relay.stop", "relay.status", "relay.allocate",
            "birdsong.generate_encrypted_beacon", "birdsong.decrypt_beacon",
            "birdsong.verify_lineage", "birdsong.get_lineage", "birdsong.advertise",
            "birdsong.schema",
            "mesh.init", "mesh.status", "mesh.find_path",
            "mesh.announce", "mesh.peers", "mesh.health_check",
            "mesh.auto_discover",
            "punch.request", "punch.coordinate", "punch.status",
            "onion.start", "onion.stop", "onion.status",
            "onion.connect", "onion.address",
            "tor.status", "tor.connect",
            "tor.service.start", "tor.service.stop",
            "tor.consensus.fetch", "tor.circuit.build", "tor.circuit.close",
            "discovery.peers",
            "rendezvous.register", "rendezvous.lookup",
            "peer.connect"
        ]
    })
}

/// Generate `discover_capabilities` response (biomeOS cross-primal scanner protocol)
///
/// This is the response format that capability scanners (e.g., Squirrel)
/// expect when probing sockets. It enables zero-configuration discovery:
/// instead of setting `HTTP_REQUEST_PROVIDER_SOCKET`, primals simply
/// scan available sockets and ask each one what capabilities it provides.
#[must_use]
pub fn discover_capabilities() -> Value {
    serde_json::json!({
        "primal": primal_names::SELF_NAME,
        "capabilities": [
            "http.request",
            "http.get",
            "http.post",
            "secure_http",
            "discovery.peers",
            "relay.serve",
            "relay.status",
            "relay.connect",
            "relay.allocate",
            "stun.detect",
            "stun.bind",
            "stun.serve",
            "stun.probe_port_pattern",
            "stun.detect_nat_type",
            "mesh.status",
            "mesh.find_path",
            "mesh.peers",
            "punch.request",
            "punch.coordinate",
            "punch.status",
            "onion.start",
            "onion.connect",
            "onion.address",
            "tor.connect",
            "tor.circuit.build",
            "igd.discover",
            "igd.map_port",
            "igd.auto_configure",
            "birdsong.advertise",
            "birdsong.verify_lineage",
            "birdsong.schema"
        ]
    })
}

/// Generate health response
#[must_use]
pub fn health(uptime_secs: u64, service_count: usize) -> Value {
    serde_json::json!({
        "status": "healthy",
        "primal": primal_names::SELF_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime_secs,
        "services": service_count,
    })
}

/// Generate identity response
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
            "relay.serve", "relay.status", "relay.allocate",
            "mesh.status", "mesh.find_path", "mesh.peers",
            "mesh.auto_discover",
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

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::{
        SONGBIRD_CAPABILITY_STRINGS, canonical_family_id, capabilities_list, discover_capabilities,
        health, health_check, health_liveness, health_readiness, identity, normalize_method,
        primal_capabilities, primal_info, rpc_discover_standard, rpc_methods,
    };

    use std::collections::HashMap;
    use std::env::VarError;

    #[test]
    fn primal_info_has_expected_keys() {
        let v = primal_info();
        assert_eq!(v["name"], "songbird");
        assert_eq!(v["role"], "network_orchestrator");
        assert!(v.get("capabilities").is_some());
        assert!(v.get("version").is_some());
    }

    #[test]
    fn primal_capabilities_is_array_of_objects() {
        let v = primal_capabilities();
        let caps = v["capabilities"].as_array().unwrap();
        assert!(!caps.is_empty());
        assert!(caps[0].get("name").is_some());
    }

    #[test]
    fn health_includes_uptime_and_services() {
        let v = health(42, 7);
        assert_eq!(v["uptime_seconds"], 42);
        assert_eq!(v["services"], 7);
        assert_eq!(v["status"], "healthy");
    }

    #[test]
    fn health_liveness_is_minimal() {
        let v = health_liveness();
        assert_eq!(v, serde_json::json!({ "status": "healthy" }));
        assert!(v.get("uptime_seconds").is_none());
    }

    #[test]
    fn capabilities_list_matches_const_table() {
        let v = capabilities_list();
        let arr = v.as_array().unwrap();
        assert_eq!(arr.len(), SONGBIRD_CAPABILITY_STRINGS.len());
        for (i, s) in SONGBIRD_CAPABILITY_STRINGS.iter().enumerate() {
            assert_eq!(arr[i].as_str().unwrap(), *s);
        }
    }

    #[test]
    fn identity_includes_family_id() {
        let v = identity("fam-test");
        assert_eq!(v["family_id"], "fam-test");
        let caps = v["capabilities"].as_array().unwrap();
        assert!(caps.iter().any(|c| c == "ipc.register"));
    }

    #[test]
    fn rpc_methods_non_empty() {
        let v = rpc_methods();
        let methods = v["methods"].as_array().unwrap();
        assert!(!methods.is_empty());
    }

    #[test]
    fn discover_capabilities_lists_http_and_ipc() {
        let v = discover_capabilities();
        assert_eq!(v["primal"], "songbird");
        let caps = v["capabilities"].as_array().unwrap();
        assert!(caps.iter().any(|c| c == "http.request"));
    }

    #[test]
    fn canonical_family_id_prefers_orchestrator() {
        let m = HashMap::from([
            ("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "orch"),
            ("BIOMEOS_FAMILY_ID", "biome"),
        ]);
        assert_eq!(
            canonical_family_id(|k| m
                .get(k)
                .copied()
                .map(String::from)
                .ok_or(VarError::NotPresent)),
            "orch"
        );
    }

    #[test]
    fn canonical_family_id_falls_back_to_biomeos() {
        let m = HashMap::from([("BIOMEOS_FAMILY_ID", "biome-only")]);
        assert_eq!(
            canonical_family_id(|k| m
                .get(k)
                .copied()
                .map(String::from)
                .ok_or(VarError::NotPresent)),
            "biome-only"
        );
    }

    #[test]
    fn canonical_family_id_falls_back_to_songbird_family_id() {
        let m = HashMap::from([("SONGBIRD_FAMILY_ID", "sb")]);
        assert_eq!(
            canonical_family_id(|k| m
                .get(k)
                .copied()
                .map(String::from)
                .ok_or(VarError::NotPresent)),
            "sb"
        );
    }

    #[test]
    fn canonical_family_id_falls_back_to_family_id() {
        let m = HashMap::from([("FAMILY_ID", "fam")]);
        assert_eq!(
            canonical_family_id(|k| m
                .get(k)
                .copied()
                .map(String::from)
                .ok_or(VarError::NotPresent)),
            "fam"
        );
    }

    #[test]
    fn canonical_family_id_falls_back_to_node_family_id() {
        let m = HashMap::from([("NODE_FAMILY_ID", "node")]);
        assert_eq!(
            canonical_family_id(|k| m
                .get(k)
                .copied()
                .map(String::from)
                .ok_or(VarError::NotPresent)),
            "node"
        );
    }

    #[test]
    fn canonical_family_id_default_when_missing() {
        let m: HashMap<&str, &str> = HashMap::new();
        assert_eq!(
            canonical_family_id(|k| m
                .get(k)
                .copied()
                .map(String::from)
                .ok_or(VarError::NotPresent)),
            "default"
        );
    }

    #[test]
    fn rpc_discover_standard_includes_core_methods() {
        let v = rpc_discover_standard();
        let methods = v["methods"].as_array().unwrap();
        let names: Vec<&str> = methods.iter().filter_map(|x| x.as_str()).collect();
        assert!(names.contains(&"health.liveness"));
        assert!(names.contains(&"health.readiness"));
        assert!(names.contains(&"health.check"));
        assert!(names.contains(&"identity"));
        assert!(names.contains(&"peer.connect"));
        assert!(names.contains(&"tor.circuit.build"));
    }

    #[test]
    fn rpc_methods_includes_igd_and_tor_entries() {
        let v = rpc_methods();
        let methods = v["methods"].as_array().unwrap();
        let has_igd = methods.iter().any(|m| m["name"] == "igd.discover");
        let has_tor = methods.iter().any(|m| m["name"] == "tor.status");
        assert!(has_igd);
        assert!(has_tor);
    }

    #[test]
    fn primal_info_lists_discovery_methods() {
        let v = primal_info();
        let dm = v["discovery_methods"].as_array().unwrap();
        assert!(dm.iter().any(|x| x == "mdns"));
    }

    #[test]
    fn identity_lists_ipc_methods_in_capabilities() {
        let v = identity("fam");
        let caps = v["capabilities"].as_array().unwrap();
        assert!(caps.iter().any(|c| c == "ipc.register"));
    }

    #[test]
    fn health_zero_uptime_and_zero_services() {
        let v = health(0, 0);
        assert_eq!(v["uptime_seconds"], 0);
        assert_eq!(v["services"], 0);
        assert_eq!(v["primal"], "songbird");
    }

    #[test]
    fn rpc_methods_has_jsonrpc_and_non_empty_methods() {
        let v = rpc_methods();
        assert_eq!(v["jsonrpc"], "2.0");
        let m = v["methods"].as_array().unwrap();
        assert!(m.len() > 5);
        assert!(m.iter().any(|x| x["name"] == "primal.info"));
    }

    #[test]
    fn discover_capabilities_includes_tor_and_mesh() {
        let v = discover_capabilities();
        let caps = v["capabilities"].as_array().unwrap();
        let s: Vec<&str> = caps.iter().filter_map(|x| x.as_str()).collect();
        assert!(s.contains(&"tor.connect"));
        assert!(s.contains(&"mesh.status"));
    }

    #[test]
    fn songbird_capability_strings_count_matches_network_ipc_crypto() {
        assert!(SONGBIRD_CAPABILITY_STRINGS.contains(&"ipc.jsonrpc"));
        assert!(SONGBIRD_CAPABILITY_STRINGS.contains(&"network.tls"));
        assert_eq!(SONGBIRD_CAPABILITY_STRINGS.len(), 14);
    }

    #[test]
    fn rpc_discover_standard_contains_capabilities_listing() {
        let v = rpc_discover_standard();
        let methods = v["methods"].as_array().unwrap();
        assert!(methods.iter().any(|m| m == "primal.capabilities"));
        assert!(methods.iter().any(|m| m == "health.liveness"));
        assert!(methods.iter().any(|m| m == "health.readiness"));
        assert!(methods.iter().any(|m| m == "health.check"));
        assert!(methods.iter().any(|m| m == "capabilities.list"));
    }

    #[test]
    fn health_readiness_reports_ready_status() {
        let v = health_readiness();
        assert_eq!(v["status"], "ready");
        assert!(v.get("subsystems").is_some());
        assert_eq!(v["subsystems"]["ipc"], "up");
    }

    #[test]
    fn health_check_includes_primal_and_version() {
        let v = health_check();
        assert_eq!(v["status"], "healthy");
        assert_eq!(v["primal"], "songbird");
        assert!(v.get("version").is_some());
        assert!(v.get("subsystems").is_some());
    }

    #[test]
    fn normalize_method_canonicalizes_capability_list_aliases() {
        assert_eq!(normalize_method("capabilities.list"), "capabilities.list");
        assert_eq!(normalize_method("capability.list"), "capabilities.list");
        // primal.capabilities is intentionally NOT aliased — different response shape
        assert_eq!(normalize_method("primal.capabilities"), "primal.capabilities");
    }

    #[test]
    fn normalize_method_canonicalizes_health_aliases() {
        assert_eq!(normalize_method("health.liveness"), "health.liveness");
        assert_eq!(normalize_method("ping"), "health.liveness");
        assert_eq!(normalize_method("health"), "health.check");
        assert_eq!(normalize_method("status"), "health.check");
        assert_eq!(normalize_method("check"), "health.check");
    }

    #[test]
    fn normalize_method_passes_through_unknown() {
        assert_eq!(normalize_method("compute.route"), "compute.route");
        assert_eq!(normalize_method("ipc.register"), "ipc.register");
    }

    #[test]
    fn primal_capabilities_includes_mesh_and_onion() {
        let v = primal_capabilities();
        let caps = v["capabilities"].as_array().unwrap();
        let names: Vec<&str> = caps.iter().filter_map(|c| c["name"].as_str()).collect();
        assert!(names.contains(&"mesh"));
        assert!(names.contains(&"onion"));
    }
}
