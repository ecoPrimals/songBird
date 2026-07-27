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
use super::http_proxy::{CapabilityProxyRouter, ProxyRoute};
use std::fmt::Write;
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

    // JSON-RPC forwarding endpoint — always accessible (used by esotericWebb, sourDough-pattern consumers)
    if path == "/jsonrpc" && method == "POST" {
        return handle_jsonrpc_forward(reader.into_inner(), body.as_deref(), peer).await;
    }

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
        warn!(peer = %peer, path, "drawbridge: no capability route for path — check SONGBIRD_DRAWBRIDGE_ROUTES and SONGBIRD_PROXY_ROUTES env");
        let stream = reader.into_inner();
        let mut stream = stream;
        let avail = router
            .list_capabilities()
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(",");
        let cap = capability.map_or_else(|| "null".to_string(), |c| format!("\"{c}\""));
        let err_body = format!(
            r#"{{"error":"no_capability_route","path":"{path}","capability":{cap},"available_capabilities":[{avail}],"hint":"Set SONGBIRD_DRAWBRIDGE_ROUTES (path=capability) and SONGBIRD_PROXY_ROUTES (capability=url)"}}"#,
        );
        let resp = format!(
            "HTTP/1.1 502 Bad Gateway\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{err_body}",
            err_body.len()
        );
        stream.write_all(resp.as_bytes()).await?;
        return Ok(());
    };

    use super::http_proxy::BackendProtocol;

    match route.protocol {
        BackendProtocol::JsonRpcIpc => {
            let method_name = derive_jsonrpc_method(path, &config.routes);
            debug!(
                peer = %peer,
                path,
                method = %method_name,
                "drawbridge: routing to JSON-RPC IPC backend"
            );
            proxy_to_jsonrpc_backend(
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
            proxy_to_backend(reader.into_inner(), method, &backend_url, &headers, body.as_deref())
                .await
        }
    }
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

/// Derive a JSON-RPC method name from an HTTP request path.
///
/// Strips the matched route prefix, converts remaining path segments to
/// dot-separated method name. E.g. `/api/mesh/status` with prefix `/api/` → `mesh.status`.
/// Query string is stripped before conversion.
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

/// Translate an HTTP request into a JSON-RPC call via IPC, return response as HTTP.
///
/// Connects to the specified IPC socket (or default if empty), sends a JSON-RPC
/// request derived from the HTTP path→method mapping, and returns the JSON-RPC
/// response as an `HTTP 200 application/json` body.
async fn proxy_to_jsonrpc_backend(
    mut client_stream: tokio::net::TcpStream,
    method: &str,
    http_body: Option<&str>,
    socket_path_hint: &str,
    peer: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::AsyncReadExt;

    let socket_path = if socket_path_hint.is_empty() {
        songbird_types::defaults::paths::primary_ipc_socket_path()
    } else {
        std::path::PathBuf::from(socket_path_hint)
    };

    let body_json: Option<serde_json::Value> =
        http_body.and_then(|b| serde_json::from_str(b).ok());

    let (effective_method, params) = if method.is_empty() {
        if let Some(ref obj) = body_json {
            let m = obj.get("method").and_then(|v| v.as_str()).unwrap_or("");
            let p = obj.get("params").cloned().unwrap_or(serde_json::Value::Null);
            (m.to_string(), p)
        } else {
            (String::new(), serde_json::Value::Null)
        }
    } else {
        (method.to_string(), body_json.clone().unwrap_or(serde_json::Value::Null))
    };

    let jsonrpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": effective_method,
        "params": params,
        "id": body_json.as_ref()
            .and_then(|v| v.get("id"))
            .cloned()
            .unwrap_or(serde_json::json!(1))
    });

    let path_str = socket_path.to_string_lossy();
    let mut ipc = match songbird_types::IpcStream::connect(&path_str).await {
        Ok(s) => s,
        Err(e) => {
            debug!(peer = %peer, error = %e, method, "drawbridge jsonrpc: cannot connect to IPC");
            let err_body = format!(
                r#"{{"jsonrpc":"2.0","error":{{"code":-32603,"message":"IPC unavailable: {e}"}},"id":1}}"#,
            );
            let resp = format!(
                "HTTP/1.1 502 Bad Gateway\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{err_body}",
                err_body.len()
            );
            client_stream.write_all(resp.as_bytes()).await?;
            return Ok(());
        }
    };

    let mut request_bytes = serde_json::to_vec(&jsonrpc_request)?;
    request_bytes.push(b'\n');
    ipc.write_all(&request_bytes).await?;

    let mut response = Vec::with_capacity(4096);
    let mut buf = [0u8; 4096];
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, ipc.read(&mut buf)).await {
            Ok(Ok(0)) => break,
            Ok(Ok(n)) => {
                response.extend_from_slice(&buf[..n]);
                if response.contains(&b'\n') {
                    break;
                }
            }
            Ok(Err(_)) => break,
            Err(_) => break,
        }
    }

    let response_str = String::from_utf8_lossy(&response);
    let trimmed = response_str.trim();

    let http_resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{trimmed}",
        trimmed.len()
    );
    client_stream.write_all(http_resp.as_bytes()).await?;
    Ok(())
}

/// Forward a JSON-RPC request to the local songBird IPC endpoint and return
/// the response as an HTTP 200 JSON body. Enables esotericWebb and other
/// NDJSON JSON-RPC consumers to call songBird methods over HTTP.
async fn handle_jsonrpc_forward(
    mut client_stream: tokio::net::TcpStream,
    body: Option<&str>,
    peer: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::AsyncReadExt;

    let Some(json_body) = body else {
        client_stream
            .write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: 42\r\n\r\n{\"error\":\"POST /jsonrpc requires a body\"}")
            .await?;
        return Ok(());
    };

    let socket_path = songbird_types::defaults::paths::primary_ipc_socket_path();
    let path_str = socket_path.to_string_lossy();
    let mut ipc = match songbird_types::IpcStream::connect(&path_str).await {
        Ok(s) => s,
        Err(e) => {
            debug!(peer = %peer, error = %e, "jsonrpc forward: cannot connect to IPC");
            let err_body = format!(
                "{{\"jsonrpc\":\"2.0\",\"error\":{{\"code\":-32603,\"message\":\"IPC unavailable: {e}\"}},\"id\":null}}"
            );
            let resp = format!(
                "HTTP/1.1 502 Bad Gateway\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{err_body}",
                err_body.len()
            );
            client_stream.write_all(resp.as_bytes()).await?;
            return Ok(());
        }
    };

    let mut request_bytes = json_body.as_bytes().to_vec();
    if !request_bytes.ends_with(b"\n") {
        request_bytes.push(b'\n');
    }
    ipc.write_all(&request_bytes).await?;

    let mut response = Vec::with_capacity(4096);
    let mut buf = [0u8; 4096];
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, ipc.read(&mut buf)).await {
            Ok(Ok(0)) => break,
            Ok(Ok(n)) => {
                response.extend_from_slice(&buf[..n]);
                if response.contains(&b'\n') {
                    break;
                }
            }
            Ok(Err(_)) => break,
            Err(_) => break,
        }
    }

    let response_str = String::from_utf8_lossy(&response);
    let trimmed = response_str.trim();

    let http_resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{trimmed}",
        trimmed.len()
    );
    client_stream.write_all(http_resp.as_bytes()).await?;
    Ok(())
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

    let host = authority.split(':').next().unwrap_or(authority);

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
#[path = "drawbridge_tests.rs"]
mod tests;
