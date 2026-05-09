// SPDX-License-Identifier: AGPL-3.0-or-later
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
use crate::ipc::pure_rust_server::method_gate::{
    CallerContext, MethodGate, dispatch_auth_method, extract_bearer_token,
};

static BIN_GATE: std::sync::LazyLock<MethodGate> = std::sync::LazyLock::new(MethodGate::from_env);

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
    if let Some(ref path) = args.config {
        tracing::info!("   Config file: {path}");
    } else {
        tracing::info!("   Config source: Environment variables");
    }
    let mut config = CanonicalSongbirdConfig::from_env()?;

    // Override bind address and port from CLI (CLI takes precedence over config/env)
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

    // Step 4: Start the orchestrator (non-blocking, returns handle)
    tracing::info!("🔧 Starting orchestrator components...");
    let mut orchestrator = app::start_orchestrator(config).await?;
    tracing::info!("   Orchestrator: ✅ Started");

    tracing::info!("✅ Songbird ready!");
    tracing::info!("");

    // Step 4.5: Start IPC server (Unix socket or TCP based on args)
    // wateringHole standard: `server --port <PORT>` binds newline-delimited TCP JSON-RPC.
    // When --listen is not explicit, derive it from --port for compliance.
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
    let _ipc_handle = if let Some(ref listen_addr) = effective_listen {
        // TCP IPC mode (Android/Universal)
        tracing::info!("");
        tracing::info!("🌐 Starting TCP IPC Server (Android/Universal mode)...");
        tracing::info!("   Listen: {}", listen_addr);
        tracing::info!("   Protocol: JSON-RPC 2.0 over TCP");
        if let Some(ref fam) = family_identity {
            tracing::info!("   Family: {}", fam);
        }

        let security_socket = args.security_socket.clone().unwrap_or_else(|| {
            songbird_crypto_provider::socket_discovery::discover_security_socket()
        });
        tracing::info!("   Security provider: {}", security_socket);
        tracing::info!("   Capabilities: http, stun, discovery");

        let listen_clone = listen_addr.clone();
        Some(tokio::spawn(async move {
            match start_tcp_ipc_server(&listen_clone, &security_socket).await {
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

        let security_socket = args.security_socket.unwrap_or_else(|| {
            songbird_crypto_provider::socket_discovery::discover_security_socket()
        });
        tracing::info!("   Security provider: {}", security_socket);
        tracing::info!("   Capabilities: http, discovery, secure_http");

        // Spawn IPC server in background task (Unix only)
        #[cfg(unix)]
        let ipc_task = {
            let socket_clone = socket_path;
            Some(tokio::spawn(async move {
                match start_ipc_server(&socket_clone, &security_socket).await {
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

/// Handle a single connection with BTSP auto-detection.
///
/// Reads the first line from the stream. If it looks like a BTSP `ClientHello`
/// (`"protocol":"btsp"`), performs the NDJSON BTSP handshake before falling
/// through to JSON-RPC. Otherwise treats it as a plain JSON-RPC request.
async fn handle_connection<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
    stream: S,
    handler: Arc<IpcServiceHandler>,
    security_client: Arc<songbird_http_client::SecurityRpcClient>,
    peer_label: &str,
    caller: &CallerContext,
) {
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);
    let mut first_line = String::new();

    match reader.read_line(&mut first_line).await {
        Ok(0) => {
            tracing::debug!("{peer_label} disconnected before sending data");
            return;
        }
        Ok(_) => {}
        Err(e) => {
            tracing::error!("{peer_label} read error on first line: {e}");
            return;
        }
    }

    if crate::ipc::btsp::is_btsp_client_hello(&first_line) {
        tracing::info!("{peer_label} BTSP ClientHello detected — starting NDJSON handshake");
        match crate::ipc::btsp::perform_server_handshake_ndjson(
            &first_line,
            &mut reader,
            &mut writer,
            &security_client,
        )
        .await
        {
            Ok(session) => {
                tracing::info!(
                    "{peer_label} BTSP handshake complete (session={}, cipher={})",
                    session.session_id,
                    session.cipher,
                );
            }
            Err(e) => {
                tracing::warn!("{peer_label} BTSP handshake failed: {e}");
                let err_frame =
                    serde_json::json!({"error":"handshake_failed","reason":e.to_string()});
                let mut bytes = serde_json::to_vec(&err_frame).unwrap_or_default();
                bytes.push(b'\n');
                let _ = writer.write_all(&bytes).await;
                let _ = writer.flush().await;
                return;
            }
        }
    } else if !first_line.trim().is_empty() {
        dispatch_json_rpc_line(&first_line, &mut writer, &handler, peer_label, caller).await;
    }

    handle_json_rpc_lines(&mut reader, &mut writer, &handler, &security_client, peer_label, caller)
        .await;
}

/// Process a stream of newline-delimited JSON-RPC requests.
///
/// If the client sends `btsp.negotiate`, the Phase 3 negotiation is handled
/// inline: the NDJSON response is sent, and (if a real cipher was negotiated)
/// the connection switches to encrypted framing for all subsequent traffic.
async fn handle_json_rpc_lines<R, W>(
    reader: &mut R,
    writer: &mut W,
    handler: &Arc<IpcServiceHandler>,
    security_client: &Arc<songbird_http_client::SecurityRpcClient>,
    peer_label: &str,
    caller: &CallerContext,
) where
    R: tokio::io::AsyncBufRead + tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
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

                if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(&line)
                    && request.method == "btsp.negotiate"
                {
                    let id = request.id.unwrap_or(serde_json::Value::Null);
                    let params = request.params.unwrap_or(serde_json::Value::Null);
                    let (result, keys) =
                        crate::ipc::btsp_phase3::handle_negotiate(&params, security_client).await;

                    let resp = JsonRpcResponse::success(
                        serde_json::to_value(&result).unwrap_or_default(),
                        id,
                    );
                    if let Ok(json) = serde_json::to_string(&resp) {
                        let _ = writer.write_all(json.as_bytes()).await;
                        let _ = writer.write_all(b"\n").await;
                        let _ = writer.flush().await;
                    }

                    if let Some(session_keys) = keys {
                        tracing::debug!(
                            "{peer_label} BTSP Phase 3: switching to encrypted framing"
                        );
                        // Safety: reader is the same BufReader used for negotiate line
                        // I/O — any bytes buffered ahead are preserved. We do NOT call
                        // into_inner() (avoids barraCuda/coralReef bug class).
                        handle_encrypted_json_rpc(
                            reader,
                            writer,
                            handler,
                            session_keys,
                            peer_label,
                            caller,
                        )
                        .await;
                        return;
                    }
                    continue;
                }

                dispatch_json_rpc_line(&line, writer, handler, peer_label, caller).await;
            }
            Err(e) => {
                tracing::error!("{peer_label} read error: {e}");
                break;
            }
        }
    }
}

/// Encrypted JSON-RPC loop (BTSP Phase 3) for the `bin_interface` server path.
async fn handle_encrypted_json_rpc<R, W>(
    reader: &mut R,
    writer: &mut W,
    handler: &Arc<IpcServiceHandler>,
    keys: crate::ipc::btsp_phase3::SessionKeys,
    peer_label: &str,
    caller: &CallerContext,
) where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    tracing::info!("{peer_label} BTSP Phase 3: encrypted session active");
    loop {
        let frame = match crate::ipc::btsp_phase3::read_encrypted_frame(reader).await {
            Ok(f) => f,
            Err(e) => {
                let msg = format!("{e:#}");
                if msg.contains("UnexpectedEof") || msg.contains("failed to read frame length") {
                    tracing::debug!("{peer_label} BTSP Phase 3: client disconnected");
                } else {
                    tracing::error!("{peer_label} BTSP Phase 3 frame read error: {e:#}");
                }
                break;
            }
        };

        let plaintext = match keys.decrypt(&frame) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("{peer_label} BTSP Phase 3 decrypt failed: {e:#}");
                break;
            }
        };

        let mut request = match serde_json::from_slice::<JsonRpcRequest>(&plaintext) {
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
                if let Ok(resp_bytes) = serde_json::to_vec(&resp)
                    && let Ok(encrypted) = keys.encrypt(&resp_bytes)
                {
                    let _ =
                        crate::ipc::btsp_phase3::write_encrypted_frame(writer, &encrypted).await;
                }
                continue;
            }
        };

        let is_notification = request.id.is_none();
        let id = request.id.unwrap_or(serde_json::Value::Null);
        tracing::debug!(
            "{peer_label} BTSP Phase 3 JSON-RPC: {} (notification={is_notification})",
            request.method
        );

        // Extract _bearer_token from params and enrich CallerContext
        let caller = if let Some(ref mut params) = request.params {
            if let Some(token) = extract_bearer_token(params) {
                caller.clone().with_bearer_token(token)
            } else {
                caller.clone()
            }
        } else {
            caller.clone()
        };

        // JH-0: Pre-dispatch method gate
        let response =
            if let Some(auth_result) = dispatch_auth_method(&request.method, &BIN_GATE, &caller) {
                JsonRpcResponse::success(auth_result, id.clone())
            } else if let Err(gate_err) = BIN_GATE.check(&request.method, &caller) {
                JsonRpcResponse::error(
                    JsonRpcError {
                        code: gate_err.code,
                        message: gate_err.message,
                        data: gate_err.data,
                    },
                    id.clone(),
                )
            } else {
                match handler
                    .handle(&request.method, request.params.unwrap_or(serde_json::Value::Null))
                    .await
                {
                    Ok(result) => JsonRpcResponse::success(result, id.clone()),
                    Err(message) => {
                        JsonRpcResponse::error(JsonRpcError::internal_error(message), id.clone())
                    }
                }
            };

        if !is_notification
            && let Ok(resp_bytes) = serde_json::to_vec(&response)
            && let Ok(encrypted) = keys.encrypt(&resp_bytes)
            && let Err(e) = crate::ipc::btsp_phase3::write_encrypted_frame(writer, &encrypted).await
        {
            tracing::error!("{peer_label} BTSP Phase 3 write error: {e:#}");
            break;
        }
    }
}

