// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for the drawbridge HTTP listener — routing, auth, external proxy.

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::super::drawbridge_auth::{
    AuthGate, ExternalProxyAllowlist, ExternalService, TrustedNetwork, extract_host_from_url,
    percent_decode,
};
use super::super::http_proxy::{BackendProtocol, ProxyRoute};
use super::*;
use std::net::IpAddr;

fn empty_allowlist() -> ExternalProxyAllowlist {
    ExternalProxyAllowlist {
        services: std::collections::HashMap::new(),
    }
}

#[test]
fn config_from_env_defaults() {
    songbird_process_env::remove_var("SONGBIRD_DRAWBRIDGE_ADDR");
    songbird_process_env::remove_var("SONGBIRD_DRAWBRIDGE_ROUTES");
    songbird_process_env::remove_var("SONGBIRD_DRAWBRIDGE_AUTH_TOKENS");
    songbird_process_env::remove_var("SONGBIRD_DRAWBRIDGE_PUBLIC_PATHS");
    songbird_process_env::remove_var("SONGBIRD_DRAWBRIDGE_TRUSTED_PEERS");
    songbird_process_env::remove_var("SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST");
    let config = DrawbridgeConfig::from_env();
    assert_eq!(config.bind_addr, songbird_types::defaults::ports::DEFAULT_DRAWBRIDGE_ADDR);
    assert!(config.routes.is_empty());
    assert!(!config.auth.is_enforcing());
    assert!(!config.external_allowlist.is_active());
}

#[test]
fn config_parses_routes() {
    let config = DrawbridgeConfig {
        bind_addr: String::from("127.0.0.1:7780"),
        routes: vec![
            DrawbridgeRoute {
                path_prefix: String::from("/hub"),
                capability: String::from("jupyter"),
                public: false,
            },
            DrawbridgeRoute {
                path_prefix: String::from("/api"),
                capability: String::from("inference"),
                public: false,
            },
        ],
        auth: AuthGate {
            tokens: vec![],
            public_paths: vec![],
            trusted_peers: vec![],
        },
        external_allowlist: empty_allowlist(),
    };
    assert_eq!(config.routes.len(), 2);
    assert_eq!(config.routes[0].path_prefix, "/hub");
    assert_eq!(config.routes[0].capability, "jupyter");
    assert_eq!(config.routes[1].path_prefix, "/api");
    assert_eq!(config.routes[1].capability, "inference");
}

#[test]
fn resolve_route_matches_prefix() {
    let config = DrawbridgeConfig {
        bind_addr: String::new(),
        routes: vec![
            DrawbridgeRoute {
                path_prefix: String::from("/hub"),
                capability: String::from("jupyter"),
                public: false,
            },
            DrawbridgeRoute {
                path_prefix: String::from("/api"),
                capability: String::from("inference"),
                public: false,
            },
        ],
        auth: AuthGate {
            tokens: vec![],
            public_paths: vec![],
            trusted_peers: vec![],
        },
        external_allowlist: empty_allowlist(),
    };
    assert_eq!(config.resolve_route("/hub/login").map(|r| r.capability.as_str()), Some("jupyter"));
    assert_eq!(
        config.resolve_route("/api/v1/models").map(|r| r.capability.as_str()),
        Some("inference")
    );
    assert!(config.resolve_route("/unknown").is_none());
}

#[test]
fn build_backend_url_strips_prefix() {
    let route = ProxyRoute {
        base_url: String::from("http://192.168.4.237:8000"),
        protocol: BackendProtocol::Http,
        default_headers: std::collections::HashMap::new(),
        api_key_env: None,
        timeout_ms: 30_000,
    };
    let routes = vec![DrawbridgeRoute {
        path_prefix: String::from("/hub"),
        capability: String::from("jupyter"),
        public: false,
    }];
    assert_eq!(build_backend_url(&route, "/hub/login", &routes), "http://192.168.4.237:8000/login");
    assert_eq!(
        build_backend_url(&route, "/hub/api/status", &routes),
        "http://192.168.4.237:8000/api/status"
    );
}

