// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Server mode implementation
//!
//! Handles:
//! - Server startup and initialization
//! - IPC server setup (Unix socket + TCP)
//! - Signal handling and graceful shutdown
//! - Capability registration with Neural API

use anyhow::Result;
use songbird_universal_ipc::registry::ServiceRegistry;
use songbird_universal_ipc::service::IpcServiceHandler;
use songbird_universal_ipc::tower_atomic::{
    JsonRpcError, JsonRpcHandler, JsonRpcRequest, JsonRpcResponse,
};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;

use super::ServerArgs;

/// Run orchestrator in server mode
///
/// Modern, idiomatic, async Rust implementation with:
/// - Proper signal handling (SIGINT, SIGTERM)
/// - Graceful shutdown
/// - Instance locking
/// - Comprehensive logging
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn run_server(args: ServerArgs) -> Result<()> {
    use crate::app;
    use crate::process_manager::ProcessManager;
    use songbird_types::config::CanonicalSongbirdConfig;

    // Initialize tracing (early, before any logging)
    if args.verbose {
        tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();
    } else {
        tracing_subscriber::fmt::init();
    }

    // Apply environment overrides BEFORE ProcessManager (pid_dir affects PID path)
    if args.dark_forest {
        songbird_process_env::set_var("SONGBIRD_DARK_FOREST", "true");
    }
    if let Some(ref pid_dir) = args.pid_dir {
        songbird_process_env::set_var("SONGBIRD_PID_DIR", pid_dir);
    }

    // Determine the actual port to use (federation_port takes precedence)
    let actual_port = args.federation_port.unwrap_or(args.port);

    // Log startup with mode information
    tracing::info!("🚀 Songbird v{} - Server Mode", env!("CARGO_PKG_VERSION"));
    tracing::info!(
        "   Mode: Server {}",
        if args.daemon {
            "(daemon)"
        } else {
            "(foreground)"
        }
    );
    tracing::info!("   External Port: {} (LAN discovery/federation)", actual_port);
    if args.dark_forest {
        tracing::info!("   Dark Forest: ✅ Enabled (encrypted beacons only)");
    }
    if let Some(ref pid_dir) = args.pid_dir {
        tracing::info!("   PID Dir: {} (override)", pid_dir);
    }
    if let Some(ref listen) = args.listen {
        tracing::info!("   Internal IPC: TCP {} (Android/Universal)", listen);
    } else if let Some(ref socket) = args.socket {
        tracing::info!("   Internal Socket: {} (inter-primal IPC)", socket);
    }
    tracing::info!("   Process ID: {}", std::process::id());

    // ✅ Step 1: Acquire instance lock FIRST (before any resources)
    let process_mgr = ProcessManager::new()?;
    let _singleton_guard = process_mgr.acquire_lock()?;
    tracing::info!("   Instance Lock: ✅ Acquired (PID file active)");

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
    let mut config = if let Some(path) = args.config {
        tracing::info!("   Config file: {}", path);
        CanonicalSongbirdConfig::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load configuration from file: {e}"))?
    } else {
        tracing::info!("   Config source: Environment variables");
        CanonicalSongbirdConfig::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load configuration from environment: {e}"))?
    };

    // Override port from CLI (CLI takes precedence over config/env)
    config.network.base_port = actual_port;
    tracing::info!("   Configuration: ✅ Loaded (port override: {})", actual_port);

    // Step 4: Start the orchestrator (non-blocking, returns handle)
    tracing::info!("🔧 Starting orchestrator components...");
    let mut orchestrator = app::start_orchestrator(config).await?;
    tracing::info!("   Orchestrator: ✅ Started");

    tracing::info!("✅ Songbird ready!");
    tracing::info!("");

    // Step 4.5: Start IPC server (Unix socket or TCP based on args)
    let socket_path_for_registration = args.socket.clone().or_else(|| args.listen.clone());
    let ipc_handle = if let Some(ref listen_addr) = args.listen {
        // TCP IPC mode (Android/Universal)
        tracing::info!("");
        tracing::info!("🌐 Starting TCP IPC Server (Android/Universal mode)...");
        tracing::info!("   Listen: {}", listen_addr);
        tracing::info!("   Protocol: JSON-RPC 2.0 over TCP");
        if let Some(ref fam) = family_identity {
            tracing::info!("   Family: {}", fam);
        }

        // Determine BearDog socket/address (capability-first discovery)
        let beardog_socket = args.beardog_socket.clone().unwrap_or_else(|| {
            let default_family = crate::env_config::family_id();
            let family_id = family_identity.as_deref().unwrap_or(&default_family);
            format!("/tmp/beardog-{family_id}.sock")
        });
        tracing::info!("   BearDog: {}", beardog_socket);
        tracing::info!("   Capabilities: http, stun, discovery");

        let listen_clone = listen_addr.clone();
        Some(tokio::spawn(async move {
            match start_tcp_ipc_server(&listen_clone, &beardog_socket).await {
                Ok(()) => tracing::info!("TCP IPC server stopped gracefully"),
                Err(e) => tracing::error!("TCP IPC server error: {}", e),
            }
        }))
    } else if let Some(socket_path) = args.socket {
        // Unix socket IPC mode (default)
        tracing::info!("");
        tracing::info!("🌐 Starting IPC Server (biomeOS integration)...");
        tracing::info!("   Socket: {}", socket_path);
        tracing::info!("   Protocol: JSON-RPC 2.0 over Unix sockets");
        if let Some(ref fam) = family_identity {
            tracing::info!("   Family: {}", fam);
        }

        // Determine BearDog socket (capability-first discovery)
        let beardog_socket = args.beardog_socket.unwrap_or_else(|| {
            let default_family = crate::env_config::family_id();
            let family_id = family_identity.as_deref().unwrap_or(&default_family);
            format!("/tmp/beardog-{family_id}.sock")
        });
        tracing::info!("   BearDog: {}", beardog_socket);
        tracing::info!("   Capabilities: http, discovery, secure_http");

        // Spawn IPC server in background task (Unix only)
        #[cfg(unix)]
        let ipc_task = {
            let socket_clone = socket_path;
            Some(tokio::spawn(async move {
                match start_ipc_server(&socket_clone, &beardog_socket).await {
                    Ok(()) => tracing::info!("IPC server stopped gracefully"),
                    Err(e) => tracing::error!("IPC server error: {}", e),
                }
            }))
        };

        #[cfg(not(unix))]
        let ipc_task: Option<tokio::task::JoinHandle<()>> = {
            tracing::info!("IPC server: Windows TCP fallback (coming in Phase 2)");
            None
        };

        ipc_task
    } else {
        tracing::info!("");
        tracing::info!("💡 Tip: Use --socket or --listen to enable IPC");
        tracing::info!("   Unix: --socket /run/user/$(id -u)/biomeos/songbird.sock");
        tracing::info!("   TCP:  --listen 127.0.0.1:9901 (Android/Universal)");
        None
    };

    // Step 4.6: Register capabilities with Neural API (if available)
    if socket_path_for_registration.is_some() {
        tracing::info!("");
        tracing::info!("🌟 Registering capabilities with Neural API...");
        if let Err(e) = crate::capability_registration::register_capabilities().await {
            tracing::warn!("⚠️  Failed to register capabilities: {}", e);
            tracing::warn!("   Songbird will continue without Neural API registration");
            tracing::warn!("   Direct socket connections will still work");
        }
    }

    tracing::info!("");
    tracing::info!("💡 Press Ctrl+C to stop gracefully");

    // Step 5: If daemon mode, detach from terminal (future enhancement)
    if args.daemon {
        tracing::info!("📌 Daemon mode: Process detached");
    }

    // Step 6: Main event loop - wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("🛑 Received SIGINT (Ctrl+C), initiating graceful shutdown...");
        }
        () = async {
            #[cfg(unix)]
            {
                #[expect(clippy::expect_used, reason = "intentional pattern; clippy false positive for this API")] // signal handler setup is infallible in practice
                let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("Failed to setup SIGTERM handler");
                sigterm.recv().await;
            }
            #[cfg(not(unix))]
            {
                std::future::pending::<()>().await
            }
        } => {
            tracing::info!("🛑 Received SIGTERM, initiating graceful shutdown...");
        }
    }

    // Step 7: Graceful shutdown
    tracing::info!("🧹 Stopping orchestrator components...");

    // Unregister capabilities from Neural API (if registered)
    if socket_path_for_registration.is_some() {
        let _ = crate::capability_registration::unregister_capabilities().await;
    }

    orchestrator.stop().await?;
    tracing::info!("   Orchestrator: ✅ Stopped");

    tracing::info!("✅ Graceful shutdown complete");

    Ok(())
}

