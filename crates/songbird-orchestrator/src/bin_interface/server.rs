// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Server lifecycle: startup, config, signal handling, and shutdown.
//!
//! The per-connection IPC protocol (BTSP auto-detect, JSON-RPC dispatch,
//! encrypted framing) lives in [`ipc_session`](super::ipc_session).

use anyhow::Result;
use songbird_universal_ipc::registry::ServiceRegistry;
use songbird_universal_ipc::service::IpcServiceHandler;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::ServerArgs;
use super::ipc_session::handle_connection;
use crate::ipc::pure_rust_server::method_gate::{CallerContext, MethodGate};

pub(super) static BIN_GATE: std::sync::LazyLock<MethodGate> =
    std::sync::LazyLock::new(MethodGate::from_env);

/// Run orchestrator in server mode.
///
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn run_server(args: ServerArgs) -> Result<()> {
    use crate::app;
    use crate::process_manager::ProcessManager;
    use songbird_types::config::CanonicalSongbirdConfig;

    if args.verbose {
        tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();
    } else {
        tracing_subscriber::fmt::init();
    }

    if args.dark_forest {
        songbird_process_env::set_var("SONGBIRD_DARK_FOREST", "true");
    }
    if let Some(ref pid_dir) = args.pid_dir {
        songbird_process_env::set_var("SONGBIRD_PID_DIR", pid_dir);
    }

    let actual_port = args.federation_port.unwrap_or(args.port);

    tracing::info!("🚀 Songbird v{} - Server Mode", env!("CARGO_PKG_VERSION"));
    tracing::info!(
        "   Mode: Server {}",
        if args.daemon {
            "(daemon)"
        } else {
            "(foreground)"
        }
    );
    tracing::info!("   HTTP Bind: {}:{}", args.bind, actual_port);
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

    let process_mgr = ProcessManager::new()?;
    let _singleton_guard = process_mgr.acquire_lock()?;
    tracing::info!("   Instance Lock: ✅ Acquired (PID file active)");

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

    tracing::info!("📋 Loading configuration...");
    if let Some(ref path) = args.config {
        tracing::info!("   Config file: {path}");
    } else {
        tracing::info!("   Config source: Environment variables");
    }
    let mut config = CanonicalSongbirdConfig::from_env()?;

    let (bind_host, bind_port_override) = parse_bind_flag(&args.bind);
    config.network.bind_host = bind_host.to_string();
    if let Some(bp) = bind_port_override {
        config.network.base_port = bp;
    } else {
        config.network.base_port = actual_port;
    }
    tracing::info!(
        "   Configuration: ✅ Loaded (bind: {}:{})",
        config.network.bind_host,
        config.network.base_port,
    );

    tracing::info!("🧹 Cleaning stale sockets...");
    crate::env_config::cleanup_stale_sockets();

    tracing::info!("🔧 Starting orchestrator components...");
    let mut orchestrator = app::start_orchestrator(config).await?;
    tracing::info!("   Orchestrator: ✅ Started");
    tracing::info!("✅ Songbird ready!");
    tracing::info!("");

    let effective_listen = args.listen.clone().or_else(|| {
        if args.socket.is_none() {
            let addr =
                format!("{}:{}", songbird_types::constants::PRODUCTION_BIND_ADDRESS, actual_port);
            tracing::info!(
                "   TCP JSON-RPC: {} (derived from --port per wateringHole standard)",
                addr
            );
            Some(addr)
        } else {
            None
        }
    });
    let socket_path_for_registration = args.socket.clone().or_else(|| effective_listen.clone());
    let _ipc_handle =
        spawn_ipc_listener(&args, effective_listen.as_ref(), family_identity.as_ref());

    if socket_path_for_registration.is_some() {
        tracing::info!("");
        tracing::info!("🌟 Registering capabilities with Neural API...");
        if let Err(e) = crate::capability_registration::register_capabilities().await {
            tracing::warn!("⚠️  Failed to register capabilities: {}", e);
            tracing::warn!("   Songbird will continue without Neural API registration");
        }
    }

    tracing::info!("");
    tracing::info!("💡 Press Ctrl+C to stop gracefully");
    if args.daemon {
        tracing::info!("📌 Daemon mode: Process detached");
    }

    await_shutdown_signal().await;

    tracing::info!("🧹 Stopping orchestrator components...");
    if socket_path_for_registration.is_some() {
        let _ = crate::capability_registration::unregister_capabilities().await;
    }
    orchestrator.stop().await?;
    tracing::info!("   Orchestrator: ✅ Stopped");
    tracing::info!("✅ Graceful shutdown complete");

    Ok(())
}

// ── Lifecycle helpers ────────────────────────────────────────────────────

