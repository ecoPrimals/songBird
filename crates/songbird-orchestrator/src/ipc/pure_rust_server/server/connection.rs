// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unix/TCP accept loops, per-connection framing, and TCP discovery file I/O.
//!
//! When BTSP is active (production mode with `FAMILY_ID` set), every accepted
//! UDS connection is routed through first-line auto-detection:
//!
//! - First byte `0x7B` (`{`) → read full first line, then:
//!   - Contains `"protocol":"btsp"` → BTSP JSON-line (NDJSON) handshake,
//!     then persistent NDJSON JSON-RPC session (primalSpring, springs)
//!   - Otherwise → plain NDJSON JSON-RPC (biomeOS composition, local tooling)
//! - Any other first byte → BTSP length-prefixed binary framing
//!
//! This matches the security provider's TCP auto-detect pattern and the
//! storage provider's UDS pattern per
//! `UPSTREAM_CROSSTALK_AND_DOWNSTREAM_ABSORPTION.md`.

use anyhow::{Context, Result, bail};
use bytes::Bytes;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{self, Poll};
use std::time::Duration;
use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader,
    ReadBuf,
};
use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info, warn};

use super::super::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use super::UnixSocketServer;
use crate::ipc::btsp;
use crate::ipc::btsp_phase3;

// ─── PeekedStream adapter ────────────────────────────────────────────────────
//
// Combines a `BufReader<R>` (reader with peeked bytes still in its buffer) and
// a writer `W` into a single type implementing `AsyncRead + AsyncWrite + Unpin`.
// This allows the BTSP handshake (which needs a bidirectional stream) to work
// after we've already peeked via `BufReader::fill_buf`.

#[cfg(unix)]
struct PeekedStream<R, W> {
    reader: BufReader<R>,
    writer: W,
}

#[cfg(unix)]
impl<R: AsyncRead + Unpin, W: Unpin> AsyncRead for PeekedStream<R, W> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().reader).poll_read(cx, buf)
    }
}

#[cfg(unix)]
impl<R: Unpin, W: AsyncWrite + Unpin> AsyncWrite for PeekedStream<R, W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.get_mut().writer).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().writer).poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().writer).poll_shutdown(cx)
    }
}

// ─── Accept loops ────────────────────────────────────────────────────────────

impl UnixSocketServer {
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

        crate::env_config::create_domain_socket_symlink(&self.socket_path);

        let btsp_active = btsp::btsp_required();

        self.is_running.store(true, std::sync::atomic::Ordering::Release);
        self.is_ready.store(true, std::sync::atomic::Ordering::Release);
        self.ready_notify.notify_waiters();

        info!("✅ Unix socket JSON-RPC server listening: {}", self.socket_path.display());
        info!("   Protocol: JSON-RPC 2.0 (pure Rust)");
        if btsp_active {
            info!("   Security: BTSP with first-line auto-detect (FAMILY_ID set)");
            info!(
                "   Peek: '{{' + protocol:btsp → NDJSON BTSP | '{{' → plain JSON-RPC | other → binary BTSP"
            );
        } else {
            info!("   Security: Development mode (no BTSP handshake)");
        }
        info!("   APIs: 14 (3 P2P + 4 registry + 4 graph + 3 coordination)");
        info!("   Status: READY ✅ (atomic flag set)");

