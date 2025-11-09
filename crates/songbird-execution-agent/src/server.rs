//! HTTP server for execution agent

use crate::{
    executor::CommandExecutor,
    job_manager::{JobManager, JobStats},
    security::SecurityValidator,
    types::*,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
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
    
    /// Start the server
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let state = ServerState {
            job_manager: self.job_manager.clone(),
            executor: self.executor.clone(),
            security: self.security.clone(),
        };
        
        let app = Router::new()
            .route("/health", get(health_check))
            .route("/api/v1/execution/command", post(execute_command))
            .route("/api/v1/execution/jobs", get(list_jobs))
            .route("/api/v1/execution/jobs/:job_id", get(get_job))
            .route("/api/v1/execution/jobs/:job_id/stop", post(stop_job))
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
        
        // Spawn task to monitor the job
        let job_id = job.id.clone();
        tokio::spawn(async move {
            // In a real implementation, we'd monitor the process and update job status
            // For now, this is a placeholder
            info!("Background job {} is running", job_id);
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
async fn list_jobs(
    State(state): State<ServerState>,
) -> Result<Json<Vec<JobInfo>>, AppError> {
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
async fn get_stats(
    State(state): State<ServerState>,
) -> Result<Json<JobStats>, AppError> {
    let stats = state.job_manager.get_stats().await;
    Ok(Json(stats))
}

/// Application error wrapper for proper HTTP responses
struct AppError(SongbirdError);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            SongbirdError::Validation { ref message, .. } => {
                (StatusCode::BAD_REQUEST, message.clone())
            }
            SongbirdError::Security(ref sec_err) => {
                (StatusCode::UNAUTHORIZED, sec_err.message.clone())
            }
            SongbirdError::Registry { ref message, .. } => {
                (StatusCode::NOT_FOUND, message.clone())
            }
            SongbirdError::Configuration { ref message, .. } => {
                (StatusCode::TOO_MANY_REQUESTS, message.clone())
            }
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
mod tests {
    use super::*;
    use crate::{AgentConfig, ResourceLimits};

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

