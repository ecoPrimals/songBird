// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Pure Rust IPC Server Infrastructure (Isomorphic - TRUE ecoBin v2.0)
//!
//! v3.22.0: Evolved from jsonrpsee to pure Rust implementation (`security provider` pattern)
//! v8.23.0: Added automatic TCP fallback (isomorphic adaptation)
//!
//! ## Design Principles
//!
//! 1. **Zero External RPC Libraries**: Pure `tokio::net` + JSON
//! 2. **Zero Hardcoding**: Socket path from env vars, automatic fallback
//! 3. **Modern Async**: tokio + async/await
//! 4. **Thread-Safe**: Arc + atomic readiness flags
//! 5. **Observable**: Structured logging
//! 6. **Graceful Shutdown**: Cleanup on drop
//! 7. **TRUE Isomorphism**: Try → Detect → Adapt → Succeed
//!
//! ## Platform Support
//!
//! - **Unix/Linux/macOS**: Unix domain sockets (optimal)
//! - **Android/SELinux**: TCP localhost fallback (automatic)
//! - **Windows**: TCP localhost (future)
//!
//! ## Automatic Adaptation
//!
//! Server detects platform constraints (`SELinux`, permissions) and automatically
//! falls back to TCP without requiring configuration. Same binary works everywhere!

mod connection;
mod handlers;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn};

use crate::app::connection_manager::ConnectionManager;
use crate::ipc::handlers::IpcHandlers;
use crate::ipc::registry::ServiceRegistry;
use songbird_discovery::anonymous::AnonymousDiscoveryListener;

/// Pure Rust Unix socket JSON-RPC server for inter-primal communication
pub struct UnixSocketServer {
    socket_path: PathBuf,
    handlers: Arc<IpcHandlers>,
    is_ready: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
    ready_notify: Arc<tokio::sync::Notify>,
}

impl UnixSocketServer {
    pub fn new(
        service_registry: Arc<ServiceRegistry>,
        discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,
        connection_manager: Arc<ConnectionManager>,
        security_client: Arc<songbird_http_client::SecurityRpcClient>,
    ) -> Self {
        let socket_path = Self::socket_path_from_env();
        let handlers = Arc::new(IpcHandlers::new(
            service_registry,
            discovery_listener,
            connection_manager,
            security_client,
        ));

        Self {
            socket_path,
            handlers,
            is_ready: Arc::new(AtomicBool::new(false)),
            is_running: Arc::new(AtomicBool::new(false)),
            ready_notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    #[must_use]
    pub fn socket_path_from_env() -> PathBuf {
        Self::socket_path_with_env(|key| songbird_process_env::var(key))
    }

    pub fn socket_path_with_env<F>(env_reader: F) -> PathBuf
    where
        F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
    {
        if let Ok(socket_path) = env_reader("SONGBIRD_ORCHESTRATOR_SOCKET")
            && !socket_path.is_empty()
        {
            info!("📍 Using SONGBIRD_ORCHESTRATOR_SOCKET: {}", socket_path);
            return PathBuf::from(socket_path);
        }

        if let Ok(socket_path) = env_reader("SONGBIRD_SOCKET")
            && !socket_path.is_empty()
        {
            info!("📍 Using SONGBIRD_SOCKET: {}", socket_path);
            return PathBuf::from(socket_path);
        }

        if let Ok(socket_path) = env_reader("BIOMEOS_SOCKET_PATH")
            && !socket_path.is_empty()
        {
            info!("📍 Using BIOMEOS_SOCKET_PATH: {}", socket_path);
            return PathBuf::from(socket_path);
        }

        let socket_path = crate::env_config::socket_path();
        info!("📍 Using socket path (TRUE PRIMAL self-knowledge): {}", socket_path.display());
        socket_path
    }

    #[must_use]
    pub fn get_family_id() -> String {
        Self::get_family_id_with_env(|key| songbird_process_env::var(key))
    }

    /// Same priority as [`Self::get_family_id`], with injectable env (tests, embedders).
    #[must_use]
    pub fn get_family_id_with_env<F>(env_reader: F) -> String
    where
        F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
    {
        env_reader("SONGBIRD_ORCHESTRATOR_FAMILY_ID")
            .or_else(|_| env_reader("SONGBIRD_ORCHESTRATOR_FAMILY"))
            .or_else(|_| env_reader("BIOMEOS_FAMILY_ID"))
            .or_else(|_| env_reader("SONGBIRD_FAMILY_ID"))
            .unwrap_or_else(|_| "default".to_string())
    }

    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    #[must_use]
    pub fn readiness_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_ready)
    }

