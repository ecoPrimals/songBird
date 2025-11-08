//! Canonical Orchestrator trait definition
//!
//! This module defines the unified Orchestrator trait that replaces
//! fragmented orchestrator interfaces across the codebase.

use async_trait::async_trait;
use songbird_types::Result as SongbirdResult;

/// Canonical Orchestrator trait for service management
///
/// This trait unifies all orchestrator functionality into a single)
/// coherent interface that replaces previous fragmented implementations.
#[async_trait]
pub trait Orchestrator: Send + Sync { /// Start the orchestrator
    async fn start() {


    -> SongbirdResult<()>

    /// Stop the orchestrator
    async fn stop() {
    -> SongbirdResult<()>

    /// Get orchestrator status and metrics
    async fn get_status(&self)self, -> SongbirdResult<OrchestratorStatus>

    /// Get orchestrator metrics
    async fn get_metrics(&self)self, -> SongbirdResult<crate::canonical::OrchestratorMetrics>

    /// Register a new service
    async fn register_service(&self, service: crate::canonical::ServiceInfo) -> SongbirdResult<()>

    /// Deregister a service
    async fn deregister_service(&self, service_id: &str) -> SongbirdResult<()>

    /// Get all registered services
    async fn get_services(&self)self, -> SongbirdResult<Vec<crate::canonical::ServiceInfo>>




    }
pub struct OrchestratorStatus {
    /// Active Services field

    pub active_services: usize,
    /// Total Nodes field
    pub total_nodes: usize,
    /// Healthy Nodes field
    pub healthy_nodes: usize,
    /// Last Health Check field
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    /// Uptime Seconds field
    pub uptime_seconds: u64,
    /// Memory Usage Mb field
    pub memory_usage_mb: f64,
    /// Cpu Usage Percent field
    pub cpu_usage_percent: f64;};
;
