// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Smart port binding with automatic fallback.
//!
//! Uses [`SovereignBinder`] for wildcard addresses and incremental port
//! fallback when the requested port is already in use.

use anyhow::Result;
use std::net::SocketAddr;
use tracing::{info, warn};

/// Smart port binding with automatic fallback using Sovereign Socket.
///
/// When the bind address is a wildcard (`0.0.0.0` or `[::]`), uses the
/// `SovereignBinder` multi-strategy approach (IPv6 dual-stack → IPv4 → localhost).
///
/// When a specific IP is requested (e.g. `127.0.0.1`), binds directly to that
/// address — this is the secure-by-default path triggered by `--bind 127.0.0.1`.
pub(super) async fn bind_with_fallback(
    addr: &SocketAddr,
) -> Result<(tokio::net::TcpListener, SocketAddr)> {
    let port = addr.port();
    let ip = addr.ip();

    let is_wildcard = ip.is_unspecified();

    if is_wildcard {
        use crate::network::SovereignBinder;

        if port == 0 {
            info!("🦅 Ephemeral port requested (port 0) — OS will assign");
            let (listener, actual_addr) = SovereignBinder::bind_sovereign(0).await?;
            info!("✅ Ephemeral bind successful: {}", actual_addr);
            return Ok((listener, actual_addr));
        }

        info!("🦅 Using sovereign socket binding for port {} (wildcard)", port);

        match SovereignBinder::bind_sovereign(port).await {
            Ok((listener, actual_addr)) => {
                info!("✅ Sovereign bind successful: {}", actual_addr);
                return Ok((listener, actual_addr));
            }
            Err(e) => {
                warn!("Sovereign bind to port {} failed: {}", port, e);
                warn!("Attempting fallback with incremental ports...");
            }
        }

        let max_attempts = 10;
        for attempt in 1..=max_attempts {
            let try_port = port + attempt;

            match SovereignBinder::bind_sovereign(try_port).await {
                Ok((listener, actual_addr)) => {
                    info!("✅ Sovereign bind successful on fallback port: {}", actual_addr);
                    return Ok((listener, actual_addr));
                }
                Err(_) if attempt < max_attempts => {
                    tracing::debug!("Port {} busy, trying next...", try_port);
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to bind after {max_attempts} attempts. Last error: {e}. Tried ports {port}-{try_port}"
                    ));
                }
            }
        }

        return Err(anyhow::anyhow!(
            "Port binding loop exhausted {max_attempts} attempts without returning"
        ));
    }

    // Specific IP requested — bind directly (secure-by-default path)
    info!("🔒 Binding HTTP server to {} (specific address)", addr);

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            let actual_addr = listener.local_addr()?;
            info!("✅ HTTP server bound to {}", actual_addr);
            Ok((listener, actual_addr))
        }
        Err(e) if port > 0 => {
            warn!("Bind to {} failed: {} — trying incremental ports", addr, e);
            let max_attempts = 10;
            for attempt in 1..=max_attempts {
                let try_port = port + attempt;
                let try_addr = SocketAddr::new(ip, try_port);
                match tokio::net::TcpListener::bind(try_addr).await {
                    Ok(listener) => {
                        let actual_addr = listener.local_addr()?;
                        info!("✅ HTTP server bound to fallback: {}", actual_addr);
                        return Ok((listener, actual_addr));
                    }
                    Err(_) if attempt < max_attempts => continue,
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "Failed to bind to {ip} after {max_attempts} attempts. Last error: {e}"
                        ));
                    }
                }
            }
            Err(anyhow::anyhow!(
                "Port binding loop exhausted {max_attempts} attempts without returning"
            ))
        }
        Err(e) => Err(anyhow::anyhow!("Failed to bind to {addr}: {e}")),
    }
}
