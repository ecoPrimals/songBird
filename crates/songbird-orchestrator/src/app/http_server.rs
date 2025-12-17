//! HTTP/HTTPS Server Management
//!
//! Handles HTTP and HTTPS server lifecycle including:
//! - TLS configuration and certificate management
//! - Port binding with automatic fallback
//! - Router setup with all API endpoints
//! - Background server spawning

use anyhow::Result;
use axum::Router;
use songbird_network_federation::state::FederationState;
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_types::SafeEnv;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Start HTTP server with federation API
pub async fn start_http_server(
    federation_state: Arc<FederationState>,
    federated_service_registry: Arc<FederatedServiceRegistry>,
    bind_address: &str,
    port: u16,
) -> Result<()> {
    let addr: SocketAddr = super::parse_bind_address(bind_address, port)?;

    // Build the app with all API routes
    let app = build_router(
        Arc::clone(&federation_state),
        Arc::clone(&federated_service_registry),
    );

    // Smart port management: Try configured port, auto-increment if busy
    let (listener, actual_addr) = bind_with_fallback(&addr).await?;
    let actual_port = actual_addr.port();

    if actual_port == port {
        info!("✅ Bound to configured port {}", port);
    } else {
        warn!("⚠️  Configured port {} busy, using port {} instead", port, actual_port);
    }

    // ✅ TLS support (Dec 17, 2025) - ENABLED BY DEFAULT (fail-secure)
    // Set SONGBIRD_TLS_ENABLED=false to explicitly opt-out (e.g., for local dev)
    let tls_enabled = SafeEnv::get_bool("SONGBIRD_TLS_ENABLED", true);

    if tls_enabled {
        info!("🔐 TLS enabled - configuring HTTPS server (fail-secure by default)");
        start_https_server(app, listener, actual_addr).await
    } else {
        warn!("⚠️  TLS DISABLED - Using plain HTTP (insecure)");
        warn!("   This should only be used for local development on trusted networks");
        warn!("   For production, remove SONGBIRD_TLS_ENABLED=false");
        info!("🌐 HTTP server listening on {}", actual_addr);
        start_http_server_plain(app, listener).await
    }
}

/// Build the Axum router with all API endpoints
fn build_router(
    federation_state: Arc<FederationState>,
    federated_service_registry: Arc<FederatedServiceRegistry>,
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
    let protocol_router =
        crate::server::protocol_api::protocol_routes().with_state(protocol_state);

    // Create JSON-RPC API state for universal gateway
    let jsonrpc_state = crate::server::jsonrpc_api::JsonRpcState::new(
        Arc::clone(&federation_state),
        Arc::clone(&federated_service_registry),
    );

    // Create JSON-RPC router with state
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

    Router::new()
        .nest(
            "/api/federation",
            crate::server::federation_api::federation_routes(
                Arc::clone(&federation_state),
                Arc::clone(&federated_service_registry),
            ),
        )
        .nest(
            "/api/compute",
            compute_router,
        )
        .nest(
            "/api/protocol",
            protocol_router,
        )
        .nest(
            "/jsonrpc",
            jsonrpc_router,
        )
        .nest(
            "/api/ws",
            websocket_router,
        )
        .nest(
            "/api/deployment",
            crate::server::deployment_api::deployment_routes(deployment_state),
        )
        .route("/health", axum::routing::get(|| async { "OK" }))
        .layer(axum::extract::DefaultBodyLimit::max(100 * 1024 * 1024)) // 100 MB limit
}

/// Start plain HTTP server (no TLS)
async fn start_http_server_plain(
    app: Router,
    listener: tokio::net::TcpListener,
) -> Result<()> {
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
    socket.connect("8.8.8.8:80")?;  // Doesn't actually connect, just determines route
    
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

/// Start HTTPS server with TLS
async fn start_https_server(
    app: Router,
    _listener: tokio::net::TcpListener,
    addr: SocketAddr,
) -> Result<()> {
    use songbird_network_federation::tls::{TlsConfig, TlsCertificateManager};

    // Get TLS configuration from environment
    let cert_path = SafeEnv::get_or_default("SONGBIRD_TLS_CERT", "certs/songbird.crt");
    let key_path = SafeEnv::get_or_default("SONGBIRD_TLS_KEY", "certs/songbird.key");
    
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
            user_sans
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
        );
    }
    
    // Remove duplicates
    sans_list.sort();
    sans_list.dedup();
    
    let sans = sans_list;

    // Get node ID for common name
    let node_id = SafeEnv::get_or_default("SONGBIRD_NODE_ID", "songbird");

    let sans_display = sans.join(", ");
    
    let tls_config = TlsConfig {
        cert_path: cert_path.to_string(),
        key_path: key_path.to_string(),
        sans,
        organization: "ecoPrimals".to_string(),
        common_name: node_id.to_string(),
    };

    // Create certificate manager and ensure certificates exist
    let cert_manager = TlsCertificateManager::new(tls_config);
    cert_manager.ensure_certificates().await.map_err(|e| {
        anyhow::anyhow!("Failed to ensure TLS certificates: {}", e)
    })?;

    // Load rustls server config
    let rustls_config = cert_manager.load_tls_config().await.map_err(|e| {
        anyhow::anyhow!("Failed to load TLS config: {}", e)
    })?;

    info!("✅ TLS configuration loaded, HTTPS server listening on https://{}", addr);
    info!("   Certificate: {}", cert_path);
    info!("   Key: {}", key_path);
    info!("   SANs: {}", sans_display);
    info!("   🔒 SECURE BY DEFAULT - All connections encrypted");
    info!("   💡 To disable TLS (not recommended): export SONGBIRD_TLS_ENABLED=false");

    // Use axum-server for TLS support
    let tls_config_for_server = axum_server::tls_rustls::RustlsConfig::from_config(Arc::new(rustls_config));
    
    // Spawn HTTPS server in background
    tokio::spawn(async move {
        if let Err(e) = axum_server::bind_rustls(addr, tls_config_for_server)
            .serve(app.into_make_service())
            .await
        {
            error!("❌ HTTPS server error: {}", e);
        }
    });

    Ok(())
}

/// Smart port binding with automatic fallback
///
/// Tries the requested port first, then auto-increments until it finds an available port.
/// Maximum 10 attempts before giving up.
async fn bind_with_fallback(
    addr: &SocketAddr,
) -> Result<(tokio::net::TcpListener, SocketAddr)> {
    let host = addr.ip();
    let mut port = addr.port();
    let max_attempts = 10;

    for attempt in 1..=max_attempts {
        let try_addr = SocketAddr::new(host, port);

        match tokio::net::TcpListener::bind(try_addr).await {
            Ok(listener) => {
                let actual_addr = listener.local_addr()?;
                if attempt > 1 {
                    info!("✅ Found available port {} (after {} attempts)", port, attempt);
                }
                return Ok((listener, actual_addr));
            }
            Err(_e) if attempt < max_attempts => {
                tracing::debug!(
                    "Port {} busy, trying {} (attempt {}/{})",
                    port,
                    port + 1,
                    attempt,
                    max_attempts
                );
                port += 1;
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to bind after {} attempts. Last error: {}. Tried ports {}-{}",
                    max_attempts,
                    e,
                    addr.port(),
                    port
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

