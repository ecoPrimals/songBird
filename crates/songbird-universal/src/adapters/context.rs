//! # Shared Adapter Context
//!
//! Common context types and utilities used across all universal adapters.

use uuid::Uuid;

/// Zero-Cost Adapter Context - passed through async call chains
///
/// This context provides request tracking, telemetry, and performance metrics
/// across all adapter types without runtime overhead.
#[derive(Debug, Clone)]
pub struct AdapterContext {
    /// Request ID for tracing and correlation
    pub request_id: Uuid,
    /// Source component for telemetry
    pub source: &'static str,
    /// Performance tracking start time
    pub start_time: std::time::Instant,
}

impl AdapterContext {
    /// Create new context with automatic request ID generation
    pub fn new(source: &'static str) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            source,
            start_time: std::time::Instant::now(),
        }
    }

    /// Create context with specific request ID (for correlation)
    pub fn with_request_id(source: &'static str, request_id: Uuid) -> Self {
        Self {
            request_id,
            source,
            start_time: std::time::Instant::now(),
        }
    }

    /// Get elapsed time for performance metrics
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u128 {
        self.elapsed().as_millis()
    }

    /// Create a child context for sub-operations
    pub fn child(&self, source: &'static str) -> Self {
        Self {
            request_id: self.request_id,
            source,
            start_time: std::time::Instant::now(),
        }
    }
}
