// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Compute API - Unified endpoint for intelligent task routing
//!
//! This API provides a single entry point for submitting compute tasks.
//! Tasks are intelligently routed based on complexity:
//! - Lightweight → Local or peer Songbird
//! - Moderate → Peer Songbird (with fallback)
//! - Heavy → Specialized capability (Toadstool, etc.)
//! - External providers → Registered capability providers

use crate::core::registry::CapabilityRegistry;
use crate::core::routing::enhanced_router::EnhancedCapabilityRouter;
use crate::core::routing::{CapabilityRouter, RoutingDecision, Task};
use crate::service_registry::ServiceRegistry;
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// State for compute API
#[derive(Clone)]
pub struct ComputeApiState {
    /// Capability router for intelligent task routing
    router: Arc<CapabilityRouter>,
    /// Enhanced router with Universal Port Authority (optional, preferred)
    enhanced_router: Option<Arc<EnhancedCapabilityRouter>>,
    /// Active job tracking
    active_jobs: Arc<RwLock<HashMap<Uuid, JobStatus>>>,
}

impl ComputeApiState {
    /// Create new compute API state
    #[must_use]
    pub fn new(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
    ) -> Self {
        let router = Arc::new(CapabilityRouter::new(federation_state, service_registry));
        Self {
            router,
            enhanced_router: None,
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create new compute API state with capability registry
    #[must_use]
    pub fn with_capability_registry(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
        capability_registry: Arc<CapabilityRegistry>,
    ) -> Self {
        let router = Arc::new(CapabilityRouter::with_capability_registry(
            federation_state,
            service_registry,
            capability_registry,
        ));
        Self {
            router,
            enhanced_router: None,
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create new compute API state with Enhanced Router (Universal Port Authority)
    /// This is the modern, preferred constructor that uses the Enhanced Router
    #[must_use]
    pub fn with_enhanced_router(
        federation_state: Arc<FederationState>,
        federated_service_registry: Arc<FederatedServiceRegistry>,
        service_registry: Arc<ServiceRegistry>,
        capability_registry: Option<Arc<CapabilityRegistry>>,
    ) -> Self {
        // Create enhanced router (modern approach with UPA)
        let enhanced_router = Arc::new(EnhancedCapabilityRouter::new(
            service_registry,
            Arc::clone(&federation_state),
            Arc::clone(&federated_service_registry),
        ));

        // Create legacy router for fallback
        let router = if let Some(cap_registry) = capability_registry {
            Arc::new(CapabilityRouter::with_capability_registry(
                federation_state,
                federated_service_registry,
                cap_registry,
            ))
        } else {
            Arc::new(CapabilityRouter::new(federation_state, federated_service_registry))
        };

        Self {
            router,
            enhanced_router: Some(enhanced_router),
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Create compute API routes
pub fn compute_routes() -> Router<ComputeApiState> {
    Router::new()
        .route("/task", post(submit_compute_task))
        .route("/task/:job_id", get(get_task_status))
}

/// Human-readable routing destination label (mirrors `submit_compute_task` mapping).
#[must_use]
pub(crate) fn format_compute_routed_destination(decision: &RoutingDecision) -> String {
    match decision {
        RoutingDecision::ExecuteLocally => "local".to_string(),
        RoutingDecision::RouteToSongbird {
            node_id,
            ..
        } => format!("songbird:{node_id}"),
        RoutingDecision::RouteToRegisteredService {
            service_name,
            port,
            ..
        } => format!("service:{service_name}:{port}"),
        RoutingDecision::RouteToCapability {
            capability_type,
            provider_endpoint,
        } => format!("{capability_type:?}:{provider_endpoint}"),
        RoutingDecision::RouteToExternalProvider {
            provider_id,
            ..
        } => format!("external:{provider_id}"),
    }
}

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// Submit a compute task for intelligent routing
#[tracing::instrument(skip(state, req), fields(task_type = %req.task.task_type))]
async fn submit_compute_task(
    State(state): State<ComputeApiState>,
    Json(req): Json<ComputeTaskRequest>,
) -> Result<Json<ComputeTaskResponse>, ApiError> {
    info!("Received compute task: {}", req.task.task_type);

    // Generate unique job ID
    let job_id = Uuid::new_v4();

    // Route the task intelligently
    // Prefer enhanced router if available (Universal Port Authority)
    let routing_decision = if let Some(enhanced) = &state.enhanced_router {
        info!("Using Enhanced Router with Universal Port Authority");
        enhanced.route_task(&req.task).await
    } else {
        debug!("Using legacy router");
        state.router.route_task(&req.task).await
    }
    .map_err(|e| ApiError::Routing(e.to_string()))?;

    debug!("Routing decision for job {}: {:?}", job_id, routing_decision);

    // Determine where the task was routed
    let routed_to = format_compute_routed_destination(&routing_decision);

    // Create job status
    let job_status = JobStatus {
        job_id,
        status: JobStatusType::Routing,
        routed_to: routed_to.clone(),
        progress: None,
        started_at: chrono::Utc::now(),
        completed_at: None,
        error: None,
    };

    // Store job status
    state.active_jobs.write().await.insert(job_id, job_status.clone());

    info!("Task {} routed to: {}", job_id, routed_to);

    // Execute the task based on routing decision
    // All execution is now async with proper status tracking
    // Note: Arc clones are cheap (pointer + atomic increment), not deep clones
    let router_clone = Arc::clone(&state.router);
    let active_jobs_clone = Arc::clone(&state.active_jobs);
    let task_clone = req.task.clone();

    match &routing_decision {
        RoutingDecision::ExecuteLocally => {
            // Local execution: mark as running and execute in background
            info!("Executing task {} locally", job_id);

            // Update to running immediately
            {
                let mut jobs = state.active_jobs.write().await;
                if let Some(status) = jobs.get_mut(&job_id) {
                    status.status = JobStatusType::Running;
                }
            }

            // Spawn async task for local execution using CommandExecutor
            //
            // ## Integration Pattern
            // We use the execution-agent's CommandExecutor for actual command execution.
            // This provides:
            // - Resource limiting
            // - Timeout handling
            // - Output capture
            // - Process management
            //
            // ## Future Evolution
            // When orchestrator has access to a JobManager, we can:
            // 1. Track long-running background jobs
            // 2. Support job cancellation
            // 3. Stream output updates
            // 4. Persist job state
            tokio::spawn(async move {
                use songbird_execution_agent::{
                    CommandExecutor, ExecutionRequest, ExecutionStatus, ResourceLimits,
                };

                // Create executor with reasonable defaults
                let limits = ResourceLimits {
                    max_memory_mb: Some(1024),       // 1GB per task
                    max_cpu_time_seconds: Some(300), // 5 minutes
                    default_timeout_seconds: 60,     // 1 minute default
                };
                let executor = CommandExecutor::new(limits);

                // Prepare execution request using builder pattern
                // The task_type contains the command to execute
                let exec_request =
                    ExecutionRequest::new(task_clone.task_type.as_ref()).with_timeout(60); // 1 minute timeout

                // Execute the command
                let result = executor.execute(exec_request).await;

                // Update job status based on execution result
                let mut jobs = active_jobs_clone.write().await;
                if let Some(status) = jobs.get_mut(&job_id) {
                    match result {
                        Ok(response) => {
                            // Check execution status
                            match response.status {
                                ExecutionStatus::Completed => {
                                    status.status = JobStatusType::Completed;
                                    info!(
                                        "Task {} completed successfully (exit code: {:?})",
                                        job_id, response.exit_code
                                    );
                                }
                                ExecutionStatus::Failed | ExecutionStatus::Timeout => {
                                    status.status = JobStatusType::Failed;
                                    warn!(
                                        "Task {} failed (status: {}, exit code: {:?}): {}",
                                        job_id,
                                        response.status,
                                        response.exit_code,
                                        response.stderr
                                    );
                                }
                                _ => {
                                    // Should not happen for synchronous execution
                                    status.status = JobStatusType::Failed;
                                    warn!(
                                        "Task {} in unexpected state: {}",
                                        job_id, response.status
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            status.status = JobStatusType::Failed;
                            error!("Task {} execution error: {}", job_id, e);
                        }
                    }
                    status.completed_at = Some(chrono::Utc::now());
                }
            });
        }

        RoutingDecision::RouteToRegisteredService {
            service_id,
            service_name,
            endpoint,
            port,
        } => {
            // Route to service registered via Universal Port Authority
            info!(
                "Routing task {} to registered service {} (ID: {}) at {}:{}",
                job_id, service_name, service_id, endpoint, port
            );

            let endpoint_clone = endpoint.clone();
            let service_name_clone = service_name.clone();
            let port_clone = *port;

            tokio::spawn(async move {
                // Update to running
                {
                    let mut jobs = active_jobs_clone.write().await;
                    if let Some(status) = jobs.get_mut(&job_id) {
                        status.status = JobStatusType::Running;
                    }
                }

                // Send task to registered service (Pure Rust HTTP via Tower Atomic)
                let crypto_socket = match crate::primal_discovery::discover_crypto_provider().await
                {
                    Ok(socket) => socket,
                    Err(e) => {
                        warn!(
                            "Failed to discover crypto provider: {}, task {} may fail",
                            e, job_id
                        );
                        // Update job status to failed
                        let mut jobs = active_jobs_clone.write().await;
                        if let Some(status) = jobs.get_mut(&job_id) {
                            status.status = JobStatusType::Failed;
                            status.error = Some(format!("Crypto provider discovery failed: {e}"));
                        }
                        return;
                    }
                };

                let client = songbird_http_client::SongbirdHttpClient::new(crypto_socket);
                let service_url = format!("http://{endpoint_clone}:{port_clone}/execute");

                let task_json = match serde_json::to_value(&task_clone) {
                    Ok(json) => json,
                    Err(e) => {
                        warn!("Failed to serialize task {}: {}", job_id, e);
                        let mut jobs = active_jobs_clone.write().await;
                        if let Some(status) = jobs.get_mut(&job_id) {
                            status.status = JobStatusType::Failed;
                            status.error = Some(format!("Task serialization failed: {e}"));
                        }
                        return;
                    }
                };

                let result = client.post(&service_url, task_json).await;

                // Update job status with result
                let mut jobs = active_jobs_clone.write().await;
                if let Some(status) = jobs.get_mut(&job_id) {
                    match result {
                        Ok(response) if response.status >= 200 && response.status < 300 => {
                            status.status = JobStatusType::Completed;
                            status.completed_at = Some(chrono::Utc::now());
                            info!("Task {} completed on service {}", job_id, service_name_clone);
                        }
                        Ok(response) => {
                            status.status = JobStatusType::Failed;
                            status.error = Some(format!(
                                "Service {} returned error: {}",
                                service_name_clone, response.status
                            ));
                            warn!(
                                "Task {} failed on service {}: {}",
                                job_id, service_name_clone, response.status
                            );
                        }
                        Err(e) => {
                            status.status = JobStatusType::Failed;
                            status.error = Some(format!(
                                "HTTP request to service {service_name_clone} failed: {e}"
                            ));
                            warn!(
                                "Task {} HTTP error to service {}: {}",
                                job_id, service_name_clone, e
                            );
                        }
                    }
                }
            });
        }

        RoutingDecision::RouteToSongbird {
            endpoint,
            node_id,
        } => {
            // Forward to peer Songbird via HTTP
            info!("Forwarding task {} to Songbird {} at {}", job_id, node_id, endpoint);

            let endpoint_clone = endpoint.clone();
            let node_id_clone = node_id.clone();

            tokio::spawn(async move {
                // Update to running
                {
                    let mut jobs = active_jobs_clone.write().await;
                    if let Some(status) = jobs.get_mut(&job_id) {
                        status.status = JobStatusType::Running;
                    }
                }

                // Forward task via HTTP POST to peer's /task endpoint (Pure Rust HTTP)
                let crypto_socket = match crate::primal_discovery::discover_crypto_provider().await
                {
                    Ok(socket) => socket,
                    Err(e) => {
                        warn!("Failed to discover crypto provider for peer forward: {}", e);
                        let mut jobs = active_jobs_clone.write().await;
                        if let Some(status) = jobs.get_mut(&job_id) {
                            status.status = JobStatusType::Failed;
                            status.error = Some(format!("Crypto provider discovery failed: {e}"));
                        }
                        return;
                    }
                };

                let client = songbird_http_client::SongbirdHttpClient::new(crypto_socket);
                let forward_url = format!("{endpoint_clone}/task");

                let task_json = match serde_json::to_value(&task_clone) {
                    Ok(json) => json,
                    Err(e) => {
                        warn!("Failed to serialize task for forward {}: {}", job_id, e);
                        let mut jobs = active_jobs_clone.write().await;
                        if let Some(status) = jobs.get_mut(&job_id) {
                            status.status = JobStatusType::Failed;
                            status.error = Some(format!("Task serialization failed: {e}"));
                        }
                        return;
                    }
                };

                // Note: SongbirdHttpClient doesn't need .send() - post() returns the future directly
                let result = tokio::time::timeout(
                    tokio::time::Duration::from_secs(300),
                    client.post(&forward_url, task_json),
                )
                .await;

                // Update job status with result
                let mut jobs = active_jobs_clone.write().await;
                if let Some(status) = jobs.get_mut(&job_id) {
                    match result {
                        Ok(Ok(response)) if response.status >= 200 && response.status < 300 => {
                            status.status = JobStatusType::Completed;
                            status.completed_at = Some(chrono::Utc::now());
                            info!("Task {} completed on peer {}", job_id, node_id_clone);
                        }
                        Ok(Ok(response)) => {
                            status.status = JobStatusType::Failed;
                            status.error =
                                Some(format!("Peer returned error: {}", response.status));
                            status.completed_at = Some(chrono::Utc::now());
                            warn!(
                                "Task {} failed on peer {}: {}",
                                job_id, node_id_clone, response.status
                            );
                        }
                        Ok(Err(e)) => {
                            status.status = JobStatusType::Failed;
                            status.error = Some(format!("HTTP request to peer failed: {e}"));
                            status.completed_at = Some(chrono::Utc::now());
                            warn!("Task {} HTTP error to peer {}: {}", job_id, node_id_clone, e);
                        }
                        Err(_timeout) => {
                            status.status = JobStatusType::Failed;
                            status.error = Some(format!(
                                "Forward to peer {node_id_clone} timed out after 300s"
                            ));
                            status.completed_at = Some(chrono::Utc::now());
                            warn!("Task {} forward to peer {} timed out", job_id, node_id_clone);
                        }
                    }
                }
            });
        }

        RoutingDecision::RouteToCapability {
            provider_endpoint,
            capability_type,
        } => {
            // Forward to capability provider (Toadstool, security provider, etc.)
            info!(
                "Forwarding task {} to {:?} capability at {}",
                job_id, capability_type, provider_endpoint
            );

            let endpoint_clone = provider_endpoint.clone();

            tokio::spawn(async move {
                let result =
                    router_clone.execute_on_external_provider(&endpoint_clone, &task_clone).await;

                // Update job status with result
                let mut jobs = active_jobs_clone.write().await;
                if let Some(status) = jobs.get_mut(&job_id) {
                    match result {
                        Ok(_data) => {
                            status.status = JobStatusType::Completed;
                            status.completed_at = Some(chrono::Utc::now());
                            info!("Task {} completed on capability provider", job_id);
                        }
                        Err(e) => {
                            status.status = JobStatusType::Failed;
                            status.error = Some(e.to_string());
                            status.completed_at = Some(chrono::Utc::now());
                            warn!("Task {} failed on capability provider: {}", job_id, e);
                        }
                    }
                }
            });
        }

        RoutingDecision::RouteToExternalProvider {
            execution_endpoint,
            provider_id,
            ..
        } => {
            // Execute on external provider (fully implemented)
            info!(
                "Executing task {} on external provider {} at {}",
                job_id, provider_id, execution_endpoint
            );

            let endpoint_clone = execution_endpoint.clone();

            tokio::spawn(async move {
                // Update status to running
                {
                    let mut jobs = active_jobs_clone.write().await;
                    if let Some(status) = jobs.get_mut(&job_id) {
                        status.status = JobStatusType::Running;
                    }
                }

                let result =
                    router_clone.execute_on_external_provider(&endpoint_clone, &task_clone).await;

                // Update job status with result
                let mut jobs = active_jobs_clone.write().await;
                if let Some(status) = jobs.get_mut(&job_id) {
                    match result {
                        Ok(_data) => {
                            status.status = JobStatusType::Completed;
                            status.completed_at = Some(chrono::Utc::now());
                            info!("Task {} completed successfully", job_id);
                        }
                        Err(e) => {
                            status.status = JobStatusType::Failed;
                            status.error = Some(e.to_string());
                            status.completed_at = Some(chrono::Utc::now());
                            warn!("Task {} failed: {}", job_id, e);
                        }
                    }
                }
            });
        }
    }

    Ok(Json(ComputeTaskResponse {
        job_id,
        routed_to,
        status: "routing".to_string(),
        estimated_completion: None,
    }))
}

/// Get the status of a compute task
#[tracing::instrument(skip(state))]
async fn get_task_status(
    State(state): State<ComputeApiState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobStatus>, ApiError> {
    debug!("Querying status for job: {}", job_id);

    let jobs = state.active_jobs.read().await;
    let job_status = jobs
        .get(&job_id)
        .ok_or_else(|| ApiError::NotFound(format!("Job {job_id} not found")))?
        .clone();

    Ok(Json(job_status))
}

/// API errors
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
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use songbird_config::capability_endpoints::CapabilityType;

    fn create_test_state() -> ComputeApiState {
        let federation_state = Arc::new(FederationState::new("default".to_string()));
        let service_registry = Arc::new(FederatedServiceRegistry::new());
        ComputeApiState::new(federation_state, service_registry)
    }

    #[tokio::test]
    async fn test_submit_lightweight_task() {
        let state = create_test_state();

        let req = ComputeTaskRequest {
            task: Task::new("health_check"),
            priority: Some(5),
            timeout_secs: Some(30),
        };

        let response = submit_compute_task(State(state.clone()), Json(req))
            .await
            .expect("Test task submission should succeed");

        assert_eq!(response.status, "routing");
        assert_eq!(response.routed_to, "local");

        // Verify job was stored
        let jobs = state.active_jobs.read().await;
        assert!(jobs.contains_key(&response.job_id));
    }

    #[tokio::test]
    async fn test_submit_heavy_task() {
        // Set compute capability endpoint for test
        songbird_process_env::set_var("CAPABILITY_COMPUTE_ENDPOINT", "http://localhost:9000");

        let state = create_test_state();

        let req = ComputeTaskRequest {
            task: Task::builder("ml_training")
                .with_gpu()
                .with_cpu(8.0)
                .with_memory(16384)
                .with_duration(600)
                .build(),
            priority: Some(10),
            timeout_secs: Some(1800),
        };

        let response = submit_compute_task(State(state.clone()), Json(req))
            .await
            .expect("heavy task should route");

        assert_eq!(response.status, "routing");
        // Should route to capability (Compute)
        assert!(response.routed_to.starts_with("Compute:"));

        // Cleanup
        songbird_process_env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    }

    #[tokio::test]
    async fn test_get_task_status_not_found() {
        let state = create_test_state();
        let non_existent_id = Uuid::new_v4();

        let result = get_task_status(State(state), Path(non_existent_id)).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::Routing("Test error".to_string());
        assert_eq!(err.to_string(), "Routing error: Test error");
    }

    #[test]
    fn test_api_error_into_response_status_codes() {
        let cases = [
            (ApiError::Routing("r".into()), StatusCode::INTERNAL_SERVER_ERROR),
            (ApiError::Execution("e".into()), StatusCode::INTERNAL_SERVER_ERROR),
            (ApiError::InvalidRequest("i".into()), StatusCode::BAD_REQUEST),
            (ApiError::NotFound("n".into()), StatusCode::NOT_FOUND),
        ];
        for (err, expected) in cases {
            let resp = err.into_response();
            assert_eq!(resp.status(), expected);
        }
    }

    #[test]
    fn test_job_status_type_serde_lowercase_roundtrip() {
        let json = serde_json::to_string(&JobStatusType::Queued).expect("serialize");
        assert_eq!(json, "\"queued\"");
        let back: JobStatusType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, JobStatusType::Queued);
    }

    #[test]
    fn job_status_type_all_variants_serde() {
        let cases = [
            (JobStatusType::Queued, "\"queued\""),
            (JobStatusType::Routing, "\"routing\""),
            (JobStatusType::Running, "\"running\""),
            (JobStatusType::Completed, "\"completed\""),
            (JobStatusType::Failed, "\"failed\""),
            (JobStatusType::Cancelled, "\"cancelled\""),
        ];
        for (variant, expected) in cases {
            let j = serde_json::to_string(&variant).expect("serialize");
            assert_eq!(j, expected);
            let back: JobStatusType = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn compute_task_request_roundtrip() {
        let req = ComputeTaskRequest {
            task: Task::new("ping"),
            priority: Some(3),
            timeout_secs: Some(12),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let back: ComputeTaskRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.priority, Some(3));
        assert_eq!(back.timeout_secs, Some(12));
    }

    #[test]
    fn api_error_is_std_error() {
        let err = ApiError::Routing("x".into());
        let _: &dyn std::error::Error = &err;
        assert_eq!(format!("{err}"), "Routing error: x");
    }

    #[test]
    fn format_compute_routed_destination_all_variants() {
        assert_eq!(format_compute_routed_destination(&RoutingDecision::ExecuteLocally), "local");
        assert_eq!(
            format_compute_routed_destination(&RoutingDecision::RouteToSongbird {
                node_id: "n1".to_string(),
                endpoint: "https://peer".to_string(),
            }),
            "songbird:n1"
        );
        assert_eq!(
            format_compute_routed_destination(&RoutingDecision::RouteToRegisteredService {
                service_id: "sid".to_string(),
                service_name: "toad".to_string(),
                endpoint: "127.0.0.1".to_string(),
                port: 8080,
            }),
            "service:toad:8080"
        );
        assert_eq!(
            format_compute_routed_destination(&RoutingDecision::RouteToCapability {
                capability_type: CapabilityType::Compute,
                provider_endpoint: "unix:///run/c.sock".to_string(),
            })
            .contains("Compute"),
            true
        );
        assert_eq!(
            format_compute_routed_destination(&RoutingDecision::RouteToExternalProvider {
                provider_id: "prov-1".to_string(),
                execution_endpoint: "https://x/exec".to_string(),
                capability_name: "compute_heavy".to_string(),
            }),
            "external:prov-1"
        );
    }

    #[test]
    fn api_error_display_covers_all_variants() {
        assert_eq!(ApiError::Execution("e".into()).to_string(), "Execution error: e");
        assert_eq!(ApiError::InvalidRequest("bad".into()).to_string(), "Invalid request: bad");
        assert_eq!(ApiError::NotFound("n".into()).to_string(), "Not found: n");
    }

    #[test]
    fn compute_task_response_roundtrip_json() -> Result<(), serde_json::Error> {
        let id = Uuid::nil();
        let resp = ComputeTaskResponse {
            job_id: id,
            routed_to: "local".to_string(),
            status: "routing".to_string(),
            estimated_completion: None,
        };
        let json = serde_json::to_string(&resp)?;
        let back: ComputeTaskResponse = serde_json::from_str(&json)?;
        assert_eq!(back.job_id, id);
        assert_eq!(back.routed_to, "local");
        Ok(())
    }

    #[test]
    fn job_status_roundtrip_json() -> Result<(), serde_json::Error> {
        let started = chrono::Utc::now();
        let js = JobStatus {
            job_id: Uuid::nil(),
            status: JobStatusType::Failed,
            routed_to: "x".to_string(),
            progress: Some(0.5),
            started_at: started,
            completed_at: None,
            error: Some("oops".to_string()),
        };
        let json = serde_json::to_string(&js)?;
        let back: JobStatus = serde_json::from_str(&json)?;
        assert_eq!(back.status, JobStatusType::Failed);
        assert_eq!(back.error.as_deref(), Some("oops"));
        Ok(())
    }
}
