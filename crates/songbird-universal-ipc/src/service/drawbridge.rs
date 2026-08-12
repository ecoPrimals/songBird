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
use super::drawbridge_proxy;
use super::http_proxy::{BackendProtocol, CapabilityProxyRouter};
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
        let bind_addr =
            songbird_process_env::var("SONGBIRD_DRAWBRIDGE_ADDR").unwrap_or_else(|_| {
                String::from(songbird_types::defaults::ports::DEFAULT_DRAWBRIDGE_ADDR)
            });

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
                let (cap, public) =
                    cap_raw.strip_suffix("!public").map_or((cap_raw, false), |c| (c, true));
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
    reason = "connection lifecycle is cohesive — splitting would scatter related stream-ownership logic"
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
        let mut stream = reader.into_inner();
        stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n").await?;
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

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

    if path == "/jsonrpc" && method == "POST" {
        return drawbridge_proxy::handle_jsonrpc_forward(
            reader.into_inner(),
            body.as_deref(),
            peer,
        )
        .await;
    }

    if let Some(token) = path.strip_prefix("/.well-known/acme-challenge/") {
        let response = serve_acme_challenge(token);
        let mut stream = reader.into_inner();
        stream.write_all(response.as_bytes()).await?;
        return Ok(());
    }

    let matched_route = config.resolve_route(path);
    let route_is_public = matched_route.is_some_and(|r| r.public);

    let auth_header = headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("authorization"))
        .map(|(_, v)| v.as_str());

    if !route_is_public && !config.auth.is_authorized(peer.ip(), path, auth_header) {
        debug!(peer = %peer, path, "drawbridge: auth denied");
        let mut stream = reader.into_inner();
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

    if capability == Some(EXTERNAL_PROXY_CAPABILITY) {
        let route_prefix = matched_route.map_or("", |r| r.path_prefix.as_str());
        let path_after_prefix = &path[route_prefix.len()..];

        let Some(external_url) = resolve_external_url(config, path_after_prefix) else {
            debug!(peer = %peer, path, "drawbridge: external proxy — service not in allowlist");
            let mut stream = reader.into_inner();
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
        return drawbridge_proxy::proxy_to_external(
            reader.into_inner(),
            method,
            &external_url,
            &headers,
            body.as_deref(),
        )
        .await;
    }

    let route = capability.map_or_else(
        || {
            let caps = router.list_capabilities();
            if caps.len() == 1 {
                router.route(&caps[0])
            } else {
                None
            }
        },
        |cap| router.route(cap),
    );

    let Some(route) = route else {
        return send_no_route_error(reader.into_inner(), router, path, capability, peer).await;
    };

    match route.protocol {
        BackendProtocol::JsonRpcIpc => {
            let method_name = derive_jsonrpc_method(path, &config.routes);
            debug!(
                peer = %peer,
                path,
                method = %method_name,
                "drawbridge: routing to JSON-RPC IPC backend"
            );
            drawbridge_proxy::proxy_to_jsonrpc_backend(
                reader.into_inner(),
                &method_name,
                body.as_deref(),
                &route.base_url,
                peer,
            )
            .await
        }
        BackendProtocol::Http => {
            let backend_url = build_backend_url(&route, path, &config.routes);
            debug!(
                peer = %peer,
                host = %host,
                path,
                backend = %backend_url,
                "drawbridge: routing to HTTP backend"
            );
            drawbridge_proxy::proxy_to_backend(
                reader.into_inner(),
                method,
                &backend_url,
                &headers,
                body.as_deref(),
            )
            .await
        }
    }
}

/// Resolve and forward a request to an allowlisted external service.
fn resolve_external_url(config: &DrawbridgeConfig, path_after_prefix: &str) -> Option<String> {
    let resolved = config
        .external_allowlist
        .parse_and_validate(path_after_prefix)
        .map(|(service, remainder)| ExternalProxyAllowlist::build_url(service, remainder));

    resolved.or_else(|| {
        let query = path_after_prefix
            .strip_prefix('?')
            .or_else(|| path_after_prefix.find('?').map(|i| &path_after_prefix[i + 1..]))?;
        let url_param = query.split('&').find_map(|p| p.strip_prefix("url="))?;
        let decoded = percent_decode(url_param);
        config.external_allowlist.validate_url(&decoded)?;
        Some(decoded)
    })
}

async fn send_no_route_error(
    mut stream: tokio::net::TcpStream,
    router: &CapabilityProxyRouter,
    path: &str,
    capability: Option<&str>,
    peer: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::fmt::Write;
    warn!(peer = %peer, path, "drawbridge: no capability route for path — check SONGBIRD_DRAWBRIDGE_ROUTES and SONGBIRD_PROXY_ROUTES env");
    let avail =
        router.list_capabilities().iter().map(|c| format!("\"{c}\"")).collect::<Vec<_>>().join(",");
    let cap = capability.map_or_else(|| "null".to_string(), |c| format!("\"{c}\""));
    let mut err_body = String::new();
    let _ = write!(
        err_body,
        r#"{{"error":"no_capability_route","path":"{path}","capability":{cap},"available_capabilities":[{avail}],"hint":"Set SONGBIRD_DRAWBRIDGE_ROUTES (path=capability) and SONGBIRD_PROXY_ROUTES (capability=url)"}}"#,
    );
    let resp = format!(
        "HTTP/1.1 502 Bad Gateway\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{err_body}",
        err_body.len()
    );
    stream.write_all(resp.as_bytes()).await?;
    Ok(())
}

fn build_backend_url(
    route: &super::http_proxy::ProxyRoute,
    request_path: &str,
    routes: &[DrawbridgeRoute],
) -> String {
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

/// Derive a JSON-RPC method name from an HTTP request path.
///
/// Strips the matched route prefix, converts remaining path segments to
/// dot-separated method name. E.g. `/api/mesh/status` with prefix `/api/` → `mesh.status`.
fn derive_jsonrpc_method(request_path: &str, routes: &[DrawbridgeRoute]) -> String {
    let matched_prefix = routes
        .iter()
        .find(|r| request_path.starts_with(&r.path_prefix))
        .map_or("", |r| r.path_prefix.as_str());

    let suffix = if matched_prefix.is_empty() {
        request_path
    } else {
        &request_path[matched_prefix.len()..]
    };

    let path_only = suffix.split('?').next().unwrap_or(suffix);
    path_only.trim_matches('/').replace('/', ".")
}

// --- ACME HTTP-01 Challenge Infrastructure ---

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

static ACME_CHALLENGES: LazyLock<RwLock<HashMap<String, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Register an ACME challenge token+authorization pair.
/// Called by bearDog via `acme.challenge_ready` JSON-RPC.
pub fn register_acme_challenge(token: &str, authorization: &str) {
    if let Ok(mut store) = ACME_CHALLENGES.write() {
        info!(token = %token, "ACME challenge registered");
        store.insert(token.to_string(), authorization.to_string());
    }
}

/// Remove an ACME challenge after validation completes.
pub fn remove_acme_challenge(token: &str) {
    if let Ok(mut store) = ACME_CHALLENGES.write() {
        store.remove(token);
    }
}

fn serve_acme_challenge(token: &str) -> String {
    let authorization = ACME_CHALLENGES.read().ok().and_then(|store| store.get(token).cloned());

    if let Some(auth) = authorization {
        info!(token = %token, "ACME challenge served");
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{auth}",
            auth.len(),
        )
    } else {
        debug!(token = %token, "ACME challenge not found");
        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string()
    }
}

#[cfg(test)]
#[path = "drawbridge_tests.rs"]
mod tests;
