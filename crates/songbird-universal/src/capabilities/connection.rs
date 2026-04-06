// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Primal connection types and health tracking

use std::collections::HashMap;

use super::types::PrimalType;

/// Health status of primal connection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionHealth {
    /// Connection is healthy and responsive
    Healthy,
    /// Connection is degraded but functional
    Degraded,
    /// Connection is unhealthy or non-responsive
    Unhealthy,
    /// Connection status is unknown
    Unknown,
}

/// Health check result with detailed information
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Whether the service is healthy
    pub is_healthy: bool,
    /// Status message
    pub status: String,
    /// Timestamp of check
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Connection to a primal
#[derive(Debug, Clone)]
pub struct PrimalConnection {
    /// Primal name
    pub name: String,
    /// Primal type
    pub primal_type: PrimalType,
    /// Endpoint URL
    pub endpoint: String,
    /// Connection health
    pub health: ConnectionHealth,
    /// Last successful communication
    pub last_contact: chrono::DateTime<chrono::Utc>,
    /// Last health check result
    pub last_health_check: Option<HealthCheckResult>,
    /// Connection metadata
    pub metadata: HashMap<String, String>,
}
