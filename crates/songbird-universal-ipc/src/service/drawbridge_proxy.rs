// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Outbound proxy backends for drawbridge: HTTP, HTTPS/TLS, and JSON-RPC over IPC.

use std::fmt::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, warn};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Forward HTTP request to an HTTP backend and stream the response back to the client.
pub(super) async fn proxy_to_backend(
    mut client_stream: tokio::net::TcpStream,
    method: &str,
    backend_url: &str,
    headers: &[(String, String)],
    body: Option<&str>,
) -> Result<(), BoxError> {
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

/// Forward to an external allowlisted service (dispatches HTTP vs HTTPS).
pub(super) async fn proxy_to_external(
    client_stream: tokio::net::TcpStream,
    method: &str,
    external_url: &str,
    headers: &[(String, String)],
    body: Option<&str>,
) -> Result<(), BoxError> {
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
) -> Result<(), BoxError> {
    use rustls::pki_types::ServerName;

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

/// Send a JSON-RPC request over IPC and return the response body as a string.
///
/// Shared core between `proxy_to_jsonrpc_backend` and `handle_jsonrpc_forward`.
/// Connects to the given IPC socket, sends the request, reads until newline or timeout.
pub(super) async fn ipc_jsonrpc_roundtrip(
    socket_path: &std::path::Path,
    request_bytes: &[u8],
) -> Result<String, String> {
    let path_str = socket_path.to_string_lossy();
    let mut ipc = songbird_types::IpcStream::connect(&path_str)
        .await
        .map_err(|e| format!("IPC unavailable: {e}"))?;

    ipc.write_all(request_bytes).await.map_err(|e| format!("IPC write failed: {e}"))?;

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

    Ok(String::from_utf8_lossy(&response).trim().to_string())
}

/// Forward a JSON-RPC request to a specific IPC socket and write the HTTP response.
pub(super) async fn proxy_to_jsonrpc_backend(
    mut client_stream: tokio::net::TcpStream,
    method: &str,
    http_body: Option<&str>,
    socket_path_hint: &str,
    peer: SocketAddr,
) -> Result<(), BoxError> {
    let socket_path = if socket_path_hint.is_empty() {
        songbird_types::defaults::paths::primary_ipc_socket_path()
    } else {
        std::path::PathBuf::from(socket_path_hint)
    };

    let body_json: Option<serde_json::Value> = http_body.and_then(|b| serde_json::from_str(b).ok());

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
            .unwrap_or_else(|| serde_json::json!(1))
    });

    let mut request_bytes = serde_json::to_vec(&jsonrpc_request)?;
    request_bytes.push(b'\n');

    match ipc_jsonrpc_roundtrip(&socket_path, &request_bytes).await {
        Ok(response_body) => {
            let http_resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{response_body}",
                response_body.len()
            );
            client_stream.write_all(http_resp.as_bytes()).await?;
        }
        Err(e) => {
            debug!(peer = %peer, error = %e, method, "drawbridge jsonrpc: IPC error");
            let err_body =
                format!(r#"{{"jsonrpc":"2.0","error":{{"code":-32603,"message":"{e}"}},"id":1}}"#,);
            let resp = format!(
                "HTTP/1.1 502 Bad Gateway\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{err_body}",
                err_body.len()
            );
            client_stream.write_all(resp.as_bytes()).await?;
        }
    }

    Ok(())
}

/// Forward a raw JSON-RPC body to the primary IPC endpoint and return the HTTP response.
pub(super) async fn handle_jsonrpc_forward(
    mut client_stream: tokio::net::TcpStream,
    body: Option<&str>,
    peer: SocketAddr,
) -> Result<(), BoxError> {
    let Some(json_body) = body else {
        client_stream
            .write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: 42\r\n\r\n{\"error\":\"POST /jsonrpc requires a body\"}")
            .await?;
        return Ok(());
    };

    let socket_path = songbird_types::defaults::paths::primary_ipc_socket_path();

    let mut request_bytes = json_body.as_bytes().to_vec();
    if !request_bytes.ends_with(b"\n") {
        request_bytes.push(b'\n');
    }

    match ipc_jsonrpc_roundtrip(&socket_path, &request_bytes).await {
        Ok(response_body) => {
            let http_resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{response_body}",
                response_body.len()
            );
            client_stream.write_all(http_resp.as_bytes()).await?;
        }
        Err(e) => {
            debug!(peer = %peer, error = %e, "jsonrpc forward: IPC error");
            let err_body = format!(
                "{{\"jsonrpc\":\"2.0\",\"error\":{{\"code\":-32603,\"message\":\"{e}\"}},\"id\":null}}"
            );
            let resp = format!(
                "HTTP/1.1 502 Bad Gateway\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{err_body}",
                err_body.len()
            );
            client_stream.write_all(resp.as_bytes()).await?;
        }
    }

    Ok(())
}
