//! Evolved Response System for Songbird
//!
//! This module provides a unified, idiomatic Rust response system that seamlessly
//! integrates AI-first patterns with standard Result types. It leverages zero-cost
//! abstractions and trait-based design for maximum flexibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;
use uuid::Uuid;

// Re-export AI types for compatibility
pub use crate::ai_first::{AIErrorCategory, AIFirstError, RetryStrategy, ErrorSeverity, SuggestedAction};

/// The evolved response type that unifies all Songbird response patterns
///
/// This type provides both AI-first capabilities and idiomatic Rust patterns.
/// It can be used directly or converted to/from standard Result types seamlessly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolvedResponse<T> {
    /// The actual response data (strongly typed)
    pub data: T,
    
    /// AI-optimized metadata for decision making
    pub ai_metadata: AIMetadata,
    
    /// Performance tracking information
    pub performance: PerformanceMetrics,
    
    /// Request tracing information
    pub trace_info: TraceInfo,
    
    /// Human interaction context (when applicable)
    pub human_context: Option<HumanContext>,
}

/// AI-specific metadata for enhanced decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMetadata {
    /// Confidence score for AI decision making (0.0-1.0)
    pub confidence: f64,
    
    /// Suggested next actions for AI agents
    pub suggested_actions: Vec<SuggestedAction>,
    
    /// Automation hints for AI processing
    pub automation_hints: Vec<String>,
    
    /// Whether this operation requires human intervention
    pub requires_human_intervention: bool,
}

/// Performance metrics for response tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    
    /// Memory usage in bytes (if tracked)
    pub memory_usage_bytes: Option<u64>,
    
    /// Network latency in milliseconds (if applicable)
    pub network_latency_ms: Option<u64>,
    
    /// Cache hit/miss information
    pub cache_hit: Option<bool>,
}

/// Request tracing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceInfo {
    /// Unique request identifier
    pub request_id: Uuid,
    
    /// Parent request ID (for nested operations)
    pub parent_id: Option<Uuid>,
    
    /// Operation timestamp
    pub timestamp: SystemTime,
    
    /// Service/component that generated this response
    pub source: String,
}

/// Human interaction context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanContext {
    /// Human-readable description
    pub description: String,
    
    /// Urgency level for human attention
    pub urgency: HumanUrgency,
    
    /// Additional context data
    pub context_data: HashMap<String, String>,
}

/// Urgency levels for human intervention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HumanUrgency {
    /// Low priority - can be handled later
    Low,
    /// Medium priority - should be handled soon
    Medium,
    /// High priority - needs immediate attention
    High,
    /// Critical - requires urgent human intervention
    Critical,
}

impl<T> EvolvedResponse<T> {
    /// Create a new successful response with default metadata
    pub fn success(data: T) -> Self {
        Self {
            data,
            ai_metadata: AIMetadata::default(),
            performance: PerformanceMetrics::default(),
            trace_info: TraceInfo::new("songbird-core"),
            human_context: None,
        }
    }
    
    /// Create a response with custom AI metadata
    pub fn with_ai_metadata(data: T, ai_metadata: AIMetadata) -> Self {
        Self {
            data,
            ai_metadata,
            performance: PerformanceMetrics::default(),
            trace_info: TraceInfo::new("songbird-core"),
            human_context: None,
        }
    }
    
    /// Create a response with performance tracking
    pub fn with_performance(data: T, performance: PerformanceMetrics) -> Self {
        Self {
            data,
            ai_metadata: AIMetadata::default(),
            performance,
            trace_info: TraceInfo::new("songbird-core"),
            human_context: None,
        }
    }
    
    /// Transform the data while preserving all metadata
    pub fn map<U, F>(self, f: F) -> EvolvedResponse<U>
    where
        F: FnOnce(T) -> U,
    {
        EvolvedResponse {
            data: f(self.data),
            ai_metadata: self.ai_metadata,
            performance: self.performance,
            trace_info: self.trace_info,
            human_context: self.human_context,
        }
    }
    
    /// Try to transform the data, preserving metadata on success
    pub fn and_then<U, F, E>(self, f: F) -> Result<EvolvedResponse<U>, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        match f(self.data) {
            Ok(new_data) => Ok(EvolvedResponse {
                data: new_data,
                ai_metadata: self.ai_metadata,
                performance: self.performance,
                trace_info: self.trace_info,
                human_context: self.human_context,
            }),
            Err(e) => Err(e),
        }
    }
    
    /// Extract just the data, discarding metadata
    pub fn into_data(self) -> T {
        self.data
    }
    
    /// Get a reference to the data
    pub const fn data(&self) -> &T {
        &self.data
    }
    
    /// Get the confidence score
    pub const fn confidence(&self) -> f64 {
        self.ai_metadata.confidence
    }
    
    /// Check if human intervention is required
    pub const fn requires_human_intervention(&self) -> bool {
        self.ai_metadata.requires_human_intervention
    }
    
    /// Add human context to the response
    pub fn with_human_context(mut self, context: HumanContext) -> Self {
        self.human_context = Some(context);
        self
    }
    
    /// Set confidence score
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.ai_metadata.confidence = confidence.clamp(0.0, 1.0);
        self
    }
    
    /// Add suggested actions
    pub fn with_suggestions(mut self, actions: Vec<SuggestedAction>) -> Self {
        self.ai_metadata.suggested_actions = actions;
        self
    }
}

