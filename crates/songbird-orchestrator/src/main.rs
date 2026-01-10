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

    let family_identity =
        std::env::var("SONGBIRD_FAMILY_ID").or_else(|_| std::env::var("FAMILY_ID")).ok();

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

    // Step 5: Start the orchestrator (non-blocking, returns handle)
    // v3.18.2: Modern idiomatic Rust - clear separation of concerns
    let mut orchestrator = app::start_orchestrator(config).await?;

    tracing::info!("✅ Songbird Orchestrator started successfully");
    tracing::info!("✅ Orchestrator running. Press Ctrl+C to stop.");

    // Step 6: Main event loop - wait for shutdown signal
    // This is the ONLY signal handler (no duplication)
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("🛑 Received SIGINT (Ctrl+C), initiating graceful shutdown...");
        }
        _ = async {
            #[cfg(unix)]
            {
                let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("Failed to setup SIGTERM handler");
                sigterm.recv().await;
            }
            #[cfg(not(unix))]
            {
                // Windows: only Ctrl+C is available
                std::future::pending::<()>().await
            }
        } => {
            tracing::info!("🛑 Received SIGTERM, initiating graceful shutdown...");
        }
    }

    // Step 7: Graceful shutdown - stop orchestrator components
    tracing::info!("🧹 Stopping orchestrator components...");
    orchestrator.stop().await?;

    tracing::info!("🧹 Cleaning up resources...");
    tracing::info!("   • Releasing instance lock (PID file)");
    tracing::info!("   • Closing network connections");
    tracing::info!("   • Flushing logs");

    // Add a small delay to ensure logs are flushed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    tracing::info!("✅ Graceful shutdown complete");

    Ok(())
    // _singleton_guard drops here, removing PID file cleanly
    // This is the RAII pattern - cleanup is automatic, panic-safe
}
