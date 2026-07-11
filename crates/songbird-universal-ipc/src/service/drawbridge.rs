// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Drawbridge HTTP listener — the single crossing point between Gatehouse and Darkforest.
//!
//! Accepts plain HTTP connections (from bearDog after TLS termination) and routes
//! them to capability backends via [`CapabilityProxyRouter`]. This is the native
//! HTTP bridge that translates external web traffic into capability-routed requests.
//!
//! The drawbridge runs as a standalone async task, listening on a configurable
//! address (`SONGBIRD_DRAWBRIDGE_ADDR`, default `127.0.0.1:7780`) or Unix socket
//! (`SONGBIRD_DRAWBRIDGE_SOCKET`).
//!
//! Route mapping: path prefix → capability (from `SONGBIRD_DRAWBRIDGE_ROUTES`).
//! Example: `/hub=jupyter,/api=inference` means `/hub/login` → jupyter backend.
//!
//! ## Auth Gate (EXP-06)
//!
//! The drawbridge enforces authentication before proxying:
//! - `SONGBIRD_DRAWBRIDGE_AUTH_TOKENS`: comma-separated bearer tokens (required for auth)
//! - `SONGBIRD_DRAWBRIDGE_PUBLIC_PATHS`: comma-separated paths that skip auth
//! - `SONGBIRD_DRAWBRIDGE_TRUSTED_PEERS`: comma-separated CIDR ranges that bypass auth
//!
//! If no auth tokens are configured, the drawbridge operates in open mode (backward compat).

use super::http_proxy::{CapabilityProxyRouter, ProxyRoute};
use std::fmt::Write;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tracing::{debug, info, warn};

/// Route mapping: path prefix → capability name.
#[derive(Debug, Clone)]
pub struct DrawbridgeRoute {
    pub path_prefix: String,
    pub capability: String,
    /// If true, this route bypasses auth enforcement regardless of global auth config.
    pub public: bool,
}

/// CIDR-style network range for trusted peer matching.
#[derive(Debug, Clone)]
pub struct TrustedNetwork {
    addr: IpAddr,
    prefix_len: u8,
}

impl TrustedNetwork {
    /// Parse a CIDR notation string (e.g., "192.168.0.0/16", "127.0.0.1/8").
    fn parse(cidr: &str) -> Option<Self> {
        let (addr_str, prefix_str) = cidr.split_once('/')?;
        let addr: IpAddr = addr_str.trim().parse().ok()?;
        let prefix_len: u8 = prefix_str.trim().parse().ok()?;
        Some(Self { addr, prefix_len })
    }

    /// Check if an IP address falls within this network range.
    fn contains(&self, ip: IpAddr) -> bool {
        match (self.addr, ip) {
            (IpAddr::V4(net), IpAddr::V4(target)) => {
                if self.prefix_len == 0 {
                    return true;
                }
                let mask = u32::MAX.checked_shl(32 - u32::from(self.prefix_len)).unwrap_or(0);
                (u32::from(net) & mask) == (u32::from(target) & mask)
            }
            (IpAddr::V6(net), IpAddr::V6(target)) => {
                if self.prefix_len == 0 {
                    return true;
                }
                let mask = u128::MAX.checked_shl(128 - u32::from(self.prefix_len)).unwrap_or(0);
                (u128::from(net) & mask) == (u128::from(target) & mask)
            }
            _ => false,
        }
    }
}

/// Authentication gate for the drawbridge.
///
/// Enforces bearer token auth with configurable bypass for trusted peers and public paths.
#[derive(Debug, Clone)]
pub struct AuthGate {
    tokens: Vec<String>,
    public_paths: Vec<String>,
    trusted_peers: Vec<TrustedNetwork>,
}

impl AuthGate {
    /// Load auth configuration from environment variables.
    ///
    /// - `SONGBIRD_DRAWBRIDGE_AUTH_TOKENS`: comma-separated valid bearer tokens
    /// - `SONGBIRD_DRAWBRIDGE_PUBLIC_PATHS`: comma-separated path prefixes that skip auth
    /// - `SONGBIRD_DRAWBRIDGE_TRUSTED_PEERS`: comma-separated CIDR ranges that bypass auth
    #[must_use]
    pub fn from_env() -> Self {
        let tokens: Vec<String> = songbird_process_env::var("SONGBIRD_DRAWBRIDGE_AUTH_TOKENS")
            .unwrap_or_default()
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        let public_paths: Vec<String> =
            songbird_process_env::var("SONGBIRD_DRAWBRIDGE_PUBLIC_PATHS")
                .unwrap_or_default()
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();

        let trusted_peers: Vec<TrustedNetwork> =
            songbird_process_env::var("SONGBIRD_DRAWBRIDGE_TRUSTED_PEERS")
                .unwrap_or_default()
                .split(',')
                .filter_map(|c| TrustedNetwork::parse(c.trim()))
                .collect();

        Self { tokens, public_paths, trusted_peers }
    }