    #[must_use]
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Acquire)
    }

    pub fn shutdown(&self) {
        info!("🛑 Shutdown requested for Unix socket server");
        self.is_running.store(false, Ordering::Release);
        self.is_ready.store(false, Ordering::Release);
    }

    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.is_ready.load(Ordering::Acquire)
    }

    pub async fn wait_ready(&self, timeout: std::time::Duration) -> bool {
        if self.is_ready() {
            return true;
        }

        match tokio::time::timeout(timeout, async {
            loop {
                self.ready_notify.notified().await;
                if self.is_ready() {
                    return;
                }
            }
        })
        .await
        {
            Ok(()) => true,
            Err(_) => self.is_ready(),
        }
    }

    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("🔌 Starting IPC server (isomorphic mode)...");
        info!("   Socket path: {}", self.socket_path.display());

        match self.clone().try_unix_server().await {
            Ok(()) => Ok(()),
            Err(e) => {
                if self.is_platform_constraint(&e) {
                    warn!("⚠️  Unix sockets unavailable: {}", e);
                    warn!("   Platform constraint detected (SELinux/permissions)");
                    warn!("   Falling back to TCP IPC...");

                    self.start_tcp_fallback().await
                } else {
                    Err(e).context("Failed to start IPC server")
                }
            }
        }
    }

    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping Unix socket JSON-RPC server...");

        self.is_ready.store(false, Ordering::Release);

        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).context("Failed to remove socket file")?;
            info!("🧹 Removed socket: {}", self.socket_path.display());
        }

        info!("✅ Unix socket server stopped");
        Ok(())
    }
}

impl Drop for UnixSocketServer {
    fn drop(&mut self) {
        if self.socket_path.exists()
            && let Err(e) = std::fs::remove_file(&self.socket_path)
        {
            warn!("⚠️  Failed to remove socket file on drop: {}", e);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::app::connection_manager::ConnectionManager;
    use crate::ipc::pure_rust_server::protocol::{JsonRpcRequest, JsonRpcResponse};
    use crate::ipc::registry::ServiceRegistry;

    fn mock_env(
        vars: HashMap<&str, &str>,
    ) -> impl Fn(&str) -> std::result::Result<String, std::env::VarError> {
        let owned: HashMap<String, String> =
            vars.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |key: &str| owned.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    }

    fn test_server() -> Arc<UnixSocketServer> {
        let registry = Arc::new(ServiceRegistry::new());
        let conn_mgr = Arc::new(ConnectionManager::new());
        let security = Arc::new(songbird_http_client::SecurityRpcClient::new_direct(
            "/tmp/songbird-test-l2.sock",
        ));
        Arc::new(UnixSocketServer::new(registry, None, conn_mgr, security))
    }

    fn jsonrpc_req(method: &str) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: None,
            id: Some(serde_json::json!(1)),
        }
    }

    fn assert_success(resp: &JsonRpcResponse, label: &str) {
        assert!(resp.error.is_none(), "{label}: expected success, got error: {:?}", resp.error);
        assert!(resp.result.is_some(), "{label}: expected result payload");
    }

    #[tokio::test]
    async fn wire_standard_l2_capabilities_list_on_socket() {
        let server = test_server();
        let resp = server.handle_jsonrpc_request(jsonrpc_req("capabilities.list")).await;
        assert_success(&resp, "capabilities.list");
        let result = resp.result.unwrap();
        assert_eq!(result["primal"].as_str().unwrap(), "songbird");
        assert!(result["version"].is_string());
        assert!(result["methods"].is_array());
    }

    #[tokio::test]
    async fn wire_standard_l2_capabilities_methods_on_socket() {
        let server = test_server();
        let resp = server.handle_jsonrpc_request(jsonrpc_req("capabilities.methods")).await;
        assert_success(&resp, "capabilities.methods");
        let result = resp.result.unwrap();
        assert!(result.is_object(), "capabilities.methods should return a map");
    }

    #[tokio::test]
    async fn wire_standard_l2_identity_get_on_socket() {
        let server = test_server();
        let resp = server.handle_jsonrpc_request(jsonrpc_req("identity.get")).await;
        assert_success(&resp, "identity.get");
        let result = resp.result.unwrap();
        assert_eq!(result["primal"].as_str().unwrap(), "songbird");
        assert_eq!(result["domain"].as_str().unwrap(), "network");
        assert_eq!(result["license"].as_str().unwrap(), "AGPL-3.0-or-later");
        assert!(result["version"].is_string());
    }

    #[tokio::test]
    async fn wire_standard_l2_identity_on_socket() {
        let server = test_server();
        let resp = server.handle_jsonrpc_request(jsonrpc_req("identity")).await;
        assert_success(&resp, "identity");
        let result = resp.result.unwrap();
        assert_eq!(result["primal"].as_str().unwrap(), "songbird");
        assert!(result["version"].is_string());
    }

