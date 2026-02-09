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

/// Generate primal info (self-knowledge only)
pub fn primal_info() -> Value {
    serde_json::json!({
        "name": "songbird",
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
                "operations": ["get_public_address", "bind", "serve", "stop", "status"],
                "description": "NAT traversal via STUN",
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
                "provider": "beardog"
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
                "operations": ["request", "status"],
                "description": "UDP hole punching for direct P2P connections"
            },
            {
                "name": "onion",
                "operations": ["start", "stop", "status", "connect", "address"],
                "description": "Sovereign .onion service for NAT traversal",
                "encryption": "x25519_chacha20poly1305",
                "provider": "beardog"
            },
            {
                "name": "tor",
                "operations": ["status", "connect", "service.start", "service.stop",
                    "consensus.fetch", "circuit.build", "circuit.close"],
                "description": "Pure Rust Tor protocol for symmetric NAT traversal",
                "implementation": "from_scratch",
                "provider": "beardog"
            }
        ]
    })
}

/// Generate JSON-RPC method listing
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
        ]
    })
}

/// Generate biomeOS standard rpc.discover response
pub fn rpc_discover_standard() -> Value {
    serde_json::json!({
        "methods": [
            "health", "identity", "rpc.discover",
            "discover_capabilities",
            "primal.info", "primal.capabilities", "rpc.methods",
            "ipc.register", "ipc.resolve", "ipc.discover", "ipc.list",
            "http.request", "http.get", "http.post",
            "stun.get_public_address", "stun.bind",
            "stun.serve", "stun.stop", "stun.status",
            "igd.discover", "igd.map_port", "igd.unmap_port",
            "igd.status", "igd.external_ip", "igd.auto_configure",
            "relay.serve", "relay.stop", "relay.status", "relay.allocate",
            "birdsong.generate_encrypted_beacon", "birdsong.decrypt_beacon",
            "birdsong.verify_lineage", "birdsong.get_lineage", "birdsong.advertise",
            "mesh.init", "mesh.status", "mesh.find_path",
            "mesh.announce", "mesh.peers", "mesh.health_check",
            "mesh.auto_discover",
            "punch.request", "punch.status",
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

/// Generate discover_capabilities response (biomeOS cross-primal scanner protocol)
///
/// This is the response format that capability scanners (e.g., Squirrel)
/// expect when probing sockets. It enables zero-configuration discovery:
/// instead of setting `HTTP_REQUEST_PROVIDER_SOCKET`, primals simply
/// scan available sockets and ask each one what capabilities it provides.
pub fn discover_capabilities() -> Value {
    serde_json::json!({
        "primal": "songbird",
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
            "mesh.status",
            "mesh.find_path",
            "mesh.peers",
            "punch.request",
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
            "birdsong.verify_lineage"
        ]
    })
}

/// Generate health response
pub fn health(uptime_secs: u64, service_count: usize) -> Value {
    serde_json::json!({
        "status": "healthy",
        "primal": "songbird",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime_secs,
        "services": service_count,
    })
}

/// Generate identity response
pub fn identity(family_id: &str) -> Value {
    serde_json::json!({
        "primal": "songbird",
        "version": env!("CARGO_PKG_VERSION"),
        "family_id": family_id,
        "capabilities": [
            "ipc.register", "ipc.resolve", "ipc.discover", "ipc.list",
            "http.request", "http.get", "http.post",
            "secure_http",
            "stun.get_public_address", "stun.bind",
            "igd.discover", "igd.map_port", "igd.auto_configure",
            "birdsong.generate_encrypted_beacon", "birdsong.decrypt_beacon",
            "birdsong.verify_lineage", "birdsong.get_lineage",
            "birdsong.advertise",
            "relay.serve", "relay.status", "relay.allocate",
            "mesh.status", "mesh.find_path", "mesh.peers",
            "mesh.auto_discover",
            "punch.request", "punch.status",
            "onion.start", "onion.connect", "onion.address",
            "tor.connect", "tor.circuit.build",
            "discovery.peers",
            "rendezvous.register", "rendezvous.lookup",
            "peer.connect",
            "discover_capabilities"
        ]
    })
}

