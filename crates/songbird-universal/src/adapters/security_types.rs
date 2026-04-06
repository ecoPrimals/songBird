// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Types for the security capability adapter (metrics, health, auth results).

use serde::{Deserialize, Serialize};

/// Security metrics from any security capability provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    /// Number of active authenticated sessions
    pub active_sessions: u32,
    /// Number of failed authentication attempts in the last hour
    pub failed_auth_attempts: u32,
    /// Number of currently blocked IPs
    pub blocked_ips: u32,
    /// Security score (0.0 - 1.0)
    pub security_score: f64,
    /// Timestamp of metrics collection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl SecurityMetrics {
    /// Check if security is under attack
    #[must_use]
    pub const fn is_under_attack(&self) -> bool {
        self.failed_auth_attempts > 100 || self.blocked_ips > 50
    }

    /// Get security health status
    #[must_use]
    pub fn health_status(&self) -> SecurityHealth {
        if self.security_score < 0.5 || self.is_under_attack() {
            SecurityHealth::Critical
        } else if self.security_score < 0.7 || self.failed_auth_attempts > 50 {
            SecurityHealth::Warning
        } else {
            SecurityHealth::Healthy
        }
    }

    /// Create healthy metrics for testing
    #[cfg(test)]
    pub fn healthy() -> Self {
        Self {
            active_sessions: 10,
            failed_auth_attempts: 2,
            blocked_ips: 0,
            security_score: 0.95,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Security health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityHealth {
    /// Security is healthy
    Healthy,
    /// Security warnings present
    Warning,
    /// Security is critical
    Critical,
}

/// Authentication result from security capability provider
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
