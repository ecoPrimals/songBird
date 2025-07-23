//! OS Substrate Integration with Performance Optimizations
//!
//! This module provides platform-agnostic OS operations by delegating to the
//! toadstool and biomeOS substrate systems with comprehensive performance
//! optimizations including TTL caching, connection pooling, and circuit breakers.
//!
//! ## Refactored Architecture
//!
//! The substrate system is organized into focused modules for better maintainability:
//! - `cache` - TTL caching with LRU eviction
//! - `circuit_breaker` - Circuit breaker resilience pattern
//! - `connection_pool` - HTTP connection pool management
//! - `clients` - Toadstool client implementation
//! - `types` - Data structures and enums
//! - `metrics` - Performance metrics tracking
//! - `substrate` - Main OSSubstrate implementation

pub mod cache;
pub mod circuit_breaker;
pub mod clients;
pub mod connection_pool;
pub mod metrics;
pub mod os_substrate;
pub mod types;

// Re-export all public types for backward compatibility
pub use cache::*;
pub use circuit_breaker::*;
pub use clients::*;
pub use connection_pool::*;
pub use metrics::*;
pub use os_substrate::*;
pub use types::*;
