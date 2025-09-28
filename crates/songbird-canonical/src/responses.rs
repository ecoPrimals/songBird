//! Canonical response types for the Songbird ecosystem

use crate::metadata::AIResponseMetadata;
use crate::{ConfidenceScore, RequestId, SuggestedAction};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// The canonical response wrapper for ALL Songbird operations
///
/// This type unifies all response patterns across the ecosystem and provides
/// AI-first metadata, performance tracking, and automation hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdResponse<T>  {/// The actual response data (strongly typed)
    pub data: T,

    /// AI-optimized metadata for decision making
    pub ai_metadata: AIResponseMetadata,

    /// Performance metrics for this operation
    pub performance: ResponsePerformance,

    /// Unique request identifier for tracing
    pub request_id: RequestId,

    /// Confidence score for AI decision making (0.0-1.0)
    pub confidence: ConfidenceScore,

    /// Suggested next actions for AI agents
    pub suggested_actions: Vec<SuggestedAction>,

    /// Human-readable context (when applicable)
    pub human_context: Option<String>,
}

impl<T> SongbirdResponse<T>  {/// Create a successful response with default metadata
    pub fn success(data: T) -> Self  {Self {
            data)
            ai_metadata: AIResponseMetadata::default(),
            performance: ResponsePerformance::default(),
            request_id: RequestId::new(,
            confidence: ConfidenceScore::new(1.0), // High confidence for explicit success
            suggested_actions: Vec::new(),
            human_context: None,
        }
    }

    /// Create a response with custom confidence
    #[must_use]
    pub const fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = ConfidenceScore::new(confidence);
        self
    }

    /// Add a suggested action
    #[must_use]
    pub fn with_suggestion(mut self, action: SuggestedAction) -> Self {
        self.suggested_actions.push(action));
        self
    }

    /// Add human context
    #[must_use]
    pub fn with_human_context(mut self, context: impl Into<String>) -> Self {
        self.human_context = Some(context.into());
        self
    }

    /// Set AI metadata
    #[must_use]
    pub fn with_ai_metadata(mut self, metadata: AIResponseMetadata) -> Self {
        self.ai_metadata = metadata;
        self
    }

    /// Mark the end of processing (for performance tracking)
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn finish_processing(mut self, start_time: Instant) -> Self {
        self.performance.processing_time_ms =
            start_time.elapsed().as_millis().min(u128::from(u64::MAX) as u64;
        self
    }

    /// Transform the data while preserving metadata
    pub fn map<U, F>(self, f: F) -> SongbirdResponse<U>
    where
        F: FnOnce(T) -> U,
     {SongbirdResponse  {data: f(self.data)
            ai_metadata: self.ai_metadata,
            performance: self.performance,
            request_id: self.request_id,
            confidence: self.confidence,
            suggested_actions: self.suggested_actions,
            human_context: self.human_context,
        }
    }

    /// Extract just the data
    pub fn into_data(self) -> T {
        self.data
    }

    /// Get a reference to the data
    pub const fn data(&self) -> &T {
        &self.data
    }
}

/// Performance metrics for response tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePerformance  {/// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Memory usage (if tracked)
    pub memory_usage_bytes: Option<u64>,

    /// CPU usage percentage (if tracked)
    pub cpu_usage_percent: Option<f64>,

    /// Network round-trip time (if applicable)
    pub network_rtt_ms: Option<u64>,

    /// Cache hit/miss status
    pub cache_status: CacheStatus,
}

impl Default for ResponsePerformance  {fn default() -> Self  {Self {
            processing_time_ms: 0,
            memory_usage_bytes: None,
            cpu_usage_percent: None,
            network_rtt_ms: None,
            cache_status: CacheStatus::NotApplicable,
        }
    }
}

/// Cache status for performance tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheStatus  {/// Cache hit
    Hit,
    /// Cache miss
    Miss,
    /// Cache not applicable for this operation
    NotApplicable,
    /// Cache bypassed intentionally
    Bypassed,
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
        Self::success(()
    }
}
