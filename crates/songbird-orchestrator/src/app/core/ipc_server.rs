// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unix socket and TCP IPC server startup for inter-primal communication.

use anyhow::Result;
use std::sync::Arc;
#[cfg(unix)]
use tracing::error;
use tracing::{info, warn};

use super::SongbirdOrchestrator;

impl SongbirdOrchestrator {
    /// Start Unix Socket IPC server for inter-primal communication (Jan 4, 2026)
    #[cfg(unix)]
    pub(crate) async fn start_ipc_server(&self) -> Result<()> {
        use crate::ipc::{ServiceRegistry, UnixSocketServer};

        info!("🎧 Starting Unix Socket IPC server (v3.20.0 - Service Registry Mode)");
        info!(
            "   Family ID: {}",
            songbird_process_env::var("SONGBIRD_FAMILY_ID")
                .unwrap_or_else(|_| String::from("default"))
        );
        info!("   Protocol: JSON-RPC 2.0");

        let service_registry = Arc::new(ServiceRegistry::new());
        let discovery_listener_clone = self.discovery_listener.clone();
        let connection_manager_clone = Arc::clone(&self.connection_manager);

        let crypto_socket = crate::env_config::security_crypto_ipc_socket_from_env(|| {
            let family_id = songbird_process_env::var("SONGBIRD_FAMILY_ID")
                .or_else(|_| songbird_process_env::var("FAMILY_ID"))
                .unwrap_or_else(|_| String::from("default"));
            songbird_types::defaults::paths::family_scoped_crypto_socket_path(&family_id)
                .to_string_lossy()
                .into_owned()
        });

        info!("🔐 Security provider (Direct): {}", crypto_socket);
        let security_client =
            Arc::new(songbird_http_client::SecurityRpcClient::new_direct(crypto_socket));

        let server = Arc::new(UnixSocketServer::new(
            service_registry,
            discovery_listener_clone,
            connection_manager_clone,
            security_client,
        ));

        let server_arc = Arc::clone(&server);
        let server_task = tokio::spawn(async move {
            if let Err(e) = server_arc.start().await {
                error!("❌ Unix Socket IPC server error: {}", e);
            }
        });

        if !server.wait_ready(songbird_types::defaults::timeouts::DEFAULT_REQUEST_TIMEOUT).await {
            warn!("⚠️  Unix Socket IPC server did not become ready within 5 seconds");
        }

        info!("✅ Unix Socket IPC server started successfully");
        info!("   APIs: 11 total");
        info!(
            "   - Service Registry: register_service, discover_by_capability, get_service_health, health_check"
        );
        info!(
            "   - P2P Discovery: discover_by_family, create_genetic_tunnel, announce_capabilities"
        );
        info!(
            "   - Graph Intelligence: graph.validate, graph.check_availability, graph.suggest_alternatives, coordination.validate_pattern"
        );
        info!("   🌱 Primals can now register and discover each other!");

        drop(server_task);

        Ok(())
    }

    /// Start the JSON-RPC IPC server on non-Unix platforms (Windows, etc.).
    #[cfg(not(unix))]
    pub(crate) async fn start_ipc_server(&self) -> Result<()> {
        use songbird_types::defaults::ports::DEFAULT_IPC_LISTEN_PORT;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let ipc_port = songbird_process_env::var("SONGBIRD_IPC_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(DEFAULT_IPC_LISTEN_PORT);

        let bind_host = &self._config.network.bind_host;
        let listen_addr = format!("{bind_host}:{ipc_port}");

        info!("🎧 Starting TCP IPC server (non-Unix platform: {})", std::env::consts::OS);
        info!("   Listen: {listen_addr}");
        info!("   Protocol: JSON-RPC 2.0 over TCP");
        info!(
            "   Family ID: {}",
            songbird_process_env::var("SONGBIRD_FAMILY_ID")
                .unwrap_or_else(|_| String::from("default"))
        );

        let shared_handler = self.shared_ipc_handler.clone().unwrap_or_else(|| {
            Arc::new(songbird_universal_ipc::service::IpcServiceHandler::with_federation_state(
                Arc::new(tokio::sync::RwLock::new(
                    songbird_universal_ipc::registry::ServiceRegistry::new(),
                )),
                Arc::clone(&self.federation_state),
            ))
        });

        let addr: std::net::SocketAddr = listen_addr
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid IPC listen address '{listen_addr}': {e}"))?;

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| anyhow::anyhow!("TCP IPC server failed to bind {listen_addr}: {e}"))?;

        info!("✅ TCP IPC server started on {listen_addr}");
        info!("   🌱 Primals can now register and discover each other (TCP transport)");

        let shared_handler_clone = Arc::clone(&shared_handler);
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        let handler = Arc::clone(&shared_handler_clone);
                        tokio::spawn(async move {
                            let (reader, mut writer) = stream.into_split();
                            let mut buf_reader = BufReader::new(reader);

                            if let Ok(buf) = buf_reader.fill_buf().await {
                                if buf.len() >= 2
                                    && songbird_types::constants::ribocipher::is_signal_byte(buf[0])
                                {
                                    let consume = if buf[1]
                                        == songbird_types::constants::ribocipher::VERSION_1
                                    {
                                        2
                                    } else {
                                        1
                                    };
                                    buf_reader.consume(consume);
                                }
                            }

                            let mut lines = buf_reader.lines();
                            while let Ok(Some(line)) = lines.next_line().await {
                                let line = line.trim().to_string();
                                if line.is_empty() {
                                    continue;
                                }
                                let parsed: serde_json::Value = match serde_json::from_str(&line) {
                                    Ok(v) => v,
                                    Err(_) => continue,
                                };
                                let method =
                                    parsed["method"].as_str().unwrap_or_default().to_string();
                                let params = parsed
                                    .get("params")
                                    .cloned()
                                    .unwrap_or(serde_json::Value::Object(Default::default()));
                                let id = parsed.get("id").cloned();

                                let result = handler.handle(&method, params).await;
                                let response = match result {
                                    Ok(data) => serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "result": data,
                                        "id": id,
                                    }),
                                    Err(msg) => serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "error": { "code": -32603, "message": msg },
                                        "id": id,
                                    }),
                                };

                                let mut resp_line =
                                    serde_json::to_string(&response).unwrap_or_default();
                                resp_line.push('\n');
                                if writer.write_all(resp_line.as_bytes()).await.is_err() {
                                    break;
                                }
                            }
                            debug!("TCP IPC connection closed: {peer_addr}");
                        });
                    }
                    Err(e) => warn!("Failed to accept TCP IPC connection: {e}"),
                }
            }
        });

        Ok(())
    }
}
