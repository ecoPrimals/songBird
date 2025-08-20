//! Response types for the Songbird ecosystem
//!
//! Moved from songbird-canonical to break circular dependency

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The canonical response wrapper for ALL Songbird operations
///
/// This type unifies all response patterns across the ecosystem and provides
/// AI-first metadata, performance tracking, and automation hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdResponse<T> {
    /// The actual response data (strongly typed)
    pub data: T,

    /// Performance metrics for this operation
    pub performance: ResponsePerformance,

    /// Unique request identifier for tracing
    pub request_id: Uuid,

    /// Confidence score for AI decision making (0.0-1.0)
    pub confidence: f64,

    /// Human-readable context (when applicable)
    pub human_context: Option<String>,
}

impl<T> SongbirdResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            data,
            performance: ResponsePerformance::default(),
            request_id: Uuid::new_v4(),
            confidence: 1.0,
            human_context: None,
        }
    }

    /// Check if this response represents a successful operation
    ///
    /// For SongbirdResponse, this always returns true since errors are handled
    /// via the Result type wrapper (SongbirdResult<SongbirdResponse<T>>)
    pub const fn is_success(&self) -> bool {
        true
    }

    /// Check if this response represents an error
    ///
    /// For SongbirdResponse, this always returns false since errors are handled
    /// via the Result type wrapper (SongbirdResult<SongbirdResponse<T>>)
    pub const fn is_error(&self) -> bool {
        false
    }

    /// Transform the data while preserving metadata
    pub fn map<U, F>(self, f: F) -> SongbirdResponse<U>
    where
        F: FnOnce(T) -> U,
    {
        SongbirdResponse {
            data: f(self.data),
            performance: self.performance,
            request_id: self.request_id,
            confidence: self.confidence,
            human_context: self.human_context,
        }
    }

    /// Extract just the data
    pub fn into_data(self) -> T {
        self.data
    }

    /// Extract just the data (alias for into_data for compatibility)
    pub fn unwrap_data(self) -> T {
        self.data
    }

    /// Get a reference to the data
    pub const fn data(&self) -> &T {
        &self.data
    }
}

/// Performance metrics for response tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponsePerformance {
    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Memory usage (if tracked)
    pub memory_usage_bytes: Option<u64>,

    /// CPU usage percentage (if tracked)
    pub cpu_usage_percent: Option<f64>,

    /// Network round-trip time (if applicable)
    pub network_rtt_ms: Option<u64>,
}

// Convenience implementations for common types
impl<T> From<T> for SongbirdResponse<T> {
    fn from(data: T) -> Self {
        Self::success(data)
    }
}

impl SongbirdResponse<()> {
    /// Create a unit response (for operations that don't return data)
    #[must_use]
    pub fn unit() -> Self {
        Self::success(())
    }
}
