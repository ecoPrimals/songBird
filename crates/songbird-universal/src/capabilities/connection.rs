//! Primal connection types and health tracking

use std::collections::HashMap;

use super::types::PrimalType;

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
    /// Connection metadata
    pub metadata: HashMap<String, String>,
}

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
