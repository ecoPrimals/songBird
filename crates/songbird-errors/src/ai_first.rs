/// # AI-First Response Format
///
/// Implementation of the Universal AI-first response format as required
/// by the `EcoPrimals` AI-First Citizen API Standard.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

/// Universal AI-first response format - MANDATORY for ecosystem compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdResponse<T> {
    /// Operation success status (machine-readable)
    pub success: bool,

    /// Strongly-typed response data
    pub data: T,

    /// AI-optimized error information
    pub error: Option<AIFirstError>,

    /// Unique request identifier for tracing
    pub request_id: Uuid,

    /// Processing time in milliseconds
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

impl<T> SongbirdResponse<T> {
    /// Create a successful AI-first response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            error: None,
            request_id: Uuid::new_v4(),
            processing_time_ms: 0, // Will be set by middleware
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score: 1.0,
            suggested_actions: vec![],
        }
    }

    /// Create a successful response with confidence and suggestions
    pub fn success_with_ai_context(
        data: T,
        confidence: f64,
        suggestions: Vec<SuggestedAction>,
    ) -> Self {
        Self {
            success: true,
            data,
            error: None,
            request_id: Uuid::new_v4(),
            processing_time_ms: 0,
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score: confidence.clamp(0.0, 1.0),
            suggested_actions: suggestions,
        }
    }

    /// Create an error response with data
    pub fn error_with_data(data: T, error: AIFirstError) -> Self {
        Self {
            success: false,
            data,
            error: Some(error),
            request_id: Uuid::new_v4(),
            processing_time_ms: 0,
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score: 0.0,
            suggested_actions: vec![],
        }
    }

    /// Create an error response
    ///
    /// # Errors
    ///
    /// This function always creates an error response and should be used carefully.
    #[must_use]
    pub fn error(error: AIFirstError) -> SongbirdResponse<()> {
        SongbirdResponse {
            success: false,
            data: (),
            error: Some(error),
            request_id: Uuid::new_v4(),
            processing_time_ms: 0,
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score: 0.0,
            suggested_actions: vec![],
        }
    }

    /// Check if this response is successful
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.success
    }

    /// Check if this response is an error
    #[must_use]
    pub const fn is_error(&self) -> bool {
        !self.success
    }

    /// Get the data from the response
    #[must_use]
    pub const fn get_data(&self) -> &T {
        &self.data
    }

    /// Convert to standard Result type
    ///
    /// # Errors
    /// Returns an error if the response indicates failure or contains error information.
    pub fn into_result(self) -> Result<T, Box<AIFirstError>> {
        if self.success && self.error.is_none() {
            Ok(evolved_success(self.data))
        } else {
            Err(Box::new(self.error.unwrap_or_else(|| AIFirstError {
                code: "UNKNOWN_ERROR".to_string(),
                message: "Response marked as failed but no error provided".to_string(),
                category: AIErrorCategory::Unknown,
                retry_strategy: RetryStrategy::no_retry(),
                automation_hints: vec![],
                severity: ErrorSeverity::Medium,
                requires_human_intervention: false,
                context: HashMap::new(),
            })))
        }
    }

    /// Map the data to a new type
    pub fn map<U, F>(self, f: F) -> SongbirdResponse<U>
    where
        F: FnOnce(T) -> U,
    {
        SongbirdResponse {
            success: self.success,
            data: f(self.data),
            error: self.error,
            request_id: self.request_id,
            processing_time_ms: self.processing_time_ms,
            ai_metadata: self.ai_metadata,
            human_context: self.human_context,
            confidence_score: self.confidence_score,
            suggested_actions: self.suggested_actions,
        }
    }
}

// Automatic conversion from standard Result to SongbirdResponse
impl<T, E> From<Result<T, E>> for SongbirdResponse<Option<T>>
where
    E: std::fmt::Display,
{
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(evolved_success(data)) => Self::success(Some(data)),
            Err(e) => Self::error_with_data(
                None,
                AIFirstError {
                    code: "CONVERSION_ERROR".to_string(),
                    message: e.to_string(),
                    category: AIErrorCategory::Unknown,
                    retry_strategy: RetryStrategy::no_retry(),
                    automation_hints: vec![],
                    severity: ErrorSeverity::Medium,
                    requires_human_intervention: false,
                    context: HashMap::new(),
                },
            ),
        }
    }
}

/// Helper trait for easily converting values to AI-first response format
///
/// This trait provides a convenient way to wrap any value in an `SongbirdResponse`
/// while maintaining type safety and ensuring all responses follow the AI-first
/// citizen API standard.
pub trait IntoSongbirdResponse<T> {
    /// Convert this value into an AI-first response format
    ///
    /// # Returns
    ///
    /// Returns an `SongbirdResponse<T>` with success status and the wrapped value
    fn into_ai_response(self) -> SongbirdResponse<T>;
}

impl<T> IntoSongbirdResponse<T> for T {
    fn into_ai_response(self) -> SongbirdResponse<T> {
        SongbirdResponse::success(self)
    }
}

