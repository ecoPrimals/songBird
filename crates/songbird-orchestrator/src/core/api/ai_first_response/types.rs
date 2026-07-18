// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Types, DTOs, and nested metadata for [`super::AIFirstResponse`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// AI-optimized error structure with automation hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"]
pub struct AIFirstError {
    /// Machine-readable error code (`UPPER_SNAKE_CASE`)
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
            code: String::from("SERVICE_MESH_FAILURE"),
            message: message.into(),
            category: AIErrorCategory::ServiceMeshFailure,
            retry_strategy: RetryStrategy {
                should_retry: true,
                delay_ms: 1000,
                max_attempts: 3,
                backoff_strategy: BackoffType::Exponential {
                    base: 2.0,
                },
                retry_conditions: vec![String::from("service_available")],
                success_probability: 0.7,
            },
            automation_hints: vec![
                String::from("Check service health"),
                String::from("Try alternative service endpoint"),
            ],
            severity: ErrorSeverity::High,
            requires_human_intervention: false,
            context: {
                let mut ctx = HashMap::new();
                ctx.insert(
                    String::from("failed_service"),
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
            code: String::from("HUMAN_INTERVENTION_REQUIRED"),
            message: format!("Human intervention required: {reason}"),
            category: AIErrorCategory::HumanInterventionRequired,
            retry_strategy: RetryStrategy {
                should_retry: false,
                delay_ms: 0,
                max_attempts: 0,
                backoff_strategy: BackoffType::Linear,
                retry_conditions: vec![String::from("human_approval_received")],
                success_probability: 1.0,
            },
            automation_hints: vec![
                String::from("Escalate to human operator"),
                String::from("Provide context for decision"),
            ],
            severity: ErrorSeverity::Medium,
            requires_human_intervention: true,
            context: {
                let mut ctx = HashMap::new();
                ctx.insert(String::from("intervention_reason"), serde_json::Value::String(reason));
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
