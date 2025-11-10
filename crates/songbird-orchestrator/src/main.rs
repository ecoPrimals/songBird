//! Songbird Gaming Bridge - Anti-Monolith Orchestrator
//!
//! Main entry point for the modular gaming network bridge

use anyhow::Result;
use songbird_types::config::CanonicalSongbirdConfig;
use songbird_orchestrator::app;

/// Main entry point for the Songbird Orchestrator
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("🚀 Starting Songbird Orchestrator...");

    // Load configuration from environment
    let config = CanonicalSongbirdConfig::from_env()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration from environment: {}. Check environment variables and config files.", e))?;

    // Start the orchestrator
    app::start_orchestrator(config).await?;

    tracing::info!("✅ Songbird Orchestrator started successfully");

    // Keep running until interrupted
    tokio::signal::ctrl_c().await?;

    tracing::info!("🛑 Shutting down Songbird Orchestrator...");

    Ok(())
}
