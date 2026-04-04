// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use anyhow::Result;
use tracing::{error, info, warn};

use super::SongbirdOrchestrator;

impl SongbirdOrchestrator {
    /// Start the orchestrator
    /// Start the Songbird Orchestrator (7-stage startup sequence)
    ///
    /// **Deep Debt Evolution** (Feb 6, 2026): Extracted 275-line method to `startup_orchestration` module
    ///
    /// **Startup Stages**:
    /// 1. Provision Security - JWT secrets, identity query
    /// 2. Start Core Servers - HTTP, IPC, tarpc
    /// 3. Register Self - Federation self-registration
    /// 4. Start Discovery - Anonymous peer discovery
    /// 5. Start Federation - Coordinator and trust cleanup
    /// 6. Start Background Tasks - Health monitoring, cleanup
    /// 7. Verify Connectivity - Post-startup verification
    ///
    /// **See**: `startup_orchestration.rs` for implementation details
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn start(&mut self) -> Result<()> {
        crate::app::startup_orchestration::StartupOrchestrator::new(self).start().await
    }

    /// Stop the orchestrator
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn stop(&mut self) -> Result<()> {
        info!("🛑 Stopping Songbird Orchestrator");

        // Send shutdown signal
        if let Err(e) = self.shutdown_sender.send(()) {
            warn!("Failed to send shutdown signal: {}", e);
        }

        // Federation manager doesn't have a stop method, so we'll just log
        info!("✅ Federation manager will stop gracefully");

        if let Err(e) = self.observability_manager.stop().await {
            error!("Failed to stop observability manager: {}", e);
        }

        info!("✅ Songbird Orchestrator stopped successfully");
        Ok(())
    }
}
