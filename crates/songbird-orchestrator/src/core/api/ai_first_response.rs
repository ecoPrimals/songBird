// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! AI-first universal response envelope for orchestrator APIs.
//!
//! Endpoints return [`AIFirstResponse`] so automation and humans share the same structured
//! outcome, timing, and optional follow-up actions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Universal AI-first response format for Songbird orchestrator endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"]
pub struct AIFirstResponse<T> {
    /// Operation success status (machine-readable)
    pub success: bool,
    /// Strongly-typed response data
    pub data: T,
    /// AI-optimized error information
    pub error: Option<AIFirstError>,
    /// Unique request identifier for tracing and correlation
    pub request_id: Uuid,
    /// Processing time in milliseconds for performance monitoring
    pub processing_time_ms: u64,
    /// AI-specific metadata for decision making
    pub ai_metadata: AIResponseMetadata,
    /// Human interaction context (when applicable)
    pub human_context: Option<HumanInteractionContext>,
    /// Confidence score for AI decision making (0.0 - 1.0)
    pub confidence_score: f64,
    /// Suggested next actions for AI agents
    pub suggested_actions: Vec<SuggestedAction>,
}

impl<T> AIFirstResponse<T> {
    /// Create a successful AI-first response.
    #[must_use]
    pub fn success(
        data: T,
        request_id: Uuid,
        processing_time_ms: u64,
        confidence_score: f64,
    ) -> Self {
        Self {
            success: true,
            data,
            error: None,
            request_id,
            processing_time_ms,
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score,
            suggested_actions: Vec::new(),
        }
    }

    /// Create a failed AI-first response (still carries `data`, often partial or placeholder).
    #[must_use]
    pub fn error(data: T, error: AIFirstError, request_id: Uuid, processing_time_ms: u64) -> Self {
        Self {
            success: false,
            data,
            error: Some(error),
            request_id,
            processing_time_ms,
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score: 0.0,
            suggested_actions: Vec::new(),
        }
    }

    /// Whether the response indicates success.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.success
    }

    /// Whether the response indicates an error.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        !self.success
    }

    /// Consume the response and return the payload.
    #[must_use]
    pub fn into_data(self) -> T {
        self.data
    }

    /// Attach human interaction context (builder).
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]
    pub fn with_human_context(mut self, context: HumanInteractionContext) -> Self {
        self.human_context = Some(context);
        self
    }

    /// Replace AI metadata (builder).
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]
    pub fn with_ai_metadata(mut self, metadata: AIResponseMetadata) -> Self {
        self.ai_metadata = metadata;
        self
    }

    /// Attach suggested follow-up actions (builder).
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]
    pub fn with_suggested_actions(mut self, actions: Vec<SuggestedAction>) -> Self {
        self.suggested_actions = actions;
        self
    }
}

/// AI-optimized error structure with automation hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"]
pub struct AIFirstError {
    /// Machine-readable error code (UPPER_SNAKE_CASE)
    pub code: String,
    /// Human-readable message (for logging/debugging)
    pub message: String,
    /// Error category for AI classification
    pub category: AIErrorCategory,
    /// Retry guidance
    pub retry_strategy: RetryStrategy,
    /// Actionable hints for AI automation
    pub automation_hints: Vec<String>,
    /// Severity level for prioritization
    pub severity: ErrorSeverity,
    /// Whether human intervention is required
    pub requires_human_intervention: bool,
    /// Related error context for debugging
    pub context: HashMap<String, serde_json::Value>,
}

impl AIFirstError {
    /// Service mesh / routing style failure with retry defaults.
    #[must_use]
    pub fn service_mesh_failure(service: &str, message: impl Into<String>) -> Self {
        Self {
            code: "SERVICE_MESH_FAILURE".to_string(),
            message: message.into(),
            category: AIErrorCategory::ServiceMeshFailure,
            retry_strategy: RetryStrategy {
                should_retry: true,
                delay_ms: 1000,
                max_attempts: 3,
                backoff_strategy: BackoffType::Exponential {
                    base: 2.0,
                },
                retry_conditions: vec!["service_available".to_string()],
                success_probability: 0.7,
            },
            automation_hints: vec![
                "Check service health".to_string(),
                "Try alternative service endpoint".to_string(),
            ],
            severity: ErrorSeverity::High,
            requires_human_intervention: false,
            context: {
                let mut ctx = HashMap::new();
                ctx.insert(
                    "failed_service".to_string(),
                    serde_json::Value::String(service.to_string()),
                );
                ctx
            },
        }
    }