        while self.is_running() {
            match tokio::time::timeout(songbird_types::defaults::timeouts::DEFAULT_ACCEPT_POLL_INTERVAL, listener.accept()).await {
                Ok(Ok((stream, _addr))) => {
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        let result = if btsp_active {
                            server.handle_connection_with_peek(stream).await
                        } else {
                            server.handle_connection(stream).await
                        };
                        if let Err(e) = result {
                            error!("Connection handler error: {}", e);
                        }
                    });
                }
                Ok(Err(e)) => {
                    error!("Failed to accept connection: {}", e);
                }
                Err(_) => {}
            }
        }

        info!("Unix socket server stopped gracefully");
        Ok(())
    }

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

        let bind_addr = format!("{}:0", songbird_types::constants::DEVELOPMENT_BIND_ADDRESS);
        let listener = TcpListener::bind(&bind_addr)
            .await
            .context("Failed to bind TCP localhost for IPC fallback")?;

        let bound_addr = listener.local_addr()?;
        info!("✅ TCP IPC listening on {}", bound_addr);

        self.write_tcp_discovery_file(bound_addr.port())?;

        self.is_running.store(true, std::sync::atomic::Ordering::Release);
        self.is_ready.store(true, std::sync::atomic::Ordering::Release);
        self.ready_notify.notify_waiters();

        info!("   APIs: 14 (3 P2P + 4 registry + 4 graph + 3 coordination)");
        info!("   Status: READY ✅ (isomorphic TCP fallback active)");

        while self.is_running() {
            match tokio::time::timeout(songbird_types::defaults::timeouts::DEFAULT_ACCEPT_POLL_INTERVAL, listener.accept()).await {
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

        let (reader, writer) = stream.into_split();
        let reader = BufReader::new(reader);
        self.handle_ndjson_session(reader, writer).await
    }

    fn write_tcp_discovery_file(&self, port: u16) -> Result<()> {
        let port_file = if let Ok(runtime_dir) = songbird_process_env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir).join("songbird-ipc-port")
        } else if let Ok(home) = songbird_process_env::var("HOME") {
            let share_dir = PathBuf::from(home).join(".local/share");
            if let Err(e) = std::fs::create_dir_all(&share_dir) {
                warn!("Failed to create {:?}: {}", share_dir, e);
                songbird_types::defaults::paths::ipc_port_file_path()
            } else {
                share_dir.join("songbird-ipc-port")
            }
        } else {
            songbird_types::defaults::paths::ipc_port_file_path()
        };

        let content = format!("tcp:{}:{port}", songbird_types::constants::DEVELOPMENT_BIND_ADDRESS);
        std::fs::write(&port_file, content)
            .context(format!("Failed to write TCP discovery file: {}", port_file.display()))?;

        info!("   Discovery file: {}", port_file.display());
        Ok(())
    }

    // ─── Per-connection protocol detection ───────────────────────────────────

    /// Per-connection protocol auto-detection via first-byte peek + first-line
    /// discrimination.
    ///
    /// Peeks the first byte using `BufReader::fill_buf` (no data consumed):
    ///
    /// - `0x7B` (`{`) → reads the full first line, then:
    ///   - Contains `"protocol":"btsp"` → BTSP JSON-line (NDJSON) handshake
    ///     followed by persistent NDJSON JSON-RPC (primalSpring wire format)
    ///   - Otherwise → plain NDJSON-RPC (biomeOS, local tooling)
    /// - Any other byte → BTSP length-prefixed binary handshake
    ///
    /// For the binary BTSP path, the peeked byte is preserved in the
    /// `BufReader` and passed through via [`PeekedStream`].
    #[cfg(unix)]
    async fn handle_connection_with_peek(&self, stream: UnixStream) -> Result<()> {
        const PEEK_TIMEOUT: Duration = songbird_types::defaults::timeouts::DEFAULT_PEEK_TIMEOUT;

        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        let first_byte = match tokio::time::timeout(PEEK_TIMEOUT, reader.fill_buf()).await {
            Ok(Ok(buf)) if !buf.is_empty() => buf[0],
            Ok(Ok(_)) => {
                debug!("UDS peek: client disconnected before sending data");
                return Ok(());
            }
            Ok(Err(e)) => {
                return Err(e).context("UDS peek I/O error");
            }
            Err(_) => {
                debug!("UDS peek: timeout after {}s — dropping connection", PEEK_TIMEOUT.as_secs());
                return Ok(());
            }
        };

        if first_byte == b'{' {
            let mut first_line = String::new();
            reader.read_line(&mut first_line).await.context("UDS: failed to read first line")?;

            if first_line.trim().is_empty() {
                debug!("UDS: empty first line after peek");
                return Ok(());
            }

            if btsp::is_btsp_client_hello(&first_line) {
                debug!("UDS peek: BTSP JSON-line ClientHello detected — NDJSON handshake");
                let session = match btsp::perform_server_handshake_ndjson(
                    &first_line,
                    &mut reader,
                    &mut write_half,
                    &self.security_client,
                )
                .await
                {
                    Ok(s) => s,
                    Err(e) => {
                        let err_frame =
                            serde_json::json!({"error":"handshake_failed","reason":e.to_string()});
                        let mut bytes = serde_json::to_vec(&err_frame).unwrap_or_default();
                        bytes.push(b'\n');
                        let _ = write_half.write_all(&bytes).await;
                        let _ = write_half.flush().await;
                        return Err(e).context("BTSP NDJSON handshake failed");
                    }
                };

                info!(
                    "BTSP NDJSON session {} authenticated (cipher: {})",
                    session.session_id, session.cipher
                );
                self.handle_ndjson_session(reader, write_half).await
            } else {
                debug!("UDS peek: JSON-RPC detected — plain NDJSON session");
                self.handle_ndjson_first_line_then_session(first_line, reader, write_half).await
            }
        } else {
            debug!("UDS peek: binary protocol detected (0x{first_byte:02X}) — BTSP handshake");
            let stream = PeekedStream {
                reader,
                writer: write_half,
            };
            self.handle_btsp_on_stream(stream).await
        }
    }

    // ─── BTSP handlers (generic over AsyncRead + AsyncWrite) ─────────────────

    /// Run the full BTSP lifecycle (handshake → persistent framed JSON-RPC) on
    /// any bidirectional async stream. Works with both raw `UnixStream` and the
    /// `PeekedStream` adapter used after first-byte auto-detection.
    async fn handle_btsp_on_stream<S>(&self, mut stream: S) -> Result<()>
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin + Send + 'static,
    {
        debug!("New IPC connection (BTSP mode)");

        let session = btsp::perform_server_handshake(&mut stream, &self.security_client)
            .await
            .context("BTSP handshake failed")?;

        info!("BTSP session {} authenticated (cipher: {})", session.session_id, session.cipher);

        self.handle_btsp_frames(stream, &session).await
    }

    /// Persistent BTSP frame loop: reads length-prefixed JSON-RPC frames until
    /// the client disconnects or a read timeout expires.
    ///
    /// Intercepts `btsp.negotiate` requests to perform Phase 3 cipher upgrade.
    /// On successful negotiation with a real cipher, transitions to encrypted
    /// framing via [`Self::handle_encrypted_session`].
    async fn handle_btsp_frames<S>(
        &self,
        mut stream: S,
        session: &btsp::BtspSession,
    ) -> Result<()>
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin + Send + 'static,
    {
        let mut len_buf = [0u8; 4];
        loop {
            match tokio::time::timeout(songbird_types::defaults::timeouts::DEFAULT_IDLE_TIMEOUT, stream.read_exact(&mut len_buf))
                .await
            {
                Ok(Ok(_)) => {}
                Ok(Err(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    debug!("BTSP client disconnected (session {})", session.session_id);
                    break;
                }
                Ok(Err(e)) => {
                    return Err(e).context("BTSP frame length read error");
                }
                Err(_) => {
                    debug!("BTSP idle timeout (session {})", session.session_id);
                    break;
                }
            }

            let frame_len = u32::from_be_bytes(len_buf) as usize;
            if frame_len > 16 * 1024 * 1024 {
                bail!("BTSP frame exceeds 16 MiB limit ({frame_len})");
            }

            let mut payload = vec![0u8; frame_len];
            stream.read_exact(&mut payload).await.context("BTSP payload read error")?;

            let request = match serde_json::from_slice::<JsonRpcRequest>(&payload) {
                Ok(req) => req,
                Err(e) => {
                    let resp = JsonRpcResponse::error(
                        JsonRpcError::parse_error(format!("Failed to parse JSON-RPC request: {e}")),
                        serde_json::Value::Null,
                    );
                    Self::write_btsp_response(&mut stream, &resp).await?;
                    continue;
                }
            };

            if request.method == "btsp.negotiate" {
                let id = request.id.clone().unwrap_or(serde_json::Value::Null);
                let params = request.params.unwrap_or(serde_json::Value::Null);
                let (result, keys) =
                    btsp_phase3::handle_negotiate(&params, &self.security_client).await;

                let resp = JsonRpcResponse::success(
                    serde_json::to_value(&result).unwrap_or_default(),
                    id,
                );
                Self::write_btsp_response(&mut stream, &resp).await?;

                if let Some(session_keys) = keys {
                    debug!("BTSP Phase 3: switching binary-framed session to encrypted framing");
                    let (reader, writer) = tokio::io::split(stream);
                    return self
                        .handle_encrypted_session(reader, writer, session_keys)
                        .await;
                }
                continue;
            }

            let is_notification = request.id.is_none();
            debug!(
                "BTSP JSON-RPC: {} (notification={}, session={})",
                request.method, is_notification, session.session_id
            );
            let response = self.handle_jsonrpc_request(request).await;

            if !is_notification {
                Self::write_btsp_response(&mut stream, &response).await?;
            }
        }

        Ok(())
    }

    async fn write_btsp_response<W: AsyncWriteExt + Unpin>(
        writer: &mut W,
        response: &JsonRpcResponse,
    ) -> Result<()> {
        let resp_bytes = serde_json::to_vec(response)?;
        let resp_len = u32::try_from(resp_bytes.len()).context("response exceeds u32::MAX")?;
        writer.write_all(&resp_len.to_be_bytes()).await?;
        writer.write_all(&resp_bytes).await?;
        writer.flush().await?;
        Ok(())
    }

    // ─── NDJSON handler (shared by UDS plain + TCP + peek-bypass) ────────────

    /// Handle a pre-consumed first line as JSON-RPC, then continue with a
    /// persistent NDJSON session. Used when the first-line discrimination read
    /// the line to check for BTSP but found normal JSON-RPC instead.
    async fn handle_ndjson_first_line_then_session<R, W>(
        &self,
        first_line: String,
        reader: R,
        mut writer: W,
    ) -> Result<()>
    where
        R: AsyncBufRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        if !first_line.trim().is_empty() {
            match serde_json::from_str::<JsonRpcRequest>(&first_line) {
                Ok(request) => {
                    let is_notification = request.id.is_none();
                    debug!(
                        "JSON-RPC request: {} (notification={})",
                        request.method, is_notification
                    );
                    let response = self.handle_jsonrpc_request(request).await;
                    if !is_notification {
                        let mut payload = serde_json::to_vec(&response)?;
                        payload.push(b'\n');
                        writer.write_all(&Bytes::from(payload)).await?;
                        writer.flush().await?;
                    }
                }
                Err(e) => {
                    let resp = JsonRpcResponse::error(
                        JsonRpcError::parse_error(format!("Failed to parse JSON-RPC request: {e}")),
                        serde_json::Value::Null,
                    );
                    let mut payload = serde_json::to_vec(&resp)?;
                    payload.push(b'\n');
                    writer.write_all(&Bytes::from(payload)).await?;
                    writer.flush().await?;
                }
            }
        }

        self.handle_ndjson_session(reader, writer).await
    }

    /// Persistent newline-delimited JSON-RPC session: reads requests in a loop
    /// until the client disconnects. Generic over any buffered reader + writer pair.
    ///
    /// If the client sends a `btsp.negotiate` request, the handler processes
    /// the Phase 3 negotiation, sends the NDJSON response, and (if a real
    /// cipher was negotiated) switches to the encrypted frame loop for all
    /// subsequent traffic.
    async fn handle_ndjson_session<R, W>(&self, mut reader: R, mut writer: W) -> Result<()>
    where
        R: AsyncBufRead + AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("Client disconnected");
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
                                JsonRpcError::parse_error(format!("Failed to parse JSON-RPC request: {e}")),
                                serde_json::Value::Null,
                            );
                            let mut payload = serde_json::to_vec(&resp)?;
                            payload.push(b'\n');
                            writer.write_all(&Bytes::from(payload)).await?;
                            writer.flush().await?;
                            continue;
                        }
                    };

                    if request.method == "btsp.negotiate" {
                        let id = request.id.clone().unwrap_or(serde_json::Value::Null);
                        let params = request.params.unwrap_or(serde_json::Value::Null);
                        let (result, keys) =
                            btsp_phase3::handle_negotiate(&params, &self.security_client).await;

                        let resp = JsonRpcResponse::success(
                            serde_json::to_value(&result).unwrap_or_default(),
                            id,
                        );
                        let mut payload = serde_json::to_vec(&resp)?;
                        payload.push(b'\n');
                        writer.write_all(&Bytes::from(payload)).await?;
                        writer.flush().await?;

                        if let Some(session_keys) = keys {
                            debug!("BTSP Phase 3: switching to encrypted framing");
                            return self
                                .handle_encrypted_session(reader, writer, session_keys)
                                .await;
                        }
                        continue;
                    }

                    let is_notification = request.id.is_none();
                    debug!(
                        "JSON-RPC request: {} (notification={})",
                        request.method, is_notification
                    );
                    let response = self.handle_jsonrpc_request(request).await;

                    if !is_notification {
                        let mut payload = serde_json::to_vec(&response)?;
                        payload.push(b'\n');
                        writer.write_all(&Bytes::from(payload)).await?;
                        writer.flush().await?;
                    }
                }
                Err(e) => {
                    error!("Failed to read from socket: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Persistent encrypted JSON-RPC session (BTSP Phase 3).
    ///
    /// After `btsp.negotiate` upgrades the connection, all subsequent traffic
    /// uses length-prefixed encrypted frames:
    /// `[4B len (BE u32)][12B nonce][ciphertext + Poly1305 tag]`
    async fn handle_encrypted_session<R, W>(
        &self,
        mut reader: R,
        mut writer: W,
        keys: btsp_phase3::SessionKeys,
    ) -> Result<()>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        info!("BTSP Phase 3: encrypted session active");
        loop {
            let frame = match btsp_phase3::read_encrypted_frame(&mut reader).await {
                Ok(f) => f,
                Err(e) => {
                    let msg = format!("{e:#}");
                    if msg.contains("UnexpectedEof") || msg.contains("failed to read frame length")
                    {
                        debug!("BTSP Phase 3: client disconnected");
                        break;
                    }
                    return Err(e).context("BTSP Phase 3: frame read error");
                }
            };

            let plaintext = keys.decrypt(&frame).context("BTSP Phase 3: decryption failed")?;

            let request = match serde_json::from_slice::<JsonRpcRequest>(&plaintext) {
                Ok(req) => req,
                Err(e) => {
                    let resp = JsonRpcResponse::error(
                        JsonRpcError::parse_error(format!("Failed to parse JSON-RPC request: {e}")),
                        serde_json::Value::Null,
                    );
                    let resp_bytes = serde_json::to_vec(&resp)?;
                    let encrypted = keys.encrypt(&resp_bytes)?;
                    btsp_phase3::write_encrypted_frame(&mut writer, &encrypted).await?;
                    continue;
                }
            };

            let is_notification = request.id.is_none();
            debug!("BTSP Phase 3 JSON-RPC: {} (notification={})", request.method, is_notification);
            let response = self.handle_jsonrpc_request(request).await;

            if !is_notification {
                let resp_bytes = serde_json::to_vec(&response)?;
                let encrypted = keys.encrypt(&resp_bytes)?;
                btsp_phase3::write_encrypted_frame(&mut writer, &encrypted).await?;
            }
        }

        Ok(())
    }

    /// Handle a single UDS client connection with plain JSON-RPC 2.0 (no BTSP).
    #[cfg(unix)]
    pub(crate) async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        debug!("New IPC connection (development mode)");
        let (read_half, write_half) = stream.into_split();
        let reader = BufReader::new(read_half);
        self.handle_ndjson_session(reader, write_half).await
    }
}

#[cfg(test)]
#[path = "connection_tests.rs"]
mod tests;

