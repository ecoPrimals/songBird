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
    );

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
                // ✅ GRACEFUL DEGRADATION: If HTTPS fails (e.g., BearDog unavailable),
                // fall back to plain HTTP so the server still starts
                warn!("⚠️  HTTPS server failed to start: {}", e);
                warn!("   Most likely cause: BearDog crypto provider not available");
                warn!("   DEGRADING TO PLAIN HTTP (insecure, but functional)");
                warn!("   To resolve: Start BearDog or set SONGBIRD_TLS_ENABLED=false");

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
fn build_router(
    federation_state: Arc<FederationState>,
    federated_service_registry: Arc<FederatedServiceRegistry>,
    service_registry: Arc<crate::service_registry::ServiceRegistry>,
) -> Router {
    // Build the app with federation and deployment routes
    let deployment_state = crate::server::deployment_api::DeploymentState::new();

    // Create compute API state for intelligent routing
    let compute_state = crate::server::compute_api::ComputeApiState::new(
        Arc::clone(&federation_state),
        Arc::clone(&federated_service_registry),
    );

    // Create compute API router with state
    let compute_router = crate::server::compute_api::compute_routes().with_state(compute_state);

    // Create protocol API state for progressive enhancement
    let protocol_state = crate::server::protocol_api::ProtocolApiState::new();

    // Create protocol API router with state
    let protocol_router = crate::server::protocol_api::protocol_routes().with_state(protocol_state);

    // Create JSON-RPC API state for universal gateway
    // ✅ EVOLUTION (Feb 9, 2026): Wire IpcServiceHandler for full method forwarding on TCP
    // This makes TCP /jsonrpc equivalent to Unix socket for inter-gate mesh communication
    let ipc_registry = Arc::new(tokio::sync::RwLock::new(
        songbird_universal_ipc::registry::ServiceRegistry::new(),
    ));
    let ipc_handler =
        Arc::new(songbird_universal_ipc::service::IpcServiceHandler::new(ipc_registry));

    let jsonrpc_state = crate::server::jsonrpc_api::JsonRpcState::with_ipc_handler(
        Arc::clone(&federation_state),
        Arc::clone(&federated_service_registry),
        ipc_handler,
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

    Router::new()
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
        .nest("/api/v1/services", service_registry_router) // NEW: Universal Port Authority
        .merge(info_router) // NEW: Orchestrator info for discovery
        .route("/health", axum::routing::get(|| async { "OK" }))
        .layer(axum::extract::DefaultBodyLimit::max(100 * 1024 * 1024)) // 100 MB limit
}

/// Start plain HTTP server (no TLS)
async fn start_http_server_plain(app: Router, listener: tokio::net::TcpListener) -> Result<()> {
    // Spawn server in background
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("❌ HTTP server error: {}", e);
        }
    });

    Ok(())
}

/// Get local IP address for certificate SANs
async fn get_local_ip() -> Result<String> {
    use std::net::{IpAddr, Ipv4Addr};

    // Try to get local IP by creating a UDP socket (doesn't actually send data)
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?; // Doesn't actually connect, just determines route

    if let Ok(local_addr) = socket.local_addr() {
        let ip = local_addr.ip();
        match ip {
            IpAddr::V4(ipv4) if ipv4 != Ipv4Addr::LOCALHOST => {
                return Ok(ip.to_string());
            }
            _ => {}
        }
    }

    Err(anyhow::anyhow!("Could not determine local IP"))
}