    /// Human approval or manual step required.
    #[must_use]
    pub fn human_intervention_required(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self {
            code: "HUMAN_INTERVENTION_REQUIRED".to_string(),
            message: format!("Human intervention required: {reason}"),
            category: AIErrorCategory::HumanInterventionRequired,
            retry_strategy: RetryStrategy {
                should_retry: false,
                delay_ms: 0,
                max_attempts: 0,
                backoff_strategy: BackoffType::Linear,
                retry_conditions: vec!["human_approval_received".to_string()],
                success_probability: 1.0,
            },
            automation_hints: vec![
                "Escalate to human operator".to_string(),
                "Provide context for decision".to_string(),
            ],
            severity: ErrorSeverity::Medium,
            requires_human_intervention: true,
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("intervention_reason".to_string(), serde_json::Value::String(reason));
                ctx
            },
        }
    }
}

/// High-level classification for automation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[must_use = "This type represents an outcome that must be handled"]
pub enum AIErrorCategory {
    ServiceMeshFailure,
    ServiceDiscoveryFailure,
    LoadBalancingFailure,
    ConfigurationIssue,
    SecurityViolation,
    NetworkFailure,
    HumanInterventionRequired,
    DependencyFailure,
    RateLimiting,
    ResourceExhaustion,
    CircuitBreakerOpen,
    PrimalIntegrationFailure,
    SystemError,
    Unknown,
}

/// Retry policy attached to errors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryStrategy {
    /// Whether automatic retry is recommended
    pub should_retry: bool,
    /// Initial delay in milliseconds
    pub delay_ms: u64,
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Backoff strategy type
    pub backoff_strategy: BackoffType,
    /// Conditions that must be met for retry
    pub retry_conditions: Vec<String>,
    /// Estimated success probability for retry
    pub success_probability: f64,
}

/// Backoff strategy for retries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackoffType {
    Linear,
    Exponential {
        base: f64,
    },
    Fibonacci,
    Custom {
        formula: String,
    },
}

/// Severity for triage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[must_use = "This type represents an outcome that must be handled"]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Metadata block for AI-facing consumers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"]
pub struct AIResponseMetadata {
    /// Performance characteristics
    pub performance: PerformanceMetrics,
    /// Resource utilization
    pub resource_usage: ResourceUsage,
    /// Quality indicators
    pub quality_metrics: QualityMetrics,
    /// Caching information
    pub cache_info: CacheInfo,
    /// Rate limiting status
    pub rate_limit_status: RateLimitStatus,
    /// Related operations or dependencies
    pub dependencies: Vec<String>,
    /// Service mesh routing information
    pub routing_metadata: RoutingMetadata,
}

/// Performance slice of [`AIResponseMetadata`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Request processing latency
    pub latency_ms: f64,
    /// Service mesh routing time
    pub routing_time_ms: f64,
    /// Backend service response time
    pub backend_response_time_ms: f64,
    /// Network overhead
    pub network_overhead_ms: f64,
    /// Throughput metrics
    pub throughput_rps: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            latency_ms: 0.0,
            routing_time_ms: 0.0,
            backend_response_time_ms: 0.0,
            network_overhead_ms: 0.0,
            throughput_rps: 0.0,
        }
    }
}

/// Resource usage slice of [`AIResponseMetadata`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU utilization percentage
    pub cpu_percent: f64,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Disk usage in bytes
    pub disk_bytes: u64,
    /// Network bandwidth usage in bytes per second
    pub network_bytes_per_sec: u64,
    /// Custom resource usage metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_bytes: 0,
            disk_bytes: 0,
            network_bytes_per_sec: 0,
            custom_metrics: HashMap::new(),
        }
    }
}

/// Quality slice of [`AIResponseMetadata`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Accuracy score (0.0 - 1.0)
    pub accuracy: f64,
    /// Completeness score (0.0 - 1.0)
    pub completeness: f64,
    /// Consistency score (0.0 - 1.0)
    pub consistency: f64,
    /// Data freshness (seconds since last update)
    pub freshness_seconds: u64,
    /// Service reliability score (0.0 - 1.0)
    pub reliability: f64,
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            accuracy: 1.0,
            completeness: 1.0,
            consistency: 1.0,
            freshness_seconds: 0,
            reliability: 1.0,
        }
    }
}

/// Cache slice of [`AIResponseMetadata`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    /// Whether response is cached
    pub is_cached: bool,
    /// Hit ratio when cached
    pub hit_ratio: f64,
    /// Time to live for cached data
    pub ttl_seconds: Option<u64>,
    /// Cache key used
    pub cache_key: Option<String>,
    /// Cache generation timestamp
    pub cached_at: Option<DateTime<Utc>>,
}

