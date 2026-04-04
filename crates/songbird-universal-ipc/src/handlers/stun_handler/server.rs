// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Embedded STUN server lifecycle for the JSON-RPC handler (`stun.serve` / `stun.stop` / `stun.status`).
//!
//! Owns the background task handle and bind metadata for the in-process [`songbird_stun::StunServer`].

use serde_json::{Value, json};
use songbird_stun::StunServer;
use std::net::SocketAddr;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use super::StunHandler;

#[derive(Debug)]
pub(super) struct ServerInstance {
    /// Tokio task handle for the running server
    pub(super) handle: JoinHandle<()>,

    /// Bind address the server is listening on
    pub(super) bind_addr: SocketAddr,

    /// Server start time
    pub(super) start_time: std::time::Instant,
}

impl StunHandler {
    /// Handle `stun.serve` method - Start STUN server
    pub async fn handle_serve(&self, params: Value) -> Result<Value, String> {
        {
            let instance = self.server_handle.read().await;
            if instance.is_some() {
                return Err("STUN server is already running (use stun.stop first)".to_string());
            }
        }

        let bind_addr_str =
            params.get("bind_addr").and_then(|v| v.as_str()).unwrap_or("0.0.0.0:3478");

        let bind_addr: SocketAddr = bind_addr_str
            .parse()
            .map_err(|e| format!("Invalid bind address '{bind_addr_str}': {e}"))?;

        info!("🌐 Starting STUN server on {}", bind_addr);

        let server = StunServer::new(bind_addr);

        let handle = tokio::spawn(async move {
            match server.run().await {
                Ok(()) => {
                    info!("✅ STUN server shut down gracefully");
                }
                Err(e) => {
                    warn!("⚠️  STUN server error: {}", e);
                }
            }
        });

        {
            let mut instance = self.server_handle.write().await;
            *instance = Some(ServerInstance {
                handle,
                bind_addr,
                start_time: std::time::Instant::now(),
            });
        }

        debug!("✅ STUN server started successfully");

        Ok(json!({
            "status": "started",
            "bind_addr": bind_addr.to_string(),
            "comment": "STUN server running in background (use stun.stop to stop)"
        }))
    }

    /// Handle `stun.stop` method - Stop STUN server
    pub async fn handle_stop(&self, _params: Value) -> Result<Value, String> {
        let mut instance_guard = self.server_handle.write().await;

        if let Some(instance) = instance_guard.take() {
            let uptime = instance.start_time.elapsed().as_secs();
            let bind_addr = instance.bind_addr.to_string();

            info!("🛑 Stopping STUN server (uptime: {}s)", uptime);

            instance.handle.abort();

            Ok(json!({
                "status": "stopped",
                "uptime_seconds": uptime,
                "bind_addr": bind_addr
            }))
        } else {
            Err("STUN server is not running".to_string())
        }
    }

    /// Handle `stun.status` method - Get server status
    pub async fn handle_status(&self, _params: Value) -> Result<Value, String> {
        let instance = self.server_handle.read().await;

        instance.as_ref().map_or_else(
            || {
                Ok(json!({
                    "running": false,
                    "comment": "STUN server is not running (use stun.serve to start)"
                }))
            },
            |instance| {
                let uptime = instance.start_time.elapsed().as_secs();

                Ok(json!({
                    "running": true,
                    "bind_addr": instance.bind_addr.to_string(),
                    "uptime_seconds": uptime
                }))
            },
        )
    }
}