#[test]
fn build_backend_url_footprint_proxy_path() {
    let route = ProxyRoute {
        base_url: String::from("http://127.0.0.1:8090"),
        protocol: BackendProtocol::Http,
        default_headers: std::collections::HashMap::new(),
        api_key_env: None,
        timeout_ms: 30_000,
    };
    let routes = vec![DrawbridgeRoute {
        path_prefix: String::from("/footprint"),
        capability: String::from("footprint"),
        public: false,
    }];
    assert_eq!(
        build_backend_url(&route, "/footprint/ext/geocode", &routes),
        "http://127.0.0.1:8090/ext/geocode"
    );
    assert_eq!(
        build_backend_url(&route, "/footprint/ext/elevation", &routes),
        "http://127.0.0.1:8090/ext/elevation"
    );
    assert_eq!(
        build_backend_url(&route, "/footprint/api/projects", &routes),
        "http://127.0.0.1:8090/api/projects"
    );
}

// ── Auth Gate Tests ──────────────────────────────────────────────────

#[test]
fn auth_gate_open_mode_allows_everything() {
    let gate = AuthGate {
        tokens: vec![],
        public_paths: vec![],
        trusted_peers: vec![],
    };
    assert!(!gate.is_enforcing());
    let peer: IpAddr = "1.2.3.4".parse().unwrap();
    assert!(gate.is_authorized(peer, "/hub/login", None));
}

#[test]
fn auth_gate_enforcing_rejects_without_token() {
    let gate = AuthGate {
        tokens: vec![String::from("secret-abc")],
        public_paths: vec![],
        trusted_peers: vec![],
    };
    assert!(gate.is_enforcing());
    let peer: IpAddr = "8.8.8.8".parse().unwrap();
    assert!(!gate.is_authorized(peer, "/hub/login", None));
}

#[test]
fn auth_gate_accepts_valid_bearer_token() {
    let gate = AuthGate {
        tokens: vec![String::from("tok-1"), String::from("tok-2")],
        public_paths: vec![],
        trusted_peers: vec![],
    };
    let peer: IpAddr = "8.8.8.8".parse().unwrap();
    assert!(gate.is_authorized(peer, "/hub/login", Some("Bearer tok-1")));
    assert!(gate.is_authorized(peer, "/hub/login", Some("Bearer tok-2")));
    assert!(!gate.is_authorized(peer, "/hub/login", Some("Bearer invalid")));
}

#[test]
fn auth_gate_accepts_raw_token_without_bearer_prefix() {
    let gate = AuthGate {
        tokens: vec![String::from("raw-token")],
        public_paths: vec![],
        trusted_peers: vec![],
    };
    let peer: IpAddr = "8.8.8.8".parse().unwrap();
    assert!(gate.is_authorized(peer, "/path", Some("raw-token")));
}

#[test]
fn auth_gate_public_path_bypasses_auth() {
    let gate = AuthGate {
        tokens: vec![String::from("secret")],
        public_paths: vec![String::from("/health"), String::from("/public/")],
        trusted_peers: vec![],
    };
    let peer: IpAddr = "8.8.8.8".parse().unwrap();
    assert!(gate.is_authorized(peer, "/health", None));
    assert!(gate.is_authorized(peer, "/health/status", None));
    assert!(gate.is_authorized(peer, "/public/page", None));
    assert!(!gate.is_authorized(peer, "/hub/login", None));
}

#[test]
fn auth_gate_trusted_peer_bypasses_auth() {
    let gate = AuthGate {
        tokens: vec![String::from("secret")],
        public_paths: vec![],
        trusted_peers: vec![
            TrustedNetwork::parse("127.0.0.0/8").unwrap(),
            TrustedNetwork::parse("192.168.0.0/16").unwrap(),
        ],
    };
    let localhost: IpAddr = "127.0.0.1".parse().unwrap();
    let lan: IpAddr = "192.168.4.5".parse().unwrap();
    let external: IpAddr = "8.8.8.8".parse().unwrap();

    assert!(gate.is_authorized(localhost, "/hub/login", None));
    assert!(gate.is_authorized(lan, "/hub/login", None));
    assert!(!gate.is_authorized(external, "/hub/login", None));
}

