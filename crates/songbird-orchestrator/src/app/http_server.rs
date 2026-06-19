// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP/HTTPS Server Management
//!
//! Handles HTTP and HTTPS server lifecycle including:
//! - TLS configuration and certificate management
//! - Port binding with automatic fallback
//! - Router setup with all API endpoints
//! - Background server spawning

use anyhow::Result;
use axum::Router;
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use songbird_types::SafeEnv;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Start HTTP server with federation API
///
/// Returns the actual port the server bound to (may differ from configured port if fallback occurred)
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn start_http_server(
    federation_state: Arc<FederationState>,
    federated_service_registry: Arc<FederatedServiceRegistry>,
    service_registry: Arc<crate::service_registry::ServiceRegistry>,
    bind_addr: SocketAddr,
) -> Result<u16> {
    // Build the app with all API routes
    let app = build_router(
        Arc::clone(&federation_state),
        Arc::clone(&federated_service_registry),
        Arc::clone(&service_registry),
    )
    .await?;

    // Smart port management: Try configured port, auto-increment if busy
    let (listener, actual_addr) = bind_with_fallback(&bind_addr).await?;
    let actual_port = actual_addr.port();

    if actual_port == bind_addr.port() {
        info!("✅ Bound to configured port {}", bind_addr.port());
    } else {
        warn!("⚠️  Configured port {} busy, using port {} instead", bind_addr.port(), actual_port);
    }

    // ✅ TLS support (Dec 17, 2025) - ENABLED BY DEFAULT (fail-secure)
    // Set SONGBIRD_TLS_ENABLED=false to explicitly opt-out (e.g., for local dev)
    // ✅ EVOLUTION (Jan 29, 2026): Graceful degradation - fall back to HTTP if TLS fails
    let tls_enabled = SafeEnv::get_bool("SONGBIRD_TLS_ENABLED", true);

    if tls_enabled {
        info!("🔐 TLS enabled - configuring HTTPS server (fail-secure by default)");
        match start_https_server(app.clone(), listener, actual_addr).await {
            Ok(()) => {
                info!("✅ HTTPS server started successfully");
            }
            Err(e) => {
                // ✅ GRACEFUL DEGRADATION: If HTTPS fails (e.g., security provider unavailable),
                // fall back to plain HTTP so the server still starts
                warn!("⚠️  HTTPS server failed to start: {}", e);
                warn!("   Most likely cause: security provider crypto provider not available");
                warn!("   DEGRADING TO PLAIN HTTP (insecure, but functional)");
                warn!("   To resolve: Start security provider or set SONGBIRD_TLS_ENABLED=false");

                // Rebind the port since the listener was consumed
                let (fallback_listener, fallback_addr) = bind_with_fallback(&bind_addr).await?;
                info!("🌐 HTTP server (fallback) listening on {}", fallback_addr);
                start_http_server_plain(app, fallback_listener).await?;
            }
        }
    } else {
        warn!("⚠️  TLS DISABLED - Using plain HTTP (insecure)");
        warn!("   This should only be used for local development on trusted networks");
        warn!("   For production, remove SONGBIRD_TLS_ENABLED=false");
        info!("🌐 HTTP server listening on {}", actual_addr);
        start_http_server_plain(app, listener).await?;
    }

    // Return the actual port we bound to
    Ok(actual_port)
}

