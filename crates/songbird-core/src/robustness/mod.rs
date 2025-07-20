//! Robustness and reliability patterns for Songbird Orchestrator
//!
//! This module provides advanced reliability and fault tolerance capabilities including:
//! - Circuit breaker pattern for fault isolation
//! - Exponential backoff retry logic
//! - Adaptive timeout management
//! - Bulkhead pattern for resource isolation
//! - Rate limiting and throttling
//! - Health check coordination
//! - Graceful degradation strategies
//!
//! ## Architecture
//!
//! The robustness system is organized into focused modules:
//! - `config` - Configuration structures for all patterns
//! - `error_types` - Error types and state definitions
//! - `stats` - Statistics and monitoring structures
//! - `circuit_breaker` - Circuit breaker pattern implementation
//! - `rate_limiter` - Rate limiting with multiple algorithms
//! - `bulkhead` - Resource isolation using bulkhead pattern
//! - `health_checker` - Health monitoring and status tracking
//! - `manager` - Main coordinator for all patterns
//! - `utils` - Utility functions for common operations

pub mod bulkhead;
pub mod circuit_breaker;
pub mod config;
pub mod error_types;
pub mod health_checker;
pub mod manager;
pub mod rate_limiter;
pub mod stats;
pub mod utils;

// Re-export all public types for backward compatibility and convenience
pub use bulkhead::*;
pub use circuit_breaker::*;
pub use config::*;
pub use error_types::*;
pub use health_checker::*;
pub use manager::*;
pub use rate_limiter::*;
pub use stats::*;
pub use utils::*;
