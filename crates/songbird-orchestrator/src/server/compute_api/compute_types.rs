// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Compute API types, job model, and helpers
//!
//! Domain types shared between the compute HTTP handlers and tests.
//! Extracted from `compute_api.rs` for cohesion and file-size hygiene.

use crate::core::routing::Task;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;
use uuid::Uuid;

/// Request to submit a compute task
#[derive(Debug, Serialize, Deserialize)]
pub struct ComputeTaskRequest {
    /// Task to execute
    pub task: Task,
    /// Priority (0-10, higher is more important)
    pub priority: Option<u8>,
    /// Timeout in seconds
    pub timeout_secs: Option<u64>,
}

/// Response from task submission
#[derive(Debug, Serialize, Deserialize)]
pub struct ComputeTaskResponse {
    /// Unique job identifier
    pub job_id: Uuid,
    /// Where the task was routed to
    pub routed_to: String,
    /// Current job status
    pub status: String,
    /// Estimated completion time (if known)
    pub estimated_completion: Option<String>,
}

/// Job status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    /// Job identifier
    pub job_id: Uuid,
    /// Current status
    pub status: JobStatusType,
    /// Where the job was routed
    pub routed_to: String,
    /// Progress percentage (0.0 - 1.0)
    pub progress: Option<f64>,
    /// When the job started
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// When the job completed (if finished)
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Job status types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatusType {
    /// Job is queued
    Queued,
    /// Job is routing to executor
    Routing,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed with an error
    Failed,
    /// Job was cancelled
    Cancelled,
}

/// Update a job's status in the active jobs registry
pub async fn update_job_status(
    active_jobs: &RwLock<HashMap<Uuid, JobStatus>>,
    job_id: Uuid,
    status: JobStatusType,
    error: Option<String>,
) {
    let mut jobs = active_jobs.write().await;
    if let Some(job) = jobs.get_mut(&job_id) {
        job.status = status;
        if status == JobStatusType::Completed || status == JobStatusType::Failed {
            job.completed_at = Some(chrono::Utc::now());
        }
        if let Some(err) = error {
            job.error = Some(err);
        }
    }
}

/// Discover an HTTP client via the crypto provider socket
pub async fn discover_http_client(
    active_jobs: &Arc<RwLock<HashMap<Uuid, JobStatus>>>,
    job_id: Uuid,
) -> Option<songbird_http_client::SongbirdHttpClient> {
    match crate::primal_discovery::discover_crypto_provider().await {
        Ok(socket) => Some(songbird_http_client::SongbirdHttpClient::new(socket)),
        Err(e) => {
            warn!("Failed to discover crypto provider: {e}");
            update_job_status(
                active_jobs.as_ref(),
                job_id,
                JobStatusType::Failed,
                Some(format!("Crypto provider discovery failed: {e}")),
            )
            .await;
            None
        }
    }
}

/// Serialize a task to JSON, recording failure in the job registry
pub async fn serialize_task(
    task: &Task,
    active_jobs: &Arc<RwLock<HashMap<Uuid, JobStatus>>>,
    job_id: Uuid,
) -> Option<serde_json::Value> {
    match serde_json::to_value(task) {
        Ok(json) => Some(json),
        Err(e) => {
            warn!("Failed to serialize task {job_id}: {e}");
            update_job_status(
                active_jobs.as_ref(),
                job_id,
                JobStatusType::Failed,
                Some(format!("Task serialization failed: {e}")),
            )
            .await;
            None
        }
    }
}

/// API errors for the compute endpoint
#[derive(Debug)]
pub enum ApiError {
    /// Task routing failed
    Routing(String),
    /// Task execution failed
    Execution(String),
    /// Invalid request
    InvalidRequest(String),
    /// Resource not found
    NotFound(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Routing(msg) => write!(f, "Routing error: {msg}"),
            Self::Execution(msg) => write!(f, "Execution error: {msg}"),
            Self::InvalidRequest(msg) => write!(f, "Invalid request: {msg}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
        }
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Routing(msg) | Self::Execution(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            Self::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };

        let body = Json(serde_json::json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}
