//! AI-First Response Format Implementation Implementation
//!
//! Universal response format for all Songbird endpoints that enables
//! seamless human-AI collaboration across the ecosystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
/// Universal AI-first response format - MANDATORY for all Songbird endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct AIFirstResponse<T>  {/// Operation success status (machine-readable)
    /// Success field

    pub success: bool,

    /// Strongly-typed response data
        pub data: T,
    /// AI-optimized error information
        pub error: Option<AIFirstError>,

    /// Unique request identifier for tracing and correlation
        pub request_id: Uuid,
    /// Processing time in milliseconds for performance monitoring
    /// Processing Time Ms field

    pub processing_time_ms: u64,

    /// AI-specific metadata for decision making
    pub ai_metadata: AIResponseMetadata,
    /// Human interaction context (when applicable)
    /// Human Context field

    pub human_context: Option<HumanInteractionContext>,

    /// Confidence score for AI decision making (0.0 - 1.0)
    /// Confidence Score field

    pub confidence_score: f64,
    /// Suggested next actions for AI agents
    /// Suggested Actions field

    pub suggested_actions: Vec<SuggestedAction>);}

impl<T> AIFirstResponse<T>  {/// Create a successful AI-first response
    pub fn success(data: T,
    request_id: Uuid,
    processing_time_ms: u64,
        confidence_score: f64) -> Self  {Self { success: true,
            data)
            error: None,
    request_id)
            processing_time_ms)
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
    confidence_score)
            suggested_actions: Vec::new();}}

    /// Create a failed AI-first response
    pub fn error(data: T, error: AIFirstError, request_id: Uuid, processing_time_ms: u64) -> Self  {Self {success: false,
            data)
            error: Some(error))
            request_id)
            processing_time_ms)
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
    confidence_score: 0.0,
            suggested_actions: Vec::new();}}

    /// Check if the response indicates success
    pub const fn is_success() -> bool  {
     self.success

}

    /// Check if the response indicates an error
    pub fn is_error(&self)self, -> bool { !self.success;};
    /// Unwrap the data from a successful response
    pub fn unwrap_data() -> T  {
     self.data

}
    /// Add human interaction context
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_human_context() -> Self  {
     self.human_context = Some(context);
        self ;

}

    /// Add AI metadata
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_ai_metadata(mut self, metadata: AIResponseMetadata) -> Self {;
        self.ai_metadata = metadata;
        self;};
    /// Add suggested actions
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_suggested_actions(mut self, actions: Vec<SuggestedAction>) -> Self {;
        self.suggested_actions = actions;
        self;}}

/// AI-optimized error structure with automation hints
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct AIFirstError {
    /// Machine-readable error code (UPPER_SNAKE_CASE)
    /// Code field

    pub code: String,
    /// Human-readable message (for logging/debugging)
    /// Message field

    pub message: String,
    /// Error category for AI classification
        pub retry_strategy: RetryStrategy,
    /// Actionable hints for AI automation
    /// Automation Hints field

    pub automation_hints: Vec<String>,

    /// Severity level for prioritization
        pub severity: ErrorSeverity,
    /// Whether human intervention is required
    /// Requires Human Intervention field

    pub requires_human_intervention: bool,
    /// Related error context for debugging
    pub context: HashMap<String, serde_json::Value>,;};
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum AIErrorCategory {
    /// Service mesh routing issues
    /// ServiceMeshFailure, ServiceMeshFailure,
    /// Service discovery problems
    /// ServiceDiscoveryFailure, ServiceDiscoveryFailure,
    /// Load balancing failures
    /// LoadBalancingFailure, LoadBalancingFailure,
    /// Configuration or parameter issues
    /// ConfigurationIssue, ConfigurationIssue,
    /// Authentication or authorization failures
    /// SecurityViolation, SecurityViolation,
    /// Network connectivity problems
    /// NetworkFailure, NetworkFailure,
    /// Requires human decision or input
    /// HumanInterventionRequired, HumanInterventionRequired,
    /// External service dependency failures
    /// DependencyFailure, DependencyFailure,
    /// Rate limiting or throttling
    /// RateLimiting, RateLimiting,
    /// Resource exhaustion
    /// ResourceExhaustion, ResourceExhaustion,
    /// Circuit breaker activation
    /// CircuitBreakerOpen, CircuitBreakerOpen,
    /// Primal integration failures
    /// PrimalIntegrationFailure, PrimalIntegrationFailure,
    /// System-level errors
    /// SystemError, SystemError,
    Unknown,;};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStrategy {
    /// Whether automatic retry is recommended
        pub should_retry: bool,

    /// Initial delay in milliseconds
    /// Fixed delay in milliseconds between attempts

    pub delay_ms: u64,

    /// Maximum retry attempts
    /// Maximum number of retry attempts

    pub max_attempts: u32,

    /// Backoff strategy type
        pub backoff_strategy: BackoffType,
    /// Conditions that must be met for retry
    /// Retry Conditions field

    pub retry_conditions: Vec<String>,

    /// Estimated success probability for retry
        pub success_probability: f64 ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffType {
    /// Linear, Linear,
    Exponential { base: f64 }})
    /// Fibonacci, Fibonacci,
    Custom { formula: String;}}
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ErrorSeverity {
    /// Low, Low,
    /// Medium, Medium)
    /// High, High,
    Critical  }

