/// # Ecosystem Compliance Types
///
/// This module provides types and utilities for ecosystem compliance,
/// automation suggestions, and structured error recovery.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ecosystem-wide error interface
pub trait EcosystemError {
    /// Get error code for machine processing
    fn error_code(&self) -> String;

    /// Get automation suggestions
    fn automation_suggestions(&self) -> Vec<AutomationSuggestion>;

    /// Get retry strategy
    fn retry_strategy(&self) -> crate::ai_first::RetryStrategy;

    /// Check if human intervention is required
    fn requires_human_intervention(&self) -> bool;
}

/// Automation suggestion for AI agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationSuggestion {
    /// Action to take
    pub action: String,

    /// Confidence in this suggestion (0.0 - 1.0)
    pub confidence: f64,

    /// Parameters for the action
    pub parameters: HashMap<String, serde_json::Value>,

    /// Expected outcome
    pub expected_outcome: Option<String>,

    /// Estimated success rate (0.0 - 1.0)
    pub estimated_success_rate: f64,
}

impl AutomationSuggestion {
    /// Create a new automation suggestion
    pub fn new(action: &str, confidence: f64) -> Self {
        Self {
            action: action.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            parameters: HashMap::new(),
            expected_outcome: None,
            estimated_success_rate: 0.7,
        }
    }

    /// Add parameter to the suggestion
    pub fn with_parameter(mut self, key: &str, value: serde_json::Value) -> Self {
        self.parameters.insert(key.to_string(), value);
        self
    }

    /// Set expected outcome
    pub fn with_outcome(mut self, outcome: &str) -> Self {
        self.expected_outcome = Some(outcome.to_string());
        self
    }

    /// Set success rate estimate
    pub fn with_success_rate(mut self, rate: f64) -> Self {
        self.estimated_success_rate = rate.clamp(0.0, 1.0);
        self
    }
}

/// Error recovery context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryContext {
    /// Error occurrence timestamp
    pub timestamp: std::time::SystemTime,

    /// Number of previous failures
    pub failure_count: u32,

    /// Previous recovery attempts
    pub recovery_attempts: Vec<RecoveryAttempt>,

    /// Current system state
    pub system_state: HashMap<String, serde_json::Value>,
}

/// Recovery attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    /// Attempt timestamp
    pub timestamp: std::time::SystemTime,

    /// Recovery action taken
    pub action: String,

    /// Whether the attempt was successful
    pub success: bool,

    /// Additional context
    pub context: Option<String>,
}

/// Ecosystem error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcosystemSeverity {
    /// Information - no action required
    Info,
    /// Warning - attention recommended
    Warning,
    /// Error - action required
    Error,
    /// Critical - immediate action required
    Critical,
    /// Fatal - system failure
    Fatal,
}

/// Error classification for ecosystem routing
#[derive(Debug, Clone, Serialize, Deserialize)]
// NOTE: Custom error enum - consider migration to SongbirdError variants
pub enum ErrorClassification {
    /// Transient error that may resolve itself
    Transient,
    /// Persistent error requiring intervention
    Persistent,
    /// Configuration error requiring changes
    Configuration,
    /// Resource error requiring scaling
    Resource,
    /// Security error requiring attention
    Security,
    /// System error requiring investigation
    System,
}

/// Structured error reporting for ecosystem monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    /// Error identifier
    pub id: uuid::Uuid,

    /// Error classification
    pub classification: ErrorClassification,

    /// Severity level
    pub severity: EcosystemSeverity,

    /// Error message
    pub message: String,

    /// Service or component that generated the error
    pub source: String,

    /// Automation suggestions
    pub suggestions: Vec<AutomationSuggestion>,

    /// Recovery context
    pub recovery_context: Option<RecoveryContext>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ErrorReport {
    /// Create a new error report
    pub fn new(
        classification: ErrorClassification,
        severity: EcosystemSeverity,
        message: &str,
        source: &str,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            classification,
            severity,
            message: message.to_string(),
            source: source.to_string(),
            suggestions: vec![],
            recovery_context: None,
            metadata: HashMap::new(),
        }
    }

    /// Add automation suggestion
    pub fn with_suggestion(mut self, suggestion: AutomationSuggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add recovery context
    pub fn with_recovery_context(mut self, context: RecoveryContext) -> Self {
        self.recovery_context = Some(context);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}
