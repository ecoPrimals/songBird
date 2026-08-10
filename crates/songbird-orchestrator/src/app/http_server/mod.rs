// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP/HTTPS Server Management
//!
//! Handles HTTP and HTTPS server lifecycle including:
//! - TLS configuration and certificate management
//! - Port binding with automatic fallback
//! - Router setup with all API endpoints
//! - Background server spawning
//! - RiboCipher federation dispatch on `:7700`

mod federation_dispatch;
mod port_binding;
mod tls_server;

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
    shared_ipc_handler: Option<Arc<songbird_universal_ipc::service::IpcServiceHandler>>,
) -> Result<u16> {
    let federation_ipc_handler = shared_ipc_handler.as_ref().map(Arc::clone);

    let app = build_router(
        Arc::clone(&federation_state),
        Arc::clone(&federated_service_registry),
        Arc::clone(&service_registry),
        shared_ipc_handler,
    )
    .await?;

    let (listener, actual_addr) = port_binding::bind_with_fallback(&bind_addr).await?;
    let actual_port = actual_addr.port();

    if actual_port == bind_addr.port() {
        info!("✅ Bound to configured port {}", bind_addr.port());
    } else {
        warn!(
            "⚠️  Configured port {} busy, using port {} instead",
            bind_addr.port(),
            actual_port
        );
    }

    let tls_enabled = SafeEnv::get_bool("SONGBIRD_TLS_ENABLED", true);

    if tls_enabled {
        info!("🔐 TLS enabled - configuring HTTPS server (fail-secure by default)");
        match tls_server::start_https_server(
            app.clone(),
            federation_ipc_handler,
            listener,
            actual_addr,
        )
        .await
        {
            Ok(()) => {
                info!("✅ HTTPS server started successfully");
            }
            Err(e) => {
                warn!("⚠️  HTTPS server failed to start: {}", e);
                warn!("   Most likely cause: security provider crypto provider not available");
                warn!("   DEGRADING TO PLAIN HTTP (insecure, but functional)");
                warn!("   To resolve: Start security provider or set SONGBIRD_TLS_ENABLED=false");

                let (fallback_listener, fallback_addr) =
                    port_binding::bind_with_fallback(&bind_addr).await?;
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

    Ok(actual_port)
}

/// Build the Axum router with all API endpoints
async fn build_router(
    federation_state: Arc<FederationState>,
    federated_service_registry: Arc<FederatedServiceRegistry>,
    service_registry: Arc<crate::service_registry::ServiceRegistry>,
    shared_ipc_handler: Option<Arc<songbird_universal_ipc::service::IpcServiceHandler>>,
) -> Result<Router> {
    let deployment_state = crate::server::deployment_api::DeploymentState::new();

    let compute_state = crate::server::compute_api::ComputeApiState::new(
        Arc::clone(&federation_state),
        Arc::clone(&federated_service_registry),
    );

    let compute_router =
        crate::server::compute_api::compute_routes().with_state(compute_state.clone());

    let protocol_state = crate::server::protocol_api::ProtocolApiState::new();

    let protocol_router =
        crate::server::protocol_api::protocol_routes().with_state(protocol_state.clone());

    let _ = std::fs::create_dir_all(crate::env_config::data_dir());
    let task_db_url = crate::env_config::data_dir().join("task_lifecycle.db").display().to_string();
    let task_manager = Arc::new(
        crate::task_lifecycle::TaskLifecycleManager::new(&task_db_url)
            .await
            .map_err(|e| anyhow::anyhow!("task lifecycle database: {e}"))?,
    );
    let consent_manager = Arc::new(crate::consent_management::ConsentManager::new());

    let ipc_handler: Arc<songbird_universal_ipc::service::IpcServiceHandler> =
        if let Some(handler) = shared_ipc_handler {
            handler
        } else {
            let ipc_registry = Arc::new(tokio::sync::RwLock::new(
                songbird_universal_ipc::registry::ServiceRegistry::new(),
            ));
            Arc::new(songbird_universal_ipc::service::IpcServiceHandler::with_federation_state(
                ipc_registry,
                Arc::clone(&federation_state),
            ))
        };

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

    let jsonrpc_router = crate::server::jsonrpc_api::jsonrpc_routes().with_state(jsonrpc_state);

    let event_broadcaster = Arc::new(crate::server::events::EventBroadcaster::new());

    let websocket_state = crate::server::websocket_api::WebSocketApiState::new(
        Arc::clone(&federation_state),
        Arc::clone(&federated_service_registry),
        Arc::clone(&event_broadcaster),
    );

    let websocket_router =
        crate::server::websocket_api::websocket_routes().with_state(websocket_state);

    let service_registry_router =
        crate::server::service_registry_api::service_registry_routes((*service_registry).clone());

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
        .nest(
            "/api/deployment",
            crate::server::deployment_api::deployment_routes(deployment_state),
        )
        .nest(
            "/api",
            crate::server::consent_api::consent_routes().with_state(
                crate::server::consent_api::ConsentApiState::new(Arc::clone(&consent_manager)),
            ),
        )
        .nest(
            "/api/v1",
            crate::server::task_api::task_lifecycle_router(Arc::clone(&task_manager)),
        )
        .nest("/api/v1/services", service_registry_router)
        .merge(info_router)
        .route("/health", axum::routing::get(|| async { "OK" }))
        .layer(axum::extract::DefaultBodyLimit::max(100 * 1024 * 1024)))
}

/// Start plain HTTP server (no TLS)
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
async fn start_http_server_plain(app: Router, listener: tokio::net::TcpListener) -> Result<()> {
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("❌ HTTP server error: {}", e);
        }
    });

    Ok(())
}

/// Get local IP address for certificate SANs.
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
async fn get_local_ip() -> Result<String> {
    crate::network::route_detect::resolve_local_ipv4()
}

/// Start tarpc server for high-performance native RPC.
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