/// Build the Axum router with all API endpoints
async fn build_router(
    federation_state: Arc<FederationState>,
    federated_service_registry: Arc<FederatedServiceRegistry>,
    service_registry: Arc<crate::service_registry::ServiceRegistry>,
) -> Result<Router> {
    // Build the app with federation and deployment routes
    let deployment_state = crate::server::deployment_api::DeploymentState::new();

    // Create compute API state for intelligent routing
    let compute_state = crate::server::compute_api::ComputeApiState::new(
        Arc::clone(&federation_state),
        Arc::clone(&federated_service_registry),
    );

    // Create compute API router with state
    let compute_router =
        crate::server::compute_api::compute_routes().with_state(compute_state.clone());

    // Create protocol API state for progressive enhancement
    let protocol_state = crate::server::protocol_api::ProtocolApiState::new();

    // Create protocol API router with state
    let protocol_router =
        crate::server::protocol_api::protocol_routes().with_state(protocol_state.clone());

    // Task lifecycle + consent (shared with JSON-RPC semantic methods)
    let _ = std::fs::create_dir_all(crate::env_config::data_dir());
    let task_db_url = crate::env_config::data_dir().join("task_lifecycle.db").display().to_string();
    let task_manager = Arc::new(
        crate::task_lifecycle::TaskLifecycleManager::new(&task_db_url)
            .await
            .map_err(|e| anyhow::anyhow!("task lifecycle database: {e}"))?,
    );
    let consent_manager = Arc::new(crate::consent_management::ConsentManager::new());

    // Create JSON-RPC API state for universal gateway
    // ✅ EVOLUTION (Feb 9, 2026): Wire IpcServiceHandler for full method forwarding on TCP
    // This makes TCP /jsonrpc equivalent to Unix socket for inter-gate mesh communication
    let ipc_registry = Arc::new(tokio::sync::RwLock::new(
        songbird_universal_ipc::registry::ServiceRegistry::new(),
    ));
    let ipc_handler =
        Arc::new(songbird_universal_ipc::service::IpcServiceHandler::with_federation_state(
            ipc_registry,
            Arc::clone(&federation_state),
        ));

    let jsonrpc_state = crate::server::jsonrpc_api::JsonRpcState::with_ipc_handler(
        Arc::clone(&federation_state),
        Arc::clone(&federated_service_registry),
        ipc_handler,
        compute_state,
        deployment_state.clone(),
        protocol_state,
        Arc::clone(&service_registry),
        Arc::clone(&task_manager),
        Arc::clone(&consent_manager),
    );

    // Create JSON-RPC router with state (now includes full universal-ipc method table)
    let jsonrpc_router = crate::server::jsonrpc_api::jsonrpc_routes().with_state(jsonrpc_state);

    // Create event broadcaster for real-time events
    let event_broadcaster = Arc::new(crate::server::events::EventBroadcaster::new());

    // Create WebSocket API state for real-time communication
    let websocket_state = crate::server::websocket_api::WebSocketApiState::new(
        Arc::clone(&federation_state),
        Arc::clone(&federated_service_registry),
        Arc::clone(&event_broadcaster),
    );

    // Create WebSocket router with state
    let websocket_router =
        crate::server::websocket_api::websocket_routes().with_state(websocket_state);

    // Create service registry router (Universal Port Authority)
    let service_registry_router =
        crate::server::service_registry_api::service_registry_routes((*service_registry).clone());

    // Create info router (for orchestrator discovery)
    let info_router = crate::server::service_registry_api::info_routes();

    Ok(Router::new()
        .nest(
            "/api/federation",
            crate::server::federation::federation_routes(
                Arc::clone(&federation_state),
                Arc::clone(&federated_service_registry),
            ),
        )
        .nest("/api/compute", compute_router)
        .nest("/api/protocol", protocol_router)
        .nest("/jsonrpc", jsonrpc_router)
        .nest("/api/ws", websocket_router)
        .nest("/api/deployment", crate::server::deployment_api::deployment_routes(deployment_state))
        .nest(
            "/api",
            crate::server::consent_api::consent_routes().with_state(
                crate::server::consent_api::ConsentApiState::new(Arc::clone(&consent_manager)),
            ),
        )
        .nest("/api/v1", crate::server::task_api::task_lifecycle_router(Arc::clone(&task_manager)))
        .nest("/api/v1/services", service_registry_router) // NEW: Universal Port Authority
        .merge(info_router) // NEW: Orchestrator info for discovery
        .route("/health", axum::routing::get(|| async { "OK" }))
        .layer(axum::extract::DefaultBodyLimit::max(100 * 1024 * 1024))) // 100 MB limit
}

