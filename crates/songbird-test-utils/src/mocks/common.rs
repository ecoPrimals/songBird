// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Common types and utilities for mock primal servers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Health status for mock services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HealthStatus {
    /// Service is healthy and accepting requests
    #[default]
    Healthy,
    /// Service is degraded but still functional
    Degraded,
    /// Service is unhealthy and should not receive traffic
    Unhealthy,
    /// Service is starting up
    Starting,
}

/// Standard mock response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockResponse<T> {
    /// Response data
    pub data: T,
    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Service health at time of response
    pub health: HealthStatus,
}

impl<T> MockResponse<T> {
    /// Create a new mock response
    pub fn new(data: T, health: HealthStatus) -> Self {
        Self {
            data,
            timestamp: chrono::Utc::now(),
            health,
        }
    }
}

/// Base trait for mock primal servers
pub trait MockPrimalServer {
    /// Get the server's listening port
    fn port(&self) -> u16;

    /// Get the server's endpoint URL
    fn endpoint(&self) -> String {
        format!("http://localhost:{}", self.port())
    }

    /// Set the server's health status
    fn set_health(&self, status: HealthStatus);

    /// Get the server's current health status
    fn get_health(&self) -> HealthStatus;
}

/// Shared state for mock servers
#[derive(Debug)]
pub struct MockServerState {
    /// TCP port the mock HTTP server binds.
    pub port: u16,
    /// Latest [`HealthStatus`] for readiness probes.
    pub health: Arc<RwLock<HealthStatus>>,
    /// Total requests observed (for assertions).
    pub request_count: Arc<RwLock<usize>>,
    /// Arbitrary key/value tags (e.g. feature toggles under test).
    pub metadata: Arc<RwLock<HashMap<String, String>>>,
}

impl MockServerState {
    /// Create new mock server state
    #[must_use]
    pub fn new(port: u16) -> Self {
        Self {
            port,
            health: Arc::new(RwLock::new(HealthStatus::Healthy)),
            request_count: Arc::new(RwLock::new(0)),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Increment request count
    ///
    /// # Panics
    ///
    /// Panics if the internal request count lock is poisoned.
    pub fn increment_requests(&self) {
        let mut count = self.request_count.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        *count += 1;
    }

    /// Get request count
    ///
    /// # Panics
    ///
    /// Panics if the internal request count lock is poisoned.
    #[must_use]
    pub fn get_request_count(&self) -> usize {
        *self.request_count.read().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        })
    }

    /// Set health status
    ///
    /// # Panics
    ///
    /// Panics if the internal health lock is poisoned.
    pub fn set_health(&self, status: HealthStatus) {
        let mut health = self.health.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        *health = status;
    }

    /// Get health status
    ///
    /// # Panics
    ///
    /// Panics if the internal health lock is poisoned.
    #[must_use]
    pub fn get_health(&self) -> HealthStatus {
        *self.health.read().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        })
    }
}
