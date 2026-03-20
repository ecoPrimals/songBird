// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Mock `BearDog` Security Primal
//!
//! Provides HTTP endpoints that simulate `BearDog`'s security, authentication, and deployment features.

#![allow(clippy::unused_async)]

use super::common::{HealthStatus, MockPrimalServer, MockServerState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Authentication result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthResult {
    /// Authentication succeeded
    Authorized,
    /// Authentication failed
    Unauthorized,
    /// Token expired
    Expired,
    /// Invalid credentials
    Invalid,
}

/// Security metrics from `BearDog`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    /// Number of active sessions
    pub active_sessions: u32,
    /// Number of failed auth attempts (last hour)
    pub failed_auth_attempts: u32,
    /// Number of blocked IPs
    pub blocked_ips: u32,
    /// Security score (0.0 - 1.0)
    pub security_score: f64,
}

impl Default for SecurityMetrics {
    fn default() -> Self {
        Self {
            active_sessions: 10,
            failed_auth_attempts: 2,
            blocked_ips: 0,
            security_score: 0.95,
        }
    }
}

/// Mock `BearDog` security server
#[derive(Debug, Clone)]
pub struct MockBearDog {
    state: Arc<MockServerState>,
    auth_results: Arc<RwLock<HashMap<String, AuthResult>>>,
    security_metrics: Arc<RwLock<SecurityMetrics>>,
}

impl MockBearDog {
    /// Create a new mock `BearDog` server
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(MockServerState::new(0)),
            auth_results: Arc::new(RwLock::new(HashMap::new())),
            security_metrics: Arc::new(RwLock::new(SecurityMetrics::default())),
        }
    }

    /// Start the mock server
    ///
    /// # Errors
    ///
    /// Currently never returns an error, but signature allows for future error cases.
    pub async fn start(&mut self) -> Result<u16, Box<dyn std::error::Error>> {
        let port = fastrand::u16(10000..60000);
        self.state = Arc::new(MockServerState::new(port));
        Ok(port)
    }

    /// Stop the mock server
    pub async fn stop(&self) {
        // Server cleanup
    }

    /// Configure authentication result for a specific token
    ///
    /// # Panics
    ///
    /// Panics if the internal auth results lock is poisoned.
    pub fn set_auth_result(&self, token: impl Into<String>, result: AuthResult) {
        let mut results = self.auth_results.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        results.insert(token.into(), result);
    }

    /// Check authentication for a token
    ///
    /// # Panics
    ///
    /// Panics if the internal auth results lock is poisoned.
    #[must_use]
    pub fn check_auth(&self, token: &str) -> AuthResult {
        self.state.increment_requests();

        let results = self.auth_results.read().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        results.get(token).cloned().unwrap_or(AuthResult::Unauthorized)
    }

    /// Set active sessions count
    ///
    /// # Panics
    ///
    /// Panics if the internal security metrics lock is poisoned.
    pub fn set_active_sessions(&self, count: u32) {
        let mut metrics = self.security_metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.active_sessions = count;
    }

    /// Set failed auth attempts
    ///
    /// # Panics
    ///
    /// Panics if the internal security metrics lock is poisoned.
    pub fn set_failed_auth_attempts(&self, count: u32) {
        let mut metrics = self.security_metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.failed_auth_attempts = count;
    }

    /// Get security metrics
    ///
    /// # Panics
    ///
    /// Panics if the internal security metrics lock is poisoned.
    #[must_use]
    pub fn get_metrics(&self) -> SecurityMetrics {
        self.security_metrics
            .read()
            .unwrap_or_else(|poisoned| {
                tracing::warn!("RwLock poisoned in test mock, recovering");
                poisoned.into_inner()
            })
            .clone()
    }

    /// Simulate security incident
    ///
    /// # Panics
    ///
    /// Panics if the internal security metrics lock is poisoned.
    pub fn simulate_security_incident(&self) {
        let mut metrics = self.security_metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.failed_auth_attempts = 150;
        metrics.blocked_ips = 25;
        metrics.security_score = 0.45;
        drop(metrics);
        self.state.set_health(HealthStatus::Degraded);
    }

    /// Simulate normal operation
    ///
    /// # Panics
    ///
    /// Panics if the internal security metrics lock is poisoned.
    pub fn simulate_normal_operation(&self) {
        let mut metrics = self.security_metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.failed_auth_attempts = 2;
        metrics.blocked_ips = 0;
        metrics.security_score = 0.95;
        drop(metrics);
        self.state.set_health(HealthStatus::Healthy);
    }
}

