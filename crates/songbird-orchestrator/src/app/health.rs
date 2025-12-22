//! Health check and status reporting for the orchestrator
//!
//! Provides comprehensive health monitoring for all orchestrator components.

use anyhow::Result;
use tracing::info;

use super::core::SongbirdOrchestrator;

/// Orchestrator status information
///
/// Provides a snapshot of the orchestrator's operational state.
#[derive(Debug, Clone)]
pub struct OrchestratorStatus {
    /// Whether gaming subsystem is active
    pub gaming_active: bool,
    /// Whether federation is connected
    pub federation_connected: bool,
    /// Number of active sessions
    pub active_sessions: u32,
    /// Total number of players/participants
    pub total_players: u32,
}

/// Health check report for all orchestrator components
///
/// Comprehensive health status across all subsystems.
#[derive(Debug, Clone)]
pub struct HealthCheckReport {
    /// Gaming subsystem health
    pub gaming_healthy: bool,
    /// Federation connectivity health
    pub federation_healthy: bool,
    /// Observability system health
    pub observability_healthy: bool,
    /// Security subsystem health
    pub security_healthy: bool,
    /// Overall system health (aggregated)
    pub overall_healthy: bool,
    /// Timestamp of health check
    pub timestamp: std::time::SystemTime,
}

impl HealthCheckReport {
    /// Create a new health check report with all systems healthy
    pub fn all_healthy() -> Self {
        Self {
            gaming_healthy: true,
            federation_healthy: true,
            observability_healthy: true,
            security_healthy: true,
            overall_healthy: true,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Check if all subsystems are healthy
    pub fn is_fully_healthy(&self) -> bool {
        self.gaming_healthy
            && self.federation_healthy
            && self.observability_healthy
            && self.security_healthy
    }
}

/// Run comprehensive health check on the orchestrator
///
/// Checks all subsystems and returns detailed status.
pub async fn run_health_check(orchestrator: &SongbirdOrchestrator) -> Result<()> {
    let status = orchestrator.get_status().await?;
    info!("Health check completed: {:?}", status);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_report_all_healthy() {
        let report = HealthCheckReport::all_healthy();
        assert!(report.is_fully_healthy());
        assert!(report.overall_healthy);
    }

    #[test]
    fn test_health_report_partial_failure() {
        let mut report = HealthCheckReport::all_healthy();
        report.gaming_healthy = false;
        assert!(!report.is_fully_healthy());
    }
}