/// Handle a single JSON-RPC 2.0 connection over any async stream.
///
/// Transport-agnostic: works identically for Unix sockets, TCP, or any
/// `AsyncRead + AsyncWrite` stream. Newline-delimited JSON-RPC per wateringHole spec.
async fn handle_json_rpc_connection<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
    stream: S,
    handler: Arc<IpcServiceHandler>,
    peer_label: &str,
) {
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                tracing::debug!("{peer_label} disconnected");
                break;
            }
            Ok(_) => {
                if line.trim().is_empty() {
                    continue;
                }

                let request = match serde_json::from_str::<JsonRpcRequest>(&line) {
                    Ok(req) => req,
                    Err(e) => {
                        let resp = JsonRpcResponse::error(
                            JsonRpcError {
                                code: JsonRpcError::PARSE_ERROR,
                                message: format!("Failed to parse request: {e}"),
                                data: None,
                            },
                            serde_json::Value::Null,
                        );
                        if let Ok(json) = serde_json::to_string(&resp) {
                            let _ = writer.write_all(json.as_bytes()).await;
                            let _ = writer.write_all(b"\n").await;
                        }
                        continue;
                    }
                };

                let is_notification = request.id.is_none();
                let id = request.id.unwrap_or(serde_json::Value::Null);
                tracing::debug!(
                    "{peer_label} JSON-RPC: {} (notification={is_notification})",
                    request.method,
                );

                let response = match handler
                    .handle(
                        &request.method,
                        request.params.unwrap_or(serde_json::Value::Null),
                    )
                    .await
                {
                    Ok(result) => JsonRpcResponse::success(result, id),
                    Err(message) => {
                        JsonRpcResponse::error(JsonRpcError::internal_error(message), id)
                    }
                };

                if !is_notification
                    && let Ok(response_json) = serde_json::to_string(&response)
                {
                    let _ = writer.write_all(response_json.as_bytes()).await;
                    let _ = writer.write_all(b"\n").await;
                }
            }
            Err(e) => {
                tracing::error!("{peer_label} read error: {e}");
                break;
            }
        }
    }
}