#[test]
fn auth_gate_query_param_token_works() {
    let gate = AuthGate {
        tokens: vec![String::from("browser-tok")],
        public_paths: vec![],
        trusted_peers: vec![],
    };
    let peer: IpAddr = "8.8.8.8".parse().unwrap();
    assert!(gate.is_authorized(peer, "/hub/login?_sb_token=browser-tok", None));
    assert!(!gate.is_authorized(peer, "/hub/login?_sb_token=wrong", None));
    assert!(gate.is_authorized(peer, "/hub?foo=bar&_sb_token=browser-tok&x=1", None));
}

#[test]
fn trusted_network_cidr_matching() {
    let net = TrustedNetwork::parse("10.13.37.0/24").unwrap();
    assert!(net.contains("10.13.37.1".parse().unwrap()));
    assert!(net.contains("10.13.37.254".parse().unwrap()));
    assert!(!net.contains("10.13.38.1".parse().unwrap()));
    assert!(!net.contains("192.168.1.1".parse().unwrap()));

    let wide = TrustedNetwork::parse("10.0.0.0/8").unwrap();
    assert!(wide.contains("10.255.255.255".parse().unwrap()));
    assert!(!wide.contains("11.0.0.1".parse().unwrap()));
}

#[test]
fn trusted_network_parse_invalid_returns_none() {
    assert!(TrustedNetwork::parse("not-a-cidr").is_none());
    assert!(TrustedNetwork::parse("192.168.1.1").is_none());
    assert!(TrustedNetwork::parse("/24").is_none());
}

#[test]
fn per_route_public_flag_parsed_from_env_format() {
    let config = DrawbridgeConfig {
        bind_addr: String::from("127.0.0.1:7780"),
        routes: vec![
            DrawbridgeRoute {
                path_prefix: String::from("/health"),
                capability: String::from("status"),
                public: true,
            },
            DrawbridgeRoute {
                path_prefix: String::from("/hub"),
                capability: String::from("jupyter"),
                public: false,
            },
        ],
        auth: AuthGate {
            tokens: vec![String::from("secret")],
            public_paths: vec![],
            trusted_peers: vec![],
        },
        external_allowlist: empty_allowlist(),
    };
    let health_route = config.resolve_route("/health/check").unwrap();
    assert!(health_route.public);
    let hub_route = config.resolve_route("/hub/login").unwrap();
    assert!(!hub_route.public);
}

// ── External Proxy Allowlist Tests ───────────────────────────────────

#[test]
fn external_allowlist_empty_is_inactive() {
    let al = empty_allowlist();
    assert!(!al.is_active());
    assert!(al.allowed_services().is_empty());
}

#[test]
fn external_allowlist_parses_services() {
    let mut services = std::collections::HashMap::new();
    services.insert(
        String::from("osm"),
        ExternalService {
            base_url: String::from("http://127.0.0.1:7781"),
            name: String::from("osm"),
        },
    );
    services.insert(
        String::from("fema"),
        ExternalService {
            base_url: String::from("http://127.0.0.1:7782"),
            name: String::from("fema"),
        },
    );
    let al = ExternalProxyAllowlist {
        services,
    };

    assert!(al.is_active());
    assert_eq!(al.allowed_services().len(), 2);
    assert!(al.resolve("osm").is_some());
    assert!(al.resolve("fema").is_some());
    assert!(al.resolve("evil").is_none());
}

#[test]
fn external_allowlist_parse_and_validate_path() {
    let mut services = std::collections::HashMap::new();
    services.insert(
        String::from("osm"),
        ExternalService {
            base_url: String::from("http://127.0.0.1:7781"),
            name: String::from("osm"),
        },
    );
    services.insert(
        String::from("usgs"),
        ExternalService {
            base_url: String::from("http://127.0.0.1:7783"),
            name: String::from("usgs"),
        },
    );
    let al = ExternalProxyAllowlist {
        services,
    };

    let (svc, remainder) = al.parse_and_validate("/osm/16/32000/21000.png").unwrap();
    assert_eq!(svc.name, "osm");
    assert_eq!(remainder, "/16/32000/21000.png");

    let (svc, remainder) = al.parse_and_validate("/usgs/epqs/pqs.php").unwrap();
    assert_eq!(svc.name, "usgs");
    assert_eq!(remainder, "/epqs/pqs.php");

    assert!(al.parse_and_validate("/evil/data").is_none());
    assert!(al.parse_and_validate("/arcgis/rest").is_none());
}

