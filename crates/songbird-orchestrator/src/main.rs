//! Songbird Gaming Bridge - Anti-Monolith Orchestrator
//!
//! Main entry point for the modular gaming network bridge

use anyhow::Result;
use songbird_orchestrator::app;
use songbird_types::config::CanonicalSongbirdConfig;

/// Main entry point for the Songbird Orchestrator
#[tokio::main]
async fn main() -> Result<()> {
    // ✅ Initialize rustls crypto provider FIRST (before any TLS operations)
    // This prevents "Could not automatically determine the process-level CryptoProvider" panic
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("Crypto provider already initialized"))?;

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
