// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! JSON-RPC method listings, biomeOS discovery, and method-name normalization.

use serde_json::Value;
use songbird_types::primal_names;

/// Normalize a JSON-RPC method name for legacy tolerance.
#[must_use]
pub fn normalize_method(method: &str) -> &str {
    songbird_types::normalize_json_rpc_method_name(method)
}

/// Generate JSON-RPC method listing
#[must_use]
pub fn rpc_methods() -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "methods": [
            {"name": "discover_capabilities", "description": "Cross-primal discovery: list capabilities this primal provides", "params": []},
            {"name": "primal.info", "description": "Get primal metadata and capabilities", "params": []},
            {"name": "primal.capabilities", "description": "Get detailed capability descriptions", "params": []},
            {"name": "rpc.methods", "description": "List all available JSON-RPC methods", "params": []},
            {"name": "ipc.register", "description": "Register a primal in the IPC registry", "params": ["primal_id", "capabilities", "endpoint"]},
            {"name": "ipc.resolve", "description": "Resolve by primal ID or capability", "params": ["primal_id?", "capability?"]},
            {"name": "ipc.discover", "description": "Discover primals by capability", "params": ["capability"]},
            {"name": "ipc.list", "description": "List all registered primals", "params": []},
            {"name": "http.request", "description": "Full HTTP/HTTPS request", "params": ["method", "url", "headers?", "body?"]},
            {"name": "http.get", "description": "HTTP GET request", "params": ["url", "headers?"]},
            {"name": "http.post", "description": "HTTP POST request", "params": ["url", "body", "headers?"]},
            {"name": "stun.get_public_address", "description": "Get public IP and port via STUN", "params": ["stun_server?"]},
            {"name": "stun.bind", "description": "Bind to port and get mapping", "params": ["local_port?", "stun_server?"]},
            {"name": "stun.serve", "description": "Start STUN server", "params": ["bind_addr?"]},
            {"name": "stun.stop", "description": "Stop STUN server", "params": []},
            {"name": "stun.status", "description": "Get STUN server status", "params": []},
            {"name": "stun.probe_port_pattern", "description": "Probe NAT port allocation pattern for coordinated punch", "params": ["stun_server", "probes?"]},
            {"name": "stun.detect_nat_type", "description": "Detect NAT type (full-cone, symmetric, etc.)", "params": ["stun_server?"]},
            {"name": "igd.discover", "description": "Discover router IGD capabilities", "params": []},
            {"name": "igd.map_port", "description": "Request port forwarding", "params": ["external_port?", "internal_port?", "protocol?", "ttl?"]},
            {"name": "igd.unmap_port", "description": "Remove port forwarding", "params": ["external_port", "protocol?"]},
            {"name": "igd.status", "description": "Query all current mappings", "params": []},
            {"name": "igd.external_ip", "description": "Get external IP from router", "params": []},
            {"name": "igd.auto_configure", "description": "All-in-one setup + verify", "params": ["port?", "protocol?"]},
            {"name": "discovery.peers", "description": "Discover peers on local network", "params": []},
            {"name": "rendezvous.register", "description": "Register with rendezvous server", "params": ["server_url", "peer_id", "connection_info"]},
            {"name": "rendezvous.lookup", "description": "Lookup peer on rendezvous server", "params": ["server_url", "peer_id"]},
            {"name": "peer.connect", "description": "Connect to peer directly", "params": ["peer_address", "peer_port"]},
            {"name": "birdsong.generate_encrypted_beacon", "description": "Generate family-encrypted discovery beacon", "params": ["node_id", "capabilities"]},
            {"name": "birdsong.decrypt_beacon", "description": "Decrypt received beacon (family gate)", "params": ["encrypted_beacon"]},
            {"name": "birdsong.verify_lineage", "description": "Verify peer lineage via challenge-response", "params": ["peer_node_id", "our_node_id"]},
            {"name": "birdsong.get_lineage", "description": "Get own lineage info", "params": []},
            {"name": "birdsong.advertise", "description": "Generate beacon with onion endpoint", "params": ["node_id", "capabilities"]},
            {"name": "birdsong.schema", "description": "Introspect beacon request schema (fields, types, required/optional)", "params": []},
            {"name": "relay.serve", "description": "Start relay server", "params": ["bind_addr?"]},
            {"name": "relay.stop", "description": "Stop relay server", "params": []},
            {"name": "relay.status", "description": "Get relay server status", "params": []},
            {"name": "relay.allocate", "description": "Allocate relay session", "params": ["target_node_id"]},
            {"name": "mesh.init", "description": "Initialize mesh network", "params": []},
            {"name": "mesh.status", "description": "Get mesh status", "params": []},
            {"name": "mesh.find_path", "description": "Find path to peer via mesh", "params": ["target_node_id"]},
            {"name": "mesh.announce", "description": "Announce presence on mesh", "params": []},
            {"name": "mesh.peers", "description": "List mesh peers", "params": []},
            {"name": "mesh.topology", "description": "Get full mesh network topology graph", "params": []},
            {"name": "mesh.health_check", "description": "Check mesh health", "params": []},
            {"name": "mesh.auto_discover", "description": "Auto-discover mesh peers", "params": []},
            {"name": "punch.request", "description": "Request UDP hole punch to peer", "params": ["peer_address", "peer_port"]},
            {"name": "punch.coordinate", "description": "Relay-assisted coordinated punch for symmetric NATs", "params": ["target_node_id", "relay_session_id", "our_port_pattern", "peer_port_pattern"]},
            {"name": "punch.status", "description": "Get hole punch status", "params": []},
            {"name": "onion.start", "description": "Start sovereign .onion service", "params": []},
            {"name": "onion.stop", "description": "Stop .onion service", "params": []},
            {"name": "onion.status", "description": "Get .onion service status", "params": []},
            {"name": "onion.connect", "description": "Connect via .onion address", "params": ["onion_address"]},
            {"name": "onion.address", "description": "Get .onion address", "params": []},
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
            "mesh.announce", "mesh.peers", "mesh.topology",
            "mesh.health_check", "mesh.auto_discover",
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
            "mesh.topology",
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
