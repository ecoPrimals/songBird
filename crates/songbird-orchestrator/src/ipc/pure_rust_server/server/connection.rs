// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Unix/TCP accept loops, per-connection framing, and TCP discovery file I/O.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info, warn};

use super::super::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use super::UnixSocketServer;

impl UnixSocketServer {
    /// Try to start Unix socket server (existing optimal path)
    #[cfg(unix)]
    pub(crate) async fn try_unix_server(self: Arc<Self>) -> Result<()> {
        info!("   Trying Unix socket IPC (optimal)...");

        if let Some(parent) = self.socket_path.parent()
            && !parent.exists()
        {
            debug!("   Creating socket directory: {:?}", parent);
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create socket directory: {}", parent.display()))?;
        }

        if self.socket_path.exists() {
            debug!("   Removing stale socket file");
            std::fs::remove_file(&self.socket_path)
                .context("Failed to remove stale socket file")?;
        }

        let listener = UnixListener::bind(&*self.socket_path)
            .context(format!("Failed to bind Unix socket: {}", self.socket_path.display()))?;

        self.is_running.store(true, std::sync::atomic::Ordering::Release);
        self.is_ready.store(true, std::sync::atomic::Ordering::Release);
        self.ready_notify.notify_waiters();

        info!("✅ Unix socket JSON-RPC server listening: {}", self.socket_path.display());
        info!("   Protocol: JSON-RPC 2.0 (pure Rust)");
        info!("   APIs: 14 (3 P2P + 4 registry + 4 graph + 3 Squirrel)");
        info!("   Status: READY ✅ (atomic flag set)");

        while self.is_running() {
            match tokio::time::timeout(Duration::from_millis(100), listener.accept()).await {
                Ok(Ok((stream, _addr))) => {
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("❌ Connection handler error: {}", e);
                        }
                    });
                }
                Ok(Err(e)) => {
                    error!("❌ Failed to accept connection: {}", e);
                }
                Err(_) => {}
            }
        }

        info!("🛑 Unix socket server stopped gracefully");
        Ok(())
    }

    /// Try to start Unix socket server (existing optimal path)
    #[cfg(not(unix))]
    pub(crate) async fn try_unix_server(self: Arc<Self>) -> Result<()> {
        let _ = self;
        Err(anyhow::anyhow!("Unix sockets not supported on this platform"))
    }

    pub(crate) fn is_platform_constraint(&self, error: &anyhow::Error) -> bool {
        let error_str = format!("{error:#}");
        let lower = error_str.to_lowercase();

        if lower.contains("permission denied") {
            debug!("   Detected permission denied (potential SELinux)");

            #[cfg(target_os = "android")]
            {
                return true;
            }

            #[cfg(not(target_os = "android"))]
            {
                if self.is_selinux_enforcing() {
                    debug!("   Confirmed: SELinux is enforcing");
                    return true;
                }
            }
        }

        if lower.contains("address family not supported")
            || lower.contains("protocol not supported")
            || lower.contains("not supported")
        {
            debug!("   Detected unsupported platform feature");
            return true;
        }

        false
    }

    fn is_selinux_enforcing(&self) -> bool {
        let _ = self;
        std::fs::read_to_string("/sys/fs/selinux/enforce")
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok())
            .is_some_and(|v| v == 1)
    }

    pub(crate) async fn start_tcp_fallback(self: Arc<Self>) -> Result<()> {
        info!("🌐 Starting TCP IPC fallback (isomorphic mode)");
        info!("   Protocol: JSON-RPC 2.0 (same as Unix socket)");

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("Failed to bind TCP localhost for IPC fallback")?;

        let bound_addr = listener.local_addr()?;
        info!("✅ TCP IPC listening on {}", bound_addr);

        self.write_tcp_discovery_file(bound_addr.port())?;

        self.is_running.store(true, std::sync::atomic::Ordering::Release);
        self.is_ready.store(true, std::sync::atomic::Ordering::Release);
        self.ready_notify.notify_waiters();

        info!("   APIs: 14 (3 P2P + 4 registry + 4 graph + 3 Squirrel)");
        info!("   Status: READY ✅ (isomorphic TCP fallback active)");

        while self.is_running() {
            match tokio::time::timeout(Duration::from_millis(100), listener.accept()).await {
                Ok(Ok((stream, addr))) => {
                    debug!("📥 TCP IPC connection from {}", addr);
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_tcp_connection(stream).await {
                            error!("❌ TCP connection handler error: {}", e);
                        }
                    });
                }
                Ok(Err(e)) => {
                    error!("❌ Failed to accept TCP connection: {}", e);
                }
                Err(_) => {}
            }
        }

        info!("🛑 TCP IPC server stopped gracefully");
        Ok(())
    }

    async fn handle_tcp_connection(&self, stream: tokio::net::TcpStream) -> Result<()> {
        debug!("📥 New TCP IPC connection");

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("📤 TCP client disconnected");
                    break;
                }
                Ok(_) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                        Ok(request) => {
                            debug!("📨 TCP JSON-RPC request: {}", request.method);
                            self.handle_jsonrpc_request(request).await
                        }
                        Err(e) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError::parse_error(format!(
                                "Failed to parse JSON-RPC request: {e}"
                            ))),
                            id: serde_json::Value::Null,
                        },
                    };

                    let response_json = serde_json::to_string(&response)?;
                    writer.write_all(response_json.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                    writer.flush().await?;

                    debug!("✅ TCP response sent, closing connection");
                    break;
                }
                Err(e) => {
                    error!("❌ Failed to read from TCP socket: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    fn write_tcp_discovery_file(&self, port: u16) -> Result<()> {
        let port_file = if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir).join("songbird-ipc-port")
        } else if let Ok(home) = std::env::var("HOME") {
            let share_dir = PathBuf::from(home).join(".local/share");
            if let Err(e) = std::fs::create_dir_all(&share_dir) {
                warn!("⚠️  Failed to create {:?}: {}", share_dir, e);
                PathBuf::from("/tmp/songbird-ipc-port")
            } else {
                share_dir.join("songbird-ipc-port")
            }
        } else {
            PathBuf::from("/tmp/songbird-ipc-port")
        };

        let content = format!("tcp:127.0.0.1:{port}");
        std::fs::write(&port_file, content)
            .context(format!("Failed to write TCP discovery file: {}", port_file.display()))?;

        info!("   Discovery file: {}", port_file.display());
        Ok(())
    }

    /// Handle a single client connection with JSON-RPC 2.0
    #[cfg(unix)]
    pub(crate) async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        debug!("📥 New IPC connection");

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("📤 Client disconnected");
                    break;
                }
                Ok(_) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                        Ok(request) => {
                            debug!("📨 JSON-RPC request: {}", request.method);
                            self.handle_jsonrpc_request(request).await
                        }
                        Err(e) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError::parse_error(format!(
                                "Failed to parse JSON-RPC request: {e}"
                            ))),
                            id: serde_json::Value::Null,
                        },
                    };

                    let response_json = serde_json::to_string(&response)?;
                    writer.write_all(response_json.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                    writer.flush().await?;

                    debug!("✅ Response sent, closing connection");
                    break;
                }
                Err(e) => {
                    error!("❌ Failed to read from socket: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}
