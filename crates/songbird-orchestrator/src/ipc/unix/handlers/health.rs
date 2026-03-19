// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2025 ecoPrimals
//! Health & Diagnostics Handlers
//!
//! Handlers for health checks and diagnostic information.
//! Provides both legacy (primal.health) and biomeOS-standard (health) methods.

use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

use crate::app::connection_manager::ConnectionManager;
use crate::ipc::jsonrpc::JsonRpcError;
use crate::ipc::primal_registry::PrimalRegistry;

/// Handle primal.health - Get health status (legacy, backward compat)
pub async fn handle_health(
    registry: Arc<RwLock<PrimalRegistry>>,
) -> Result<Value, JsonRpcError> {
    let reg = registry.read().await;
    let primal_count = reg.list_all().await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        .len();
    
    Ok(serde_json::json!({
        "status": "healthy",
        "registered_primals": primal_count,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Handle health - biomeOS-standard health check (Feb 4, 2026)
///
/// NEW: Bare `health` method (no prefix) as required by biomeOS.
/// Returns health status with uptime, peer count, and BearDog connectivity.
///
/// EVOLVED (Phase 5A): Real uptime tracking instead of hardcoded value
pub async fn handle_health_standard(
    registry: Arc<RwLock<PrimalRegistry>>,
    connection_manager: Option<Arc<ConnectionManager>>,
    start_time: Option<Arc<RwLock<std::time::Instant>>>,
) -> Result<Value, JsonRpcError> {
    let reg = registry.read().await;
    let primal_count = reg.list_all().await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        .len();
    
    // Get peer count if connection manager available
    let peers_connected = if let Some(manager) = connection_manager {
        manager.get_peer_count().await
    } else {
        0
    };
    
    // Check BearDog connectivity (best effort)
    let beardog_connected = {
        let beardog_socket = songbird_http_client::discover_beardog_socket();
        std::path::Path::new(&beardog_socket).exists()
    };
    
    // Calculate actual uptime (Phase 5A Evolution - Feb 4, 2026)
    let uptime_seconds = if let Some(start_time_arc) = start_time {
        let start_time_guard = start_time_arc.read().await;
        start_time_guard.elapsed().as_secs()
    } else {
        // Fallback if start_time not available (shouldn't happen in production)
        warn!("⚠️  Start time not available, using estimated uptime");
        3600
    };
    
    Ok(serde_json::json!({
        "status": "healthy",
        "uptime_seconds": uptime_seconds,
        "peers_connected": peers_connected,
        "beardog_connected": beardog_connected,
        "registered_primals": primal_count
    }))
}

/// Handle primal.ping - Simple ping/pong
pub async fn handle_ping() -> Result<Value, JsonRpcError> {
    Ok(serde_json::json!({
        "pong": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
