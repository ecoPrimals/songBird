//! # Songbird Rendezvous Server
//!
//! **Privacy-First Rendezvous Server** for internet-wide Songbird federation.
//!
//! ## Purpose
//!
//! Enables Songbird nodes to discover and connect across the internet without
//! exposing IP addresses publicly. Acts as a coordinator for peer-to-peer
//! connections.
//!
//! ## Architecture
//!
//! ```text
//! Node A ──► Rendezvous ◄── Node B
//!   │                        │
//!   └────► Direct P2P ◄──────┘
//!          (after coordination)
//! ```
//!
//! ## Security Model
//!
//! - **Honest but Curious**: Rendezvous follows protocol but may log everything
//! - **Zero Trust**: End-to-end encryption, rendezvous can't read content
//! - **Signed Messages**: All messages cryptographically signed (via BearDog)
//! - **Ephemeral Sessions**: Session IDs rotate every 10-15 minutes
//! - **No IP Exposure**: Nodes never share IPs via rendezvous

mod api;
mod coordination;
mod messages;
mod registry;
mod security;
mod websocket;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::registry::SessionRegistry;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder().with_max_level(Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🌍 Starting Songbird Rendezvous Server");
    info!("   Privacy-First Internet Federation");

    // Create session registry
    let registry = Arc::new(SessionRegistry::new());

    // Start registry cleanup task
    let cleanup_registry = Arc::clone(&registry);
    tokio::spawn(async move {
        cleanup_registry.start_cleanup_loop().await;
    });

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        .route("/", get(root))
        // Registration endpoints
        .route("/api/v1/register", post(api::register_presence))
        .route("/api/v1/heartbeat", post(api::heartbeat))
        // Query endpoints
        .route("/api/v1/query", post(api::query_peers))
        .route("/api/v1/peers/:session_id", get(api::get_peer_info))
        // Connection coordination
        .route("/api/v1/connect", post(api::request_connection))
        .route("/api/v1/respond", post(api::respond_connection))
        // WebSocket for real-time coordination
        .route("/ws/:session_id", get(websocket::websocket_handler))
        // Middleware
        .layer(CorsLayer::new().allow_origin(Any))
        .layer(TraceLayer::new_for_http())
        // Shared state
        .with_state(registry);

    // Listen on all interfaces (for internet deployment)
    let addr = SocketAddr::from(([0, 0, 0, 0], 8888));
    info!("🚀 Listening on {}", addr);
    info!("   Endpoints:");
    info!("      POST /api/v1/register - Register node presence");
    info!("      POST /api/v1/query    - Query for peers");
    info!("      POST /api/v1/connect  - Request connection");
    info!("      WS   /ws/:session_id  - Real-time coordination");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn root() -> &'static str {
    "Songbird Rendezvous Server - Privacy-First Federation"
}
