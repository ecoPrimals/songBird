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
//! Auth and external proxy policy are in [`super::drawbridge_auth`].

use super::drawbridge_auth::{
    AuthGate, DrawbridgeRoute, EXTERNAL_PROXY_CAPABILITY, ExternalProxyAllowlist, percent_decode,
};
#[cfg(test)]
use super::drawbridge_auth::{ExternalService, TrustedNetwork, extract_host_from_url};
use super::http_proxy::{CapabilityProxyRouter, ProxyRoute};
use std::fmt::Write;
#[cfg(test)]
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tracing::{debug, info, warn};

/// Drawbridge HTTP listener configuration.
#[derive(Debug, Clone)]
pub struct DrawbridgeConfig {
    pub bind_addr: String,
    pub routes: Vec<DrawbridgeRoute>,
    pub auth: AuthGate,
    pub external_allowlist: ExternalProxyAllowlist,
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

        Self {
            bind_addr,
            routes,
            auth: AuthGate::from_env(),
            external_allowlist: ExternalProxyAllowlist::from_env(),
        }
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
        external_proxy = config.external_allowlist.is_active(),
        allowed_services = ?config.external_allowlist.allowed_services(),
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

#[expect(
    clippy::too_many_lines,
    reason = "cohesive connection handler — splitting would scatter related logic"
)]
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
            let name_trimmed = name.trim();
            let value = value.trim().to_string();
            if name_trimmed.eq_ignore_ascii_case("host") {
                host.clone_from(&value);
            } else if name_trimmed.eq_ignore_ascii_case("content-length") {
                content_length = value.parse().unwrap_or(0);
            }
            headers.push((name_trimmed.to_string(), value));
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
        .find(|(n, _)| n.eq_ignore_ascii_case("authorization"))
        .map(|(_, v)| v.as_str());

    if !route_is_public && !config.auth.is_authorized(peer.ip(), path, auth_header) {
        debug!(peer = %peer, path, "drawbridge: auth denied");
        let stream = reader.into_inner();
        let mut stream = stream;
        stream
            .write_all(
                b"HTTP/1.1 401 Unauthorized\r\n\
              WWW-Authenticate: Bearer realm=\"drawbridge\"\r\n\
              Content-Length: 0\r\n\r\n",
            )
            .await?;
        return Ok(());
    }

    let capability = matched_route.map(|r| r.capability.as_str());

    // External proxy handling — domain-validated forward proxy
    if capability == Some(EXTERNAL_PROXY_CAPABILITY) {
        let route_prefix = matched_route.map_or("", |r| r.path_prefix.as_str());
        let path_after_prefix = &path[route_prefix.len()..];

        let resolved = config
            .external_allowlist
            .parse_and_validate(path_after_prefix)
            .map(|(service, remainder)| ExternalProxyAllowlist::build_url(service, remainder));

        // Fallback: ?url=<encoded_url> query-string compatibility (Express-style proxy)
        let external_url = resolved.or_else(|| {
            let query = path_after_prefix
                .strip_prefix('?')
                .or_else(|| path_after_prefix.find('?').map(|i| &path_after_prefix[i + 1..]))?;
            let url_param = query.split('&').find_map(|p| p.strip_prefix("url="))?;
            let decoded = percent_decode(url_param);
            config.external_allowlist.validate_url(&decoded)?;
            Some(decoded)
        });

        let Some(external_url) = external_url else {
            debug!(peer = %peer, path, "drawbridge: external proxy — service not in allowlist");
            let stream = reader.into_inner();
            let mut stream = stream;
            stream
                .write_all(
                    b"HTTP/1.1 403 Forbidden\r\n\
                  Content-Type: text/plain\r\n\
                  Content-Length: 30\r\n\r\n\
                  Service not in proxy allowlist",
                )
                .await?;
            return Ok(());
        };
        debug!(
            peer = %peer,
            external_url = %external_url,
            "drawbridge: external proxy — forwarding to allowlisted service"
        );

        return proxy_to_external(
            reader.into_inner(),
            method,
            &external_url,
            &headers,
            body.as_deref(),
        )
        .await;
    }

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
    let stripped = backend_url.strip_prefix("http://").unwrap_or(backend_url);

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
            client_stream
                .write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n")
                .await?;
            return Ok(());
        }
    };

    let mut request_buf = format!("{method} {path_and_query} HTTP/1.1\r\nHost: {authority}\r\n");

    for (name, value) in headers {
        if name.eq_ignore_ascii_case("host") {
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

/// Forward a request to an external allowlisted service.
///
/// Supports both HTTP and HTTPS outbound. HTTPS uses rustls with native CA roots.
/// In production, Caddy typically handles TLS termination, but this enables
/// direct drawbridge usage in local dev/test.
async fn proxy_to_external(
    client_stream: tokio::net::TcpStream,
    method: &str,
    external_url: &str,
    headers: &[(String, String)],
    body: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if external_url.starts_with("https://") {
        proxy_to_external_tls(client_stream, method, external_url, headers, body).await
    } else {
        proxy_to_backend(client_stream, method, external_url, headers, body).await
    }
}

/// Shared TLS connector for outbound HTTPS proxy requests.
/// Initialized once on first use — avoids loading native CA roots per request.
fn outbound_tls_connector() -> &'static tokio_rustls::TlsConnector {
    use std::sync::OnceLock;
    static CONNECTOR: OnceLock<tokio_rustls::TlsConnector> = OnceLock::new();
    CONNECTOR.get_or_init(|| {
        let mut root_store = rustls::RootCertStore::empty();
        let native_certs = rustls_native_certs::load_native_certs();
        for cert in native_certs.certs {
            let _ = root_store.add(cert);
        }
        let tls_config = Arc::new(
            rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth(),
        );
        tokio_rustls::TlsConnector::from(tls_config)
    })
}

/// HTTPS outbound proxy using rustls + native CA roots.
async fn proxy_to_external_tls(
    mut client_stream: tokio::net::TcpStream,
    method: &str,
    external_url: &str,
    headers: &[(String, String)],
    body: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use rustls::pki_types::ServerName;
    use tokio::io::AsyncReadExt;

    let stripped = external_url.strip_prefix("https://").unwrap_or(external_url);

    let (authority, path_and_query) = match stripped.find('/') {
        Some(i) => (&stripped[..i], &stripped[i..]),
        None => (stripped, "/"),
    };

    let host = if let Some(colon) = authority.find(':') {
        &authority[..colon]
    } else {
        authority
    };

    let addr = if authority.contains(':') {
        authority.to_string()
    } else {
        format!("{authority}:443")
    };

    let tcp = match tokio::net::TcpStream::connect(&addr).await {
        Ok(s) => s,
        Err(e) => {
            warn!(backend = %addr, error = %e, "drawbridge: TLS backend connect failed");
            client_stream
                .write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n")
                .await?;
            return Ok(());
        }
    };

    let connector = outbound_tls_connector();
    let server_name = ServerName::try_from(host.to_owned())?;

    let mut tls_stream = match connector.connect(server_name, tcp).await {
        Ok(s) => s,
        Err(e) => {
            warn!(backend = %host, error = %e, "drawbridge: TLS handshake failed");
            client_stream
                .write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n")
                .await?;
            return Ok(());
        }
    };

    let mut request_buf = format!("{method} {path_and_query} HTTP/1.1\r\nHost: {host}\r\n");

    for (name, value) in headers {
        if name.eq_ignore_ascii_case("host") {
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

    tls_stream.write_all(request_buf.as_bytes()).await?;

    let mut response_buf = Vec::with_capacity(8192);
    tls_stream.read_to_end(&mut response_buf).await?;

    client_stream.write_all(&response_buf).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(config.bind_addr, "127.0.0.1:7780");
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
        assert_eq!(
            config.resolve_route("/hub/login").map(|r| r.capability.as_str()),
            Some("jupyter")
        );
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
            default_headers: std::collections::HashMap::new(),
            api_key_env: None,
            timeout_ms: 30_000,
        };
        let routes = vec![DrawbridgeRoute {
            path_prefix: String::from("/hub"),
            capability: String::from("jupyter"),
            public: false,
        }];
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

        // Valid service
        let (svc, remainder) = al.parse_and_validate("/osm/16/32000/21000.png").unwrap();
        assert_eq!(svc.name, "osm");
        assert_eq!(remainder, "/16/32000/21000.png");

        let (svc, remainder) = al.parse_and_validate("/usgs/epqs/pqs.php").unwrap();
        assert_eq!(svc.name, "usgs");
        assert_eq!(remainder, "/epqs/pqs.php");

        // Non-allowlisted service
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
            super::percent_decode("https%3A%2F%2Ftile.openstreetmap.org%2F16%2F32000%2F21000.png"),
            "https://tile.openstreetmap.org/16/32000/21000.png"
        );
        assert_eq!(super::percent_decode("hello+world"), "hello world");
        assert_eq!(super::percent_decode("query%3Fx%3D1%26y%3D2"), "query?x=1&y=2");
        assert_eq!(super::percent_decode("plain"), "plain");
    }

    #[test]
    fn extract_host_from_url_works() {
        assert_eq!(super::extract_host_from_url("https://example.com/path"), Some("example.com"));
        assert_eq!(super::extract_host_from_url("http://host:8080/"), Some("host"));
        assert_eq!(super::extract_host_from_url("ftp://bad"), None);
        assert_eq!(super::extract_host_from_url("not-a-url"), None);
    }
}