/// Start HTTPS server with Pure Rust TLS (songbird-tls + BearDog)
async fn start_https_server(
    app: Router,
    listener: tokio::net::TcpListener,
    addr: SocketAddr,
) -> Result<()> {
    use songbird_tls::cert::test_utils::generate_test_certificate;
    use songbird_tls::crypto::BeardogCryptoClient;
    use songbird_tls::{TlsAcceptor, TlsServerConfig};

    // Get TLS configuration from environment
    let _cert_path = SafeEnv::get_or_default("SONGBIRD_TLS_CERT", "certs/songbird.crt");
    let _key_path = SafeEnv::get_or_default("SONGBIRD_TLS_KEY", "certs/songbird.key");

    // Get Subject Alternative Names (SANs) for certificate
    // Include localhost, local IPs, and any user-specified SANs
    let mut sans_list = vec!["localhost".to_string(), "127.0.0.1".to_string()];

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

    // PURE RUST TLS: songbird-tls + BearDog crypto
    // Generate test certificate (in production, use proper cert management)
    let test_cert = generate_test_certificate(&node_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to generate test certificate: {}", e))?;

    // Extract certificate data (first entry in chain)
    let certificate_der = test_cert
        .certificate_list
        .first()
        .ok_or_else(|| anyhow::anyhow!("No certificate in chain"))?
        .cert_data
        .clone();

    // Create BearDog crypto client for TLS operations
    // This will discover BearDog via Unix socket at runtime
    let crypto_client = BeardogCryptoClient::new()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create BearDog crypto client: {}", e))?;

    // Create Pure Rust TLS server config
    let tls_config = TlsServerConfig {
        crypto_client,
        certificate: certificate_der,
        key_id: format!("{}_tls_key", node_id), // Key ID for BearDog signing
    };

    // Create Pure Rust TLS acceptor (wrap in Arc for sharing across tasks)
    let tls_acceptor = Arc::new(TlsAcceptor::new(tls_config));

    info!("✅ Pure Rust TLS configuration loaded, server listening on {}", addr);
    info!("   Certificate: Generated (test cert for '{}')", node_id);
    info!("   Crypto: BearDog via Unix socket");
    info!("   SANs: {}", sans_display);
    info!("   🔒 100% PURE RUST - Zero C dependencies!");
    info!("   🎯 Protocol: songbird-tls | Crypto: BearDog");
    info!("   🔄 Protocol Detection: HTTP and HTTPS on same port");
    info!("   💡 To disable TLS (not recommended): export SONGBIRD_TLS_ENABLED=false");

    // DEEP DEBT SOLUTION: 100% Pure Rust TLS with songbird-tls
    //
    // This is the sovereign pattern:
    // - songbird-tls for TLS 1.3 protocol (Pure Rust)
    // - BearDog for all cryptographic operations (Pure Rust)
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

            let tls_acceptor = tls_acceptor.clone();
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
                    {
                        if !e.to_string().contains("connection closed") {
                            error!("Error serving HTTPS connection from {}: {}", remote_addr, e);
                        }
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
                    {
                        if !e.to_string().contains("connection closed") {
                            error!("Error serving HTTP connection from {}: {}", remote_addr, e);
                        }
                    }
                }
            });
        }
    });

    Ok(())
}

/// Smart port binding with automatic fallback using Sovereign Socket
///
/// Uses the SovereignBinder for truly sovereign network configuration.
/// No external tools, no sudo, no manual setup required.
///
/// The sovereign binder will:
/// - Try IPv4 wildcard (0.0.0.0) first
/// - Fall back to IPv6 wildcard (::) if needed
/// - Configure optimal socket options
/// - Enable port reuse for zero-downtime restarts
async fn bind_with_fallback(addr: &SocketAddr) -> Result<(tokio::net::TcpListener, SocketAddr)> {
    use crate::network::SovereignBinder;

    let port = addr.port();

    info!("🦅 Using sovereign socket binding for port {}", port);

    // Try sovereign binding on the requested port
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

    // Fallback: try incrementing ports
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
                    "Failed to bind after {} attempts. Last error: {}. Tried ports {}-{}",
                    max_attempts,
                    e,
                    port,
                    try_port
                ));
            }
        }
    }

    unreachable!("Loop should have returned or errored");
}

/// Start tarpc server for high-performance native RPC
///
/// tarpc provides binary RPC with ~50μs latency (100x faster than JSON-RPC!)
/// for native Rust client-to-server communication.
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
