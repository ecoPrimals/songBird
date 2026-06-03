// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Cloudflared quick-tunnel orchestration (Tier 5 emergency fallback).
//!
//! Spawns `cloudflared tunnel --url localhost:<port>` as a quick-tunnel,
//! monitors stderr for the generated `*.trycloudflare.com` endpoint, and
//! provides a handle to the running process. Dropping the handle kills the child.

use crate::error::{LineageRelayError, Result};
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tracing::{debug, info};

/// Maximum time to wait for `cloudflared` to emit its quick-tunnel URL.
const CLOUDFLARED_URL_TIMEOUT: Duration = Duration::from_secs(30);

/// Default local origin port that `cloudflared` will proxy to.
pub const DEFAULT_TUNNEL_ORIGIN_PORT: u16 = 7844;

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
            LineageRelayError::NoRelayAvailable("cannot capture cloudflared stderr".to_string())
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
                return Err(LineageRelayError::Timeout(
                    "cloudflared did not emit tunnel URL within 30 s".to_string(),
                ));
            }

            let line = match tokio::time::timeout(remaining, lines.next_line()).await {
                Ok(Ok(Some(line))) => line,
                Ok(Ok(None)) => {
                    return Err(LineageRelayError::NoRelayAvailable(
                        "cloudflared exited before emitting tunnel URL".to_string(),
                    ));
                }
                Ok(Err(e)) => {
                    return Err(LineageRelayError::NetworkError(format!(
                        "cloudflared stderr read error: {e}"
                    )));
                }
                Err(_) => {
                    return Err(LineageRelayError::Timeout(
                        "cloudflared did not emit tunnel URL within 30 s".to_string(),
                    ));
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
#[must_use]
pub fn extract_trycloudflare_url(line: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_trycloudflare_url_parses_typical_log_line() {
        let line =
            "2026-05-20T12:00:00Z INF +---------------------------------------------------+\n";
        assert_eq!(extract_trycloudflare_url(line), None);

        let line = "2026-05-20T12:00:01Z INF | https://foo-bar-baz.trycloudflare.com |";
        assert_eq!(
            extract_trycloudflare_url(line),
            Some("https://foo-bar-baz.trycloudflare.com".to_string())
        );
    }

    #[test]
    fn extract_trycloudflare_url_ignores_non_tunnel_urls() {
        let line = "connecting to https://region1.argotunnel.com:7844";
        assert_eq!(extract_trycloudflare_url(line), None);
    }
}
