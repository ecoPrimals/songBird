//! Resilience patterns for fault-tolerant systems
//!
//! This module provides modern resilience patterns for building robust,
//! fault-tolerant distributed systems.
//!
//! ## Patterns Implemented
//!
//! - **Circuit Breaker**: Prevent cascading failures, provide graceful degradation
//! - **Health Checks**: Standardized health monitoring for all components
//! - **Retry**: (Future) Automatic retry with exponential backoff
//! - **Timeout**: (Future) Bounded operation timeouts
//! - **Bulkhead**: (Future) Resource isolation
//!
//! ## Deep Debt Principles
//!
//! - ✅ Modern idiomatic Rust: async/await, type-safe, compile-time guarantees
//! - ✅ Zero unsafe code: All patterns implemented in safe Rust
//! - ✅ Composable: Patterns can be combined for sophisticated behavior
//! - ✅ Observable: Built-in statistics and state introspection
//!
//! ## Usage
//!
//! ```rust
//! use songbird_orchestrator::resilience::circuit_breaker::CircuitBreaker;
//! use songbird_orchestrator::resilience::health::{HealthCheck, HealthStatus};
//! use async_trait::async_trait;
//! use std::time::Duration;
//!
//! # struct MyService;
//! #[async_trait]
//! impl HealthCheck for MyService {
//!     async fn health(&self) -> HealthStatus {
//!         HealthStatus::healthy("my-service")
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create circuit breaker for external service calls
//!     let breaker = CircuitBreaker::builder()
//!         .failure_threshold(5)
//!         .timeout(Duration::from_secs(60))
//!         .build()?;
//!
//!     // Use circuit breaker to protect calls
//!     let result = breaker.call(|| async {
//!         // Call external service
//!         external_service_call().await
//!     }).await?;
//!
//!     Ok(())
//! }
//!
//! async fn external_service_call() -> Result<String, std::io::Error> {
//!     Ok("success".to_string())
//! }
//! ```

pub mod circuit_breaker;
pub mod health;

// Re-export main types for convenience
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitState};
pub use health::{AggregatedHealth, HealthCheck, HealthChecker, HealthStatus, Status};