// Default implementations
impl Default for AIMetadata {
    fn default() -> Self {
        Self {
            confidence: 1.0,
            suggested_actions: Vec::new(),
            automation_hints: Vec::new(),
            requires_human_intervention: false,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            processing_time_ms: 0,
            memory_usage_bytes: None,
            network_latency_ms: None,
            cache_hit: None,
        }
    }
}

impl TraceInfo {
    pub fn new(source: &str) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            parent_id: None,
            timestamp: SystemTime::now(),
            source: source.to_string(),
        }
    }
    
    pub fn with_parent(source: &str, parent_id: Uuid) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            parent_id: Some(parent_id),
            timestamp: SystemTime::now(),
            source: source.to_string(),
        }
    }
}

impl Default for HumanUrgency {
    fn default() -> Self {
        Self::Low
    }
}

// Conversion traits for seamless integration

/// Convert from EvolvedResponse to standard Result
impl<T> From<EvolvedResponse<T>> for Result<T, ()> {
    fn from(response: EvolvedResponse<T>) -> Self {
        Ok(response.data)
    }
}

/// Convert from Result to EvolvedResponse (for Ok case)
impl<T, E> From<Result<T, E>> for EvolvedResponse<T>
where
    E: fmt::Debug,
{
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(data) => EvolvedResponse::success(data),
            Err(_) => panic!("Cannot convert Err to EvolvedResponse - use try_from instead"),
        }
    }
}

/// Evolved Result type that combines Result with EvolvedResponse
pub type EvolvedResult<T, E = crate::SongbirdError> = Result<EvolvedResponse<T>, E>;

/// Type alias for the most common Songbird result pattern
pub type SongbirdEvolvedResult<T> = EvolvedResult<T, crate::SongbirdError>;

// Convenience functions

/// Create a successful evolved response
pub fn evolved_success<T>(data: T) -> EvolvedResponse<T> {
    EvolvedResponse::success(data)
}

/// Create a successful evolved result
pub fn evolved_ok<T>(data: T) -> EvolvedResult<T> {
    Ok(EvolvedResponse::success(data))
}

/// Create an error evolved result
pub fn evolved_err<T>(error: crate::SongbirdError) -> EvolvedResult<T> {
    Err(error)
}

// AI-First compatibility layer

/// Legacy SongbirdResponse type alias for backward compatibility
pub type SongbirdResponse<T> = EvolvedResponse<T>;

/// Legacy success function for backward compatibility
pub fn success<T>(data: T) -> SongbirdResponse<T> {
    EvolvedResponse::success(data)
}

/// Simple success function that returns just the data (for SongbirdResult<()> patterns)
pub fn simple_success() -> () {
    ()
}

/// Simple success function that returns just the data (for Result<T, E> patterns)
pub fn simple_ok<T>(data: T) -> T {
    data
}

/// Legacy SongbirdResult type for backward compatibility
pub type SongbirdResult<T> = EvolvedResult<T>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_evolved_response_creation() {
        let response = EvolvedResponse::success("test data");
        assert_eq!(response.data(), &"test data");
        assert_eq!(response.confidence(), 1.0);
        assert!(!response.requires_human_intervention());
    }
    
    #[test]
    fn test_response_mapping() {
        let response = EvolvedResponse::success(42);
        let mapped = response.map(|x| x * 2);
        assert_eq!(mapped.data(), &84);
    }
    
    #[test]
    fn test_confidence_clamping() {
        let response = EvolvedResponse::success("test")
            .with_confidence(1.5); // Should clamp to 1.0
        assert_eq!(response.confidence(), 1.0);
        
        let response = EvolvedResponse::success("test")
            .with_confidence(-0.5); // Should clamp to 0.0
        assert_eq!(response.confidence(), 0.0);
    }
    
    #[test]
    fn test_and_then_success() {
        let response = EvolvedResponse::success(42);
        let result = response.and_then(|x| Ok::<_, &str>(x * 2));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data(), &84);
    }
    
    #[test]
    fn test_and_then_error() {
        let response = EvolvedResponse::success(42);
        let result = response.and_then(|_| Err::<i32, &str>("error"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "error");
    }
} 