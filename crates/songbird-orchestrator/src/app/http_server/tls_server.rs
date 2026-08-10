// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HTTPS server with Pure Rust TLS (songbird-tls + security provider).
//!
//! Handles the TLS accept loop with protocol detection:
//! - `0xEC`/`0xED`/`0xEE` → riboCipher tier routing (federation)
//! - `0x16` → TLS ClientHello → HTTPS
//! - ASCII → plain HTTP (deprecated, error-logged)

use anyhow::Result;
use axum::Router;
use songbird_types::SafeEnv;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

use super::federation_dispatch::dispatch_ribocipher_rpc;

/// Start HTTPS server with Pure Rust TLS (songbird-tls + security provider).
pub(super) async fn start_https_server(
    app: Router,
    federation_ipc_handler: Option<Arc<songbird_universal_ipc::service::IpcServiceHandler>>,
    listener: tokio::net::TcpListener,
    addr: SocketAddr,
) -> Result<()> {
    use songbird_tls::cert::generator::CertificateGenerator;
    use songbird_tls::crypto::SecurityTlsCryptoClient;
    use songbird_tls::{TlsAcceptor, TlsServerConfig};

    let mut sans_list = vec![
        songbird_types::constants::LOCALHOST.to_string(),
        songbird_types::constants::DEVELOPMENT_BIND_ADDRESS.to_string(),
    ];

    if let Ok(local_ip) = super::get_local_ip().await {
        sans_list.push(local_ip);
    }

    let user_sans = SafeEnv::get_or_default("SONGBIRD_TLS_SANS", "");
    if !user_sans.is_empty() {
        sans_list.extend(
            user_sans.split(',').map(str::trim).filter(|s| !s.is_empty()).map(String::from),
        );
    }

    sans_list.sort();
    sans_list.dedup();

    let sans = sans_list;
    let node_id =
        SafeEnv::get_or_default("SONGBIRD_NODE_ID", songbird_types::primal_names::SELF_NAME);
    let sans_display = sans.join(", ");

    let cert_gen = CertificateGenerator::new()
        .map_err(|e| anyhow::anyhow!("Failed to create cert generator: {e}"))?;
    let (cert, _signing_key) = cert_gen
        .generate_self_signed(&node_id, 365)
        .map_err(|e| anyhow::anyhow!("Failed to generate self-signed certificate: {e}"))?;

    let certificate_der = cert
        .certificate_list
        .first()
        .ok_or_else(|| anyhow::anyhow!("No certificate in chain"))?
        .cert_data
        .clone();

    let crypto_client = SecurityTlsCryptoClient::new()
        .map_err(|e| anyhow::anyhow!("Failed to create security provider crypto client: {e}"))?;

    let tls_config = TlsServerConfig {
        crypto_client,
        certificate: certificate_der,
        key_id: format!("{node_id}_tls_key"),
    };

    let tls_acceptor = Arc::new(TlsAcceptor::new(tls_config));

    info!("✅ Pure Rust TLS configuration loaded, server listening on {}", addr);
    info!("   Certificate: Generated (test cert for '{}')", node_id);
    info!("   Crypto: security provider via Unix socket");
    info!("   SANs: {}", sans_display);
    info!("   🔒 100% PURE RUST - Zero C dependencies!");
    info!("   🎯 Protocol: songbird-tls | Crypto: security provider");
    info!("   🔄 Protocol Detection: HTTP and HTTPS on same port");
    info!("   💡 To disable TLS (not recommended): export SONGBIRD_TLS_ENABLED=false");

    tokio::spawn(async move {
        loop {
            let (tcp_stream, remote_addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to accept TCP connection: {}", e);
                    continue;
                }
            };

            let tls_acceptor = Arc::clone(&tls_acceptor);
            let app = app.clone();
            let ipc_handler = federation_ipc_handler.clone();

            tokio::spawn(async move {
                let mut peek_buf = [0u8; 1];
                let peek_result = tcp_stream.peek(&mut peek_buf).await;

                let first_byte = match peek_result {
                    Ok(1) => peek_buf[0],
                    Ok(0) => {
                        tracing::debug!("Empty connection from {}, closing", remote_addr);
                        return;
                    }
                    Ok(_) => return,
                    Err(e) => {
                        error!("Failed to peek connection from {}: {}", remote_addr, e);
                        return;
                    }
                };

                if songbird_types::constants::ribocipher::is_signal_byte(first_byte) {
                    handle_ribocipher_connection(tcp_stream, remote_addr, first_byte, ipc_handler)
                        .await;
                    return;
                }

                tracing::error!(
                    "Federation connection from {} without riboCipher signal (0x{first_byte:02X}) — legacy path (deprecated Wave 112, reject Wave 113)",
                    remote_addr
                );

                let is_tls = first_byte == 0x16;
                serve_http_or_tls(tcp_stream, remote_addr, is_tls, app, tls_acceptor).await;
            });
        }
    });

    Ok(())
}