    #[tokio::test]
    async fn wire_standard_l2_health_triad_on_socket() {
        let server = test_server();
        for method in &["health.liveness", "health.readiness", "health.check"] {
            let resp = server.handle_jsonrpc_request(jsonrpc_req(method)).await;
            assert_success(&resp, method);
        }
    }

    #[tokio::test]
    async fn socket_unknown_method_returns_error() {
        let server = test_server();
        let resp = server.handle_jsonrpc_request(jsonrpc_req("nonexistent.method")).await;
        assert!(resp.error.is_some(), "unknown method should return error");
        assert!(resp.result.is_none());
    }

    #[test]
    fn test_socket_path_explicit_songbird_socket() {
        let env = mock_env(HashMap::from([("SONGBIRD_SOCKET", "/tmp/test-socket.sock")]));
        let path = UnixSocketServer::socket_path_with_env(env);
        assert_eq!(path, PathBuf::from("/tmp/test-socket.sock"));
    }

    #[test]
    fn test_socket_path_orchestrator_socket_priority() {
        let env = mock_env(HashMap::from([
            ("SONGBIRD_ORCHESTRATOR_SOCKET", "/run/orchestrator.sock"),
            ("SONGBIRD_SOCKET", "/tmp/override.sock"),
        ]));
        let path = UnixSocketServer::socket_path_with_env(env);
        assert_eq!(path, PathBuf::from("/run/orchestrator.sock"));
    }

    #[test]
    fn test_socket_path_biomeos_path() {
        let env = mock_env(HashMap::from([("BIOMEOS_SOCKET_PATH", "/biomeos/songbird.sock")]));
        let path = UnixSocketServer::socket_path_with_env(env);
        assert_eq!(path, PathBuf::from("/biomeos/songbird.sock"));
    }

    #[test]
    fn test_socket_path_empty_env_ignored() {
        let env = mock_env(HashMap::from([
            ("SONGBIRD_SOCKET", ""),
            ("SONGBIRD_ORCHESTRATOR_SOCKET", ""),
            ("BIOMEOS_SOCKET_PATH", ""),
        ]));
        let path = UnixSocketServer::socket_path_with_env(env);
        let path_str = path.to_string_lossy();
        assert!(path_str.ends_with(".sock"), "Expected .sock extension, got: {path_str}");
        assert!(
            path_str.contains("network") || path_str.contains("songbird"),
            "Expected domain stem 'network' (or legacy 'songbird') in path, got: {path_str}"
        );
    }

    #[test]
    fn test_socket_path_default_is_primal_standard() {
        let env = mock_env(HashMap::new());
        let path = UnixSocketServer::socket_path_with_env(env);
        let path_str = path.to_string_lossy();

        assert!(path_str.ends_with(".sock"), "Path should end with .sock, got: {path_str}");
        assert!(
            path_str.contains("network") || path_str.contains("songbird"),
            "Path must contain domain stem 'network' (or legacy 'songbird'), got: {path_str}"
        );
    }

    #[test]
    fn test_socket_path_priority_order() {
        let env1 = mock_env(HashMap::from([
            ("SONGBIRD_ORCHESTRATOR_SOCKET", "/p1.sock"),
            ("SONGBIRD_SOCKET", "/p2.sock"),
            ("BIOMEOS_SOCKET_PATH", "/p3.sock"),
        ]));
        assert_eq!(UnixSocketServer::socket_path_with_env(env1), PathBuf::from("/p1.sock"));

        let env2 = mock_env(HashMap::from([
            ("SONGBIRD_SOCKET", "/p2.sock"),
            ("BIOMEOS_SOCKET_PATH", "/p3.sock"),
        ]));
        assert_eq!(UnixSocketServer::socket_path_with_env(env2), PathBuf::from("/p2.sock"));

        let env3 = mock_env(HashMap::from([("BIOMEOS_SOCKET_PATH", "/p3.sock")]));
        assert_eq!(UnixSocketServer::socket_path_with_env(env3), PathBuf::from("/p3.sock"));
    }

    #[test]
    fn test_concurrent_socket_path_resolution() {
        use std::thread;
        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let env = mock_env(HashMap::from([(
                        "SONGBIRD_SOCKET",
                        Box::leak(format!("/sock-{i}.sock").into_boxed_str()) as &str,
                    )]));
                    let path = UnixSocketServer::socket_path_with_env(env);
                    assert_eq!(path, PathBuf::from(format!("/sock-{i}.sock")));
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
    }
}
