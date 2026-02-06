//! Command Handler Module
//!
//! **Purpose**: Handle CLI commands sent to the orchestrator
//!
//! **Deep Debt Evolution** (Feb 6, 2026):
//! - Extracted from 47-line `handle_command()` method in core.rs
//! - Follows Single Responsibility Principle
//! - Easy to add new commands
//! - Testable in isolation
//!
//! **Supported Commands**:
//! - `status` - Get orchestrator status
//! - `health` - Run comprehensive health check
//!
//! **Future Commands**:
//! - `peers` - List discovered peers
//! - `capabilities` - List registered capabilities
//! - `federation` - Federation status
//! - `restart` - Graceful restart

use anyhow::Result;

use super::core::SongbirdOrchestrator;

/// Command handler for orchestrator CLI commands
pub struct CommandHandler<'a> {
    orchestrator: &'a SongbirdOrchestrator,
}

impl<'a> CommandHandler<'a> {
    /// Create new command handler
    pub fn new(orchestrator: &'a SongbirdOrchestrator) -> Self {
        Self { orchestrator }
    }

    /// Handle incoming CLI command
    ///
    /// **Commands**:
    /// - `status` - Get orchestrator status
    /// - `health` - Run comprehensive health check
    /// - unknown - Return error message
    ///
    /// **Returns**: String response to display to user
    pub async fn handle(&self, command: &str) -> Result<String> {
        match command {
            "status" => self.handle_status().await,
            "health" => self.handle_health().await,
            _ => self.handle_unknown(command),
        }
    }

    /// Handle status command
    ///
    /// Returns current orchestrator status (Active, Inactive, etc.)
    async fn handle_status(&self) -> Result<String> {
        let status = self.orchestrator.get_status().await?;
        Ok(format!("Status: {status:?}"))
    }

    /// Handle health command
    ///
    /// Runs comprehensive health check on all subsystems:
    /// - Gaming Manager
    /// - Federation Manager
    /// - Observability Manager
    /// - Security Integration
    ///
    /// Returns formatted health report with component status
    async fn handle_health(&self) -> Result<String> {
        // Comprehensive health check implementation
        let health_result = self.orchestrator.run_comprehensive_health_check().await;
        match health_result {
            Ok(health_report) => {
                let status = if health_report.overall_healthy {
                    "HEALTHY"
                } else {
                    "UNHEALTHY"
                };
                Ok(format!(
                    "Health Check Status: {}\n\nComponent Health:\n- Gaming Manager: {}\n- Federation Manager: {}\n- Observability Manager: {}\n- Security Integration: {}\n\nLast Check: {:?}",
                    status,
                    if health_report.gaming_healthy {
                        "✅ HEALTHY"
                    } else {
                        "❌ UNHEALTHY"
                    },
                    if health_report.federation_healthy {
                        "✅ HEALTHY"
                    } else {
                        "❌ UNHEALTHY"
                    },
                    if health_report.observability_healthy {
                        "✅ HEALTHY"
                    } else {
                        "❌ UNHEALTHY"
                    },
                    if health_report.security_healthy {
                        "✅ HEALTHY"
                    } else {
                        "❌ UNHEALTHY"
                    },
                    health_report.timestamp
                ))
            }
            Err(e) => Ok(format!("Health check failed: {e}")),
        }
    }

    /// Handle unknown command
    ///
    /// Returns error message with list of available commands
    fn handle_unknown(&self, command: &str) -> Result<String> {
        Ok(format!("Unknown command: {command}"))
    }
}