#[test]
fn external_allowlist_build_url() {
    let svc = ExternalService {
        base_url: String::from("http://127.0.0.1:7781"),
        name: String::from("osm"),
    };
    assert_eq!(
        ExternalProxyAllowlist::build_url(&svc, "/16/32000/21000.png"),
        "http://127.0.0.1:7781/16/32000/21000.png"
    );
    assert_eq!(ExternalProxyAllowlist::build_url(&svc, "/"), "http://127.0.0.1:7781/");
    assert_eq!(ExternalProxyAllowlist::build_url(&svc, ""), "http://127.0.0.1:7781/");
}

#[test]
fn external_allowlist_preserves_https_scheme() {
    let mut services = std::collections::HashMap::new();
    services.insert(
        String::from("secure"),
        ExternalService {
            base_url: String::from("https://api.example.com"),
            name: String::from("secure"),
        },
    );
    let al = ExternalProxyAllowlist {
        services,
    };
    let (svc, _) = al.parse_and_validate("/secure/path").unwrap();
    let url = ExternalProxyAllowlist::build_url(svc, "/path");
    assert!(url.starts_with("https://"));
}

#[test]
fn validate_url_matches_allowlisted_domain() {
    let mut services = std::collections::HashMap::new();
    services.insert(
        String::from("osm"),
        ExternalService {
            base_url: String::from("https://tile.openstreetmap.org"),
            name: String::from("osm"),
        },
    );
    services.insert(
        String::from("usgs"),
        ExternalService {
            base_url: String::from("https://epqs.nationalmap.gov"),
            name: String::from("usgs"),
        },
    );
    let al = ExternalProxyAllowlist {
        services,
    };

    let svc = al.validate_url("https://tile.openstreetmap.org/16/32000/21000.png");
    assert!(svc.is_some());
    assert_eq!(svc.unwrap().name, "osm");

    let svc = al.validate_url("https://epqs.nationalmap.gov/v1/json?x=-83&y=42");
    assert!(svc.is_some());
    assert_eq!(svc.unwrap().name, "usgs");

    assert!(al.validate_url("https://evil.example.com/steal").is_none());
    assert!(al.validate_url("not-a-url").is_none());
}

#[test]
fn percent_decode_works() {
    assert_eq!(
        percent_decode("https%3A%2F%2Ftile.openstreetmap.org%2F16%2F32000%2F21000.png"),
        "https://tile.openstreetmap.org/16/32000/21000.png"
    );
    assert_eq!(percent_decode("hello+world"), "hello world");
    assert_eq!(percent_decode("query%3Fx%3D1%26y%3D2"), "query?x=1&y=2");
    assert_eq!(percent_decode("plain"), "plain");
}

#[test]
fn extract_host_from_url_works() {
    assert_eq!(extract_host_from_url("https://example.com/path"), Some("example.com"));
    assert_eq!(extract_host_from_url("http://host:8080/"), Some("host"));
    assert_eq!(extract_host_from_url("ftp://bad"), None);
    assert_eq!(extract_host_from_url("not-a-url"), None);
}

#[test]
fn derive_jsonrpc_method_converts_path_to_dotted() {
    use super::super::drawbridge_auth::DrawbridgeRoute;

    let routes = vec![DrawbridgeRoute {
        path_prefix: String::from("/api"),
        capability: String::from("network"),
        public: false,
    }];

    assert_eq!(derive_jsonrpc_method("/api/mesh/status", &routes), "mesh.status");
    assert_eq!(derive_jsonrpc_method("/api/mesh/prune_stale", &routes), "mesh.prune_stale");
    assert_eq!(derive_jsonrpc_method("/api/health/ping", &routes), "health.ping");
    assert_eq!(derive_jsonrpc_method("/api/tower/health", &routes), "tower.health");
    assert_eq!(derive_jsonrpc_method("/api/tower/mesh_status", &routes), "tower.mesh_status");
    assert_eq!(derive_jsonrpc_method("/api/mesh/status?timeout=5", &routes), "mesh.status");

    let empty_routes: Vec<DrawbridgeRoute> = vec![];
    assert_eq!(derive_jsonrpc_method("/mesh/peers", &empty_routes), "mesh.peers");
}
