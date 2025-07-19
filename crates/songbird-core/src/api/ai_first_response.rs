//! AI-First Response Format Implementation
//!
//! Universal response format for all Songbird endpoints that enables
//! seamless human-AI collaboration across the ecosystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Universal AI-first response format - MANDATORY for all Songbird endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Create a successful AI-first response
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

    /// Create a failed AI-first response
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

    /// Add human interaction context
    pub fn with_human_context(mut self, context: HumanInteractionContext) -> Self {
        self.human_context = Some(context);
        self
    }

    /// Add AI metadata
    pub fn with_ai_metadata(mut self, metadata: AIResponseMetadata) -> Self {
        self.ai_metadata = metadata;
        self
    }

    /// Add suggested actions
    pub fn with_suggested_actions(mut self, actions: Vec<SuggestedAction>) -> Self {
        self.suggested_actions = actions;
        self
    }
}

/// AI-optimized error structure with automation hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIFirstError {
    /// Machine-readable error code (UPPER_SNAKE_CASE)
    pub code: String,

    /// Human-readable message (for logging/debugging)
    pub message: String,

    /// Error category for AI classification
    pub category: AIErrorCategory,

    /// Automated retry strategy
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIErrorCategory {
    /// Service mesh routing issues
    ServiceMeshFailure,

    /// Service discovery problems
    ServiceDiscoveryFailure,

    /// Load balancing failures
    LoadBalancingFailure,

    /// Configuration or parameter issues
    ConfigurationIssue,

    /// Authentication or authorization failures
    SecurityViolation,

    /// Network connectivity problems
    NetworkFailure,

    /// Requires human decision or input
    HumanInterventionRequired,

    /// External service dependency failures
    DependencyFailure,

    /// Rate limiting or throttling
    RateLimiting,

    /// Resource exhaustion
    ResourceExhaustion,

    /// Circuit breaker activation
    CircuitBreakerOpen,

    /// Primal integration failures
    PrimalIntegrationFailure,

    /// System-level errors
    SystemError,

    /// Unknown error category
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffType {
    Linear,
    Exponential { base: f64 },
    Fibonacci,
    Custom { formula: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Metadata specifically designed for AI decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for AIResponseMetadata {
    fn default() -> Self {
        Self {
            performance: PerformanceMetrics::default(),
            resource_usage: ResourceUsage::default(),
            quality_metrics: QualityMetrics::default(),
            cache_info: CacheInfo::default(),
            rate_limit_status: RateLimitStatus::default(),
            dependencies: Vec::new(),
            routing_metadata: RoutingMetadata::default(),
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    /// Whether response is cached
    pub is_cached: bool,

    /// Cache hit ratio (0.0 - 1.0)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    /// Whether rate limiting is active
    pub is_rate_limited: bool,

    /// Requests remaining in current window
    pub requests_remaining: Option<u32>,

    /// Rate limit window reset time
    pub reset_time: Option<DateTime<Utc>>,

    /// Rate limit window duration
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for RoutingMetadata {
    fn default() -> Self {
        Self {
            selected_endpoint: None,
            available_endpoints: 0,
            load_balancing_algorithm: None,
            service_health_scores: HashMap::new(),
            decision_factors: Vec::new(),
        }
    }
}

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

/// Context for human-AI collaborative operations in service mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanInteractionContext {
    /// Human user identifier (when applicable)
    pub user_id: Option<String>,

    /// Current interaction mode
    pub interaction_mode: InteractionMode,

    /// User preferences for AI operations  
    pub preferences: AIUserPreferences,

    /// Whether human approval is required for this operation
    pub approval_required: bool,

    /// Confidence threshold for auto-execution
    pub confidence_threshold: f64,

    /// Escalation configuration
    pub escalation_config: EscalationConfig,

    /// Session context for multi-step operations
    pub session_context: Option<SessionContext>,

    /// Service mesh specific context
    pub service_mesh_context: ServiceMeshContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionMode {
    /// AI operates completely autonomously
    FullyAutonomous,

    /// AI suggests actions, human approves before execution
    HumanApproval,

    /// Real-time collaboration between human and AI
    Collaborative,

    /// Human directs strategy, AI executes tactics
    HumanDirected,

    /// AI monitors and alerts, human makes key decisions
    HumanSupervised,

    /// Emergency mode - AI acts immediately, notifies human
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIUserPreferences {
    /// Preferred AI models for different operation types
    pub model_preferences: HashMap<String, String>,

    /// Auto-approval thresholds by operation category
    pub auto_approval_thresholds: HashMap<String, f64>,

    /// Notification preferences
    pub notifications: NotificationPreferences,

    /// Resource usage limits and preferences
    pub resource_limits: AIResourceLimits,

    /// Risk tolerance levels
    pub risk_tolerance: RiskTolerance,

    /// Learning preferences (whether AI should learn from user behavior)
    pub learning_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// Email notifications enabled
    pub email_enabled: bool,

    /// Slack notifications enabled
    pub slack_enabled: bool,

    /// Webhook notifications enabled
    pub webhook_enabled: bool,

    /// Minimum severity for notifications
    pub minimum_severity: ErrorSeverity,

    /// Notification channels by category
    pub channels_by_category: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResourceLimits {
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f64,

    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,

    /// Maximum execution time
    pub max_execution_time: Duration,

    /// Maximum cost per operation
    pub max_cost_per_operation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskTolerance {
    /// Risk tolerance level (0.0 - 1.0, higher = more risk tolerant)
    pub level: f64,

    /// Risk categories and their specific tolerances
    pub category_tolerances: HashMap<String, f64>,

    /// Whether to allow experimental features
    pub allow_experimental: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    /// Service mesh specific escalation triggers
    pub escalation_triggers: Vec<ServiceMeshEscalationTrigger>,

    /// Maximum time to wait for human response
    pub human_response_timeout: Duration,

    /// Action to take if human doesn't respond in time
    pub timeout_action: TimeoutAction,

    /// Escalation chain (who to contact in order)
    pub escalation_chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMeshEscalationTrigger {
    /// Service discovery failures
    ServiceDiscoveryFailure { failure_rate: f64 },

    /// Load balancing issues
    LoadBalancingDegradation { threshold: f64 },

    /// Circuit breaker activations
    CircuitBreakerActivation { service_pattern: String },

    /// Cross-primal communication failures
    CrossPrimalFailure {
        primal_type: String,
        failure_rate: f64,
    },

    /// Security concerns in service mesh
    ServiceMeshSecurityConcern { severity: String },

    /// Unknown service behavior
    UnknownServiceBehavior { anomaly_score: f64 },

    /// Critical service impact
    CriticalServiceImpact { affected_services: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeoutAction {
    /// Proceed with default action
    ProceedWithDefault,

    /// Cancel operation
    Cancel,

    /// Escalate to next level
    Escalate,

    /// Execute with reduced confidence
    ExecuteReducedConfidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Session identifier
    pub session_id: Uuid,

    /// Session start time
    pub started_at: DateTime<Utc>,

    /// Session expiration time
    pub expires_at: Option<DateTime<Utc>>,

    /// Session state data
    pub state: HashMap<String, serde_json::Value>,

    /// Previous operations in session
    pub operation_history: Vec<OperationHistoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationHistoryItem {
    /// Operation timestamp
    pub timestamp: DateTime<Utc>,

    /// Operation type
    pub operation_type: String,

    /// Operation result
    pub result: String,

    /// Human involvement level
    pub human_involvement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshContext {
    /// Preferred service routing strategies
    pub routing_preferences: Vec<String>,

    /// Load balancing preferences
    pub load_balancing_preferences: HashMap<String, String>,

    /// Circuit breaker tolerance
    pub circuit_breaker_tolerance: f64,

    /// Human notification preferences for service issues
    pub service_notification_preferences: NotificationPreferences,
}

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

    /// Estimated execution time
    pub estimated_execution_time: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionPriority {
    Low,
    Medium,
    High,
    Urgent,
}

/// Helper functions for creating common AI-first responses
impl AIFirstError {
    /// Create a service mesh failure error
    pub fn service_mesh_failure(message: &str, service: &str) -> Self {
        Self {
            code: "SERVICE_MESH_FAILURE".to_string(),
            message: message.to_string(),
            category: AIErrorCategory::ServiceMeshFailure,
            retry_strategy: RetryStrategy {
                should_retry: true,
                delay_ms: 1000,
                max_attempts: 3,
                backoff_strategy: BackoffType::Exponential { base: 2.0 },
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

    /// Create a human intervention required error
    pub fn human_intervention_required(reason: &str) -> Self {
        Self {
            code: "HUMAN_INTERVENTION_REQUIRED".to_string(),
            message: format!("Human intervention required: {}", reason),
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
                ctx.insert(
                    "intervention_reason".to_string(),
                    serde_json::Value::String(reason.to_string()),
                );
                ctx
            },
        }
    }
}