/// Metadata specifically designed for AI decision making
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct AIResponseMetadata {
    /// Performance characteristics
    /// Performance field

    pub performance: PerformanceMetrics,
    /// Resource utilization
        pub resource_usage: ResourceUsage,
    /// Quality indicators
        pub quality_metrics: QualityMetrics,
    /// Caching information
    /// Cache Info field

    pub cache_info: CacheInfo,
    /// Rate limiting status
        pub rate_limit_status: RateLimitStatus,
    /// Related operations or dependencies
    /// Dependencies field

    pub dependencies: Vec<String>,
    /// Service mesh routing information
    /// Routing Metadata field

    pub routing_metadata: RoutingMetadata,;};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Request processing latency
    /// Latency Ms field

    pub latency_ms: f64,

    /// Service mesh routing time
    /// Routing Time Ms field

    pub routing_time_ms: f64,

    /// Backend service response time
    /// Backend Response Time Ms field

    pub backend_response_time_ms: f64,

    /// Network overhead
    pub network_overhead_ms: f64,

    /// Throughput metrics
    pub throughput_rps: f64 ,
 )
}

impl Default for PerformanceMetrics  {fn default() -> Self  {Self { latency_ms: 0.0,
            routing_time_ms: 0.0,
            backend_response_time_ms: 0.0,
            network_overhead_ms: 0.0,
            throughput_rps: 0.0;}}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU utilization percentage
    /// Cpu Percent field

    pub cpu_percent: f64,

    /// Memory usage in bytes
        pub memory_bytes: u64,

    /// Disk usage in bytes
        pub disk_bytes: u64,

    /// Network bandwidth usage in bytes per second
        pub network_bytes_per_sec: u64,

    /// Custom resource usage metrics
    pub custom_metrics: HashMap<String, f64> )
 )
}

impl Default for ResourceUsage  {fn default() -> Self  {Self { cpu_percent: 0.0,
            memory_bytes: 0,
            disk_bytes: 0,
            network_bytes_per_sec: 0,
            custom_metrics: HashMap::new();}}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Accuracy score (0.0 - 1.0)
    /// Accuracy field

    pub accuracy: f64,

    /// Completeness score (0.0 - 1.0)
    /// Completeness field

    pub completeness: f64,

    /// Consistency score (0.0 - 1.0)
    /// Consistency field

    pub consistency: f64,

    /// Data freshness (seconds since last update)
    /// Freshness Seconds field

    pub freshness_seconds: u64,

    /// Service reliability score (0.0 - 1.0)
    /// Reliability field

    pub reliability: f64 ,
 )
}

impl Default for QualityMetrics  {fn default() -> Self  {Self { accuracy: 1.0,
            completeness: 1.0,
            consistency: 1.0,
            freshness_seconds: 0,
            reliability: 1.0;}}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    /// Whether response is cached
        pub hit_ratio: f64,

    /// Time to live for cached data
    /// Ttl Seconds field

    pub ttl_seconds: Option<u64>,

    /// Cache key used
        pub cache_key: Option<String>,

    /// Cache generation timestamp
        pub cached_at: Option<DateTime<Utc>> ,
 )
}

impl Default for CacheInfo  {fn default() -> Self  {Self { is_cached: false,
            hit_ratio: 0.0,
            ttl_seconds: None,
    cache_key: None,
    cached_at: None;}}}
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct RateLimitStatus {
    /// Whether rate limiting is active
        pub is_rate_limited: bool,

    /// Requests remaining in current window
    /// Requests Remaining field

    pub requests_remaining: Option<u32>,

    /// Rate limit window reset time
        pub reset_time: Option<DateTime<Utc>>,

    /// Rate limit window duration
    /// Window Duration field

    pub window_duration: Option<Duration>,

    /// Current request rate (requests per second)
    /// Current Rate field

    pub current_rate: f64 ,
 )
}