/// Handle a riboCipher-signalled connection (federation tier).
async fn handle_ribocipher_connection(
    tcp_stream: tokio::net::TcpStream,
    remote_addr: SocketAddr,
    first_byte: u8,
    ipc_handler: Option<Arc<songbird_universal_ipc::service::IpcServiceHandler>>,
) {
    use songbird_types::constants::ribocipher;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let tier = ribocipher::tier_name(first_byte);
    tracing::info!("riboCipher {tier} signal from {} on federation port", remote_addr);

    let (reader, mut writer) = tcp_stream.into_split();
    let mut reader = BufReader::new(reader);

    let mut prefix = [0u8; 2];
    if tokio::io::AsyncReadExt::read_exact(&mut reader, &mut prefix).await.is_err() {
        tracing::warn!("riboCipher {tier}: failed to read prefix from {}", remote_addr);
        return;
    }

    if prefix[1] != ribocipher::VERSION_1 {
        tracing::warn!(
            "riboCipher {tier}: unsupported version 0x{:02X} from {}",
            prefix[1],
            remote_addr
        );
        return;
    }

    let mut line = String::new();
    while let Ok(n) = reader.read_line(&mut line).await {
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }
        let response = match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(req) => {
                let method = req["method"].as_str().unwrap_or("").to_string();
                let id = req["id"].clone();
                let params = req.get("params").cloned().unwrap_or(serde_json::Value::Null);
                tracing::debug!("riboCipher {tier} RPC from {}: {}", remote_addr, method);
                dispatch_ribocipher_rpc(&method, params, id, tier, ipc_handler.as_deref()).await
            }
            Err(e) => serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32700, "message": format!("Parse error: {e}")},
                "id": null
            }),
        };
        let mut resp_bytes = serde_json::to_vec(&response).unwrap_or_default();
        resp_bytes.push(b'\n');
        if writer.write_all(&resp_bytes).await.is_err() {
            break;
        }
        line.clear();
    }
}

/// Serve an HTTP or HTTPS connection after protocol detection.
async fn serve_http_or_tls(
    tcp_stream: tokio::net::TcpStream,
    remote_addr: SocketAddr,
    is_tls: bool,
    app: Router,
    tls_acceptor: Arc<songbird_tls::TlsAcceptor>,
) {
    use hyper::body::Incoming;
    use hyper_util::rt::TokioIo;
    use tower::Service;

    if is_tls {
        let tls_stream = match tls_acceptor.accept(tcp_stream).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("🔒 Pure Rust TLS handshake failed from {}: {}", remote_addr, e);
                return;
            }
        };

        tracing::debug!("🔒 Pure Rust TLS connection established from {}", remote_addr);

        let hyper_service = hyper::service::service_fn(move |request: hyper::Request<Incoming>| {
            let mut app = app.clone();
            async move { app.call(request).await }
        });

        if let Err(e) = hyper::server::conn::http1::Builder::new()
            .serve_connection(TokioIo::new(tls_stream), hyper_service)
            .await
            && !e.to_string().contains("connection closed")
        {
            error!("Error serving HTTPS connection from {}: {}", remote_addr, e);
        }
    } else {
        tracing::debug!("📡 Plain HTTP connection from {} (protocol detection)", remote_addr);

        let hyper_service = hyper::service::service_fn(move |request: hyper::Request<Incoming>| {
            let mut app = app.clone();
            async move { app.call(request).await }
        });

        if let Err(e) = hyper::server::conn::http1::Builder::new()
            .serve_connection(TokioIo::new(tcp_stream), hyper_service)
            .await
            && !e.to_string().contains("connection closed")
        {
            error!("Error serving HTTP connection from {}: {}", remote_addr, e);
        }
    }
}
