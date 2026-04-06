// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! STUN server & client JSON-RPC handler
//!
//! Provides JSON-RPC methods for NAT traversal via STUN (RFC 5389).
//!
//! **Server Methods**:
//! - `stun.serve` - Start STUN server
//! - `stun.stop` - Stop STUN server
//! - `stun.status` - Get server status
//!
//! **Client Methods** (NAT Traversal):
//! - `stun.get_public_address` - Discover public IP/port via external STUN servers
//! - `stun.bind` - Bind local port and discover NAT mapping
//!
//! Submodules: `config` (defaults/env), `server` (embedded server lifecycle), `client` (STUN client ops).

mod client;
mod config;
mod server;

use std::sync::Arc;
use tokio::sync::RwLock;

/// STUN server handler for JSON-RPC integration
///
/// Manages the lifecycle of the integrated STUN server and provides
/// status information via JSON-RPC methods.
#[derive(Debug)]
pub struct StunHandler {
    /// Currently running server instance
    server_handle: Arc<RwLock<Option<server::ServerInstance>>>,
}

impl StunHandler {
    /// Create new STUN handler
    #[must_use]
    pub fn new() -> Self {
        Self {
            server_handle: Arc::new(RwLock::new(None)),
        }
    }
}

impl Default for StunHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn stun_handler_default_matches_new() {
        let a = StunHandler::new();
        let b = StunHandler::default();
        assert_eq!(a.handle_status(json!({})).await.unwrap()["running"], false);
        assert_eq!(b.handle_status(json!({})).await.unwrap()["running"], false);
    }

    #[tokio::test]
    async fn get_public_address_empty_servers_errors() {
        let handler = StunHandler::new();
        let err = handler
            .handle_get_public_address(json!({ "servers": [] }))
            .await
            .expect_err("empty servers");
        assert!(err.contains("No STUN servers"));
    }

    #[tokio::test]
    async fn get_public_address_invalid_servers_parameter_errors() {
        let handler = StunHandler::new();
        let err = handler
            .handle_get_public_address(json!({ "servers": "not-an-array" }))
            .await
            .expect_err("invalid servers");
        assert!(err.contains("Invalid 'servers'"));
    }

    #[tokio::test]
    async fn detect_nat_type_too_few_servers_errors() {
        let handler = StunHandler::new();
        let err = handler
            .handle_detect_nat_type(json!({
                "servers": ["stun.example.com:3478"]
            }))
            .await
            .expect_err("need 2 servers");
        assert!(err.contains("at least 2"));
    }

    #[tokio::test]
    async fn detect_nat_type_invalid_servers_json_errors() {
        let handler = StunHandler::new();
        let err = handler
            .handle_detect_nat_type(json!({ "servers": 12345 }))
            .await
            .expect_err("bad json");
        assert!(err.contains("Invalid 'servers'"));
    }

    #[tokio::test]
    async fn test_handler_creation() {
        let handler = StunHandler::new();

        let status = handler.handle_status(json!({})).await.unwrap();
        assert_eq!(status["running"], false);
    }

    #[tokio::test]
    async fn test_status_when_not_running() {
        let handler = StunHandler::new();
        let result = handler.handle_status(json!({})).await.unwrap();

        assert_eq!(result["running"], false);
        assert!(result["comment"].as_str().unwrap().contains("not running"));
    }

    #[tokio::test]
    async fn test_stop_when_not_running() {
        let handler = StunHandler::new();
        let result = handler.handle_stop(json!({})).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not running"));
    }

    #[tokio::test]
    async fn test_serve_with_default_address() {
        let handler = StunHandler::new();

        let result = handler.handle_serve(json!({})).await.unwrap();

        assert_eq!(result["status"], "started");
        assert!(result["bind_addr"].as_str().unwrap().contains("3478"));

        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn test_serve_with_custom_address() {
        let handler = StunHandler::new();

        let result = handler
            .handle_serve(json!({
                "bind_addr": "127.0.0.1:0"
            }))
            .await
            .unwrap();

        assert_eq!(result["status"], "started");

        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn test_serve_twice_fails() {
        let handler = StunHandler::new();

        let _ = handler
            .handle_serve(json!({
                "bind_addr": "127.0.0.1:0"
            }))
            .await
            .unwrap();

        let result = handler
            .handle_serve(json!({
                "bind_addr": "127.0.0.1:0"
            }))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already running"));

        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn test_status_when_running() {
        let handler = StunHandler::new();

        let _ = handler
            .handle_serve(json!({
                "bind_addr": "127.0.0.1:0"
            }))
            .await
            .unwrap();

        let status = handler.handle_status(json!({})).await.unwrap();

        assert_eq!(status["running"], true);
        assert!(status["bind_addr"].is_string());
        assert!(status["uptime_seconds"].is_number());

        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn test_stop_after_start() {
        let handler = StunHandler::new();

        let _ = handler
            .handle_serve(json!({
                "bind_addr": "127.0.0.1:0"
            }))
            .await
            .unwrap();

        let result = handler.handle_stop(json!({})).await.unwrap();

        assert_eq!(result["status"], "stopped");
        assert!(result["uptime_seconds"].is_number());
        assert!(result["bind_addr"].is_string());

        let status = handler.handle_status(json!({})).await.unwrap();
        assert_eq!(status["running"], false);
    }

    #[tokio::test]
    async fn test_invalid_bind_address() {
        let handler = StunHandler::new();

        let result = handler
            .handle_serve(json!({
                "bind_addr": "invalid_address"
            }))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid bind address"));
    }
}
