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

use super::http_proxy::{CapabilityProxyRouter, ProxyRoute};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tracing::{debug, info, warn};

/// Route mapping: path prefix → capability name.
#[derive(Debug, Clone)]
pub struct DrawbridgeRoute {
    pub path_prefix: String,
    pub capability: String,
}

/// Drawbridge HTTP listener configuration.
#[derive(Debug, Clone)]
pub struct DrawbridgeConfig {
    pub bind_addr: String,
    pub routes: Vec<DrawbridgeRoute>,
}

impl DrawbridgeConfig {
    /// Load from environment variables.
    ///
    /// - `SONGBIRD_DRAWBRIDGE_ADDR`: bind address (default `127.0.0.1:7780`)
    /// - `SONGBIRD_DRAWBRIDGE_ROUTES`: comma-separated `path=capability` pairs
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
                let (path, cap) = entry.split_once('=')?;
                let path = path.trim();
                let cap = cap.trim();
                if path.is_empty() || cap.is_empty() {
                    return None;
                }
                Some(DrawbridgeRoute {
                    path_prefix: path.to_string(),
                    capability: cap.to_string(),
                })
            })
            .collect();

        Self { bind_addr, routes }
    }

    /// Extract unique capability names from configured routes.
    ///
    /// Used at startup to announce routable capabilities to mesh peers.
    #[must_use]
    pub fn provided_capabilities(&self) -> Vec<String> {
        let mut caps: Vec<String> = self.routes.iter().map(|r| r.capability.clone()).collect();
        caps.sort();
        caps.dedup();
        caps
    }

    fn resolve_capability(&self, path: &str) -> Option<&str> {
        for route in &self.routes {
            if path.starts_with(&route.path_prefix) {
                return Some(&route.capability);
            }
        }
        None
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
    peer: std::net::SocketAddr,
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
    let _version = parts[2];

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
                host = value.clone();
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

    let capability = config.resolve_capability(path);

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
        .map(|r| r.path_prefix.as_str())
        .unwrap_or("");

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
        request_buf.push_str(&format!("{name}: {value}\r\n"));
    }

    if let Some(b) = body {
        request_buf.push_str(&format!("Content-Length: {}\r\n", b.len()));
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
        let config = DrawbridgeConfig::from_env();
        assert_eq!(config.bind_addr, "127.0.0.1:7780");
        assert!(config.routes.is_empty());
    }

    #[test]
    fn config_parses_routes() {
        let config = DrawbridgeConfig {
            bind_addr: String::from("127.0.0.1:7780"),
            routes: vec![
                DrawbridgeRoute { path_prefix: String::from("/hub"), capability: String::from("jupyter") },
                DrawbridgeRoute { path_prefix: String::from("/api"), capability: String::from("inference") },
            ],
        };
        assert_eq!(config.routes.len(), 2);
        assert_eq!(config.routes[0].path_prefix, "/hub");
        assert_eq!(config.routes[0].capability, "jupyter");
        assert_eq!(config.routes[1].path_prefix, "/api");
        assert_eq!(config.routes[1].capability, "inference");
    }

    #[test]
    fn resolve_capability_matches_prefix() {
        let config = DrawbridgeConfig {
            bind_addr: String::new(),
            routes: vec![
                DrawbridgeRoute { path_prefix: String::from("/hub"), capability: String::from("jupyter") },
                DrawbridgeRoute { path_prefix: String::from("/api"), capability: String::from("inference") },
            ],
        };
        assert_eq!(config.resolve_capability("/hub/login"), Some("jupyter"));
        assert_eq!(config.resolve_capability("/api/v1/models"), Some("inference"));
        assert_eq!(config.resolve_capability("/unknown"), None);
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
            DrawbridgeRoute { path_prefix: String::from("/hub"), capability: String::from("jupyter") },
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
}
