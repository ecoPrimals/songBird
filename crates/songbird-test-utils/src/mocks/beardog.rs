//! Mock `BearDog` Security Primal
//!
//! Provides HTTP endpoints that simulate `BearDog`'s security, authentication, and deployment features.

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
