//! Songbird Gaming Bridge - Anti-Monolith Orchestrator
//!
//! Main entry point for the modular gaming network bridge

use anyhow::Result;
use songbird_orchestrator::app;
use songbird_orchestrator::process_manager::ProcessManager;
use songbird_types::config::CanonicalSongbirdConfig;

/// Main entry point for the Songbird Orchestrator
#[tokio::main]
async fn main() -> Result<()> {
    // ✅ Step 1: Acquire instance lock FIRST (before any resources)
    // This lock is scoped per NODE_ID, enabling multi-instance deployments
    // This prevents "Federation Split State Bug" (Dec 20, 2025)
    let process_mgr = ProcessManager::new()?;
    let _singleton_guard = process_mgr.acquire_lock()?;
    // Guard lives for entire program, auto-releases on drop

    // ✅ Step 2: Initialize rustls crypto provider (before any TLS operations)
    // This prevents "Could not automatically determine the process-level CryptoProvider" panic
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("Crypto provider already initialized"))?;

    // Step 3: Initialize tracing
    tracing_subscriber::fmt::init();

    // Get node identity for logging
    let node_identity = std::env::var("SONGBIRD_NODE_ID")
        .or_else(|_| std::env::var("NODE_ID"))
        .or_else(|_| std::env::var("SPORE_ID"))
        .ok();
    
    let family_identity = std::env::var("SONGBIRD_FAMILY_ID")
        .or_else(|_| std::env::var("FAMILY_ID"))
        .ok();

    tracing::info!("🚀 Starting Songbird Orchestrator...");
    tracing::info!("   Process ID: {}", std::process::id());
    if let Some(ref family) = family_identity {
        tracing::info!("   Family: {}", family);
    }
    if let Some(ref node) = node_identity {
        tracing::info!("   Node: {}", node);
    }
    tracing::info!("   Instance Lock: Enforced (PID file active)");

    // Step 4: Load configuration from environment
    let config = CanonicalSongbirdConfig::from_env()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration from environment: {}. Check environment variables and config files.", e))?;

    // Step 5: Start the orchestrator
    app::start_orchestrator(config).await?;

    tracing::info!("✅ Songbird Orchestrator started successfully");

    // Step 6: Keep running until interrupted
    tokio::signal::ctrl_c().await?;

    tracing::info!("🛑 Shutting down Songbird Orchestrator...");
    tracing::info!("   Instance lock will release automatically");

    Ok(())
    // _singleton_guard drops here, removing PID file cleanly
}