async fn await_shutdown_signal() {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("🛑 Received SIGINT (Ctrl+C), initiating graceful shutdown...");
        }
        () = async {
            #[cfg(unix)]
            {
                #[expect(clippy::expect_used, reason = "process-level signal handler — panicking is correct on setup failure")]
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
}

fn spawn_ipc_listener(
    args: &ServerArgs,
    effective_listen: Option<&String>,
    family_identity: Option<&String>,
) -> Option<tokio::task::JoinHandle<()>> {
    if let Some(listen_addr) = effective_listen {
        tracing::info!("");
        tracing::info!("🌐 Starting TCP IPC Server (Android/Universal mode)...");
        tracing::info!("   Listen: {}", listen_addr);
        tracing::info!("   Protocol: JSON-RPC 2.0 over TCP");
        if let Some(fam) = family_identity {
            tracing::info!("   Family: {}", fam);
        }

        let security_socket = args.security_socket.clone().unwrap_or_else(|| {
            songbird_crypto_provider::socket_discovery::discover_security_socket()
        });
        tracing::info!("   Security provider: {}", security_socket);
        log_available_methods();

        let listen_clone = listen_addr.clone();
        Some(tokio::spawn(async move {
            match start_tcp_ipc_server(&listen_clone, &security_socket).await {
                Ok(()) => tracing::info!("TCP IPC server stopped gracefully"),
                Err(e) => tracing::error!("TCP IPC server error: {}", e),
            }
        }))
    } else if let Some(socket_path) = &args.socket {
        tracing::info!("");
        tracing::info!("🌐 Starting IPC Server (biomeOS integration)...");
        tracing::info!("   Socket: {}", socket_path);
        tracing::info!("   Protocol: JSON-RPC 2.0 over Unix sockets");
        if let Some(fam) = family_identity {
            tracing::info!("   Family: {}", fam);
        }

        let security_socket = args.security_socket.clone().unwrap_or_else(|| {
            songbird_crypto_provider::socket_discovery::discover_security_socket()
        });
        tracing::info!("   Security provider: {}", security_socket);
        log_available_methods();

        #[cfg(unix)]
        {
            let socket_clone = socket_path.clone();
            Some(tokio::spawn(async move {
                match start_ipc_server(&socket_clone, &security_socket).await {
                    Ok(()) => tracing::info!("IPC server stopped gracefully"),
                    Err(e) => tracing::error!("IPC server error: {}", e),
                }
            }))
        }
        #[cfg(not(unix))]
        {
            tracing::info!("IPC server: Windows TCP fallback (coming in Phase 2)");
            None
        }
    } else {
        tracing::info!("");
        tracing::info!("💡 Tip: Use --socket or --listen to enable IPC");
        tracing::info!("   Unix: --socket /run/user/$(id -u)/biomeos/songbird.sock");
        tracing::info!("   TCP:  --listen 127.0.0.1:9901 (Android/Universal)");
        None
    }
}

// ── Listener setup ───────────────────────────────────────────────────────

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

#[cfg(unix)]
async fn start_ipc_server(socket_path: &str, security_socket: &str) -> Result<()> {
    let _ = std::fs::remove_file(socket_path);
    let shared_handler = create_shared_handler();
    let security_client =
        Arc::new(songbird_http_client::SecurityRpcClient::new_direct(security_socket.to_owned()));

    tracing::info!("✅ IPC server listening on {}", socket_path);
    tracing::info!("   BTSP auto-detect: enabled (JSON-line ClientHello → NDJSON handshake)");

    let listener = tokio::net::UnixListener::bind(socket_path)
        .map_err(|e| anyhow::anyhow!("Failed to bind to {socket_path}: {e}"))?;

    let uds_caller = CallerContext::from_unix();
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let handler = Arc::clone(&shared_handler);
                let sec = Arc::clone(&security_client);
                let caller = uds_caller.clone();
                tokio::spawn(async move {
                    handle_connection(stream, handler, sec, "IPC", &caller).await;
                });
            }
            Err(e) => tracing::error!("Failed to accept IPC connection: {e}"),
        }
    }
}

async fn start_tcp_ipc_server(listen_addr: &str, security_socket: &str) -> Result<()> {
    let addr: std::net::SocketAddr = listen_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid listen address '{listen_addr}': {e}"))?;

    let shared_handler = create_shared_handler();
    let security_client =
        Arc::new(songbird_http_client::SecurityRpcClient::new_direct(security_socket.to_owned()));

    tracing::info!("✅ TCP IPC server listening on {}", addr);
    tracing::info!("   BTSP auto-detect: enabled (JSON-line ClientHello → NDJSON handshake)");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {addr}: {e}"))?;

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                let handler = Arc::clone(&shared_handler);
                let sec = Arc::clone(&security_client);
                let label = format!("TCP:{peer_addr}");
                let caller = CallerContext::from_tcp(peer_addr);
                tokio::spawn(async move {
                    handle_connection(stream, handler, sec, &label, &caller).await;
                });
            }
            Err(e) => tracing::error!("Failed to accept TCP IPC connection: {e}"),
        }
    }
}

// ── Bind flag parsing ────────────────────────────────────────────────────

/// Parse `--bind` flag value into (host, optional port).
fn parse_bind_flag(value: &str) -> (&str, Option<u16>) {
    if let Some(bracket_end) = value.find(']') {
        if let Some(colon_after) = value[bracket_end..].find(':') {
            let port_str = &value[bracket_end + colon_after + 1..];
            if let Ok(port) = port_str.parse::<u16>() {
                return (&value[..=bracket_end], Some(port));
            }
        }
        return (value, None);
    }

    match value.rfind(':') {
        Some(pos) if value[..pos].contains(':') => (value, None),
        Some(pos) => {
            let port_str = &value[pos + 1..];
            match port_str.parse::<u16>() {
                Ok(port) => (&value[..pos], Some(port)),
                Err(_) => (value, None),
            }
        }
        None => (value, None),
    }
}

#[cfg(test)]
#[path = "server_tests.rs"]
mod tests;