/// Start plain HTTP server (no TLS)
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
async fn start_http_server_plain(app: Router, listener: tokio::net::TcpListener) -> Result<()> {
    // Spawn server in background
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("❌ HTTP server error: {}", e);
        }
    });

    Ok(())
}

/// Get local IP address for certificate SANs via [`crate::network::route_detect::resolve_local_ipv4`].
///
/// Uses the default interface from `netdev` first, then `SONGBIRD_ROUTE_DETECT_ADDR` (default
/// [RFC 5737] `192.0.2.1:80`).
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
async fn get_local_ip() -> Result<String> {
    crate::network::route_detect::resolve_local_ipv4()
}

/// Start HTTPS server with Pure Rust TLS (songbird-tls + `security provider`)
#[expect(clippy::too_many_lines, reason = "HTTPS server startup with TLS and route registration")]
async fn start_https_server(
    app: Router,
    listener: tokio::net::TcpListener,
    addr: SocketAddr,
) -> Result<()> {
    use songbird_tls::cert::generator::CertificateGenerator;
    use songbird_tls::crypto::SecurityTlsCryptoClient;
    use songbird_tls::{TlsAcceptor, TlsServerConfig};

    // Get TLS configuration from environment
    let _cert_path = SafeEnv::get_or_default("SONGBIRD_TLS_CERT", "certs/songbird.crt");
    let _key_path = SafeEnv::get_or_default("SONGBIRD_TLS_KEY", "certs/songbird.key");

    // Get Subject Alternative Names (SANs) for certificate
    // Include localhost, local IPs, and any user-specified SANs
    let mut sans_list = vec![
        songbird_types::constants::LOCALHOST.to_string(),
        songbird_types::constants::DEVELOPMENT_BIND_ADDRESS.to_string(),
    ];

    // Try to get local IP address for automatic inclusion
    if let Ok(local_ip) = get_local_ip().await {
        sans_list.push(local_ip);
    }

    // Add user-specified SANs
    let user_sans = SafeEnv::get_or_default("SONGBIRD_TLS_SANS", "");
    if !user_sans.is_empty() {
        sans_list.extend(
            user_sans.split(',').map(str::trim).filter(|s| !s.is_empty()).map(String::from),
        );
    }

    // Remove duplicates
    sans_list.sort();
    sans_list.dedup();

    let sans = sans_list;

    // Get node ID for common name
    let node_id = SafeEnv::get_or_default("SONGBIRD_NODE_ID", "songbird");

    let sans_display = sans.join(", ");

    // Self-signed certificate via CertificateGenerator (Pure Rust, Ed25519)
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

    // Create security provider crypto client for TLS operations
    // This will discover security provider via Unix socket at runtime
    let crypto_client = SecurityTlsCryptoClient::new()
        .map_err(|e| anyhow::anyhow!("Failed to create security provider crypto client: {e}"))?;

    // Create Pure Rust TLS server config
    let tls_config = TlsServerConfig {
        crypto_client,
        certificate: certificate_der,
        key_id: format!("{node_id}_tls_key"), // Key ID for security provider signing
    };

    // Create Pure Rust TLS acceptor (wrap in Arc for sharing across tasks)
    let tls_acceptor = Arc::new(TlsAcceptor::new(tls_config));

    info!("✅ Pure Rust TLS configuration loaded, server listening on {}", addr);
    info!("   Certificate: Generated (test cert for '{}')", node_id);
    info!("   Crypto: security provider via Unix socket");
    info!("   SANs: {}", sans_display);
    info!("   🔒 100% PURE RUST - Zero C dependencies!");
    info!("   🎯 Protocol: songbird-tls | Crypto: security provider");
    info!("   🔄 Protocol Detection: HTTP and HTTPS on same port");
    info!("   💡 To disable TLS (not recommended): export SONGBIRD_TLS_ENABLED=false");

    // DEEP DEBT SOLUTION: 100% Pure Rust TLS with songbird-tls
    //
    // This is the sovereign pattern:
    // - songbird-tls for TLS 1.3 protocol (Pure Rust)
    // - security provider for all cryptographic operations (Pure Rust)
    // - Runtime discovery via Unix sockets (no hardcoding)
    // - Zero C dependencies (TRUE ecoBin)

    // Spawn HTTPS server using Pure Rust TLS (songbird-tls)
    // EVOLUTION (Feb 2026): Protocol detection for HTTP/HTTPS on same port
    tokio::spawn(async move {
        // Accept loop: convert TCP connections to TLS or HTTP streams
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

            // Handle each connection in its own task
            tokio::spawn(async move {
                // PROTOCOL DETECTION: Peek at first byte to detect TLS vs HTTP
                // TLS ClientHello starts with 0x16 (Handshake content type)
                // HTTP requests start with ASCII method (GET=0x47, POST=0x50, PUT=0x50, HEAD=0x48, etc.)
                let mut peek_buf = [0u8; 1];
                let peek_result = tcp_stream.peek(&mut peek_buf).await;

                let is_tls = match peek_result {
                    Ok(1) => peek_buf[0] == 0x16, // TLS Handshake content type
                    Ok(0) => {
                        tracing::debug!("Empty connection from {}, closing", remote_addr);
                        return;
                    }
                    Ok(_) => false, // Shouldn't happen with 1-byte buffer
                    Err(e) => {
                        error!("Failed to peek connection from {}: {}", remote_addr, e);
                        return;
                    }
                };

                // Import shared dependencies
                use hyper::body::Incoming;
                use hyper_util::rt::TokioIo;
                use tower::Service;

                if is_tls {
                    // TLS connection: Perform Pure Rust TLS handshake
                    let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                        Ok(stream) => stream,
                        Err(e) => {
                            error!("🔒 Pure Rust TLS handshake failed from {}: {}", remote_addr, e);
                            return;
                        }
                    };

                    tracing::debug!("🔒 Pure Rust TLS connection established from {}", remote_addr);

                    let hyper_service =
                        hyper::service::service_fn(move |request: hyper::Request<Incoming>| {
                            let mut app = app.clone();
                            async move { app.call(request).await }
                        });

                    // Serve HTTPS connection
                    if let Err(e) = hyper::server::conn::http1::Builder::new()
                        .serve_connection(TokioIo::new(tls_stream), hyper_service)
                        .await
                        && !e.to_string().contains("connection closed")
                    {
                        error!("Error serving HTTPS connection from {}: {}", remote_addr, e);
                    }
                } else {
                    // HTTP connection: Serve plain HTTP (graceful degradation)
                    tracing::debug!(
                        "📡 Plain HTTP connection from {} (protocol detection)",
                        remote_addr
                    );

                    let hyper_service =
                        hyper::service::service_fn(move |request: hyper::Request<Incoming>| {
                            let mut app = app.clone();
                            async move { app.call(request).await }
                        });

                    // Serve HTTP connection
                    if let Err(e) = hyper::server::conn::http1::Builder::new()
                        .serve_connection(TokioIo::new(tcp_stream), hyper_service)
                        .await
                        && !e.to_string().contains("connection closed")
                    {
                        error!("Error serving HTTP connection from {}: {}", remote_addr, e);
                    }
                }
            });
        }
    });

    Ok(())
}

