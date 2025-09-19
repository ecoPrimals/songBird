//! Canonical Orchestrator trait definition
//!
//! This module defines the unified Orchestrator trait that replaces
//! fragmented orchestrator interfaces across the codebase.

use async_trait::async_trait;
use songbird_errors::Result as SongbirdResult;
use songbird_errors::SongbirdResult;

/// Canonical Orchestrator trait for service management
///
/// This trait unifies all orchestrator functionality into a single,
/// coherent interface that replaces previous fragmented implementations.
#[async_trait]
pub trait Orchestrator: Send + Sync {
    /// Start the orchestrator
    async fn start(&self) -> SongbirdResult<()>;

    /// Stop the orchestrator
    async fn stop(&self) -> SongbirdResult<()>;

    /// Get orchestrator status and metrics
    async fn get_status(&self) -> SongbirdResult<OrchestratorStatus>;

    /// Get orchestrator metrics
    async fn get_metrics(&self) -> SongbirdResult<crate::canonical::OrchestratorMetrics>;

    /// Register a new service
    async fn register_service(&self, service: crate::canonical::ServiceInfo) -> SongbirdResult<()>;

    /// Deregister a service
    async fn deregister_service(&self, service_id: &str) -> SongbirdResult<()>;

    /// Get all registered services
    async fn get_services(&self) -> SongbirdResult<Vec<crate::canonical::ServiceInfo>>;

    /// Perform health check on a specific service
    async fn perform_health_check(&self, service_id: &str) -> SongbirdResult<bool>;
}

/// Canonical orchestrator status structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrchestratorStatus {
    pub active_services: usize,
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}
