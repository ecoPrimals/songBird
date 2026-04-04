// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `tarpc://` URL parsing and [`super::TarpcClient`] construction.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tracing::debug;

use songbird_types::{SongbirdError, SongbirdResult};

use super::TarpcClient;

pub(super) fn new_client(endpoint: &str) -> SongbirdResult<TarpcClient> {
    debug!("Creating tarpc client for endpoint: {}", endpoint);

    let addr = TarpcClient::parse_endpoint(endpoint)?;

    Ok(TarpcClient {
        endpoint: endpoint.to_string(),
        addr,
        connection: Arc::new(RwLock::new(None)),
        timeout: Duration::from_secs(5),
    })
}

impl TarpcClient {
    /// Parse endpoint string to `SocketAddr` with limited hostname support (`localhost`, IPs, IPv6).
    pub(crate) fn parse_endpoint(endpoint: &str) -> SongbirdResult<SocketAddr> {
        let addr_str = endpoint.strip_prefix("tarpc://").ok_or_else(|| {
            SongbirdError::configuration(format!(
                "Invalid tarpc endpoint (expected tarpc://host:port): {endpoint}"
            ))
        })?;

        if let Ok(addr) = addr_str.parse::<SocketAddr>() {
            debug!("✅ Parsed tarpc endpoint as IP address: {}", addr);
            return Ok(addr);
        }

        let (host, port) = addr_str.rsplit_once(':').ok_or_else(|| {
            SongbirdError::configuration(format!(
                "Invalid tarpc endpoint (missing port): {addr_str}"
            ))
        })?;

        let port: u16 = port
            .parse()
            .map_err(|e| SongbirdError::configuration(format!("Invalid port '{port}': {e}")))?;

        let ip = match host {
            "localhost" | "localhost.localdomain" => {
                debug!("🔍 Resolved localhost to 127.0.0.1");
                std::net::Ipv4Addr::LOCALHOST
            }
            _ => {
                host.parse().map_err(|e| {
                    SongbirdError::configuration(format!(
                        "Invalid hostname or IP '{host}': {e}. tarpc requires IP addresses or 'localhost'."
                    ))
                })?
            }
        };

        let addr = SocketAddr::new(std::net::IpAddr::V4(ip), port);
        debug!("✅ Resolved tarpc endpoint: {} → {}", addr_str, addr);
        Ok(addr)
    }
}
