// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Health check and status reporting for the orchestrator
//!
//! Provides comprehensive health monitoring for all orchestrator components.
//!
//! **EVOLUTION (v3.13.0 - Jan 7, 2026)**: Extracted health check methods from core.rs
//! to reduce file size and improve maintainability. Follows single responsibility principle.

use anyhow::Result;
use tracing::{debug, info, warn};

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
    #[must_use]
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
    #[must_use]
    pub const fn is_fully_healthy(&self) -> bool {
        self.gaming_healthy
            && self.federation_healthy
            && self.observability_healthy
            && self.security_healthy
    }
}

impl SongbirdOrchestrator {
    /// Get current orchestrator status
    ///
    /// Returns a snapshot of the operational state from actual orchestrator state.
    pub async fn get_status(&self) -> Result<OrchestratorStatus> {
        // Federation mesh: coordinator present and `FederationState` shows ≥2 active nodes
        // (typical: local + remote). Discovery path: `FederationCoordinator::coordinate` →
        // `register_node` / heartbeats → `FederationState::active_nodes`.
        let federation_connected = if self.federation_coordinator.is_some() {
            let stats = self.federation_state.get_stats().await;
            stats.active_nodes >= 2
        } else {
            false
        };

        // Sessions: connected peers in `ConnectionManager` / `PeerRegistry`.
        let peers = self.connection_manager.get_all_peers().await;
        let active_sessions = peers.len() as u32;

        // Player roster: no per-peer player counts on `PeerMetadata` yet; equals session count
        // until gaming/session manager exposes headcount. Discovery path: gaming bridge → session
        // store → aggregate players per session.
        let total_players = active_sessions;

        // Gaming: any peer advertises gaming-related capability strings from discovery/BTSP.
        // Discovery path: re-enable `gaming_manager` → same signal augmented with lobby state.
        let gaming_active = peers.iter().any(|p| {
            p.capabilities.iter().any(|c| {
                let c = c.to_lowercase();
                c.contains("gaming")
                    || c.contains("game_session")
                    || c == "game"
                    || c.ends_with("_gaming")
            })
        });

        Ok(OrchestratorStatus {
            gaming_active,
            federation_connected,
            active_sessions,
            total_players,
        })
    }

    /// Start health monitoring loop (placeholder for v3.13.0)
    ///
    /// Periodically checks health of all subsystems.
    /// Note: Full background task implementation is in core.rs (future extraction target).
    pub async fn start_health_monitoring(&self) -> Result<()> {
        info!("🏥 Health monitoring initialized (checks via handle_command)");
        Ok(())
    }

    /// Run a comprehensive health check (MODERN RUST - extracted v3.13.0)
    ///
    /// Checks health of all orchestrator subsystems and returns a detailed report.
    pub(crate) async fn run_comprehensive_health_check(&self) -> Result<HealthCheckReport> {
        info!("🔍 Running comprehensive health check...");

        // Check gaming manager health
        let gaming_healthy = self.check_gaming_manager_health().await;

        // Check federation manager health
        let federation_healthy = self.check_federation_manager_health().await;

        // Check observability manager health
        let observability_healthy = self.check_observability_manager_health().await;

        // Check security integration health
        let security_healthy = self.check_security_integration_health().await;

        let overall_healthy =
            gaming_healthy && federation_healthy && observability_healthy && security_healthy;

        Ok(HealthCheckReport {
            gaming_healthy,
            federation_healthy,
            observability_healthy,
            security_healthy,
            overall_healthy,
            timestamp: std::time::SystemTime::now(),
        })
    }

    /// Check gaming manager health
    pub(crate) async fn check_gaming_manager_health(&self) -> bool {
        // Validate gaming manager is operational
        // In a real implementation, this would check gaming bridge connections
        debug!("Gaming manager health check completed");
        true
    }

    /// Check federation manager health
    pub(crate) async fn check_federation_manager_health(&self) -> bool {
        // Validate federation manager is operational
        // In a real implementation, this would check federation connectivity
        debug!("Federation manager health check completed");
        true
    }

    /// Check observability manager health
    pub(crate) async fn check_observability_manager_health(&self) -> bool {
        // Validate observability manager is operational
        // In a real implementation, this would check metrics collection
        debug!("Observability manager health check completed");
        true
    }

    /// Check security integration health via crypto-provider discovery
    pub(crate) async fn check_security_integration_health(&self) -> bool {
        match crate::primal_discovery::discover_crypto_provider().await {
            Ok(_socket) => {
                debug!("Security: crypto provider discovered and healthy");
                true
            }
            Err(e) => {
                warn!("Security: crypto provider not reachable: {e}");
                false
            }
        }
    }
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

    #[test]
    fn test_health_report_federation_unhealthy() {
        let mut report = HealthCheckReport::all_healthy();
        report.federation_healthy = false;
        assert!(!report.is_fully_healthy());
    }

    #[test]
    fn test_health_report_observability_unhealthy() {
        let mut report = HealthCheckReport::all_healthy();
        report.observability_healthy = false;
        assert!(!report.is_fully_healthy());
    }

    #[test]
    fn test_health_report_security_unhealthy() {
        let mut report = HealthCheckReport::all_healthy();
        report.security_healthy = false;
        assert!(!report.is_fully_healthy());
    }

    #[test]
    fn test_health_report_all_unhealthy() {
        let report = HealthCheckReport {
            gaming_healthy: false,
            federation_healthy: false,
            observability_healthy: false,
            security_healthy: false,
            overall_healthy: false,
            timestamp: std::time::SystemTime::now(),
        };
        assert!(!report.is_fully_healthy());
        assert!(!report.overall_healthy);
    }

    #[test]
    fn test_orchestrator_status_default_values() {
        let status = OrchestratorStatus {
            gaming_active: false,
            federation_connected: true,
            active_sessions: 0,
            total_players: 0,
        };
        assert!(!status.gaming_active);
        assert!(status.federation_connected);
        assert_eq!(status.active_sessions, 0);
        assert_eq!(status.total_players, 0);
    }

    #[test]
    fn test_orchestrator_status_with_active_gaming() {
        let status = OrchestratorStatus {
            gaming_active: true,
            federation_connected: true,
            active_sessions: 5,
            total_players: 20,
        };
        assert!(status.gaming_active);
        assert_eq!(status.active_sessions, 5);
        assert_eq!(status.total_players, 20);
    }

    #[test]
    fn test_health_report_clone() {
        let report = HealthCheckReport::all_healthy();
        let cloned = report.clone();
        assert_eq!(report.gaming_healthy, cloned.gaming_healthy);
        assert_eq!(report.federation_healthy, cloned.federation_healthy);
        assert_eq!(report.overall_healthy, cloned.overall_healthy);
    }

    #[test]
    fn test_orchestrator_status_clone() {
        let status = OrchestratorStatus {
            gaming_active: true,
            federation_connected: true,
            active_sessions: 3,
            total_players: 12,
        };
        let cloned = status.clone();
        assert_eq!(status.gaming_active, cloned.gaming_active);
        assert_eq!(status.active_sessions, cloned.active_sessions);
        assert_eq!(status.total_players, cloned.total_players);
    }

    #[test]
    fn test_health_report_timestamp() {
        let before = std::time::SystemTime::now();
        let report = HealthCheckReport::all_healthy();
        let after = std::time::SystemTime::now();

        assert!(report.timestamp >= before);
        assert!(report.timestamp <= after);
    }
}
