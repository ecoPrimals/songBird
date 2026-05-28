// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Axum HTTP handlers: submit compute tasks and poll job status.

use crate::core::routing::RoutingDecision;
use axum::extract::{Json, Path, State};
use uuid::Uuid;

use super::compute_routing::format_compute_routed_destination;
use super::compute_state::ComputeApiState;
use super::compute_types::{
    ApiError, ComputeTaskRequest, ComputeTaskResponse, JobStatus, JobStatusType,
    discover_http_client, serialize_task, update_job_status,
};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

fn new_routing_job_status(job_id: Uuid, routed_to: String) -> JobStatus {
    JobStatus {
        job_id,
        status: JobStatusType::Routing,
        routed_to,
        progress: None,
        started_at: chrono::Utc::now(),
        completed_at: None,
        error: None,
    }
}

fn compute_submission_ack_response(job_id: Uuid, routed_to: String) -> ComputeTaskResponse {
    ComputeTaskResponse {
        job_id,
        routed_to,
        status: "routing".to_string(),
        estimated_completion: None,
    }
}

/// Submit a compute task for intelligent routing
#[expect(clippy::too_many_lines, reason = "compute task routing and job submission pipeline")]
#[tracing::instrument(skip(state, req), fields(task_type = %req.task.task_type))]
pub async fn submit_compute_task(
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
    let job_status = new_routing_job_status(job_id, routed_to.clone());

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

                match result {
                    Ok(response) => match response.status {
                        ExecutionStatus::Completed => {
                            update_job_status(
                                active_jobs_clone.as_ref(),
                                job_id,
                                JobStatusType::Completed,
                                None,
                            )
                            .await;
                            info!(
                                "Task {} completed successfully (exit code: {:?})",
                                job_id, response.exit_code
                            );
                        }
                        ExecutionStatus::Failed | ExecutionStatus::Timeout => {
                            update_job_status(
                                active_jobs_clone.as_ref(),
                                job_id,
                                JobStatusType::Failed,
                                None,
                            )
                            .await;
                            warn!(
                                "Task {} failed (status: {}, exit code: {:?}): {}",
                                job_id, response.status, response.exit_code, response.stderr
                            );
                        }
                        _ => {
                            update_job_status(
                                active_jobs_clone.as_ref(),
                                job_id,
                                JobStatusType::Failed,
                                None,
                            )
                            .await;
                            warn!("Task {} in unexpected state: {}", job_id, response.status);
                        }
                    },
                    Err(e) => {
                        update_job_status(
                            active_jobs_clone.as_ref(),
                            job_id,
                            JobStatusType::Failed,
                            None,
                        )
                        .await;
                        error!("Task {} execution error: {}", job_id, e);
                    }
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
                {
                    let mut jobs = active_jobs_clone.write().await;
                    if let Some(status) = jobs.get_mut(&job_id) {
                        status.status = JobStatusType::Running;
                    }
                }

                let Some(client) = discover_http_client(&active_jobs_clone, job_id).await else {
                    return;
                };

                let service_url = format!("http://{endpoint_clone}:{port_clone}/execute");

                let Some(task_json) = serialize_task(&task_clone, &active_jobs_clone, job_id).await
                else {
                    return;
                };

                let result = client.post(&service_url, task_json).await;

                match result {
                    Ok(response) if response.status >= 200 && response.status < 300 => {
                        update_job_status(
                            active_jobs_clone.as_ref(),
                            job_id,
                            JobStatusType::Completed,
                            None,
                        )
                        .await;
                        info!("Task {} completed on service {}", job_id, service_name_clone);
                    }
                    Ok(response) => {
                        let mut jobs = active_jobs_clone.write().await;
                        if let Some(status) = jobs.get_mut(&job_id) {
                            status.status = JobStatusType::Failed;
                            status.error = Some(format!(
                                "Service {} returned error: {}",
                                service_name_clone, response.status
                            ));
                        }
                        warn!(
                            "Task {} failed on service {}: {}",
                            job_id, service_name_clone, response.status
                        );
                    }
                    Err(e) => {
                        let mut jobs = active_jobs_clone.write().await;
                        if let Some(status) = jobs.get_mut(&job_id) {
                            status.status = JobStatusType::Failed;
                            status.error = Some(format!(
                                "HTTP request to service {service_name_clone} failed: {e}"
                            ));
                        }
                        warn!(
                            "Task {} HTTP error to service {}: {}",
                            job_id, service_name_clone, e
                        );
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
                {
                    let mut jobs = active_jobs_clone.write().await;
                    if let Some(status) = jobs.get_mut(&job_id) {
                        status.status = JobStatusType::Running;
                    }
                }

                let Some(client) = discover_http_client(&active_jobs_clone, job_id).await else {
                    return;
                };

                let forward_url = format!("{endpoint_clone}/task");

                let Some(task_json) = serialize_task(&task_clone, &active_jobs_clone, job_id).await
                else {
                    return;
                };

                let result = tokio::time::timeout(
                    songbird_types::defaults::timeouts::DEFAULT_COMPUTE_TIMEOUT,
                    client.post(&forward_url, task_json),
                )
                .await;

                match result {
                    Ok(Ok(response)) if response.status >= 200 && response.status < 300 => {
                        update_job_status(
                            active_jobs_clone.as_ref(),
                            job_id,
                            JobStatusType::Completed,
                            None,
                        )
                        .await;
                        info!("Task {} completed on peer {}", job_id, node_id_clone);
                    }
                    Ok(Ok(response)) => {
                        update_job_status(
                            active_jobs_clone.as_ref(),
                            job_id,
                            JobStatusType::Failed,
                            Some(format!("Peer returned error: {}", response.status)),
                        )
                        .await;
                        warn!(
                            "Task {} failed on peer {}: {}",
                            job_id, node_id_clone, response.status
                        );
                    }
                    Ok(Err(e)) => {
                        update_job_status(
                            active_jobs_clone.as_ref(),
                            job_id,
                            JobStatusType::Failed,
                            Some(format!("HTTP request to peer failed: {e}")),
                        )
                        .await;
                        warn!("Task {} HTTP error to peer {}: {}", job_id, node_id_clone, e);
                    }
                    Err(_timeout) => {
                        update_job_status(
                            active_jobs_clone.as_ref(),
                            job_id,
                            JobStatusType::Failed,
                            Some(format!("Forward to peer {node_id_clone} timed out after 300s")),
                        )
                        .await;
                        warn!("Task {} forward to peer {} timed out", job_id, node_id_clone);
                    }
                }
            });
        }

        RoutingDecision::RouteToCapability {
            provider_endpoint,
            capability_type,
        } => {
            // Forward to capability provider (compute, security, storage, etc.)
            info!(
                "Forwarding task {} to {:?} capability at {}",
                job_id, capability_type, provider_endpoint
            );

            let endpoint_clone = provider_endpoint.clone();

            tokio::spawn(async move {
                let result =
                    router_clone.execute_on_external_provider(&endpoint_clone, &task_clone).await;

                match result {
                    Ok(_data) => {
                        update_job_status(
                            active_jobs_clone.as_ref(),
                            job_id,
                            JobStatusType::Completed,
                            None,
                        )
                        .await;
                        info!("Task {} completed on capability provider", job_id);
                    }
                    Err(e) => {
                        update_job_status(
                            active_jobs_clone.as_ref(),
                            job_id,
                            JobStatusType::Failed,
                            Some(e.to_string()),
                        )
                        .await;
                        warn!("Task {} failed on capability provider: {}", job_id, e);
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
                {
                    let mut jobs = active_jobs_clone.write().await;
                    if let Some(status) = jobs.get_mut(&job_id) {
                        status.status = JobStatusType::Running;
                    }
                }

                let result =
                    router_clone.execute_on_external_provider(&endpoint_clone, &task_clone).await;

                match result {
                    Ok(_data) => {
                        update_job_status(
                            active_jobs_clone.as_ref(),
                            job_id,
                            JobStatusType::Completed,
                            None,
                        )
                        .await;
                        info!("Task {} completed successfully", job_id);
                    }
                    Err(e) => {
                        update_job_status(
                            active_jobs_clone.as_ref(),
                            job_id,
                            JobStatusType::Failed,
                            Some(e.to_string()),
                        )
                        .await;
                        warn!("Task {} failed: {}", job_id, e);
                    }
                }
            });
        }
    }

    Ok(Json(compute_submission_ack_response(job_id, routed_to)))
}

/// Get the status of a compute task
#[tracing::instrument(skip(state))]
pub async fn get_task_status(
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

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::{ComputeTaskRequest, JobStatusType};
    use super::{
        compute_submission_ack_response, format_compute_routed_destination, new_routing_job_status,
    };
    use crate::core::routing::{RoutingDecision, Task};
    use songbird_config::capability_endpoints::CapabilityType;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    #[test]
    fn new_routing_job_status_is_routing_phase() {
        let id = Uuid::nil();
        let j = new_routing_job_status(id, "local".into());
        assert_eq!(j.job_id, id);
        assert_eq!(j.status, JobStatusType::Routing);
        assert_eq!(j.routed_to, "local");
        assert!(j.completed_at.is_none());
        assert!(j.error.is_none());
    }

    #[test]
    fn compute_submission_ack_matches_handler_contract() {
        let id = Uuid::nil();
        let r = compute_submission_ack_response(id, "songbird:n1".into());
        assert_eq!(r.job_id, id);
        assert_eq!(r.routed_to, "songbird:n1");
        assert_eq!(r.status, "routing");
        assert!(r.estimated_completion.is_none());
    }

    #[test]
    fn format_routed_destination_all_variants() {
        assert_eq!(format_compute_routed_destination(&RoutingDecision::ExecuteLocally), "local");
        assert_eq!(
            format_compute_routed_destination(&RoutingDecision::RouteToSongbird {
                node_id: "n1".into(),
                endpoint: "http://x".into(),
            }),
            "songbird:n1"
        );
        assert_eq!(
            format_compute_routed_destination(&RoutingDecision::RouteToRegisteredService {
                service_id: "s1".into(),
                service_name: "svc".into(),
                endpoint: "http://h".into(),
                port: 42,
            }),
            "service:svc:42"
        );
        assert_eq!(
            format_compute_routed_destination(&RoutingDecision::RouteToCapability {
                capability_type: CapabilityType::Compute,
                provider_endpoint: "http://cap".into(),
            }),
            "Compute:http://cap"
        );
        assert_eq!(
            format_compute_routed_destination(&RoutingDecision::RouteToExternalProvider {
                provider_id: "p1".into(),
                execution_endpoint: "http://e".into(),
                capability_name: "c".into(),
            }),
            "external:p1"
        );
    }

    #[test]
    fn compute_task_request_roundtrip_json() {
        let req = ComputeTaskRequest {
            task: Task {
                task_type: Arc::from("t1"),
                payload: serde_json::json!({"k": 1}),
                resource_requirements: None,
                estimated_duration_secs: Some(10),
                metadata: HashMap::from([("a".into(), "b".into())]),
            },
            priority: Some(3),
            timeout_secs: Some(60),
        };
        let v = serde_json::to_value(&req).unwrap();
        let back: ComputeTaskRequest = serde_json::from_value(v).unwrap();
        assert_eq!(back.task.task_type.as_ref(), "t1");
        assert_eq!(back.priority, Some(3));
    }
}