    /// Returns true if auth enforcement is active (tokens configured).
    #[must_use]
    pub fn is_enforcing(&self) -> bool {
        !self.tokens.is_empty()
    }

    /// Check whether a request is authorized.
    ///
    /// A request passes if ANY of:
    /// 1. No tokens configured (open mode — backward compat)
    /// 2. Path matches a public prefix
    /// 3. Peer IP is in a trusted network
    /// 4. Authorization header contains a valid bearer token
    /// 5. `_sb_token` query parameter contains a valid token
    #[must_use]
    pub fn is_authorized(&self, peer: IpAddr, path: &str, auth_header: Option<&str>) -> bool {
        if self.tokens.is_empty() {
            return true;
        }

        if self.public_paths.iter().any(|prefix| path.starts_with(prefix.as_str())) {
            return true;
        }

        if self.trusted_peers.iter().any(|net| net.contains(peer)) {
            return true;
        }

        if let Some(header) = auth_header {
            let token = header.strip_prefix("Bearer ").unwrap_or(header);
            if self.tokens.iter().any(|t| t == token) {
                return true;
            }
        }

        // Check query parameter fallback (for browser-based access)
        if let Some(query_start) = path.find('?') {
            let query = &path[query_start + 1..];
            for param in query.split('&') {
                if let Some(value) = param.strip_prefix("_sb_token=") && self.tokens.iter().any(|t| t == value) {
                    return true;
                }
            }
        }

        false
    }
}

/// Drawbridge HTTP listener configuration.
#[derive(Debug, Clone)]
pub struct DrawbridgeConfig {
    pub bind_addr: String,
    pub routes: Vec<DrawbridgeRoute>,
    pub auth: AuthGate,
}

impl DrawbridgeConfig {
    /// Load from environment variables.
    ///
    /// - `SONGBIRD_DRAWBRIDGE_ADDR`: bind address (default `127.0.0.1:7780`)
    /// - `SONGBIRD_DRAWBRIDGE_ROUTES`: comma-separated `path=capability` pairs
    ///   Suffix capability with `!public` to skip auth: `/health=status!public`
    ///   (default: empty — falls back to first registered capability)
    #[must_use]
    pub fn from_env() -> Self {
        let bind_addr = songbird_process_env::var("SONGBIRD_DRAWBRIDGE_ADDR")
            .unwrap_or_else(|_| String::from("127.0.0.1:7780"));

        let routes = songbird_process_env::var("SONGBIRD_DRAWBRIDGE_ROUTES")
            .unwrap_or_default()
            .split(',')
            .filter_map(|entry| {
                let entry = entry.trim();
                let (path, cap_raw) = entry.split_once('=')?;
                let path = path.trim();
                let cap_raw = cap_raw.trim();
                if path.is_empty() || cap_raw.is_empty() {
                    return None;
                }
                let (cap, public) = if let Some(c) = cap_raw.strip_suffix("!public") {
                    (c, true)
                } else {
                    (cap_raw, false)
                };
                Some(DrawbridgeRoute {
                    path_prefix: path.to_string(),
                    capability: cap.to_string(),
                    public,
                })
            })
            .collect();

        Self { bind_addr, routes, auth: AuthGate::from_env() }
    }

    fn resolve_route(&self, path: &str) -> Option<&DrawbridgeRoute> {
        self.routes.iter().find(|route| path.starts_with(&route.path_prefix))
    }
}

/// Start the drawbridge HTTP listener.
///
/// Accepts plain HTTP connections and proxies them to capability backends.
/// This is the sole crossing point for external HTTP traffic into the darkforest.
///
/// # Errors
///
/// Returns an error if the TCP listener cannot bind.
pub async fn serve_drawbridge(
    config: DrawbridgeConfig,
    router: Arc<CapabilityProxyRouter>,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(&config.bind_addr).await?;
    info!(
        addr = %config.bind_addr,
        routes = ?config.routes.iter().map(|r| format!("{} → {}", r.path_prefix, r.capability)).collect::<Vec<_>>(),
        auth_enforcing = config.auth.is_enforcing(),
        "drawbridge HTTP listener active (Gatehouse→Darkforest crossing)"
    );

    let config = Arc::new(config);

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                warn!(error = %e, "drawbridge accept error");
                continue;
            }
        };

        let config = Arc::clone(&config);
        let router = Arc::clone(&router);

        tokio::spawn(async move {
            if let Err(e) = handle_drawbridge_connection(stream, peer, &config, &router).await {
                debug!(peer = %peer, error = %e, "drawbridge connection error");
            }
        });
    }
}

