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

mod compute_handlers;
mod compute_routing;
mod compute_state;
mod compute_types;

pub(crate) use compute_handlers::{get_task_status, submit_compute_task};
pub use compute_state::ComputeApiState;
pub use compute_types::{
    ApiError, ComputeTaskRequest, ComputeTaskResponse, JobStatus, JobStatusType,
};

use axum::{
    Router,
    routing::{get, post},
};

/// Create compute API routes
pub fn compute_routes() -> Router<ComputeApiState> {
    Router::new()
        .route("/task", post(submit_compute_task))
        .route("/task/:job_id", get(get_task_status))
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use axum::extract::{Json, Path, State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use songbird_config::capability_endpoints::CapabilityType;
    use songbird_network_federation::service_registry::FederatedServiceRegistry;
    use songbird_network_federation::state::FederationState;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::core::routing::{RoutingDecision, Task};

    use super::compute_routing::format_compute_routed_destination;

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
        let federation_state = Arc::new(FederationState::new("default".to_string()));
        let service_registry = Arc::new(FederatedServiceRegistry::new());
        let mut overrides = HashMap::new();
        overrides.insert(CapabilityType::Compute, "http://localhost:9000".to_string());
        let state = ComputeApiState::new_with_capability_endpoint_overrides(
            federation_state,
            service_registry,
            overrides,
        );

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
        assert!(
            format_compute_routed_destination(&RoutingDecision::RouteToCapability {
                capability_type: CapabilityType::Compute,
                provider_endpoint: "unix:///run/c.sock".to_string(),
            })
            .contains("Compute")
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