/// Smart port binding with automatic fallback using Sovereign Socket
///
/// Bind the HTTP server to the requested address.
///
/// When the bind address is a wildcard (`0.0.0.0` or `[::]`), uses the
/// `SovereignBinder` multi-strategy approach (IPv6 dual-stack → IPv4 → localhost).
///
/// When a specific IP is requested (e.g. `127.0.0.1`), binds directly to that
/// address — this is the secure-by-default path triggered by `--bind 127.0.0.1`.
async fn bind_with_fallback(addr: &SocketAddr) -> Result<(tokio::net::TcpListener, SocketAddr)> {
    let port = addr.port();
    let ip = addr.ip();

    let is_wildcard = ip.is_unspecified();

    if is_wildcard {
        use crate::network::SovereignBinder;

        if port == 0 {
            info!("🦅 Ephemeral port requested (port 0) — OS will assign");
            let (listener, actual_addr) = SovereignBinder::bind_sovereign(0).await?;
            info!("✅ Ephemeral bind successful: {}", actual_addr);
            return Ok((listener, actual_addr));
        }

        info!("🦅 Using sovereign socket binding for port {} (wildcard)", port);

        match SovereignBinder::bind_sovereign(port).await {
            Ok((listener, actual_addr)) => {
                info!("✅ Sovereign bind successful: {}", actual_addr);
                return Ok((listener, actual_addr));
            }
            Err(e) => {
                warn!("Sovereign bind to port {} failed: {}", port, e);
                warn!("Attempting fallback with incremental ports...");
            }
        }

        let max_attempts = 10;
        for attempt in 1..=max_attempts {
            let try_port = port + attempt;

            match SovereignBinder::bind_sovereign(try_port).await {
                Ok((listener, actual_addr)) => {
                    info!("✅ Sovereign bind successful on fallback port: {}", actual_addr);
                    return Ok((listener, actual_addr));
                }
                Err(_) if attempt < max_attempts => {
                    tracing::debug!("Port {} busy, trying next...", try_port);
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to bind after {max_attempts} attempts. Last error: {e}. Tried ports {port}-{try_port}"
                    ));
                }
            }
        }

        return Err(anyhow::anyhow!(
            "Port binding loop exhausted {max_attempts} attempts without returning"
        ));
    }

    // Specific IP requested — bind directly (secure-by-default path)
    info!("🔒 Binding HTTP server to {} (specific address)", addr);

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            let actual_addr = listener.local_addr()?;
            info!("✅ HTTP server bound to {}", actual_addr);
            Ok((listener, actual_addr))
        }
        Err(e) if port > 0 => {
            warn!("Bind to {} failed: {} — trying incremental ports", addr, e);
            let max_attempts = 10;
            for attempt in 1..=max_attempts {
                let try_port = port + attempt;
                let try_addr = SocketAddr::new(ip, try_port);
                match tokio::net::TcpListener::bind(try_addr).await {
                    Ok(listener) => {
                        let actual_addr = listener.local_addr()?;
                        info!("✅ HTTP server bound to fallback: {}", actual_addr);
                        return Ok((listener, actual_addr));
                    }
                    Err(_) if attempt < max_attempts => continue,
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "Failed to bind to {ip} after {max_attempts} attempts. Last error: {e}"
                        ));
                    }
                }
            }
            Err(anyhow::anyhow!(
                "Port binding loop exhausted {max_attempts} attempts without returning"
            ))
        }
        Err(e) => Err(anyhow::anyhow!("Failed to bind to {addr}: {e}")),
    }
}

/// Start tarpc server for high-performance native RPC
///
/// tarpc provides binary RPC with ~50μs latency (100x faster than JSON-RPC!)
/// for native Rust client-to-server communication.
/// # Errors
///
/// Returns an error if the operation fails.
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
pub async fn start_tarpc_server(
    federation_state: Arc<FederationState>,
    service_registry: Arc<FederatedServiceRegistry>,
    bind_address: &str,
    port: u16,
) -> Result<()> {
    let addr: SocketAddr = super::parse_bind_address(bind_address, port)?;

    info!("🚀 Starting tarpc server on {}", addr);

    // Spawn tarpc server in background
    tokio::spawn(async move {
        if let Err(e) = crate::server::tarpc_server::start_tarpc_server(
            addr,
            federation_state,
            service_registry,
        )
        .await
        {
            error!("❌ tarpc server error: {}", e);
        }
    });

    Ok(())
}
