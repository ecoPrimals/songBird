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

use anyhow::{Context, Result};
use std::path::PathBuf;
#[cfg(unix)]
use std::pin::Pin;
use std::sync::Arc;
#[cfg(unix)]
use std::task::{self, Poll};
#[cfg(unix)]
use std::time::Duration;
#[cfg(not(unix))]
use tokio::io::BufReader;
#[cfg(unix)]
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf};
use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info, warn};

use super::UnixSocketServer;
#[cfg(unix)]
use crate::ipc::btsp;

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

        // Unconditional unlink before bind (prevents EADDRINUSE after crash).
        // Ignoring errors: NotFound is expected on fresh start, PermissionDenied
        // will surface as a bind error with better context below.
        let _ = std::fs::remove_file(&self.socket_path);

        let listener = UnixListener::bind(&*self.socket_path)
            .context(format!("Failed to bind Unix socket: {}", self.socket_path.display()))?;

        crate::env_config::create_domain_socket_symlink(&self.socket_path);

        let btsp_active = btsp::btsp_required();

        self.is_running.store(true, std::sync::atomic::Ordering::Release);
        self.is_ready.store(true, std::sync::atomic::Ordering::Release);
        self.ready_notify.notify_waiters();

        crate::neural_announce::spawn_announce(&self.socket_path);
        crate::mesh_seed::spawn_mesh_seed(std::sync::Arc::clone(&self.handlers.mesh_handler));

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
            match tokio::time::timeout(
                songbird_types::defaults::timeouts::DEFAULT_ACCEPT_POLL_INTERVAL,
                listener.accept(),
            )
            .await
            {
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

        crate::mesh_seed::spawn_mesh_seed(std::sync::Arc::clone(&self.handlers.mesh_handler));

        info!("   APIs: 14 (3 P2P + 4 registry + 4 graph + 3 coordination)");
        info!("   Status: READY ✅ (isomorphic TCP fallback active)");

        while self.is_running() {
            match tokio::time::timeout(
                songbird_types::defaults::timeouts::DEFAULT_ACCEPT_POLL_INTERVAL,
                listener.accept(),
            )
            .await
            {
                Ok(Ok((stream, addr))) => {
                    debug!("📥 TCP IPC connection from {}", addr);
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_tcp_connection(stream, addr).await {
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

    async fn handle_tcp_connection(
        &self,
        stream: tokio::net::TcpStream,
        peer_addr: std::net::SocketAddr,
    ) -> Result<()> {
        debug!("📥 New TCP IPC connection from {peer_addr}");

        let caller = super::super::method_gate::CallerContext::from_tcp(peer_addr);
        let (reader, writer) = stream.into_split();
        let reader = BufReader::new(reader);
        self.handle_ndjson_session(reader, writer, &caller).await
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

        // Whitespace-tolerant protocol detection: skip leading ASCII whitespace
        // before classifying. Handles clients that send `\n{...` or `  {...`.
        let first_meaningful_byte = if first_byte.is_ascii_whitespace() {
            reader.consume(1);
            loop {
                match reader.fill_buf().await {
                    Ok(buf) if !buf.is_empty() => {
                        let b = buf[0];
                        if b.is_ascii_whitespace() {
                            reader.consume(1);
                            continue;
                        }
                        break b;
                    }
                    _ => {
                        debug!("UDS peek: only whitespace received — dropping connection");
                        return Ok(());
                    }
                }
            }
        } else {
            first_byte
        };

        if songbird_types::constants::ribocipher::is_signal_byte(first_meaningful_byte) {
            // riboCipher transport signal detected — consume signal + version byte, then route.
            use songbird_types::constants::ribocipher;
            let tier = ribocipher::tier_name(first_meaningful_byte);
            reader.consume(1); // consume signal byte

            let version_byte = match reader.fill_buf().await {
                Ok(buf) if !buf.is_empty() => {
                    let v = buf[0];
                    reader.consume(1);
                    v
                }
                _ => {
                    tracing::warn!("riboCipher {tier}: missing version byte — dropping");
                    return Ok(());
                }
            };

            if version_byte != ribocipher::VERSION_1 {
                tracing::warn!(
                    "riboCipher {tier}: unsupported version 0x{version_byte:02X} — dropping"
                );
                return Ok(());
            }

            debug!("riboCipher signal: tier={tier}, version={version_byte} — routing");

            let caller = super::super::method_gate::CallerContext::from_unix();
            match first_meaningful_byte {
                ribocipher::CLEAR => {
                    // Clear tier: standard ecosystem JSON-RPC follows after signal prefix
                    self.handle_ndjson_session(reader, write_half, &caller).await
                }
                ribocipher::MITO => {
                    // Mito tier: federation inter-gate — currently routes to encrypted session
                    // (future: mito-specific obfuscation layer)
                    tracing::info!("riboCipher mito: federation-tier connection accepted");
                    self.handle_ndjson_session(reader, write_half, &caller).await
                }
                ribocipher::NUCLEAR => {
                    // Nuclear tier: high-security — route to BTSP encrypted session
                    tracing::info!("riboCipher nuclear: high-security connection accepted");
                    let stream = PeekedStream {
                        reader,
                        writer: write_half,
                    };
                    self.handle_btsp_on_stream(stream, &caller).await
                }
                _ => {
                    tracing::error!(
                        tier = first_meaningful_byte,
                        "riboCipher: unknown tier byte 0x{:02X} — dropping connection",
                        first_meaningful_byte
                    );
                    Ok(())
                }
            }
        } else if first_meaningful_byte == b'{' {
            // Wave 112: ERROR on unsignalled connections (deprecation escalation)
            error!(
                "UDS JSON-RPC connection without riboCipher signal — legacy path (deprecated Wave 112, reject Wave 113)"
            );
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
                let caller = super::super::method_gate::CallerContext::from_unix();
                self.handle_ndjson_session(reader, write_half, &caller).await
            } else {
                debug!("UDS peek: JSON-RPC detected — plain NDJSON session");
                let caller = super::super::method_gate::CallerContext::from_unix();
                self.handle_ndjson_first_line_then_session(first_line, reader, write_half, &caller)
                    .await
            }
        } else {
            // Legacy: non-riboCipher, non-JSON first byte → binary BTSP
            // Wave 112: ERROR on unsignalled connections (deprecation escalation)
            error!(
                "UDS connection without riboCipher signal (0x{first_meaningful_byte:02X}) — legacy BTSP path (deprecated Wave 112, reject Wave 113)"
            );
            let stream = PeekedStream {
                reader,
                writer: write_half,
            };
            let caller = super::super::method_gate::CallerContext::from_unix();
            self.handle_btsp_on_stream(stream, &caller).await
        }
    }

    /// Handle a single UDS client connection with plain JSON-RPC 2.0 (no BTSP).
    #[cfg(unix)]
    pub(crate) async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        debug!("New IPC connection (development mode)");
        let caller = super::super::method_gate::CallerContext::from_unix();
        let (read_half, write_half) = stream.into_split();
        let reader = BufReader::new(read_half);
        self.handle_ndjson_session(reader, write_half, &caller).await
    }
}

#[cfg(test)]
#[path = "connection_tests.rs"]
mod tests;
