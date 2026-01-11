//! Unix Socket JSON-RPC Server for Inter-Primal IPC
//!
//! v3.19.1: Modern async Rust with jsonrpsee
//! v3.20.0: Service registry + biomeOS-compatible socket path
//!
//! ## Design Principles
//!
//! 1. **Zero Hardcoding**: Socket path from env vars
//! 2. **Modern Async**: jsonrpsee + tokio
//! 3. **Thread-Safe**: Arc<RwLock> for shared state
//! 4. **Observable**: Structured logging
//! 5. **Graceful Shutdown**: Cleanup on drop

use anyhow::{Context, Result};
use jsonrpsee::server::{Server, ServerHandle};
use jsonrpsee::RpcModule;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::handlers::IpcHandlers;
use super::registry::ServiceRegistry;
use crate::app::connection_manager::ConnectionManager;
use songbird_discovery::anonymous::AnonymousDiscoveryListener;

/// Unix socket JSON-RPC server for inter-primal communication
///
/// ## Architecture
///
/// ```text
/// Primals → Unix Socket → JSON-RPC 2.0 → Songbird APIs
/// ```
///
/// ## APIs Exposed
///
/// ### P2P Discovery (v3.19.1)
/// - `discover_by_family`: Filter discovered peers by genetic family tags
/// - `create_genetic_tunnel`: Establish BTSP tunnel with genetic proof
/// - `announce_capabilities`: Update broadcaster capabilities/tags
///
/// ### Service Registry (v3.20.0)
/// - `register_service`: Register a primal with Songbird
/// - `discover_by_capability`: Find primals by capability
/// - `get_service_health`: Check primal health status
/// - `health_check`: Check Songbird's own health
pub struct UnixSocketServer {
    /// Socket path (e.g., /run/user/{uid}/songbird-{family_id}.sock)
    socket_path: PathBuf,

    /// Server handle (for graceful shutdown)
    server_handle: Option<ServerHandle>,

    /// API handlers
    handlers: Arc<IpcHandlers>,
}

impl UnixSocketServer {
    /// Create a new Unix socket server
    ///
    /// **Zero Hardcoding**: Socket path derived from family_id env var
    ///
    /// **v3.19.2**: Refactored to take individual components
    /// **v3.20.0**: Added service_registry, changed socket path to /run/user/...
    ///
    /// ## Example
    ///
    /// ```rust
    /// let server = UnixSocketServer::new(
    ///     service_registry,
    ///     discovery_listener,
    ///     connection_manager,
    /// );
    /// server.start().await?;
    /// ```
    pub fn new(
        service_registry: Arc<ServiceRegistry>,
        discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        let socket_path = Self::socket_path_from_env();
        let handlers =
            Arc::new(IpcHandlers::new(service_registry, discovery_listener, connection_manager));

        Self {
            socket_path,
            server_handle: None,
            handlers,
        }
    }

    /// Derive socket path from environment (zero hardcoding!)
    ///
    /// **v3.20.0**: Changed from `/tmp/songbird-{node_id}.sock`
    ///              to `/run/user/{uid}/songbird-{family_id}.sock`
    ///
    /// Format: `/run/user/{uid}/songbird-{family_id}.sock`
    ///
    /// ## Discovery by biomeOS
    ///
    /// 1. Read `SONGBIRD_FAMILY_ID` env var (e.g., "nat0")
    /// 2. Get current UID via `id -u` or `$UID`
    /// 3. Connect to `/run/user/{uid}/songbird-{family_id}.sock`
    fn socket_path_from_env() -> PathBuf {
        // Get family_id from env var (fallback to "default")
        let family_id =
            std::env::var("SONGBIRD_FAMILY_ID").unwrap_or_else(|_| "default".to_string());

        // Get current user ID from environment
        // Try $UID first (set by most shells), then try to parse from $USER
        let uid = std::env::var("UID").ok().and_then(|s| s.parse::<u32>().ok()).unwrap_or(1000); // Fallback to 1000 (typical first user)

        // Build path: /run/user/{uid}/songbird-{family_id}.sock
        PathBuf::from(format!("/run/user/{}/songbird-{}.sock", uid, family_id))
    }

    /// Start the Unix socket JSON-RPC server
    ///
    /// ## Lifecycle
    ///
    /// 1. Remove stale socket file (if exists)
    /// 2. Create jsonrpsee server
    /// 3. Register API methods (7 total: 3 P2P + 4 service registry)
    /// 4. Start listening (returns immediately, runs in background)
    ///
    /// ## Returns
    ///
    /// `ServerHandle` for graceful shutdown
    pub async fn start(&mut self) -> Result<ServerHandle> {
        info!("🔌 Starting Unix socket JSON-RPC server...");
        info!("   Socket path: {:?}", self.socket_path);

        // Remove stale socket file (if exists)
        if self.socket_path.exists() {
            debug!("   Removing stale socket file");
            std::fs::remove_file(&self.socket_path)
                .context("Failed to remove stale socket file")?;
        }

        // Create jsonrpsee server with Unix socket transport
        let server = Server::builder()
            .build(self.socket_path.to_str().unwrap())
            .await
            .context("Failed to create Unix socket server")?;

        // Create RPC module and register methods
        let mut module = RpcModule::new(());

        // ====================================================================
        // Service Registry APIs (v3.20.0)
        // ====================================================================

        // API 1: register_service
        let handlers_clone = self.handlers.clone();
        module.register_async_method("register_service", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move { handlers.register_service(params).await }
        })?;

