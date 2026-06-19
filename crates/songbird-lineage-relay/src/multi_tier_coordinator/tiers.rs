// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Individual tier attempt implementations for the H2-16 fallback chain.

#![forbid(unsafe_code)]

use crate::error::{LineageRelayError, Result};
use crate::types::NodeId;
use songbird_stun::TurnClient;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tracing::{debug, info};

use super::MultiTierCoordinator;

/// Default local origin port that `cloudflared` will proxy to.
pub(super) const DEFAULT_TUNNEL_ORIGIN_PORT: u16 = 7844;

/// Maximum time to wait for `cloudflared` to emit its quick-tunnel URL.
const CLOUDFLARED_URL_TIMEOUT: Duration = Duration::from_secs(30);

impl MultiTierCoordinator {
    /// Attempt a direct UDP punch to the peer.
    pub(super) async fn try_direct_punch(&self, peer_addr: SocketAddr) -> Result<Duration> {
        let start = std::time::Instant::now();

        let bind_addr = if peer_addr.is_ipv4() {
            songbird_types::constants::EPHEMERAL_BIND_ADDR
        } else {
            "[::]:0"
        };

        let socket = tokio::net::UdpSocket::bind(bind_addr)
            .await
            .map_err(|e| LineageRelayError::NetworkError(format!("bind failed: {e}")))?;

        let probe = b"SONGBIRD_PUNCH_PROBE";
        socket
            .send_to(probe, peer_addr)
            .await
            .map_err(|e| LineageRelayError::DirectConnectionFailed(format!("send failed: {e}")))?;

        let mut buf = [0u8; 64];
        match tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buf)).await {
            Ok(Ok(_)) => Ok(start.elapsed()),
            Ok(Err(e)) => {
                Err(LineageRelayError::DirectConnectionFailed(format!("recv failed: {e}")))
            }
            Err(_) => Err(LineageRelayError::DirectConnectionFailed(String::from(
                "punch probe timed out (2 s)",
            ))),
        }
    }

    /// Attempt STUN-assisted NAT discovery.
    pub(super) async fn try_stun_punch(&self, _peer: &NodeId) -> Result<SocketAddr> {
        self.discover_public_address().await
    }

    /// Attempt a TURN allocation via the configured [`TurnClient`].
    pub(super) async fn try_turn_allocation(
        &self,
        turn_client: &TurnClient,
    ) -> Result<songbird_stun::TurnAllocation> {
        let socket = tokio::net::UdpSocket::bind(songbird_types::constants::EPHEMERAL_BIND_ADDR)
            .await
            .map_err(|e| LineageRelayError::NetworkError(format!("TURN bind: {e}")))?;

        turn_client
            .allocate(&socket)
            .await
            .map_err(|e| LineageRelayError::NetworkError(format!("TURN allocate: {e}")))
    }

    /// Attempt an emergency `cloudflared` tunnel.
    ///
    /// Spawns `cloudflared tunnel --url localhost:<port>` as a quick-tunnel,
    /// parses the assigned `*.trycloudflare.com` URL from stderr, and returns
    /// it. The child process lives as long as the returned endpoint is in use
    /// (caller is responsible for keeping the [`CloudflaredTunnel`] handle).
    pub(super) async fn try_emergency_tunnel(&self) -> Result<String> {
        let tunnel = CloudflaredTunnel::spawn(DEFAULT_TUNNEL_ORIGIN_PORT).await?;
        Ok(tunnel.endpoint().to_string())
    }
}

/// A running `cloudflared tunnel` child process.
///
/// Spawns `cloudflared tunnel --url localhost:<port>` as a quick-tunnel,
/// monitors stderr for the generated `*.trycloudflare.com` endpoint, and
/// provides a handle to the running process. Dropping the handle kills the
/// child.
pub struct CloudflaredTunnel {
    endpoint: String,
    child: tokio::process::Child,
}

impl CloudflaredTunnel {
    /// Spawn a `cloudflared` quick-tunnel proxying to `localhost:<port>`.
    ///
    /// Blocks (up to 30 s) until the tunnel URL is emitted on stderr.
    ///
    /// # Errors
    ///
    /// Returns an error if `cloudflared` is not found, exits prematurely, or
    /// the URL cannot be parsed within the timeout.
    pub async fn spawn(origin_port: u16) -> Result<Self> {
        let mut child = tokio::process::Command::new("cloudflared")
            .args(["tunnel", "--url", &format!("http://localhost:{origin_port}")])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                LineageRelayError::NoRelayAvailable(format!(
                    "cloudflared binary not found or failed to start: {e}"
                ))
            })?;

        let stderr = child.stderr.take().ok_or_else(|| {
            LineageRelayError::NoRelayAvailable(String::from("cannot capture cloudflared stderr"))
        })?;

        let endpoint = Self::parse_tunnel_url(stderr).await?;
        info!("cloudflared quick-tunnel active: {endpoint}");

        Ok(Self {
            endpoint,
            child,
        })
    }

    /// The public endpoint URL assigned by Cloudflare.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Gracefully shut down the tunnel by killing the child process.
    pub async fn shutdown(&mut self) {
        let _ = self.child.kill().await;
    }

    /// Read stderr lines until we find the `*.trycloudflare.com` URL.
    async fn parse_tunnel_url(stderr: tokio::process::ChildStderr) -> Result<String> {
        let reader = tokio::io::BufReader::new(stderr);
        let mut lines = reader.lines();

        let deadline = tokio::time::Instant::now() + CLOUDFLARED_URL_TIMEOUT;

        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(LineageRelayError::Timeout(String::from(
                    "cloudflared did not emit tunnel URL within 30 s",
                )));
            }

            let line = match tokio::time::timeout(remaining, lines.next_line()).await {
                Ok(Ok(Some(line))) => line,
                Ok(Ok(None)) => {
                    return Err(LineageRelayError::NoRelayAvailable(String::from(
                        "cloudflared exited before emitting tunnel URL",
                    )));
                }
                Ok(Err(e)) => {
                    return Err(LineageRelayError::NetworkError(format!(
                        "cloudflared stderr read error: {e}"
                    )));
                }
                Err(_) => {
                    return Err(LineageRelayError::Timeout(String::from(
                        "cloudflared did not emit tunnel URL within 30 s",
                    )));
                }
            };

            debug!("cloudflared: {line}");

            if let Some(url) = extract_trycloudflare_url(&line) {
                return Ok(url);
            }
        }
    }
}

impl Drop for CloudflaredTunnel {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

/// Extract a `https://*.trycloudflare.com` URL from a log line.
pub(super) fn extract_trycloudflare_url(line: &str) -> Option<String> {
    let marker = "https://";
    let start = line.find(marker)?;
    let rest = &line[start..];
    let end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
    let url = &rest[..end];
    if url.contains("trycloudflare.com") {
        Some(url.to_string())
    } else {
        None
    }
}