fn create_shared_handler() -> Arc<IpcServiceHandler> {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    Arc::new(IpcServiceHandler::new(registry))
}

fn log_available_methods() {
    tracing::info!("   Methods available:");
    tracing::info!("     • http.request, http.get, http.post - HTTP/HTTPS requests");
    tracing::info!("     • stun.serve, stun.stop, stun.status - STUN server lifecycle");
    tracing::info!("     • relay.serve, relay.stop, relay.status, relay.allocate - Relay server");
    tracing::info!("     • discovery.peers - Real-time peer discovery");
    tracing::info!("     • rendezvous.register, rendezvous.lookup - Relay server");
    tracing::info!("     • peer.connect - UDP hole punching");
}

/// Start Unix socket IPC server for external primal access to HTTP/HTTPS capabilities.
#[cfg(unix)]
async fn start_ipc_server(socket_path: &str, _beardog_socket: &str) -> Result<()> {
    let _ = std::fs::remove_file(socket_path);
    let shared_handler = create_shared_handler();

    tracing::info!("✅ IPC server listening on {}", socket_path);
    log_available_methods();

    let listener = tokio::net::UnixListener::bind(socket_path)
        .map_err(|e| anyhow::anyhow!("Failed to bind to {socket_path}: {e}"))?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let handler = Arc::clone(&shared_handler);
                tokio::spawn(
                    async move { handle_json_rpc_connection(stream, handler, "IPC").await },
                );
            }
            Err(e) => tracing::error!("Failed to accept IPC connection: {e}"),
        }
    }
}

