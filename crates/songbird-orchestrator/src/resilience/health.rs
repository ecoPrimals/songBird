// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Health check trait and implementations
//!
//! Provides standardized health checking for all Songbird components,
//! enabling observability and automated failure detection.
//!
//! ## Deep Debt Evolution Principle
//!
//! **Before (No Standard)**:
//! ```ignore
//! // Each component implements health checks differently
//! async fn check_database() -> bool { /* ... */ }
//! async fn ping_service() -> Result<()> { /* ... */ }
//! async fn is_healthy() -> Status { /* ... */ }
//! ```
//!
//! **After (Standardized)**:
//! ```ignore
//! impl HealthCheck for Database {
//!     async fn health(&self) -> HealthStatus {
//!         // Standardized response
//!     }
//! }
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use songbird_orchestrator::resilience::health::{HealthCheck, HealthStatus, Status};
//! struct MyService;
//!
//! impl HealthCheck for MyService {
//!     async fn health(&self) -> HealthStatus {
//!         HealthStatus::healthy("my-service")
//!             .with_check("database", Status::Healthy)
//!             .with_check("cache", Status::Degraded)
//!     }
//! }
//! ```

#![expect(async_fn_in_trait, reason = "HealthCheck is the orchestrator-wide async health surface")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Health check trait for all components
///
/// Implement this trait to provide standardized health information
/// about your component or service.
pub trait HealthCheck: Send + Sync {
    /// Perform health check and return status
    ///
    /// This should be fast (<100ms) and not perform expensive operations.
    /// For expensive checks, cache results and return cached status.
    async fn health(&self) -> HealthStatus;

    /// Optional: Get component name
    ///
    /// Override this to provide a descriptive name for logging and metrics.
    fn name(&self) -> &'static str {
        "component"
    }
}

/// Overall health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Component is fully operational
    Healthy,

    /// Component is operational but degraded (e.g., high latency, partial failure)
    Degraded,

    /// Component is not operational
    Unhealthy,
}

impl Status {
    /// Check if status is healthy
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Check if status is degraded
    #[must_use]
    pub const fn is_degraded(&self) -> bool {
        matches!(self, Self::Degraded)
    }

    /// Check if status is unhealthy
    #[must_use]
    pub const fn is_unhealthy(&self) -> bool {
        matches!(self, Self::Unhealthy)
    }

    /// Get the worst status between two statuses
    #[must_use]
    pub const fn worst(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Unhealthy, _) | (_, Self::Unhealthy) => Self::Unhealthy,
            (Self::Degraded, _) | (_, Self::Degraded) => Self::Degraded,
            _ => Self::Healthy,
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Result of a health check
///
/// Contains overall status and individual component checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall status of the component
    pub status: Status,

    /// Component identifier
    pub component: String,

    /// Individual health checks
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub checks: HashMap<String, CheckResult>,

    /// Timestamp of health check
    pub timestamp: SystemTime,

    /// Optional error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Result of an individual health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Status of this check
    pub status: Status,

    /// Optional details about the check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Optional latency in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

impl HealthStatus {
    /// Create a new healthy status
    pub fn healthy(component: impl Into<String>) -> Self {
        Self {
            status: Status::Healthy,
            component: component.into(),
            checks: HashMap::new(),
            timestamp: SystemTime::now(),
            message: None,
        }
    }

    /// Create a new degraded status
    pub fn degraded(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: Status::Degraded,
            component: component.into(),
            checks: HashMap::new(),
            timestamp: SystemTime::now(),
            message: Some(message.into()),
        }
    }

    /// Create a new unhealthy status
    pub fn unhealthy(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: Status::Unhealthy,
            component: component.into(),
            checks: HashMap::new(),
            timestamp: SystemTime::now(),
            message: Some(message.into()),
        }
    }

    /// Add an individual check result (builder pattern)
    pub fn with_check(mut self, name: impl Into<String>, status: Status) -> Self {
        self.checks.insert(
            name.into(),
            CheckResult {
                status,
                message: None,
                latency_ms: None,
            },
        );
        self
    }

    /// Add an individual check with message (builder pattern)
    pub fn with_check_message(
        mut self,
        name: impl Into<String>,
        status: Status,
        message: impl Into<String>,
    ) -> Self {
        self.checks.insert(
            name.into(),
            CheckResult {
                status,
                message: Some(message.into()),
                latency_ms: None,
            },
        );
        self
    }

    /// Add an individual check with latency (builder pattern)
    pub fn with_check_latency(
        mut self,
        name: impl Into<String>,
        status: Status,
        latency_ms: u64,
    ) -> Self {
        self.checks.insert(
            name.into(),
            CheckResult {
                status,
                message: None,
                latency_ms: Some(latency_ms),
            },
        );
        self
    }

    /// Update overall status based on individual checks
    ///
    /// Sets status to the worst status among all checks.
    #[must_use]
    pub fn compute_overall_status(mut self) -> Self {
        let worst_status = self
            .checks
            .values()
            .map(|check| &check.status)
            .fold(Status::Healthy, |acc, status| acc.worst(status));

        self.status = worst_status;
        self
    }

    /// Check if overall status is healthy
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        self.status.is_healthy()
    }

    /// Check if any checks are unhealthy
    #[must_use]
    pub fn has_unhealthy_checks(&self) -> bool {
        self.checks.values().any(|check| check.status.is_unhealthy())
    }

    /// Get count of checks by status
    #[must_use]
    pub fn check_counts(&self) -> (usize, usize, usize) {
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;

        for check in self.checks.values() {
            match check.status {
                Status::Healthy => healthy += 1,
                Status::Degraded => degraded += 1,
                Status::Unhealthy => unhealthy += 1,
            }
        }

        (healthy, degraded, unhealthy)
    }
}

