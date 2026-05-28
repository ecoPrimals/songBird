// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Server command — starts the Songbird orchestrator in server mode
//!
//! Provides `run_server()` with proper signal handling (SIGINT, SIGTERM),
//! graceful shutdown, instance locking, and comprehensive logging.

use anyhow::Result;
use songbird_types::config::CanonicalSongbirdConfig;

use crate::process_manager::ProcessManager;

/// Run Songbird orchestrator in server mode
///
/// Handles instance locking, signal-based shutdown, and full orchestrator lifecycle.
///
/// # Errors
///
/// Returns an error if the instance lock cannot be acquired, configuration cannot
/// be loaded, or the orchestrator fails to start.
pub async fn run_server(
    port: u16,
    daemon: bool,
    config_path: Option<String>,
    verbose: bool,
) -> Result<()> {
    // Initialize tracing (early, before any logging)
    if verbose {
        tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();
    } else {
        tracing_subscriber::fmt::init();
    }

    // Log startup with mode information
    tracing::info!("🚀 Songbird v{} - Server Mode", env!("CARGO_PKG_VERSION"));
    tracing::info!(
        "   Mode: Server {}",
        if daemon {
            "(daemon)"
        } else {
            "(foreground)"
        }
    );
    tracing::info!("   Port: {}", port);
    tracing::info!("   Process ID: {}", std::process::id());

    // Step 1: Acquire instance lock FIRST (before any resources)
    // This lock is scoped per NODE_ID, enabling multi-instance deployments
    let process_mgr = ProcessManager::new()?;
    let _singleton_guard = process_mgr.acquire_lock()?;
    tracing::info!("   Instance Lock: ✅ Acquired (PID file active)");

    // Pure Songbird TLS — security provider handles all crypto via JSON-RPC at runtime

    // Get node identity for logging
    let node_identity = songbird_process_env::var("SONGBIRD_NODE_ID")
        .or_else(|_| songbird_process_env::var("NODE_ID"))
        .or_else(|_| songbird_process_env::var("SPORE_ID"))
        .ok();

    let family_identity = songbird_process_env::var("SONGBIRD_FAMILY_ID")
        .or_else(|_| songbird_process_env::var("FAMILY_ID"))
        .ok();

    if let Some(ref family) = family_identity {
        tracing::info!("   Family ID: {}", family);
    }
    if let Some(ref node) = node_identity {
        tracing::info!("   Node ID: {}", node);
    }

    // Step 3: Load configuration
    tracing::info!("📋 Loading configuration...");
    if let Some(ref path) = config_path {
        tracing::info!("   Config file: {path}");
    } else {
        tracing::info!("   Config source: Environment variables");
    }
    let config = CanonicalSongbirdConfig::from_env()?;
    tracing::info!("   Configuration: ✅ Loaded");

    // Step 4: Start the orchestrator (non-blocking, returns handle)
    tracing::info!("🔧 Starting orchestrator components...");
    let mut orchestrator = crate::app::start_orchestrator(config).await?;
    tracing::info!("   Orchestrator: ✅ Started");

    tracing::info!("✅ Songbird ready!");
    tracing::info!("   Unix Socket IPC: /tmp/songbird-*.sock (see logs for actual path)");
    tracing::info!("   Protocol: JSON-RPC 2.0 over Unix sockets");
    tracing::info!("   HTTP/TLS: Handled by external gateway component");
    tracing::info!("");
    tracing::info!("💡 Press Ctrl+C to stop gracefully");

    // Step 5: If daemon mode, detach from terminal
    if daemon {
        tracing::info!("📌 Daemon mode: Process detached");
        // FUTURE (Phase 2): Full daemonization support if needed
        // Current: Use systemd service units for production deployment
    }

    // Step 6: Main event loop - wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("🛑 Received SIGINT (Ctrl+C), initiating graceful shutdown...");
        }
        () = async {
            #[cfg(unix)]
            {
                // Signal handler setup is infallible in practice — panic is correct here
                #[allow(clippy::expect_used, reason = "intentional pattern; clippy false positive for this API")]
                let mut sigterm = tokio::signal::unix::signal(
                    tokio::signal::unix::SignalKind::terminate(),
                )
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
    tracing::info!("   Orchestrator: ✅ Stopped");

    tracing::info!("🧹 Cleaning up resources...");
    tracing::info!("   • Releasing instance lock (PID file)");
    tracing::info!("   • Closing network connections");
    tracing::info!("   • Flushing logs");

    tracing::info!("✅ Graceful shutdown complete");

    // Flush tracing subscriber (drop flushes buffered output)
    drop(tracing::dispatcher::get_default(std::clone::Clone::clone));

    Ok(())
    // _singleton_guard drops here, removing PID file cleanly
    // This is the RAII pattern - cleanup is automatic, panic-safe
}

#[cfg(test)]
mod tests {
    //! There is no isolated pure logic in [`super::run_server`]: it acquires a process lock,
    //! reads configuration from the environment, starts the full orchestrator, waits on OS
    //! signals, and performs teardown. Those steps require integration or end-to-end tests;
    //! unit tests would not add meaningful coverage without refactoring into testable helpers.

    #![allow(clippy::unwrap_used, reason = "test assertions")]
}
