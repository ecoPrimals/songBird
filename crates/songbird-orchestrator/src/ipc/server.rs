//! Unix Socket JSON-RPC Server for Inter-Primal IPC
//!
//! v3.19.1: Modern async Rust with jsonrpsee
//!
//! ## Design Principles
//!
//! 1. **Zero Hardcoding**: Socket path from node_id
//! 2. **Modern Async**: jsonrpsee + tokio
//! 3. **Thread-Safe**: Arc<RwLock> for shared state
//! 4. **Observable**: Structured logging
//! 5. **Graceful Shutdown**: Cleanup on drop

use anyhow::{Context, Result};
use jsonrpsee::server::{Server, ServerHandle};
use jsonrpsee::RpcModule;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::handlers::IpcHandlers;
use crate::app::connection_manager::ConnectionManager;
use songbird_discovery::anonymous::AnonymousDiscoveryListener;

/// Unix socket JSON-RPC server for inter-primal communication
///
/// ## Architecture
///
/// ```text
/// biomeOS → Unix Socket → JSON-RPC 2.0 → Songbird APIs
/// ```
///
/// ## APIs Exposed
///
/// - `discover_by_family`: Filter discovered peers by genetic family tags
/// - `create_genetic_tunnel`: Establish BTSP tunnel with genetic proof
/// - `announce_capabilities`: Update broadcaster capabilities/tags
pub struct UnixSocketServer {
    /// Socket path (e.g., /tmp/songbird-{node_id}.sock)
    socket_path: PathBuf,
    
    /// Server handle (for graceful shutdown)
    server_handle: Option<ServerHandle>,
    
    /// API handlers
    handlers: Arc<IpcHandlers>,
}

impl UnixSocketServer {
    /// Create a new Unix socket server
    ///
    /// **Zero Hardcoding**: Socket path derived from node_id
    ///
    /// **v3.19.2**: Refactored to take individual components instead of whole orchestrator
    ///
    /// ## Example
    ///
    /// ```rust
    /// let server = UnixSocketServer::new(
    ///     "tower-001",
    ///     discovery_listener,
    ///     connection_manager,
    /// );
    /// server.start().await?;
    /// ```
    pub fn new(
        node_id: &str,
        discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        let socket_path = Self::socket_path_for_node(node_id);
        let handlers = Arc::new(IpcHandlers::new(discovery_listener, connection_manager));
        
        Self {
            socket_path,
            server_handle: None,
            handlers,
        }
    }
    
    /// Derive socket path from node_id (zero hardcoding!)
    ///
    /// Format: `/tmp/songbird-{node_id}.sock`
    ///
    /// biomeOS can discover this by:
    /// 1. Reading SONGBIRD_NODE_ID env var
    /// 2. Scanning /tmp/songbird-*.sock
    /// 3. Reading from discovery announcements
    fn socket_path_for_node(node_id: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/songbird-{}.sock", node_id))
    }
    
    /// Start the Unix socket JSON-RPC server
    ///
    /// ## Lifecycle
    ///
    /// 1. Remove stale socket file (if exists)
    /// 2. Create jsonrpsee server
    /// 3. Register API methods
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
        
        // API 1: discover_by_family
        let handlers_clone = self.handlers.clone();
        module.register_async_method("discover_by_family", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move {
                handlers.discover_by_family(params).await
            }
        })?;
        
        // API 2: create_genetic_tunnel
        let handlers_clone = self.handlers.clone();
        module.register_async_method("create_genetic_tunnel", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move {
                handlers.create_genetic_tunnel(params).await
            }
        })?;
        
        // API 3: announce_capabilities
        let handlers_clone = self.handlers.clone();
        module.register_async_method("announce_capabilities", move |params, _, _| {
            let handlers = handlers_clone.clone();
            async move {
                handlers.announce_capabilities(params).await
            }
        })?;
        
        // Start server with registered methods (runs in background)
        let handle = server.start(module);
        
        info!("✅ Unix socket JSON-RPC server started");
        info!("   Listening at: {:?}", self.socket_path);
        info!("   APIs: discover_by_family, create_genetic_tunnel, announce_capabilities");
        
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
                std::fs::remove_file(&self.socket_path)
                    .context("Failed to remove socket file")?;
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
    fn test_socket_path_derivation() {
        let path = UnixSocketServer::socket_path_for_node("tower-001");
        assert_eq!(path, PathBuf::from("/tmp/songbird-tower-001.sock"));
    }
    
    #[test]
    fn test_socket_path_no_hardcoding() {
        // Different node IDs = different socket paths
        let path1 = UnixSocketServer::socket_path_for_node("alpha");
        let path2 = UnixSocketServer::socket_path_for_node("beta");
        
        assert_ne!(path1, path2);
        assert!(path1.to_str().unwrap().contains("alpha"));
        assert!(path2.to_str().unwrap().contains("beta"));
    }
}

