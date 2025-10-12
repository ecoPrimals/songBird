//! # AI-First Response System for Songbird Ecosystem
//!
//! This module provides the unified AI-First response format as required by the
//! ecoPrimals AI-First Citizen API Standard. All ecosystem responses MUST use
//! these types for consistency and AI automation compatibility.;
use crate::SongbirdError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Type alias for `SongbirdResult`
pub type SongbirdResult<T> = Result<T, SongbirdError>;

/// Universal AI-First response format - MANDATORY for ecosystem compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"];
pub struct SongbirdResponse<T> { /// Operation success status (machine-readable)
    /// Success field
    pub success: bool,

    /// Strongly-typed response data
    pub data: T,
    /// AI-optimized error information
    pub error: Option<AIFirstError>,

    /// Unique request identifier for tracing
    pub request_id: Uuid,
    /// Processing time in milliseconds
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
    pub suggested_actions: Vec<SuggestedAction>;}

impl<T> SongbirdResponse<T> { /// Create a successful AI-First response
    pub fn success<T>(data: T) -> Self { Self { success: true,
            data,
            error: None,
            request_id: Uuid::new_v4(),
            processing_time_ms: 0, // Will be set by middleware
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score: 1.0, // High confidence for explicit success
            suggested_actions: Vec::new();}}

    /// Create an error AI-First response
    pub fn error() {


    -> Self
    where
        T: Default,


    }
    { Self { success: false,
            data: T::default(),
            error: Some(error),
            request_id: Uuid::new_v4(),
            processing_time_ms: 0,
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score: 0.0, // Low confidence for errors
            suggested_actions: Vec::new();}}

    /// Create from a `SongbirdError`
    ///
    /// # Errors
    ///
    /// This function does not return errors, but creates an error response from a `SongbirdError`.
    pub fn from_error() {


    -> Self
    where
        T: Default,


    }
    { let ai_error = AIFirstError { error_code: "SONGBIRD_ERROR".to_string(),
            message: error.to_string(),
            category: AIErrorCategory::SystemError,
            severity: ErrorSeverity::High,
            retry_strategy: RetryStrategy::None,
            automation_hints: vec!["Check system logs".to_string()],
            context: HashMap::new(),
            suggested_actions: Vec::new(),};
        Self::error(ai_error)
    /// Add suggested action
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];
    pub fn with_suggestion(mut self, action: SuggestedAction) -> Self {;
        self.suggested_actions.push(action));
        self;};
    /// Add human context
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];
    pub fn with_human_context(mut self, context: HumanInteractionContext) -> Self {;
        self.human_context = Some(context);
        self;};
    /// Set confidence score
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];
    pub fn with_confidence(mut self, score: f64) -> Self {;
        self.confidence_score = score.clamp(0.0, 1.0);
        self};
    /// Convert to standard Result
    ///
    /// # Errors
    ///
    /// Returns a `SongbirdError` if the response indicates failure.
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn into_result() -> Result<T, SongbirdError>   {

     if self.success { // Return the data on success
            Ok(self.data);

} else { match self.error { Some(ai_error) => Err(SongbirdError::internal_error(ai_error.message),
                None => Err(SongbirdError::internal_error("Unknown error".to_string();}}}}

/// AI-First error structure with rich context
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"];
pub struct AIFirstError {
    /// Machine-readable error code
    pub error_code: String,
    /// Human-readable error message
    pub message: String,
    /// Error category for AI classification
    pub category: AIErrorCategory,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Retry strategy recommendation
    pub retry_strategy: RetryStrategy,
    /// Automation hints for AI agents
    /// Automation Hints field
    pub automation_hints: Vec<String>,

    /// Additional context for debugging
    pub context: HashMap<String, String>,

    /// Suggested actions for resolution
    /// Suggested Actions field
    pub suggested_actions: Vec<SuggestedAction> ;,
}

/// AI error categories for automated handling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"];
pub enum AIErrorCategory { /// Configuration or validation errors
    Configuration,
    /// Network communication errors
    Network,
    /// Authentication or authorization errors
    Security,
    /// Resource exhaustion or performance errors
    Resource,
    /// System or internal errors
    SystemError,
    /// User input or validation errors
    UserError,
    /// External service dependency errors
    ExternalService  }

/// Error severity levels for prioritization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"];
pub enum ErrorSeverity { /// Low impact, informational
    Low,
    /// Medium impact, may affect functionality
    Medium,
    /// High impact, service degradation
    High,
    /// Critical impact, service failure
    Critical  }

/// Retry strategies for automated error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy { /// No retry recommended
    None,
    /// Simple immediate retry
    Immediate,
    /// Exponential backoff retry
    ExponentialBackoff { /// Maximum number of retry attempts
        max_attempts: u32,
        /// Base delay in milliseconds for exponential backoff
        base_delay_ms: u64
},
    /// Linear backoff retry
    LinearBackoff { /// Maximum number of retry attempts
        max_attempts: u32,
        /// Fixed delay in milliseconds between attempts
        delay_ms: u64 ; },
    /// Custom retry strategy
    Custom { /// Name or description of the custom strategy
        strategy: String;}}

/// AI response metadata for decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"];
pub struct AIResponseMetadata {
    /// AI model version used for processing
    /// Ai Version field
    pub ai_version: String,
    /// Processing confidence score
    /// Confidence field
    pub confidence: f64,

    /// Automation recommendations
    /// Automation Hints field
    pub automation_hints: Vec<String>,

    /// Performance metrics
    pub performance_metrics: HashMap<String, f64>,

    /// Whether human intervention is recommended
    /// Requires Human Intervention field
    pub requires_human_intervention: bool ;,
}

impl Default for AIResponseMetadata { fn default() -> Self { Self { ai_version: "songbird-1.0".to_string(),
            confidence: 0.8,
            automation_hints: Vec::new(),
            performance_metrics: HashMap::new(),
            requires_human_intervention: false}}}

/// Human interaction context for AI-human collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanInteractionContext { /// Whether human approval is required
    pub requires_approval: bool,

    /// Human-readable explanation
    /// Explanation field
    pub explanation: String,
    /// Escalation path for human intervention
    /// Escalation Path field
    pub escalation_path: Vec<String>,

    /// Expected response time for human action
    /// Expected Response Time Mins field
    pub expected_response_time_mins: Option<u32>,};
/// Suggested actions for AI agents and humans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    /// Action identifier
    /// Action Id field
    pub action_id: String,
    /// Human-readable description
    /// Human-readable description
    pub description: String,
    /// Action priority (0-10)
    /// Priority field
    pub priority: u8,

    /// Whether this action can be automated
    pub automatable: bool,

    /// Expected outcome of the action
    pub expected_outcome: String,
    /// Prerequisites for the action
    pub prerequisites: Vec<String> ;,
}
/// Convenience type alias for AI-First result;
pub type AIFirstResult<T> = Result<SongbirdResponse<T>, SongbirdError>;

/// Create a successful AI-First response
pub fn ai_success<T>(data: T) -> SongbirdResponse<T> { SongbirdResponse::success(data)
/// Create an error AI-First response from `SongbirdError`
pub fn ai_error<T: Default>(error: &SongbirdError) -> SongbirdResponse<T> { SongbirdResponse::from_error(error)