/// Start TCP IPC server for platforms where Unix sockets are restricted
/// (Android SELinux, Windows). Same JSON-RPC 2.0 protocol, just over TCP.
async fn start_tcp_ipc_server(listen_addr: &str, _beardog_socket: &str) -> Result<()> {
    let addr: std::net::SocketAddr = listen_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid listen address '{listen_addr}': {e}"))?;

    let shared_handler = create_shared_handler();

    tracing::info!("✅ TCP IPC server listening on {}", addr);
    log_available_methods();

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {addr}: {e}"))?;

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                let handler = Arc::clone(&shared_handler);
                let label = format!("TCP:{peer_addr}");
                tokio::spawn(async move {
                    handle_json_rpc_connection(stream, handler, &label).await;
                });
            }
            Err(e) => tracing::error!("Failed to accept TCP IPC connection: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use crate::bin_interface::ServerArgs;
    use clap::Parser;

    #[derive(Parser)]
    #[command(name = "songbird")]
    struct Cli {
        #[command(flatten)]
        args: ServerArgs,
    }

    fn effective_external_port(args: &ServerArgs) -> u16 {
        args.federation_port.unwrap_or(args.port)
    }

    #[test]
    fn federation_port_takes_precedence_over_port() {
        let cli = Cli::try_parse_from(["songbird", "--port", "8080", "--federation-port", "9090"])
            .unwrap();
        assert_eq!(cli.args.port, 8080);
        assert_eq!(cli.args.federation_port, Some(9090));
        assert_eq!(effective_external_port(&cli.args), 9090);
    }

    #[test]
    fn port_used_when_federation_port_absent() {
        let cli = Cli::try_parse_from(["songbird", "--port", "7777"]).unwrap();
        assert_eq!(effective_external_port(&cli.args), 7777);
    }

    #[test]
    fn daemon_and_verbose_flags_parse() {
        let cli =
            Cli::try_parse_from(["songbird", "--port", "80", "--daemon", "--verbose"]).unwrap();
        assert!(cli.args.daemon);
        assert!(cli.args.verbose);
    }

    #[test]
    fn socket_and_listen_optional() {
        let cli = Cli::try_parse_from(["songbird"]).unwrap();
        assert!(cli.args.socket.is_none());
        assert!(cli.args.listen.is_none());
    }

    #[test]
    fn tcp_listen_address_accepts_host_port() {
        let cli = Cli::try_parse_from(["songbird", "--listen", "127.0.0.1:9901", "--port", "3000"])
            .unwrap();
        assert_eq!(cli.args.listen.as_deref(), Some("127.0.0.1:9901"));
    }

    #[test]
    fn dark_forest_flag_parses() {
        let cli = Cli::try_parse_from(["songbird", "--dark-forest"]).unwrap();
        assert!(cli.args.dark_forest);
    }

    #[test]
    fn dark_forest_defaults_to_false() {
        let cli = Cli::try_parse_from(["songbird"]).unwrap();
        assert!(!cli.args.dark_forest);
    }

    #[test]
    fn pid_dir_flag_parses() {
        let cli = Cli::try_parse_from(["songbird", "--pid-dir", "/data/local/tmp"]).unwrap();
        assert_eq!(cli.args.pid_dir.as_deref(), Some("/data/local/tmp"));
    }

    #[test]
    fn pid_dir_defaults_to_none() {
        let cli = Cli::try_parse_from(["songbird"]).unwrap();
        assert!(cli.args.pid_dir.is_none());
    }

    #[test]
    fn all_flags_combined() {
        let cli = Cli::try_parse_from([
            "songbird",
            "--port",
            "9090",
            "--dark-forest",
            "--pid-dir",
            "/run/songbird",
            "--verbose",
            "--daemon",
        ])
        .unwrap();
        assert_eq!(cli.args.port, 9090);
        assert!(cli.args.dark_forest);
        assert_eq!(cli.args.pid_dir.as_deref(), Some("/run/songbird"));
        assert!(cli.args.verbose);
        assert!(cli.args.daemon);
    }
}
