// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use std::sync::Arc;

    fn sample_job(job_id: Uuid, status: JobStatusType) -> JobStatus {
        JobStatus {
            job_id,
            status,
            routed_to: String::from("local"),
            progress: None,
            started_at: chrono::Utc::now(),
            completed_at: None,
            error: None,
        }
    }

    #[test]
    fn job_status_type_queued_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&JobStatusType::Queued).unwrap(), "\"queued\"");
        assert_eq!(
            serde_json::from_str::<JobStatusType>("\"queued\"").unwrap(),
            JobStatusType::Queued
        );
    }

    #[test]
    fn job_status_type_routing_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&JobStatusType::Routing).unwrap(), "\"routing\"");
        assert_eq!(
            serde_json::from_str::<JobStatusType>("\"routing\"").unwrap(),
            JobStatusType::Routing
        );
    }

    #[test]
    fn job_status_type_running_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&JobStatusType::Running).unwrap(), "\"running\"");
        assert_eq!(
            serde_json::from_str::<JobStatusType>("\"running\"").unwrap(),
            JobStatusType::Running
        );
    }

    #[test]
    fn job_status_type_completed_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&JobStatusType::Completed).unwrap(), "\"completed\"");
        assert_eq!(
            serde_json::from_str::<JobStatusType>("\"completed\"").unwrap(),
            JobStatusType::Completed
        );
    }

    #[test]
    fn job_status_type_failed_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&JobStatusType::Failed).unwrap(), "\"failed\"");
        assert_eq!(
            serde_json::from_str::<JobStatusType>("\"failed\"").unwrap(),
            JobStatusType::Failed
        );
    }

    #[test]
    fn job_status_type_cancelled_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&JobStatusType::Cancelled).unwrap(), "\"cancelled\"");
        assert_eq!(
            serde_json::from_str::<JobStatusType>("\"cancelled\"").unwrap(),
            JobStatusType::Cancelled
        );
    }

    #[tokio::test]
    async fn update_job_status_completed_sets_completed_at() {
        let job_id = Uuid::new_v4();
        let jobs =
            RwLock::new(HashMap::from([(job_id, sample_job(job_id, JobStatusType::Running))]));

        update_job_status(&jobs, job_id, JobStatusType::Completed, None).await;

        let jobs = jobs.read().await;
        let job = jobs.get(&job_id).unwrap();
        assert_eq!(job.status, JobStatusType::Completed);
        assert!(job.completed_at.is_some());
    }

    #[tokio::test]
    async fn update_job_status_failed_sets_completed_at() {
        let job_id = Uuid::new_v4();
        let jobs =
            RwLock::new(HashMap::from([(job_id, sample_job(job_id, JobStatusType::Running))]));

        update_job_status(&jobs, job_id, JobStatusType::Failed, Some("boom".into())).await;

        let jobs = jobs.read().await;
        let job = jobs.get(&job_id).unwrap();
        assert_eq!(job.status, JobStatusType::Failed);
        assert!(job.completed_at.is_some());
        assert_eq!(job.error.as_deref(), Some("boom"));
    }

    #[tokio::test]
    async fn update_job_status_running_does_not_set_completed_at() {
        let job_id = Uuid::new_v4();
        let jobs =
            RwLock::new(HashMap::from([(job_id, sample_job(job_id, JobStatusType::Queued))]));

        update_job_status(&jobs, job_id, JobStatusType::Running, None).await;

        let jobs = jobs.read().await;
        let job = jobs.get(&job_id).unwrap();
        assert_eq!(job.status, JobStatusType::Running);
        assert!(job.completed_at.is_none());
    }

    #[tokio::test]
    async fn api_error_routing_maps_to_internal_server_error() {
        let response = ApiError::Routing("no route".into()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn api_error_execution_maps_to_internal_server_error() {
        let response = ApiError::Execution("task failed".into()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn api_error_not_found_maps_to_404() {
        let response = ApiError::NotFound("missing job".into()).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn api_error_invalid_request_maps_to_bad_request() {
        let response = ApiError::InvalidRequest("bad payload".into()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn compute_task_request_deserializes_with_optional_fields() {
        let json = r#"{
            "task": {
                "task_type": "ml_training",
                "payload": {"epochs": 10}
            },
            "priority": 7,
            "timeout_secs": 120
        }"#;

        let req: ComputeTaskRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.task.task_type.as_ref(), "ml_training");
        assert_eq!(req.task.payload["epochs"], 10);
        assert_eq!(req.priority, Some(7));
        assert_eq!(req.timeout_secs, Some(120));
    }

    #[test]
    fn compute_task_request_deserializes_required_fields_only() {
        let json = r#"{"task": {"task_type": "health_check"}}"#;

        let req: ComputeTaskRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.task.task_type.as_ref(), "health_check");
        assert!(req.priority.is_none());
        assert!(req.timeout_secs.is_none());
    }

    #[test]
    fn compute_task_request_serde_round_trip() {
        let req = ComputeTaskRequest {
            task: Task {
                task_type: Arc::from("data_processing"),
                payload: serde_json::json!({"batch": 1}),
                resource_requirements: None,
                estimated_duration_secs: Some(60),
                metadata: HashMap::new(),
            },
            priority: Some(3),
            timeout_secs: Some(45),
        };

        let json = serde_json::to_string(&req).unwrap();
        let back: ComputeTaskRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.task.task_type.as_ref(), "data_processing");
        assert_eq!(back.priority, Some(3));
        assert_eq!(back.timeout_secs, Some(45));
    }

    #[tokio::test]
    async fn job_store_insert_and_retrieve() {
        let job_id = Uuid::new_v4();
        let jobs: Arc<RwLock<HashMap<Uuid, JobStatus>>> = Arc::new(RwLock::new(HashMap::new()));

        {
            let mut store = jobs.write().await;
            store.insert(job_id, sample_job(job_id, JobStatusType::Queued));
        }

        let store = jobs.read().await;
        let job = store.get(&job_id).expect("job should exist");
        assert_eq!(job.job_id, job_id);
        assert_eq!(job.status, JobStatusType::Queued);
    }

    #[tokio::test]
    async fn job_store_update_status() {
        let job_id = Uuid::new_v4();
        let jobs = Arc::new(RwLock::new(HashMap::from([(
            job_id,
            sample_job(job_id, JobStatusType::Queued),
        )])));

        update_job_status(jobs.as_ref(), job_id, JobStatusType::Running, None).await;

        let store = jobs.read().await;
        let job = store.get(&job_id).unwrap();
        assert_eq!(job.status, JobStatusType::Running);
    }
}