/// Parse and dispatch a single JSON-RPC line, writing the response.
async fn dispatch_json_rpc_line<W: tokio::io::AsyncWrite + Unpin>(
    line: &str,
    writer: &mut W,
    handler: &Arc<IpcServiceHandler>,
    peer_label: &str,
    caller: &CallerContext,
) {
    let mut request = match serde_json::from_str::<JsonRpcRequest>(line) {
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
            return;
        }
    };

    let is_notification = request.id.is_none();
    let id = request.id.unwrap_or(serde_json::Value::Null);
    tracing::debug!("{peer_label} JSON-RPC: {} (notification={is_notification})", request.method,);

    // Extract _bearer_token from params and enrich CallerContext
    let caller = if let Some(ref mut params) = request.params {
        if let Some(token) = extract_bearer_token(params) {
            caller.clone().with_bearer_token(token)
        } else {
            caller.clone()
        }
    } else {
        caller.clone()
    };

    // JH-0: Pre-dispatch method gate
    if let Some(auth_result) = dispatch_auth_method(&request.method, &BIN_GATE, &caller) {
        let response = JsonRpcResponse::success(auth_result, id);
        if !is_notification && let Ok(response_json) = serde_json::to_string(&response) {
            let _ = writer.write_all(response_json.as_bytes()).await;
            let _ = writer.write_all(b"\n").await;
        }
        return;
    }
    if let Err(gate_err) = BIN_GATE.check(&request.method, &caller) {
        let response = JsonRpcResponse::error(
            JsonRpcError {
                code: gate_err.code,
                message: gate_err.message,
                data: gate_err.data,
            },
            id,
        );
        if !is_notification && let Ok(response_json) = serde_json::to_string(&response) {
            let _ = writer.write_all(response_json.as_bytes()).await;
            let _ = writer.write_all(b"\n").await;
        }
        return;
    }

    let response = match handler
        .handle(&request.method, request.params.unwrap_or(serde_json::Value::Null))
        .await
    {
        Ok(result) => JsonRpcResponse::success(result, id),
        Err(message) => JsonRpcResponse::error(JsonRpcError::internal_error(message), id),
    };

    if !is_notification && let Ok(response_json) = serde_json::to_string(&response) {
        let _ = writer.write_all(response_json.as_bytes()).await;
        let _ = writer.write_all(b"\n").await;
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

/// Start Unix socket IPC server with BTSP auto-detection.
#[cfg(unix)]
async fn start_ipc_server(socket_path: &str, security_socket: &str) -> Result<()> {
    let _ = std::fs::remove_file(socket_path);
    let shared_handler = create_shared_handler();
    let security_client =
        Arc::new(songbird_http_client::SecurityRpcClient::new_direct(security_socket.to_owned()));

    tracing::info!("✅ IPC server listening on {}", socket_path);
    tracing::info!("   BTSP auto-detect: enabled (JSON-line ClientHello → NDJSON handshake)");
    log_available_methods();

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

/// Start TCP IPC server with BTSP auto-detection.
/// For platforms where Unix sockets are restricted (Android SELinux, Windows).
async fn start_tcp_ipc_server(listen_addr: &str, security_socket: &str) -> Result<()> {
    let addr: std::net::SocketAddr = listen_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid listen address '{listen_addr}': {e}"))?;

    let shared_handler = create_shared_handler();
    let security_client =
        Arc::new(songbird_http_client::SecurityRpcClient::new_direct(security_socket.to_owned()));

    tracing::info!("✅ TCP IPC server listening on {}", addr);
    tracing::info!("   BTSP auto-detect: enabled (JSON-line ClientHello → NDJSON handshake)");
    log_available_methods();

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

/// Parse `--bind` flag value into (host, optional port).
///
/// Accepts:
/// - `"127.0.0.1"` → (`"127.0.0.1"`, None)
/// - `"0.0.0.0:9200"` → (`"0.0.0.0"`, Some(9200))
/// - `"[::1]:8080"` → (`"[::1]"`, Some(8080))
/// - `"::"` → (`"::"`, None)
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
    fn bind_flag_defaults_to_localhost() {
        let cli = Cli::try_parse_from(["songbird"]).unwrap();
        assert_eq!(cli.args.bind, "127.0.0.1");
    }

    #[test]
    fn bind_flag_accepts_host() {
        let cli = Cli::try_parse_from(["songbird", "--bind", "0.0.0.0"]).unwrap();
        assert_eq!(cli.args.bind, "0.0.0.0");
    }

    #[test]
    fn bind_flag_accepts_host_port() {
        let cli = Cli::try_parse_from(["songbird", "--bind", "192.168.1.5:9200"]).unwrap();
        assert_eq!(cli.args.bind, "192.168.1.5:9200");
    }

    #[test]
    fn parse_bind_flag_host_only() {
        let (host, port) = super::parse_bind_flag("127.0.0.1");
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, None);
    }

    #[test]
    fn parse_bind_flag_host_port() {
        let (host, port) = super::parse_bind_flag("0.0.0.0:9200");
        assert_eq!(host, "0.0.0.0");
        assert_eq!(port, Some(9200));
    }

    #[test]
    fn parse_bind_flag_ipv6_bracketed_port() {
        let (host, port) = super::parse_bind_flag("[::1]:8080");
        assert_eq!(host, "[::1]");
        assert_eq!(port, Some(8080));
    }

    #[test]
    fn parse_bind_flag_ipv6_no_port() {
        let (host, port) = super::parse_bind_flag("::");
        assert_eq!(host, "::");
        assert_eq!(port, None);
    }

    #[test]
    fn parse_bind_flag_ipv6_bracketed_no_port() {
        let (host, port) = super::parse_bind_flag("[::1]");
        assert_eq!(host, "[::1]");
        assert_eq!(port, None);
    }

    #[test]
    fn all_flags_combined() {
        let cli = Cli::try_parse_from([
            "songbird",
            "--port",
            "9090",
            "--bind",
            "0.0.0.0",
            "--dark-forest",
            "--pid-dir",
            "/run/songbird",
            "--verbose",
            "--daemon",
        ])
        .unwrap();
        assert_eq!(cli.args.port, 9090);
        assert_eq!(cli.args.bind, "0.0.0.0");
        assert!(cli.args.dark_forest);
        assert_eq!(cli.args.pid_dir.as_deref(), Some("/run/songbird"));
        assert!(cli.args.verbose);
        assert!(cli.args.daemon);
    }
}