/// Aggregated health status for multiple components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedHealth {
    /// Overall status
    pub status: Status,

    /// Individual component statuses
    pub components: HashMap<String, HealthStatus>,

    /// Timestamp
    pub timestamp: SystemTime,
}

impl AggregatedHealth {
    /// Create new aggregated health
    #[must_use]
    pub fn new() -> Self {
        Self {
            status: Status::Healthy,
            components: HashMap::new(),
            timestamp: SystemTime::now(),
        }
    }

    /// Add a component health status
    #[must_use]
    pub fn add_component(mut self, status: HealthStatus) -> Self {
        let component_name = status.component.clone();
        self.status = self.status.worst(&status.status);
        self.components.insert(component_name, status);
        self
    }

    /// Check if any component is unhealthy
    #[must_use]
    pub fn has_unhealthy_components(&self) -> bool {
        self.components.values().any(|s| s.status.is_unhealthy())
    }

    /// Get component count by status
    #[must_use]
    pub fn component_counts(&self) -> (usize, usize, usize) {
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;

        for component in self.components.values() {
            match component.status {
                Status::Healthy => healthy += 1,
                Status::Degraded => degraded += 1,
                Status::Unhealthy => unhealthy += 1,
            }
        }

        (healthy, degraded, unhealthy)
    }
}

impl Default for AggregatedHealth {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for performing health checks with timeout
pub struct HealthChecker {
    timeout: std::time::Duration,
}

impl HealthChecker {
    /// Create new health checker with timeout
    #[must_use]
    pub const fn new(timeout: std::time::Duration) -> Self {
        Self {
            timeout,
        }
    }

    /// Check health with timeout
    pub async fn check<T: HealthCheck + ?Sized>(&self, component: &T) -> HealthStatus {
        match tokio::time::timeout(self.timeout, component.health()).await {
            Ok(status) => status,
            Err(_) => HealthStatus::unhealthy(
                component.name(),
                format!("Health check timeout after {:?}", self.timeout),
            ),
        }
    }

    /// Check multiple components in parallel
    pub async fn check_all<T: HealthCheck + ?Sized>(&self, components: &[&T]) -> AggregatedHealth {
        let mut health = AggregatedHealth::new();

        // Check all components in parallel
        let futures: Vec<_> = components.iter().map(|component| self.check(*component)).collect();

        let results = futures::future::join_all(futures).await;

        for status in results {
            health = health.add_component(status);
        }

        health
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHealthyService;
    struct MockDegradedService;
    struct MockUnhealthyService;

    impl HealthCheck for MockHealthyService {
        async fn health(&self) -> HealthStatus {
            HealthStatus::healthy("mock-healthy")
        }
    }

    impl HealthCheck for MockDegradedService {
        async fn health(&self) -> HealthStatus {
            HealthStatus::degraded("mock-degraded", "High latency")
        }
    }

    impl HealthCheck for MockUnhealthyService {
        async fn health(&self) -> HealthStatus {
            HealthStatus::unhealthy("mock-unhealthy", "Connection failed")
        }
    }

    #[test]
    fn test_status_worst() {
        assert_eq!(Status::Healthy.worst(&Status::Healthy), Status::Healthy);
        assert_eq!(Status::Healthy.worst(&Status::Degraded), Status::Degraded);
        assert_eq!(Status::Healthy.worst(&Status::Unhealthy), Status::Unhealthy);
        assert_eq!(Status::Degraded.worst(&Status::Unhealthy), Status::Unhealthy);
    }

    #[test]
    fn test_health_status_builder() {
        let status = HealthStatus::healthy("test")
            .with_check("database", Status::Healthy)
            .with_check("cache", Status::Degraded)
            .compute_overall_status();

        assert_eq!(status.status, Status::Degraded);
        assert_eq!(status.checks.len(), 2);
    }

    #[test]
    fn test_health_status_counts() {
        let status = HealthStatus::healthy("test")
            .with_check("db", Status::Healthy)
            .with_check("cache", Status::Degraded)
            .with_check("api", Status::Unhealthy);

        let (healthy, degraded, unhealthy) = status.check_counts();
        assert_eq!(healthy, 1);
        assert_eq!(degraded, 1);
        assert_eq!(unhealthy, 1);
    }

    #[tokio::test]
    async fn test_health_checker() {
        let service = MockHealthyService;
        let checker = HealthChecker::new(std::time::Duration::from_secs(1));

        let status = checker.check(&service).await;
        assert!(status.is_healthy());
    }

    #[tokio::test]
    async fn test_aggregated_health() {
        let health = AggregatedHealth::new()
            .add_component(HealthStatus::healthy("service1"))
            .add_component(HealthStatus::degraded("service2", "Slow"))
            .add_component(HealthStatus::unhealthy("service3", "Down"));

        assert_eq!(health.status, Status::Unhealthy);
        assert_eq!(health.components.len(), 3);

        let (healthy, degraded, unhealthy) = health.component_counts();
        assert_eq!(healthy, 1);
        assert_eq!(degraded, 1);
        assert_eq!(unhealthy, 1);
    }

    #[tokio::test]
    async fn test_check_all() {
        let checker = HealthChecker::new(std::time::Duration::from_secs(1));
        let a = checker.check(&MockHealthyService).await;
        let b = checker.check(&MockDegradedService).await;
        let c = checker.check(&MockUnhealthyService).await;
        let health = AggregatedHealth::new().add_component(a).add_component(b).add_component(c);

        assert_eq!(health.status, Status::Unhealthy);
        assert_eq!(health.components.len(), 3);
    }
}