impl<T, E> IntoSongbirdResponse<Option<T>> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn into_ai_response(self) -> SongbirdResponse<Option<T>> {
        match self {
            Ok(evolved_success(data)) => SongbirdResponse::success(Some(data)),
            Err(e) => SongbirdResponse::error_with_data(
                None,
                AIFirstError {
                    code: "CONVERSION_ERROR".to_string(),
                    message: e.to_string(),
                    category: AIErrorCategory::Unknown,
                    retry_strategy: RetryStrategy::no_retry(),
                    automation_hints: vec![],
                    severity: ErrorSeverity::Medium,
                    requires_human_intervention: false,
                    context: HashMap::new(),
                },
            ),
        }
    }
}

/// AI-optimized error structure with automation hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIFirstError {
    /// Machine-readable error code (`UPPER_SNAKE_CASE`)
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

/// Error categories for AI classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// NOTE: Custom error enum - consider migration to SongbirdError variants
pub enum AIErrorCategory {
    /// Configuration-related errors
    Configuration,
    /// Network-related errors
    Network,
    /// Authentication errors
    Authentication,
    /// Validation errors
    Validation,
    /// Resource errors
    Resource,
    /// Operation errors
    Operation,
    /// Internal system errors
    Internal,
    /// Unknown or unclassified errors
    Unknown,
    /// Insufficient computational resources
    ResourceLimitation,

    /// Configuration or parameter issues
    ConfigurationIssue,

    /// Authentication or authorization failures
    SecurityViolation,

    /// Network connectivity problems
    NetworkFailure,

    /// Runtime execution errors
    RuntimeError,

    /// System-level errors
    SystemError,

    /// Requires human decision or input
    HumanInterventionRequired,

    /// External dependency failures
    DependencyFailure,

    /// Rate limiting or throttling
    RateLimiting,
}

/// Automated retry strategy
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

impl RetryStrategy {
    /// Create exponential backoff retry strategy
    #[must_use]
    pub fn exponential_backoff(max_attempts: u32, initial_delay_ms: u64) -> Self {
        Self {
            should_retry: true,
            delay_ms: initial_delay_ms,
            max_attempts,
            backoff_strategy: BackoffType::Exponential { base: 2.0 },
            retry_conditions: vec!["network_available".to_string()],
            success_probability: 0.7,
        }
    }

    /// Create linear backoff retry strategy
    #[must_use]
    pub fn linear_backoff(max_attempts: u32, delay_ms: u64) -> Self {
        Self {
            should_retry: true,
            delay_ms,
            max_attempts,
            backoff_strategy: BackoffType::Linear,
            retry_conditions: vec!["service_available".to_string()],
            success_probability: 0.6,
        }
    }

    /// Create no-retry strategy
    #[must_use]
    pub fn no_retry() -> Self {
        Self {
            should_retry: false,
            delay_ms: 0,
            max_attempts: 0,
            backoff_strategy: BackoffType::Linear,
            retry_conditions: vec![],
            success_probability: 0.0,
        }
    }

    /// Create retry strategy with alternatives
    #[must_use]
    pub fn with_alternatives(alternatives: &[String]) -> Self {
        Self {
            should_retry: !alternatives.is_empty(),
            delay_ms: 1000,
            max_attempts: u32::try_from(alternatives.len()).unwrap_or(u32::MAX),
            backoff_strategy: BackoffType::Linear,
            retry_conditions: vec!["try_alternative_service".to_string()],
            success_probability: 0.8,
        }
    }

    /// Create backoff retry strategy (compatibility method)
    #[must_use]
    pub fn backoff_retry() -> Self {
        Self::exponential_backoff(3, 1000)
    }
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::exponential_backoff(3, 1000)
    }
}

/// Backoff strategy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffType {
    /// Linear backoff (constant delay)
    Linear,
    /// Exponential backoff with base multiplier
    Exponential {
        /// Base multiplier for exponential backoff calculation
        base: f64,
    },
    /// Fibonacci sequence backoff
    Fibonacci,
    /// Custom backoff formula
    Custom {
        /// Mathematical formula for custom retry timing calculation
        formula: String,
    },
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
// NOTE: Custom error enum - consider migration to SongbirdError variants
pub enum ErrorSeverity {
    /// Low severity - informational
    Low,
    /// Medium severity - warning
    Medium,
    /// High severity - error
    High,
    /// Critical severity - system failure
    Critical,
}

/// AI-specific metadata for decision making
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

    /// AI model recommendations
    pub recommended_models: Vec<String>,

    /// Processing complexity level
    pub complexity_level: ComplexityLevel,
}

impl Default for AIResponseMetadata {
    fn default() -> Self {
        Self {
            performance: PerformanceMetrics::default(),
            resource_usage: ResourceUsage::default(),
            quality_metrics: QualityMetrics::default(),
            cache_info: CacheInfo::default(),
            rate_limit_status: RateLimitStatus::default(),
            dependencies: vec![],
            recommended_models: vec!["gpt-4".to_string(), "claude-3".to_string()],
            complexity_level: ComplexityLevel::Medium,
        }
    }
}