async fn handle_drawbridge_connection(
    stream: tokio::net::TcpStream,
    peer: SocketAddr,
    config: &DrawbridgeConfig,
    router: &CapabilityProxyRouter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut reader = BufReader::new(stream);

    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        let stream = reader.into_inner();
        let mut stream = stream;
        stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n").await?;
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];
    let _ = parts[2];

    let mut headers: Vec<(String, String)> = Vec::new();
    let mut host = String::new();
    let mut content_length: usize = 0;

    let mut header_line = String::new();
    loop {
        header_line.clear();
        reader.read_line(&mut header_line).await?;
        if header_line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = header_line.split_once(':') {
            let name_lower = name.trim().to_lowercase();
            let value = value.trim().to_string();
            if name_lower == "host" {
                host.clone_from(&value);
            } else if name_lower == "content-length" {
                content_length = value.parse().unwrap_or(0);
            }
            headers.push((name.trim().to_string(), value));
        }
    }

    let body = if content_length > 0 {
        let mut buf = vec![0u8; content_length];
        tokio::io::AsyncReadExt::read_exact(&mut reader, &mut buf).await?;
        Some(String::from_utf8_lossy(&buf).to_string())
    } else {
        None
    };

    // Auth gate enforcement — resolve route first to check per-route public flag
    let matched_route = config.resolve_route(path);
    let route_is_public = matched_route.is_some_and(|r| r.public);

    let auth_header = headers
        .iter()
        .find(|(n, _)| n.to_lowercase() == "authorization")
        .map(|(_, v)| v.as_str());

    if !route_is_public && !config.auth.is_authorized(peer.ip(), path, auth_header) {
        debug!(peer = %peer, path, "drawbridge: auth denied");
        let stream = reader.into_inner();
        let mut stream = stream;
        stream.write_all(
            b"HTTP/1.1 401 Unauthorized\r\n\
              WWW-Authenticate: Bearer realm=\"drawbridge\"\r\n\
              Content-Length: 0\r\n\r\n",
        ).await?;
        return Ok(());
    }

    let capability = matched_route.map(|r| r.capability.as_str());

    let route = if let Some(cap) = capability {
        router.route(cap)
    } else {
        let caps = router.list_capabilities();
        if caps.len() == 1 {
            router.route(&caps[0])
        } else {
            None
        }
    };

    let Some(route) = route else {
        debug!(peer = %peer, path, "drawbridge: no capability route for path");
        let stream = reader.into_inner();
        let mut stream = stream;
        stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n").await?;
        return Ok(());
    };

    let backend_url = build_backend_url(&route, path, &config.routes);
    debug!(
        peer = %peer,
        host = %host,
        path,
        backend = %backend_url,
        "drawbridge: routing to capability backend"
    );

    proxy_to_backend(reader.into_inner(), method, &backend_url, &headers, body.as_deref()).await
}

fn build_backend_url(route: &ProxyRoute, request_path: &str, routes: &[DrawbridgeRoute]) -> String {
    let matched_prefix = routes
        .iter()
        .find(|r| request_path.starts_with(&r.path_prefix))
        .map_or("", |r| r.path_prefix.as_str());

    let suffix = if matched_prefix.is_empty() {
        request_path
    } else {
        &request_path[matched_prefix.len()..]
    };

    let base = route.base_url.trim_end_matches('/');
    if suffix.is_empty() || suffix == "/" {
        format!("{base}/")
    } else {
        format!("{base}/{}", suffix.trim_start_matches('/'))
    }
}