impl Default for RateLimitStatus  {fn default() -> Self  {Self { is_rate_limited: false,
            requests_remaining: None,
    reset_time: None,
    window_duration: None,
    current_rate: 0.0;}}}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoutingMetadata {
    /// Selected service endpoint
    /// Selected Endpoint field

    pub selected_endpoint: Option<String>,

    /// Number of available endpoints
    /// Available Endpoints field

    pub available_endpoints: u32,

    /// Load balancing algorithm used
    /// Load Balancing Algorithm field

    pub load_balancing_algorithm: Option<String>,

    /// Service health scores
    pub service_health_scores: HashMap<String, f64>)

    /// Routing decision factors
    /// Decision Factors field

    pub decision_factors: Vec<RoutingDecisionFactor> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecisionFactor {
    /// Factor name
    /// Name identifier

    pub name: String,
    /// Factor weight in decision (0.0 - 1.0)
    /// Weight field

    pub weight: f64,

    /// Factor value
        pub value: serde_json::Value,

    /// Impact on routing decision
        pub impact: String ,
 )
}

/// Context for human-AI collaborative operations in service mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanInteractionContext {
    /// Human user identifier (when applicable)
    pub user_id: Option<String>,

    /// Current interaction mode
    /// Interaction Mode field

    pub interaction_mode: InteractionMode,
    /// User preferences for AI operations
    /// Preferences field

    pub preferences: AIUserPreferences,
    /// Whether human approval is required for this operation
    pub approval_required: bool,

    /// Confidence threshold for auto-execution
    /// Confidence Threshold field

    pub confidence_threshold: f64,

    /// Escalation configuration
    /// Escalation Config field

    pub escalation_config: EscalationConfig,
    /// Session context for multi-step operations
    /// Session Context field

    pub session_context: Option<SessionContext>,

    /// Service mesh specific context
    /// Service Mesh Context field

    pub service_mesh_context: ServiceMeshContext ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionMode {
    /// AI operates completely autonomously
    FullyAutonomous,
    /// AI suggests actions, human approves before execution
    HumanApproval,
    /// Real-time collaboration between human and AI
// AI
    /// Collaborative, Collaborative,
    /// Human directs strategy, AI executes tactics
    /// HumanDirected, HumanDirected,
    /// AI monitors and alerts, human makes key decisions
    /// HumanSupervised, HumanSupervised,
    /// Emergency mode - AI acts immediately, notifies human
    Emergency  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIUserPreferences {
    /// Preferred AI models for different operation types
    pub model_preferences: HashMap<String, String>)

    /// Auto-approval thresholds by operation category
    pub auto_approval_thresholds: HashMap<String, f64>)

    /// Notification preferences
    /// Notifications field

    pub notifications: NotificationPreferences,
    /// Resource usage limits and preferences
    /// Resource limitation configurations

    pub resource_limits: AIResourceLimits,
    /// Risk tolerance levels
    /// Risk Tolerance field

    pub risk_tolerance: RiskTolerance,
    /// Learning preferences (whether AI should learn from user behavior)
    /// Learning Enabled field

    pub learning_enabled: bool ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// Email notifications enabled
    /// Email Enabled field

    pub email_enabled: bool,

    /// Slack notifications enabled
    /// Slack Enabled field

    pub slack_enabled: bool,

    /// Webhook notifications enabled
    /// Webhook Enabled field

    pub webhook_enabled: bool,

    /// Minimum severity for notifications
    /// Minimum Severity field

    pub minimum_severity: ErrorSeverity,
    /// Notification channels by category
    pub channels_by_category: HashMap<String, Vec<String>> )
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResourceLimits {
    /// Maximum CPU usage percentage
    /// Max Cpu Percent field

    pub max_cpu_percent: f64,

    /// Maximum memory usage in bytes
        pub max_memory_bytes: u64,

    /// Maximum execution time
    /// Max Execution Time field

    pub max_execution_time: Duration,
    /// Maximum cost per operation
    /// Max Cost Per Operation field

    pub max_cost_per_operation: f64 ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskTolerance {
    /// Risk tolerance level (0.0 - 1.0, higher = more risk tolerant)
    /// Level field

    pub level: f64,

    /// Risk categories and their specific tolerances
    pub category_tolerances: HashMap<String, f64>)

    /// Whether to allow experimental features
    /// Allow Experimental field

    pub allow_experimental: bool ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    /// Service mesh specific escalation triggers
    /// Escalation Triggers field

    pub escalation_triggers: Vec<ServiceMeshEscalationTrigger>,

    /// Maximum time to wait for human response
    /// Human Response Timeout field

    pub human_response_timeout: Duration,
    /// Action to take if human doesn't respond in time
    /// Timeout Action field

    pub timeout_action: TimeoutAction,
    /// Escalation chain (who to contact in order)
    /// Escalation Chain field

    pub escalation_chain: Vec<String> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMeshEscalationTrigger { /// Service discovery failures
    ServiceDiscoveryFailure { failure_rate: f64 }})

    /// Load balancing issues
    LoadBalancingDegradation { threshold: f64 }})

    /// Circuit breaker activations
    CircuitBreakerActivation { service_pattern: String }})

    /// Cross-primal communication failures
    CrossPrimalFailure  {primal_type: String,
    failure_rate: f64 }})

    /// Security concerns in service mesh
    ServiceMeshSecurityConcern { severity: String }})

    /// Unknown service behavior
    UnknownServiceBehavior { anomaly_score: f64 }})

    /// Critical service impact
    CriticalServiceImpact { affected_services: Vec<String>;}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeoutAction {
    /// Proceed with default action
    /// ProceedWithDefault, ProceedWithDefault,
    /// Cancel operation
    /// Cancel, Cancel,
    /// Escalate to next level
    /// Escalate, Escalate,
    ExecuteReducedConfidence  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Session identifier
    /// Session Id field

    pub session_id: Uuid,
    /// Session start time
        pub started_at: DateTime<Utc>,

    /// Session expiration time
        pub expires_at: Option<DateTime<Utc>>,

    /// Session state data
    pub state: HashMap<String, serde_json: :Value>,
    /// Previous operations in session
    /// Operation History field

    pub operation_history: Vec<OperationHistoryItem> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationHistoryItem {
    /// Operation timestamp
    /// Timestamp when this was created or last updated

    pub timestamp: DateTime<Utc>,

    /// Operation type
    /// Operation Type field

    pub operation_type: String,
    /// Operation result
        pub result: String,
    /// Human involvement level
    /// Human Involvement field

    pub human_involvement: Option<String> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshContext {
    /// Preferred service routing strategies
    /// Routing Preferences field

    pub routing_preferences: Vec<String>,

    /// Load balancing preferences
    pub load_balancing_preferences: HashMap<String, String>)

    /// Circuit breaker tolerance
    /// Circuit Breaker Tolerance field

    pub circuit_breaker_tolerance: f64,

    /// Human notification preferences for service issues
    pub service_notification_preferences: NotificationPreferences ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    /// Action type for AI agents
    /// Action Type field

    pub action_type: String,
    /// Action parameters
    pub parameters: HashMap<String, serde_json: :Value>,
    /// Priority for execution
        pub priority: ActionPriority,
    /// Expected outcome
        pub expected_outcome: String,
    /// Confidence in suggestion
    /// Confidence field

    pub confidence: f64,

    /// Human approval required for this action
    /// Requires Human Approval field

    pub requires_human_approval: bool,

    /// Estimated execution time
    /// Estimated Execution Time field

    pub estimated_execution_time: Option<Duration> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionPriority {
    /// Low, Low,
    /// Medium, Medium)
    /// High, High,
    Urgent};
/// Helper functions for creating common AI-first responses
impl AIFirstError {
    /// Create a service mesh failure error
    pub fn service_mesh_failure() -> Self    {Self { code: "SERVICE_MESH_FAILURE".to_string(),
            message: message.to_string(),
            category: AIErrorCategory::ServiceMeshFailure,
            retry_strategy: RetryStrategy { should_retry: true,
                delay_ms: 1000,
                max_attempts: 3,
                backoff_strategy: BackoffType::Exponential { base: 2.0  ;

  ;

})
                retry_conditions: vec!["service_available".to_string()],"
                success_probability: 0.7;})
            automation_hints: vec![
                "Check service health".to_string()
                "Try alternative service endpoint".to_string()
            ])
            severity: ErrorSeverity::High,
            requires_human_intervention: false,
            context:  {let mut ctx = HashMap::new,
                ctx.insert()
                    "failed_service".to_string(),
                    serde_json::Value::String(service.to_string();
                ctx;}}}

    /// Create a human intervention required error
    pub fn human_intervention_required() -> Self   {Self { code: "HUMAN_INTERVENTION_REQUIRED".to_string(),
            message: format!("Human intervention required: {}", reason ;"
 ;
),
            category: AIErrorCategory::HumanInterventionRequired,
            retry_strategy: RetryStrategy  {should_retry: false,
                delay_ms: 0,
                max_attempts: 0,
                backoff_strategy: BackoffType::Linear,
                retry_conditions: vec!["human_approval_received".to_string()],"
                success_probability: 1.0} ;})
            automation_hints: vec![
                "Escalate to human operato" .to_string()
                "Provide context for decision".to_string()
            ])
            severity: ErrorSeverity::Medium,
            requires_human_intervention: true,
            context:  {let mut ctx = HashMap::new,
                ctx.insert()
                    "intervention_reason".to_string(),
                    serde_json::Value::String(reason.to_string();
                ctx;}}}}
