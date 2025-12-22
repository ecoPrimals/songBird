//! Orchestrator startup and lifecycle management
//!
//! Provides the main entry point for starting and running the orchestrator.

use anyhow::Result;
use songbird_types::config::CanonicalSongbirdConfig;
use tracing::info;

use super::core::SongbirdOrchestrator;

/// Start the orchestrator with configuration
///
/// This is the main entry point for running the orchestrator. It:
/// 1. Creates a new orchestrator instance
/// 2. Starts all subsystems
/// 3. Waits for shutdown signal (Ctrl+C)
/// 4. Gracefully stops all subsystems
///
/// # Example
///
/// ```rust,ignore
/// use songbird_orchestrator::app::startup::start_orchestrator;
/// use songbird_types::config::CanonicalSongbirdConfig;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = CanonicalSongbirdConfig::from_env()?;
///     start_orchestrator(config).await
/// }
/// ```
pub async fn start_orchestrator(config: CanonicalSongbirdConfig) -> Result<()> {
    info!("🚀 Starting Songbird Orchestrator");

    let mut orchestrator = SongbirdOrchestrator::new(config).await?;
    orchestrator.start().await?;

    info!("✅ Orchestrator running. Press Ctrl+C to stop.");

    // Keep running until interrupted
    tokio::signal::ctrl_c().await?;

    info!("🛑 Shutdown signal received. Stopping orchestrator...");
    orchestrator.stop().await?;

    info!("👋 Orchestrator stopped gracefully");
    Ok(())
}

/// Simple orchestrator wrapper for basic use cases
///
/// This provides a simplified interface for applications that don't need
/// full control over the orchestrator lifecycle.
pub struct Orchestrator {
    _config: CanonicalSongbirdConfig,
}

impl Orchestrator {
    /// Create a new orchestrator wrapper
    #[must_use]
    pub fn new(config: CanonicalSongbirdConfig) -> Self {
        Self {
            _config: config,
        }
    }

    /// Start the orchestrator (convenience method)
    ///
    /// Equivalent to calling `start_orchestrator` directly.
    pub async fn run(self) -> Result<()> {
        start_orchestrator(self._config).await
    }
}