async fn proxy_to_backend(
    mut client_stream: tokio::net::TcpStream,
    method: &str,
    backend_url: &str,
    headers: &[(String, String)],
    body: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let stripped = backend_url
        .strip_prefix("http://")
        .unwrap_or(backend_url);

    let (authority, path_and_query) = match stripped.find('/') {
        Some(i) => (&stripped[..i], &stripped[i..]),
        None => (stripped, "/"),
    };

    let addr = if authority.contains(':') {
        authority.to_string()
    } else {
        format!("{authority}:80")
    };

    let mut backend = match tokio::net::TcpStream::connect(&addr).await {
        Ok(s) => s,
        Err(e) => {
            warn!(backend = %addr, error = %e, "drawbridge: backend connect failed");
            client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n").await?;
            return Ok(());
        }
    };

    let mut request_buf = format!("{method} {path_and_query} HTTP/1.1\r\nHost: {authority}\r\n");

    for (name, value) in headers {
        let name_lower = name.to_lowercase();
        if name_lower == "host" {
            continue;
        }
        let _ = write!(request_buf, "{name}: {value}\r\n");
    }

    if let Some(b) = body {
        let _ = write!(request_buf, "Content-Length: {}\r\n", b.len());
    }
    request_buf.push_str("Connection: close\r\n\r\n");

    if let Some(b) = body {
        request_buf.push_str(b);
    }

    backend.write_all(request_buf.as_bytes()).await?;

    let (mut backend_read, mut _backend_write) = backend.into_split();
    let (mut _client_read, mut client_write) = client_stream.into_split();

    tokio::io::copy(&mut backend_read, &mut client_write).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_env_defaults() {
        songbird_process_env::remove_var("SONGBIRD_DRAWBRIDGE_ADDR");
        songbird_process_env::remove_var("SONGBIRD_DRAWBRIDGE_ROUTES");
        songbird_process_env::remove_var("SONGBIRD_DRAWBRIDGE_AUTH_TOKENS");
        songbird_process_env::remove_var("SONGBIRD_DRAWBRIDGE_PUBLIC_PATHS");
        songbird_process_env::remove_var("SONGBIRD_DRAWBRIDGE_TRUSTED_PEERS");
        let config = DrawbridgeConfig::from_env();
        assert_eq!(config.bind_addr, "127.0.0.1:7780");
        assert!(config.routes.is_empty());
        assert!(!config.auth.is_enforcing());
    }

    #[test]
    fn config_parses_routes() {
        let config = DrawbridgeConfig {
            bind_addr: String::from("127.0.0.1:7780"),
            routes: vec![
                DrawbridgeRoute { path_prefix: String::from("/hub"), capability: String::from("jupyter"), public: false },
                DrawbridgeRoute { path_prefix: String::from("/api"), capability: String::from("inference"), public: false },
            ],
            auth: AuthGate { tokens: vec![], public_paths: vec![], trusted_peers: vec![] },
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
                DrawbridgeRoute { path_prefix: String::from("/hub"), capability: String::from("jupyter"), public: false },
                DrawbridgeRoute { path_prefix: String::from("/api"), capability: String::from("inference"), public: false },
            ],
            auth: AuthGate { tokens: vec![], public_paths: vec![], trusted_peers: vec![] },
        };
        assert_eq!(config.resolve_route("/hub/login").map(|r| r.capability.as_str()), Some("jupyter"));
        assert_eq!(config.resolve_route("/api/v1/models").map(|r| r.capability.as_str()), Some("inference"));
        assert!(config.resolve_route("/unknown").is_none());
    }

    #[test]
    fn build_backend_url_strips_prefix() {
        let route = ProxyRoute {
            base_url: String::from("http://192.168.4.237:8000"),
            default_headers: std::collections::HashMap::new(),
            api_key_env: None,
            timeout_ms: 30_000,
        };
        let routes = vec![
            DrawbridgeRoute { path_prefix: String::from("/hub"), capability: String::from("jupyter"), public: false },
        ];
        assert_eq!(
            build_backend_url(&route, "/hub/login", &routes),
            "http://192.168.4.237:8000/login"
        );
        assert_eq!(
            build_backend_url(&route, "/hub/api/status", &routes),
            "http://192.168.4.237:8000/api/status"
        );
    }

    // ── Auth Gate Tests ──────────────────────────────────────────────────

    #[test]
    fn auth_gate_open_mode_allows_everything() {
        let gate = AuthGate { tokens: vec![], public_paths: vec![], trusted_peers: vec![] };
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
                DrawbridgeRoute { path_prefix: String::from("/health"), capability: String::from("status"), public: true },
                DrawbridgeRoute { path_prefix: String::from("/hub"), capability: String::from("jupyter"), public: false },
            ],
            auth: AuthGate {
                tokens: vec![String::from("secret")],
                public_paths: vec![],
                trusted_peers: vec![],
            },
        };
        let health_route = config.resolve_route("/health/check").unwrap();
        assert!(health_route.public);
        let hub_route = config.resolve_route("/hub/login").unwrap();
        assert!(!hub_route.public);
    }
}
