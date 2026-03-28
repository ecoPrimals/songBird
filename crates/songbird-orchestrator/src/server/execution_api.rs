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
use songbird_types::SongbirdResult;
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
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn new() -> SongbirdResult<Self> {
        let manager = ExecutionManager::new().await?;

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
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use crate::core::execution::client::{ExecutionRequest, ExecutionStatus};
    use axum::response::IntoResponse;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn sample_request() -> ExecutionRequest {
        ExecutionRequest {
            id: Some("job-1".to_string()),
            command: "echo hi".to_string(),
            working_dir: Some(PathBuf::from("/tmp")),
            env: HashMap::from([("PATH".to_string(), "/usr/bin".to_string())]),
            background: false,
            timeout_seconds: Some(60),
            capture_output: true,
        }
    }

    #[test]
    fn single_tower_request_serde_roundtrip() {
        let req = SingleTowerRequest {
            tower_endpoint: "https://tower.example/api".to_string(),
            request: sample_request(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: SingleTowerRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tower_endpoint, req.tower_endpoint);
        assert_eq!(back.request.command, req.request.command);
    }

    #[test]
    fn broadcast_request_optional_fields_default_in_handler_logic() {
        let req = BroadcastRequest {
            tower_ids: vec!["a".to_string()],
            request: sample_request(),
            fail_fast: None,
            min_success_rate: None,
            wait_for_completion: None,
        };
        let fail_fast = req.fail_fast.unwrap_or(false);
        let min_sr = req.min_success_rate.unwrap_or(1.0);
        let wait = req.wait_for_completion.unwrap_or(true);
        assert!(!fail_fast);
        assert!((min_sr - 1.0).abs() < f64::EPSILON);
        assert!(wait);
    }

    #[test]
    fn broadcast_request_explicit_options_roundtrip() {
        let req = BroadcastRequest {
            tower_ids: vec!["t1".to_string(), "t2".to_string()],
            request: sample_request(),
            fail_fast: Some(true),
            min_success_rate: Some(0.5),
            wait_for_completion: Some(false),
        };
        let j = serde_json::to_string(&req).unwrap();
        let back: BroadcastRequest = serde_json::from_str(&j).unwrap();
        assert_eq!(back.fail_fast, Some(true));
        assert!((back.min_success_rate.unwrap() - 0.5).abs() < f64::EPSILON);
        assert_eq!(back.wait_for_completion, Some(false));
    }

    #[test]
    fn api_error_display() {
        let e = ApiError::Execution("boom".to_string());
        assert!(format!("{e}").contains("boom"));
        let i = ApiError::InvalidRequest("bad".to_string());
        assert!(format!("{i}").contains("bad"));
    }

    #[test]
    fn api_error_into_response_status() {
        use axum::http::StatusCode;

        let r = ApiError::Execution("e".to_string()).into_response();
        assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let r2 = ApiError::InvalidRequest("i".to_string()).into_response();
        assert_eq!(r2.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn execution_response_serde_shape() {
        let resp = ExecutionResponse {
            job_id: "j1".to_string(),
            status: ExecutionStatus::Completed,
            pid: Some(7),
            exit_code: Some(0),
            stdout: "out".to_string(),
            stderr: String::new(),
            started_at: SystemTime::UNIX_EPOCH,
            completed_at: Some(SystemTime::UNIX_EPOCH),
            duration_ms: Some(10),
        };
        let j = serde_json::to_string(&resp).unwrap();
        assert!(j.contains("j1"));
        assert!(j.contains("completed"));
    }
}
