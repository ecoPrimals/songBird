// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP server for execution agent

use crate::{
    executor::CommandExecutor,
    job_manager::{JobManager, JobStats},
    security::SecurityValidator,
    types::{
        ExecutionRequest, ExecutionResponse, ExecutionStatus, JobInfo, StopJobRequest,
        StopJobResponse,
    },
};
use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use songbird_types::SongbirdError;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

/// Execution server state
#[derive(Clone)]
struct ServerState {
    job_manager: Arc<JobManager>,
    executor: Arc<CommandExecutor>,
    security: Arc<SecurityValidator>,
}

/// HTTP server for remote execution
pub struct ExecutionServer {
    bind_address: String,
    port: u16,
    job_manager: Arc<JobManager>,
    executor: Arc<CommandExecutor>,
    security: Arc<SecurityValidator>,
}

impl ExecutionServer {
    /// Create a new execution server
    #[must_use]
    pub fn new(
        bind_address: String,
        port: u16,
        job_manager: JobManager,
        executor: CommandExecutor,
        auth_token: Option<String>,
    ) -> Self {
        Self {
            bind_address,
            port,
            job_manager: Arc::new(job_manager),
            executor: Arc::new(executor),
            security: Arc::new(SecurityValidator::new(auth_token.is_some(), auth_token)),
        }
    }

    /// Start the execution server
    ///
    /// # Errors
    ///
    /// Returns an error if binding to the address fails
    pub async fn serve(self) -> Result<()> {
        let state = ServerState {
            job_manager: self.job_manager.clone(),
            executor: self.executor.clone(),
            security: self.security.clone(),
        };

        let app = Router::new()
            .route("/health", get(health_check))
            .route("/api/v1/execution/command", post(execute_command))
            .route("/api/v1/execution/jobs", get(list_jobs))
            .route("/api/v1/execution/jobs/{job_id}", get(get_job))
            .route("/api/v1/execution/jobs/{job_id}/stop", post(stop_job))
            .route("/api/v1/execution/stats", get(get_stats))
            .with_state(state)
            .layer(TraceLayer::new_for_http());

        let addr = format!("{}:{}", self.bind_address, self.port);
        info!("Starting execution agent on {}", addr);

        let listener = TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "songbird-execution-agent",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Execute command endpoint
async fn execute_command(
    State(state): State<ServerState>,
    Json(request): Json<ExecutionRequest>,
) -> Result<Json<ExecutionResponse>, AppError> {
    // Validate command
    state.security.validate_command(&request.command)?;

    if request.background {
        // Background execution
        let job = state.executor.execute_background(request).await?;
        state.job_manager.add_job(job.clone()).await?;

        let job_id = job.id.clone();
        tokio::spawn(async move {
            info!("Background job {job_id} started — status tracked in job manager");
        });

        Ok(Json(ExecutionResponse {
            job_id: job.id,
            status: job.status,
            pid: job.pid,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            started_at: job.started_at,
            completed_at: None,
            duration_ms: None,
        }))
    } else {
        // Foreground execution
        let response = state.executor.execute(request).await?;
        Ok(Json(response))
    }
}

/// Get job information
async fn get_job(
    State(state): State<ServerState>,
    Path(job_id): Path<String>,
) -> Result<Json<JobInfo>, AppError> {
    let job = state.job_manager.get_job(&job_id).await?;
    Ok(Json(job))
}

/// List all jobs
async fn list_jobs(State(state): State<ServerState>) -> Result<Json<Vec<JobInfo>>, AppError> {
    let jobs = state.job_manager.list_jobs().await;
    Ok(Json(jobs))
}

/// Stop a job
async fn stop_job(
    State(state): State<ServerState>,
    Path(job_id): Path<String>,
    Json(request): Json<StopJobRequest>,
) -> Result<Json<StopJobResponse>, AppError> {
    let _pid = state.job_manager.stop_job(&job_id).await?;

    Ok(Json(StopJobResponse {
        job_id,
        status: ExecutionStatus::Stopped,
        signal: request.signal.unwrap_or_else(|| "SIGTERM".to_string()),
    }))
}

/// Get statistics
async fn get_stats(State(state): State<ServerState>) -> Result<Json<JobStats>, AppError> {
    let job_stats = state.job_manager.get_stats().await;
    Ok(Json(job_stats))
}

/// Application error wrapper for proper HTTP responses
struct AppError(SongbirdError);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            SongbirdError::Validation {
                ref message,
                ..
            } => (StatusCode::BAD_REQUEST, message.clone()),
            SongbirdError::Security(ref sec_err) => {
                (StatusCode::UNAUTHORIZED, sec_err.message.clone())
            }
            SongbirdError::Registry {
                ref message,
                ..
            } => (StatusCode::NOT_FOUND, message.clone()),
            SongbirdError::Configuration {
                ref message,
                ..
            } => (StatusCode::TOO_MANY_REQUESTS, message.clone()),
            _ => {
                error!("Internal error: {:?}", self.0);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<SongbirdError>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use songbird_types::{SecurityError, SongbirdError};

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["status"], "healthy");
        assert_eq!(v["service"], "songbird-execution-agent");
    }

    #[tokio::test]
    async fn app_error_validation_maps_to_bad_request() {
        let err = AppError(SongbirdError::Validation {
            message: "bad cmd".into(),
            field: Some("command".into()),
            suggestion: None,
        });
        let res = err.into_response();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["error"], "bad cmd");
    }

    #[tokio::test]
    async fn app_error_security_maps_to_unauthorized() {
        let err = AppError(SongbirdError::Security(SecurityError {
            message: "nope".into(),
            operation: None,
            required_permission: None,
            context: None,
            remediation: None,
        }));
        let res = err.into_response();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["error"], "nope");
    }

    #[tokio::test]
    async fn app_error_registry_maps_to_not_found() {
        let err = AppError(SongbirdError::Registry {
            message: "missing job".into(),
            service_name: Some("j1".into()),
            operation: "get".into(),
        });
        let res = err.into_response();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn app_error_configuration_maps_to_too_many_requests() {
        let err = AppError(SongbirdError::Configuration {
            message: "rate".into(),
            field: None,
            suggestion: None,
        });
        let res = err.into_response();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn app_error_other_maps_to_internal_server_error() {
        let err = AppError(SongbirdError::Network {
            message: "down".into(),
            interface: None,
            suggestion: None,
        });
        let res = err.into_response();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["error"], "Internal server error");
    }

    #[test]
    fn execution_server_new_accepts_auth_none() {
        let server = ExecutionServer::new(
            "127.0.0.1".into(),
            0,
            JobManager::new(1, 60),
            CommandExecutor::new(crate::ResourceLimits::default()),
            None,
        );
        let _ = server;
    }
}
