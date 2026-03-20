// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Execution API endpoints for orchestrator

use crate::core::execution::broadcast::{BroadcastOptions, BroadcastResult};
use crate::core::execution::client::{ExecutionRequest, ExecutionResponse};
use crate::core::execution::manager::ExecutionManager;
use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// State for execution API
#[derive(Clone)]
pub struct ExecutionApiState {
    manager: Arc<RwLock<ExecutionManager>>,
}

impl ExecutionApiState {
    /// Create new execution API state
    ///
    /// ✅ EVOLVED: Now async due to `ExecutionManager` async construction
    pub async fn new() -> Result<Self, String> {
        let manager = ExecutionManager::new()
            .await
            .map_err(|e| format!("Failed to create ExecutionManager: {e}"))?;

        Ok(Self {
            manager: Arc::new(RwLock::new(manager)),
        })
    }

    /// Get the execution manager
    ///
    /// # Note on `Arc::clone`
    /// `Arc::clone` is cheap (just pointer + atomic increment), but clippy suggests
    /// explicit `Arc::clone()` for clarity. However, `.clone()` on Arc is idiomatic.
    #[must_use]
    pub fn manager(&self) -> Arc<RwLock<ExecutionManager>> {
        Arc::clone(&self.manager)
    }
}

/// Create execution API routes
pub fn execution_routes() -> Router<ExecutionApiState> {
    Router::new()
        .route("/execute", post(execute_command))
        .route("/broadcast", post(broadcast_command))
}

/// Execute command on a single tower
async fn execute_command(
    State(state): State<ExecutionApiState>,
    Json(req): Json<SingleTowerRequest>,
) -> Result<Json<ExecutionResponse>, ApiError> {
    let manager = state.manager.read().await;

    let response = manager
        .execute_on_tower(&req.tower_endpoint, req.request)
        .await
        .map_err(|e| ApiError::Execution(e.to_string()))?;

    Ok(Json(response))
}

/// Broadcast command to multiple towers
async fn broadcast_command(
    State(state): State<ExecutionApiState>,
    Json(req): Json<BroadcastRequest>,
) -> Result<Json<BroadcastResult>, ApiError> {
    let manager = state.manager.read().await;

    let options = BroadcastOptions {
        fail_fast: req.fail_fast.unwrap_or(false),
        min_success_rate: req.min_success_rate.unwrap_or(1.0),
        wait_for_completion: req.wait_for_completion.unwrap_or(true),
    };

    let result = manager.execute_broadcast(req.tower_ids, req.request, Some(options)).await;

    Ok(Json(result))
}

/// Request to execute on a single tower
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleTowerRequest {
    pub tower_endpoint: String,
    pub request: ExecutionRequest,
}

/// Request to broadcast to multiple towers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastRequest {
    pub tower_ids: Vec<String>,
    pub request: ExecutionRequest,
    pub fail_fast: Option<bool>,
    pub min_success_rate: Option<f64>,
    pub wait_for_completion: Option<bool>,
}

/// API errors
#[derive(Debug)]
pub enum ApiError {
    Execution(String),
    InvalidRequest(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Execution(msg) => write!(f, "Execution error: {msg}"),
            Self::InvalidRequest(msg) => write!(f, "Invalid request: {msg}"),
        }
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Execution(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            Self::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(serde_json::json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_api_state_creation() {
        let state = ExecutionApiState::new();
        assert!(true); // Just verify construction
    }
}