        // API 2: discover_by_capability
        let handlers_clone = self.handlers.clone();
        module.register_async_method("discover_by_capability", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move { handlers.discover_by_capability(params).await }
        })?;

        // API 3: get_service_health
        let handlers_clone = self.handlers.clone();
        module.register_async_method("get_service_health", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move { handlers.get_service_health(params).await }
        })?;

        // API 4: health_check
        let handlers_clone = self.handlers.clone();
        module.register_async_method("health_check", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move { handlers.health_check(params).await }
        })?;

        // ====================================================================
        // P2P Discovery APIs (v3.19.1)
        // ====================================================================

        // API 5: discover_by_family
        let handlers_clone = self.handlers.clone();
        module.register_async_method("discover_by_family", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move { handlers.discover_by_family(params).await }
        })?;

        // API 6: create_genetic_tunnel
        let handlers_clone = self.handlers.clone();
        module.register_async_method("create_genetic_tunnel", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move { handlers.create_genetic_tunnel(params).await }
        })?;

        // API 7: announce_capabilities
        let handlers_clone = self.handlers.clone();
        module.register_async_method("announce_capabilities", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move { handlers.announce_capabilities(params).await }
        })?;

        // ====================================================================
        // Graph Validation APIs (v3.21.0 - Collaborative Intelligence)
        // ====================================================================

        // API 8: graph.validate
        let handlers_clone = self.handlers.clone();
        module.register_async_method("graph.validate", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move { handlers.validate_graph(params).await }
        })?;

        // ====================================================================
        // Graph Availability APIs (v3.21.0 - Collaborative Intelligence Week 2)
        // ====================================================================

        // API 9: graph.check_availability
        let handlers_clone = self.handlers.clone();
        module.register_async_method("graph.check_availability", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move { handlers.check_availability(params).await }
        })?;

        // API 10: graph.suggest_alternatives
        let handlers_clone = self.handlers.clone();
        module.register_async_method("graph.suggest_alternatives", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move { handlers.suggest_alternatives(params).await }
        })?;

        // ====================================================================
        // Coordination Validation API (v3.21.0 - Collaborative Intelligence Week 3)
        // ====================================================================

        // API 11: coordination.validate_pattern
        let handlers_clone = self.handlers.clone();
        module.register_async_method("coordination.validate_pattern", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move { handlers.validate_coordination_pattern(params).await }
        })?;

        // Start server with registered methods (runs in background)
        let handle = server.start(module);

        info!("✅ Unix socket JSON-RPC server started");
        info!("   Listening at: {:?}", self.socket_path);
        info!("   APIs: 11 (4 service registry + 3 P2P discovery + 4 graph intelligence)");

        // Store handle for graceful shutdown
        self.server_handle = Some(handle.clone());

        Ok(handle)
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.server_handle.is_some()
    }

    /// Stop the server gracefully
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.server_handle.take() {
            info!("🛑 Stopping Unix socket JSON-RPC server...");
            handle.stop()?;

            // Clean up socket file
            if self.socket_path.exists() {
                std::fs::remove_file(&self.socket_path).context("Failed to remove socket file")?;
            }

            info!("✅ Unix socket server stopped");
        }
        Ok(())
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }
}

/// Graceful cleanup on drop
impl Drop for UnixSocketServer {
    fn drop(&mut self) {
        // Try to remove socket file on drop (best effort)
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
                warn!("⚠️  Failed to remove socket file on drop: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path_from_env() {
        // Test socket path derivation
        std::env::set_var("SONGBIRD_FAMILY_ID", "nat0");
        std::env::set_var("UID", "1000");

        let path = UnixSocketServer::socket_path_from_env();
        let expected = "/run/user/1000/songbird-nat0.sock";
        assert_eq!(path.to_str().unwrap(), expected);

        // Test default fallback
        std::env::remove_var("SONGBIRD_FAMILY_ID");
        let path = UnixSocketServer::socket_path_from_env();
        let expected_default = "/run/user/1000/songbird-default.sock";
        assert_eq!(path.to_str().unwrap(), expected_default);
    }

    #[test]
    fn test_socket_path_no_hardcoding() {
        // Different family IDs = different socket paths
        std::env::set_var("UID", "1000");

        std::env::set_var("SONGBIRD_FAMILY_ID", "nat0");
        let path1 = UnixSocketServer::socket_path_from_env();

        std::env::set_var("SONGBIRD_FAMILY_ID", "lan0");
        let path2 = UnixSocketServer::socket_path_from_env();

        assert_ne!(path1, path2);
        assert!(path1.to_str().unwrap().contains("nat0"));
        assert!(path2.to_str().unwrap().contains("lan0"));
    }
}
