//! Songbird Gaming Bridge - Anti-Monolith Orchestrator
//!
//! Main entry point for the modular gaming network bridge

use anyhow::Result;
use songbird_config::SongbirdConfig;
use songbird_orchestrator::app;

/// Main entry point for the Songbird Orchestrator
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("🚀 Starting Songbird Orchestrator...");

    // Load configuration
    let config = SongbirdConfig::default();

    // Start the orchestrator
    app::start_orchestrator(config).await?;

    tracing::info!("✅ Songbird Orchestrator started successfully");

    // Keep running until interrupted
    tokio::signal::ctrl_c().await?;

    tracing::info!("🛑 Shutting down Songbird Orchestrator...");

    Ok(())
}