impl Default for CacheInfo {
    fn default() -> Self {
        Self {
            is_cached: false,
            hit_ratio: 0.0,
            ttl_seconds: None,
            cache_key: None,
            cached_at: None,
        }
    }
}

/// Rate limit slice of [`AIResponseMetadata`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    /// Whether rate limiting is active
    pub is_rate_limited: bool,
    /// Requests remaining in current window
    pub requests_remaining: Option<u32>,
    /// Rate limit window reset time
    pub reset_time: Option<DateTime<Utc>>,
    /// Rate limit window duration (not serialized; wire format uses other fields)
    #[serde(skip)]
    pub window_duration: Option<Duration>,
    /// Current request rate (requests per second)
    pub current_rate: f64,
}

impl Default for RateLimitStatus {
    fn default() -> Self {
        Self {
            is_rate_limited: false,
            requests_remaining: None,
            reset_time: None,
            window_duration: None,
            current_rate: 0.0,
        }
    }
}

/// Routing slice of [`AIResponseMetadata`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoutingMetadata {
    /// Selected service endpoint
    pub selected_endpoint: Option<String>,
    /// Number of available endpoints
    pub available_endpoints: u32,
    /// Load balancing algorithm used
    pub load_balancing_algorithm: Option<String>,
    /// Service health scores
    pub service_health_scores: HashMap<String, f64>,
    /// Routing decision factors
    pub decision_factors: Vec<RoutingDecisionFactor>,
}

/// Single routing decision factor for observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecisionFactor {
    /// Factor name
    pub name: String,
    /// Factor weight in decision (0.0 - 1.0)
    pub weight: f64,
    /// Factor value
    pub value: serde_json::Value,
    /// Impact on routing decision
    pub impact: String,
}

/// Human–AI collaboration context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanInteractionContext {
    /// Human user identifier (when applicable)
    pub user_id: Option<String>,
    /// Whether human approval is required for this operation
    pub approval_required: bool,
    /// Confidence threshold for auto-execution
    pub confidence_threshold: f64,
}

/// Suggested follow-up for an AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    /// Action type for AI agents
    pub action_type: String,
    /// Action parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Priority for execution
    pub priority: ActionPriority,
    /// Expected outcome
    pub expected_outcome: String,
    /// Confidence in suggestion
    pub confidence: f64,
    /// Human approval required for this action
    pub requires_human_approval: bool,
    /// Estimated execution time (not serialized; keep in-memory for agents)
    #[serde(skip)]
    pub estimated_execution_time: Option<Duration>,
}