impl Default for MockBearDog {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPrimalServer for MockBearDog {
    fn port(&self) -> u16 {
        self.state.port
    }

    fn set_health(&self, status: HealthStatus) {
        self.state.set_health(status);
    }

    fn get_health(&self) -> HealthStatus {
        self.state.get_health()
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default
)]
#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::float_cmp)]
#[allow(clippy::useless_vec)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
mod tests {
    #![allow(clippy::all)]
    #![allow(unused)]

    use super::*;
    use songbird_types::SongbirdError;

    #[tokio::test]
    async fn test_mock_beardog_auth() {
        let mock = MockBearDog::new();

        mock.set_auth_result("valid_token", AuthResult::Authorized);
        mock.set_auth_result("expired_token", AuthResult::Expired);

        assert_eq!(mock.check_auth("valid_token"), AuthResult::Authorized);
        assert_eq!(mock.check_auth("expired_token"), AuthResult::Expired);
        assert_eq!(mock.check_auth("unknown_token"), AuthResult::Unauthorized);
    }

    #[tokio::test]
    async fn test_mock_beardog_scenarios() {
        let mock = MockBearDog::new();

        // Test security incident
        mock.simulate_security_incident();
        let metrics = mock.get_metrics();
        assert!(metrics.failed_auth_attempts > 100);
        assert_eq!(mock.get_health(), HealthStatus::Degraded);

        // Test normal operation
        mock.simulate_normal_operation();
        let metrics = mock.get_metrics();
        assert!(metrics.failed_auth_attempts < 10);
        assert_eq!(mock.get_health(), HealthStatus::Healthy);
    }

    // ========== NEW TESTS (5 tests to improve coverage) ==========

    #[tokio::test]
    async fn test_beardog_server_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock = MockBearDog::new();
        let port = mock
            .start()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Server should start: {}", e)))?;
        assert!(port > 0);
        assert_eq!(mock.port(), port);
        mock.stop().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_security_metrics_default() {
        let mock = MockBearDog::new();
        let metrics = mock.get_metrics();
        assert_eq!(metrics.active_sessions, 10);
        assert_eq!(metrics.failed_auth_attempts, 2);
        assert_eq!(metrics.blocked_ips, 0);
        assert!((metrics.security_score - 0.95).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_auth_result_variants() {
        let mock = MockBearDog::new();
        mock.set_auth_result("auth_token", AuthResult::Authorized);
        mock.set_auth_result("invalid_token", AuthResult::Invalid);
        mock.set_auth_result("expired_token", AuthResult::Expired);

        assert_eq!(mock.check_auth("auth_token"), AuthResult::Authorized);
        assert_eq!(mock.check_auth("invalid_token"), AuthResult::Invalid);
        assert_eq!(mock.check_auth("expired_token"), AuthResult::Expired);
    }

    #[tokio::test]
    async fn test_health_status_management() {
        let mock = MockBearDog::new();
        assert_eq!(mock.get_health(), HealthStatus::Healthy);

        mock.set_health(HealthStatus::Degraded);
        assert_eq!(mock.get_health(), HealthStatus::Degraded);

        mock.set_health(HealthStatus::Unhealthy);
        assert_eq!(mock.get_health(), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_beardog_default_trait() {
        let mock = MockBearDog::default();
        assert_eq!(mock.port(), 0);
        assert_eq!(mock.get_health(), HealthStatus::Healthy);
    }
}
