//! Orchestrator status and health check types

use std::time::SystemTime;

/// Overall orchestrator status
#[derive(Debug, Clone)]
pub struct OrchestratorStatus {
    pub running: bool,
    pub gaming_active: bool,
    pub federation_active: bool,
    pub observability_active: bool,
    pub security_active: bool,
    pub uptime_seconds: u64,
    pub total_players: u32,
}

/// Health check report for all orchestrator components
#[derive(Debug, Clone)]
pub struct HealthCheckReport {
    pub gaming_healthy: bool,
    pub federation_healthy: bool,
    pub observability_healthy: bool,
    pub security_healthy: bool,
    pub overall_healthy: bool,
    pub timestamp: SystemTime,
}

impl HealthCheckReport {
    /// Create a new health check report
    pub fn new(
        gaming_healthy: bool,
        federation_healthy: bool,
        observability_healthy: bool,
        security_healthy: bool,
    ) -> Self {
        let overall_healthy =
            gaming_healthy && federation_healthy && observability_healthy && security_healthy;

        Self {
            gaming_healthy,
            federation_healthy,
            observability_healthy,
            security_healthy,
            overall_healthy,
            timestamp: SystemTime::now(),
        }
    }

    /// Check if all components are healthy
    pub fn is_healthy(&self) -> bool {
        self.overall_healthy
    }
}

impl Default for OrchestratorStatus {
    fn default() -> Self {
        Self {
            running: false,
            gaming_active: false,
            federation_active: false,
            observability_active: false,
            security_active: false,
            uptime_seconds: 0,
            total_players: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_all_healthy() {
        let report = HealthCheckReport::new(true, true, true, true);
        assert!(report.is_healthy());
        assert!(report.overall_healthy);
    }

    #[test]
    fn test_health_check_one_unhealthy() {
        let report = HealthCheckReport::new(true, false, true, true);
        assert!(!report.is_healthy());
        assert!(!report.overall_healthy);
    }

    #[test]
    fn test_orchestrator_status_default() {
        let status = OrchestratorStatus::default();
        assert!(!status.running);
        assert_eq!(status.uptime_seconds, 0);
    }
}

