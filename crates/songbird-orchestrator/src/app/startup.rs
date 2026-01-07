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
/// Start the orchestrator and return the handle
///
/// **Modern Idiomatic Rust** (v3.18.2):
/// - Spawns background tasks
/// - Returns immediately (non-blocking)
/// - Caller handles signal waiting and shutdown
/// - Separation of concerns (startup vs lifecycle management)
///
/// **Deep Debt Fixed**:
/// - No duplicate signal handlers
/// - Clear ownership: caller controls lifecycle
/// - Testable: can start/stop without blocking
pub async fn start_orchestrator(config: CanonicalSongbirdConfig) -> Result<SongbirdOrchestrator> {
    info!("🔧 Initializing orchestrator components...");

    let mut orchestrator = SongbirdOrchestrator::new(config).await?;
    orchestrator.start().await?;

    info!("✅ Orchestrator components started");

    // Return orchestrator handle to caller
    // Caller is responsible for lifecycle management
    Ok(orchestrator)
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
    /// v3.18.2: Updated for new start_orchestrator signature
    pub async fn run(self) -> Result<()> {
        let _orchestrator = start_orchestrator(self._config).await?;
        
        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        
        Ok(())
    }
}