/// Relative priority for suggested actions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use serde_json::json;

    fn rid() -> Uuid {
        Uuid::nil()
    }

    #[test]
    fn success_sets_flags_and_default_metadata() {
        let r = AIFirstResponse::success("payload", rid(), 12, 0.88);
        assert!(r.is_success());
        assert!(!r.is_error());
        assert_eq!(r.data, "payload");
        assert!(r.error.is_none());
        assert_eq!(r.processing_time_ms, 12);
        assert_eq!(r.confidence_score, 0.88);
        assert!(r.suggested_actions.is_empty());
    }

    #[test]
    fn error_sets_failure_and_zero_confidence() {
        let err = AIFirstError {
            code: "E".to_string(),
            message: "m".to_string(),
            category: AIErrorCategory::SystemError,
            retry_strategy: RetryStrategy {
                should_retry: false,
                delay_ms: 0,
                max_attempts: 0,
                backoff_strategy: BackoffType::Linear,
                retry_conditions: vec![],
                success_probability: 0.0,
            },
            automation_hints: vec![],
            severity: ErrorSeverity::Low,
            requires_human_intervention: false,
            context: HashMap::new(),
        };
        let r = AIFirstResponse::error(42_i32, err, rid(), 5);
        assert!(r.is_error());
        assert!(!r.is_success());
        assert_eq!(r.confidence_score, 0.0);
        assert_eq!(r.error.as_ref().expect("err").code, "E");
    }

    #[test]
    fn into_data_consumes_envelope() {
        let r = AIFirstResponse::success("x".to_string(), rid(), 1, 1.0);
        assert_eq!(r.into_data(), "x");
    }

    #[test]
    fn with_human_context_round_trips() {
        let ctx = HumanInteractionContext {
            user_id: Some("u1".to_string()),
            approval_required: true,
            confidence_threshold: 0.7,
        };
        let r = AIFirstResponse::success((), rid(), 0, 1.0).with_human_context(ctx.clone());
        assert_eq!(r.human_context.as_ref().expect("ctx").user_id, ctx.user_id);
    }

    #[test]
    fn with_ai_metadata_replaces_block() {
        let mut m = AIResponseMetadata::default();
        m.dependencies.push("dep".to_string());
        let r = AIFirstResponse::success(0_u8, rid(), 2, 0.5).with_ai_metadata(m.clone());
        assert_eq!(r.ai_metadata.dependencies, m.dependencies);
    }

    #[test]
    fn with_suggested_actions_preserves_vec() {
        let actions = vec![SuggestedAction {
            action_type: "retry".to_string(),
            parameters: HashMap::from([("k".to_string(), json!("v"))]),
            priority: ActionPriority::High,
            expected_outcome: "ok".to_string(),
            confidence: 0.9,
            requires_human_approval: false,
            estimated_execution_time: Some(Duration::from_secs(1)),
        }];
        let r = AIFirstResponse::success((), rid(), 0, 1.0).with_suggested_actions(actions.clone());
        assert_eq!(r.suggested_actions.len(), 1);
        assert_eq!(r.suggested_actions[0].action_type, "retry");
    }

    #[test]
    fn builder_chain_order_independent_for_metadata_and_actions() {
        let mut meta = AIResponseMetadata::default();
        meta.performance.latency_ms = 3.0;
        let actions = vec![SuggestedAction {
            action_type: "a".to_string(),
            parameters: HashMap::new(),
            priority: ActionPriority::Low,
            expected_outcome: "".to_string(),
            confidence: 1.0,
            requires_human_approval: false,
            estimated_execution_time: None,
        }];
        let r = AIFirstResponse::success(1_i32, rid(), 9, 0.5)
            .with_ai_metadata(meta.clone())
            .with_suggested_actions(actions.clone());
        assert_eq!(r.ai_metadata.performance.latency_ms, 3.0);
        assert_eq!(r.suggested_actions.len(), 1);
        let r2 = AIFirstResponse::success(1_i32, rid(), 9, 0.5)
            .with_suggested_actions(actions)
            .with_ai_metadata(meta);
        assert_eq!(r2.suggested_actions.len(), 1);
        assert_eq!(r2.ai_metadata.performance.latency_ms, 3.0);
    }

    #[test]
    fn service_mesh_failure_sets_category_and_context() {
        let e = AIFirstError::service_mesh_failure("payments", "upstream timeout");
        assert_eq!(e.category, AIErrorCategory::ServiceMeshFailure);
        assert_eq!(e.context.get("failed_service").and_then(|v| v.as_str()), Some("payments"));
        assert!(e.retry_strategy.should_retry);
        assert_eq!(
            e.retry_strategy.backoff_strategy,
            BackoffType::Exponential {
                base: 2.0
            }
        );
    }

    #[test]
    fn human_intervention_required_sets_flags_and_linear_backoff() {
        let e = AIFirstError::human_intervention_required("quota");
        assert_eq!(e.category, AIErrorCategory::HumanInterventionRequired);
        assert!(e.requires_human_intervention);
        assert!(!e.retry_strategy.should_retry);
        assert_eq!(e.retry_strategy.backoff_strategy, BackoffType::Linear);
        assert!(e.message.contains("quota"));
    }

    #[test]
    fn serde_roundtrip_ai_first_response() {
        let r = AIFirstResponse::success(json!({"a": 1}), rid(), 4, 0.33);
        let s = serde_json::to_string(&r).expect("serialize");
        let back: AIFirstResponse<serde_json::Value> =
            serde_json::from_str(&s).expect("deserialize");
        assert!(back.is_success());
        assert_eq!(back.data, json!({"a": 1}));
    }

    #[test]
    fn error_serde_preserves_code_and_category() {
        let e = AIFirstError::human_intervention_required("x");
        let r = AIFirstResponse::error((), e, rid(), 1);
        let s = serde_json::to_string(&r).expect("serialize");
        let back: AIFirstResponse<()> = serde_json::from_str(&s).expect("deserialize");
        assert!(back.is_error());
        assert_eq!(back.error.expect("e").category, AIErrorCategory::HumanInterventionRequired);
    }

    #[test]
    fn default_quality_metrics_are_sane() {
        let q = QualityMetrics::default();
        assert_eq!(q.accuracy, 1.0);
        assert_eq!(q.reliability, 1.0);
    }

    #[test]
    fn default_routing_metadata_empty_maps() {
        let r = RoutingMetadata::default();
        assert!(r.service_health_scores.is_empty());
        assert!(r.decision_factors.is_empty());
    }

    #[test]
    fn backoff_type_custom_roundtrip() {
        let b = BackoffType::Custom {
            formula: "x^2".to_string(),
        };
        let s = serde_json::to_string(&b).expect("serialize");
        let back: BackoffType = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(back, b);
    }
}
