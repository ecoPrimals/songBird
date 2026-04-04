// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use anyhow::Result;
use tracing::info;

use super::SongbirdOrchestrator;

impl SongbirdOrchestrator {
    /// Handle incoming CLI commands
    ///
    /// **Deep Debt Evolution** (Feb 6, 2026): Extracted to `command_handler` module
    ///
    /// **See**: `command_handler.rs` for implementation details
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn handle_command(&self, command: String) -> Result<String> {
        crate::app::command_handler::CommandHandler::new(self).handle(&command).await
    }

    /// Start web dashboard
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub async fn start_web_dashboard(&self) -> Result<()> {
        info!("🌐 Starting web dashboard...");
        info!(
            "✅ Web dashboard would start on http://{}:{}",
            songbird_config::canonical::constants::default_bind_address(),
            songbird_config::defaults::ports::orchestrator_port()
        );
        info!("   (Dashboard implementation available but disabled for now)");
        Ok(())
    }

    /// Detect GPU model if available
    ///
    /// Public for use in federation setup
    /// Detect GPU model (re-exported from `hardware_detection` module)
    #[must_use]
    pub fn detect_gpu() -> Option<String> {
        crate::app::hardware_detection::detect_gpu()
    }

    /// Detect storage capacity in GB (re-exported from `hardware_detection` module)
    #[must_use]
    pub fn detect_storage_capacity() -> Option<usize> {
        crate::app::hardware_detection::detect_storage_capacity()
    }
}