/// Performance metrics for AI decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,

    /// Throughput in operations per second
    pub throughput_ops_per_sec: f64,

    /// Error rate (0.0 - 1.0)
    pub error_rate: f64,

    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage_percent: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_response_time_ms: 100.0,
            throughput_ops_per_sec: 1000.0,
            error_rate: 0.01,
            cpu_usage_percent: 30.0,
        }
    }
}

/// Resource utilization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Memory usage in MB
    pub memory_mb: f64,

    /// Disk usage in MB
    pub disk_mb: f64,

    /// Network bandwidth in Mbps
    pub network_mbps: f64,

    /// Connection count
    pub connection_count: u32,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_mb: 100.0,
            disk_mb: 50.0,
            network_mbps: 10.0,
            connection_count: 10,
        }
    }
}

/// Quality metrics for AI assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Data accuracy (0.0 - 1.0)
    pub accuracy: f64,

    /// Data completeness (0.0 - 1.0)
    pub completeness: f64,

    /// Response consistency (0.0 - 1.0)
    pub consistency: f64,

    /// Service reliability (0.0 - 1.0)
    pub reliability: f64,
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            accuracy: 0.95,
            completeness: 0.90,
            consistency: 0.98,
            reliability: 0.99,
        }
    }
}

/// Caching information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    /// Whether result is cached
    pub is_cached: bool,

    /// Cache hit rate (0.0 - 1.0)
    pub hit_rate: f64,

    /// Time to live in seconds
    pub ttl_seconds: u32,

    /// Cache key used
    pub cache_key: Option<String>,
}

impl Default for CacheInfo {
    fn default() -> Self {
        Self {
            is_cached: false,
            hit_rate: 0.8,
            ttl_seconds: 300,
            cache_key: None,
        }
    }
}

/// Rate limiting status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    /// Current rate limit (requests per minute)
    pub limit: u32,

    /// Remaining requests in current window
    pub remaining: u32,

    /// Time until reset in seconds
    pub reset_in_seconds: u32,

    /// Whether currently rate limited
    pub is_limited: bool,
}

impl Default for RateLimitStatus {
    fn default() -> Self {
        Self {
            limit: 1000,
            remaining: 950,
            reset_in_seconds: 45,
            is_limited: false,
        }
    }
}

/// Processing complexity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    /// Low complexity - simple operations
    Low,
    /// Medium complexity - standard operations
    Medium,
    /// High complexity - complex operations
    High,
    /// Critical complexity - system-level operations
    Critical,
}

/// Suggested actions for AI agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    /// Action identifier
    pub action: String,

    /// Confidence in this action (0.0 - 1.0)
    pub confidence: f64,

    /// Parameters for the action
    pub parameters: HashMap<String, serde_json::Value>,

    /// Expected outcome description
    pub expected_outcome: Option<String>,

    /// Risk level of this action
    pub risk_level: RiskLevel,
}

impl SuggestedAction {
    /// Create a new suggested action
    #[must_use]
    pub fn new(action: &str, confidence: f64) -> Self {
        Self {
            action: action.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            parameters: HashMap::new(),
            expected_outcome: None,
            risk_level: RiskLevel::Low,
        }
    }

    /// Add parameter to the action
    #[must_use]
    pub fn with_parameter(mut self, key: &str, value: serde_json::Value) -> Self {
        self.parameters.insert(key.to_string(), value);
        self
    }

    /// Set expected outcome
    #[must_use]
    pub fn with_outcome(mut self, outcome: &str) -> Self {
        self.expected_outcome = Some(outcome.to_string());
        self
    }

    /// Set risk level
    #[must_use]
    pub fn with_risk(mut self, risk: RiskLevel) -> Self {
        self.risk_level = risk;
        self
    }
}

/// Risk levels for suggested actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk - safe to automate
    Low,
    /// Medium risk - proceed with caution
    Medium,
    /// High risk - requires human approval
    High,
    /// Critical risk - human intervention required
    Critical,
}

/// Human interaction context for collaborative operations
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

    /// Session context for multi-step operations
    pub session_context: Option<SessionContext>,
}

/// Human-AI interaction modes
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

/// AI user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIUserPreferences {
    /// Preferred AI models for different operation types
    pub model_preferences: HashMap<String, String>,

    /// Auto-approval thresholds by operation category
    pub auto_approval_thresholds: HashMap<String, f64>,

    /// Risk tolerance levels
    pub risk_tolerance: RiskTolerance,

    /// Learning preferences (whether AI should learn from user behavior)
    pub learning_enabled: bool,
}

/// Risk tolerance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskTolerance {
    /// Conservative - minimize risks
    Conservative,
    /// Moderate - balanced approach
    Moderate,
    /// Aggressive - accept higher risks for performance
    Aggressive,
}

/// Session context for multi-step operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Session identifier
    pub session_id: Uuid,

    /// Session start time
    pub started_at: SystemTime,

    /// Previous operations in this session
    pub operation_history: Vec<String>,

    /// Session state data
    pub state: HashMap<String, serde_json::Value>,
}
