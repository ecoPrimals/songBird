//! Pure Rust Unix Socket JSON-RPC Server for Inter-Primal IPC
//!
//! v3.22.0: Evolved from jsonrpsee to pure Rust implementation (BearDog pattern)
//!
//! ## Evolution Rationale
//!
//! **Problem**: `jsonrpsee` has complex Unix socket requirements causing "invalid socket address" errors
//! **Solution**: Pure Rust implementation using `tokio::net::UnixListener` (proven by BearDog v0.16.1)
//!
//! ## Design Principles
//!
//! 1. **Zero External RPC Libraries**: Pure `tokio::net::UnixListener` + JSON
//! 2. **Zero Hardcoding**: Socket path from env vars
//! 3. **Modern Async**: tokio + async/await
//! 4. **Thread-Safe**: Arc + atomic readiness flags
//! 5. **Observable**: Structured logging
//! 6. **Graceful Shutdown**: Cleanup on drop
//!
//! ## Inspired By
//!
//! BearDog v0.16.1's proven Unix socket IPC implementation (production-tested).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info, warn};

use super::handlers::IpcHandlers;
use super::registry::ServiceRegistry;
use crate::app::connection_manager::ConnectionManager;
use songbird_discovery::anonymous::AnonymousDiscoveryListener;

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Standard JSON-RPC 2.0 error codes
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Create a parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self {
            code: Self::PARSE_ERROR,
            message: message.into(),
            data: None,
        }
    }

    /// Create a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self {
            code: Self::METHOD_NOT_FOUND,
            message: format!("Method not found: {}", method.into()),
            data: None,
        }
    }

    /// Create an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: Self::INVALID_PARAMS,
            message: message.into(),
            data: None,
        }
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: Self::INTERNAL_ERROR,
            message: message.into(),
            data: None,
        }
    }

    /// Create a custom error with code, message, and optional data
    /// 
    /// This is a compatibility helper for migrating from jsonrpsee::types::ErrorObject::owned
    pub fn custom(code: i32, message: impl Into<String>, data: Option<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: data.map(serde_json::Value::String),
        }
    }
}

/// Pure Rust Unix socket JSON-RPC server for inter-primal communication
///
/// ## Architecture
///
/// ```text
/// Primals → Unix Socket → JSON-RPC 2.0 → Songbird APIs
/// ```
///
/// ## Evolution (v3.22.0)
///
/// **Before**: `jsonrpsee::Server` (complex, Unix socket issues)
/// **After**: `tokio::net::UnixListener` (simple, proven by BearDog)
///
/// ## APIs Exposed (11 total)
///
/// ### P2P Discovery (3 APIs)
/// - `discover_by_family`: Filter discovered peers by genetic family tags
/// - `create_genetic_tunnel`: Establish BTSP tunnel with genetic proof
/// - `announce_capabilities`: Update broadcaster capabilities/tags
///
/// ### Service Registry (4 APIs)
/// - `register_service`: Register a primal with Songbird
/// - `discover_by_capability`: Find primals by capability
/// - `get_service_health`: Check primal health status
/// - `health_check`: Check Songbird's own health
///
/// ### Graph Intelligence (4 APIs)
/// - `graph.validate`: Validate graph structure
/// - `graph.check_availability`: Check primal availability
/// - `graph.suggest_alternatives`: Suggest alternative primals
/// - `coordination.validate_pattern`: Validate coordination patterns
pub struct UnixSocketServer {
    /// Socket path (e.g., /run/user/{uid}/songbird-{family_id}.sock)
    socket_path: PathBuf,

    /// API handlers
    handlers: Arc<IpcHandlers>,

    /// Atomic readiness flag for lock-free concurrent checks (BearDog pattern)
    is_ready: Arc<AtomicBool>,

    /// Atomic running flag for graceful shutdown (concurrent-safe)
    is_running: Arc<AtomicBool>,
}

impl UnixSocketServer {
    /// Create a new Unix socket server
    ///
    /// **Zero Hardcoding**: Socket path derived from env vars
    /// **v3.22.0**: Pure Rust implementation (no jsonrpsee)
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
            handlers,
            is_ready: Arc::new(AtomicBool::new(false)),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Derive socket path from environment (zero hardcoding!)
    ///
    /// **v3.23.0**: BiomeOS Neural API compatibility (Jan 15, 2026)
    ///
    /// ## Priority Order (BiomeOS-compliant):
    ///
    /// Socket Path:
    /// 1. `SONGBIRD_ORCHESTRATOR_SOCKET` (Neural API standard)
    /// 2. `SONGBIRD_SOCKET` (alternative naming)
    /// 3. `BIOMEOS_SOCKET_PATH` (generic orchestrator)
    /// 4. Default: `/tmp/songbird-{family_id}.sock`
    ///
    /// Family ID:
    /// 1. `SONGBIRD_ORCHESTRATOR_FAMILY_ID` (Neural API standard)
    /// 2. `SONGBIRD_ORCHESTRATOR_FAMILY` (alternative)
    /// 3. `BIOMEOS_FAMILY_ID` (generic orchestrator)
    /// 4. `SONGBIRD_FAMILY_ID` (legacy)
    /// 5. Default: `"default"`
    ///
    /// ## BiomeOS Discovery
    ///
    /// BiomeOS Neural API sets environment variables and expects primals
    /// to honor them. This enables zero-configuration multi-family deployments.
    /// Derive socket path from environment variables (BiomeOS Neural API compatible)
    ///
    /// ## Priority Order (BiomeOS Standard)
    ///
    /// 1. `SONGBIRD_ORCHESTRATOR_SOCKET` (highest priority - Neural API)
    /// 2. `SONGBIRD_SOCKET` (alternative naming)
    /// 3. `BIOMEOS_SOCKET_PATH` (generic orchestrator)
    /// 4. Default: `/tmp/songbird-{family_id}.sock`
    ///
    /// ## Family ID Priority (for default path construction)
    ///
    /// 1. `SONGBIRD_ORCHESTRATOR_FAMILY_ID`
    /// 2. `SONGBIRD_ORCHESTRATOR_FAMILY`
    /// 3. `BIOMEOS_FAMILY_ID`
    /// 4. `SONGBIRD_FAMILY_ID`
    /// 5. Default: `"default"`
    ///
    /// ## Testing
    ///
    /// This method is `pub` to enable comprehensive testing of BiomeOS compatibility.
    /// See `tests/biomeos_socket_env_vars.rs` for validation.
    pub fn socket_path_from_env() -> PathBuf {
        // Priority 1: SONGBIRD_ORCHESTRATOR_SOCKET (Neural API standard)
        if let Ok(socket_path) = std::env::var("SONGBIRD_ORCHESTRATOR_SOCKET") {
            info!("📍 Using SONGBIRD_ORCHESTRATOR_SOCKET: {}", socket_path);
            return PathBuf::from(socket_path);
        }

        // Priority 2: SONGBIRD_SOCKET (alternative naming)
        if let Ok(socket_path) = std::env::var("SONGBIRD_SOCKET") {
            info!("📍 Using SONGBIRD_SOCKET: {}", socket_path);
            return PathBuf::from(socket_path);
        }

        // Priority 3: BIOMEOS_SOCKET_PATH (generic orchestrator)
        if let Ok(socket_path) = std::env::var("BIOMEOS_SOCKET_PATH") {
            info!("📍 Using BIOMEOS_SOCKET_PATH: {}", socket_path);
            return PathBuf::from(socket_path);
        }

        // No explicit socket path - construct from family ID
        // Priority order for family ID:
        let family_id = Self::get_family_id();

        // Default: /tmp/songbird-{family_id}.sock (BiomeOS standard)
        // Note: BiomeOS expects /tmp/ by default, NOT /run/user/{uid}/
        let socket_path = PathBuf::from(format!("/tmp/songbird-{}.sock", family_id));
        info!(
            "📍 Using default socket path with family '{}': {}",
            family_id,
            socket_path.display()
        );
        socket_path
    }

    /// Get family ID from environment variables (BiomeOS Neural API compatible)
    ///
    /// ## Priority Order (BiomeOS Standard)
    ///
    /// 1. `SONGBIRD_ORCHESTRATOR_FAMILY_ID` (highest priority - Neural API)
    /// 2. `SONGBIRD_ORCHESTRATOR_FAMILY` (alternative)
    /// 3. `BIOMEOS_FAMILY_ID` (generic orchestrator)
    /// 4. `SONGBIRD_FAMILY_ID` (legacy)
    /// 5. Default: `"default"`
    ///
    /// ## Testing
    ///
    /// This method is `pub` to enable comprehensive testing of BiomeOS compatibility.
    /// See `tests/biomeos_socket_env_vars.rs` for validation.
    pub fn get_family_id() -> String {
        std::env::var("SONGBIRD_ORCHESTRATOR_FAMILY_ID")
            .or_else(|_| std::env::var("SONGBIRD_ORCHESTRATOR_FAMILY"))
            .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
            .or_else(|_| std::env::var("SONGBIRD_FAMILY_ID"))
            .unwrap_or_else(|_| "default".to_string())
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Get a clone of the readiness flag (BearDog pattern)
    ///
    /// This allows checking readiness even after the server has been moved
    /// into a spawn task. This is lock-free and safe for concurrent access!
    pub fn readiness_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_ready)
    }

    /// Check if the server is running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Acquire)
    }

    /// Request server shutdown (graceful, non-blocking)
    pub fn shutdown(&self) {
        info!("🛑 Shutdown requested for Unix socket server");
        self.is_running.store(false, Ordering::Release);
        self.is_ready.store(false, Ordering::Release);
    }

    /// Check if the server is ready to accept connections (atomic, lock-free)
    pub fn is_ready(&self) -> bool {
        self.is_ready.load(Ordering::Acquire)
    }

    /// Wait for the server to be ready (non-blocking async wait)
    ///
    /// Returns `true` if ready within timeout, `false` if timeout expired.
    pub async fn wait_ready(&self, timeout: std::time::Duration) -> bool {
        let start = std::time::Instant::now();
        while !self.is_ready() {
            if start.elapsed() > timeout {
                return false;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        true
    }

    /// Start the Unix socket JSON-RPC server
    ///
    /// ## Lifecycle
    ///
    /// 1. Remove stale socket file (if exists)
    /// 2. Create parent directory (if needed)
    /// 3. Bind Unix socket listener
    /// 4. Mark as ready (atomic flag)
    /// 5. Accept connections loop
    ///
    /// ## Returns
    ///
    /// Never returns (runs until cancelled)
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("🔌 Starting pure Rust Unix socket JSON-RPC server...");
        info!("   Socket path: {}", self.socket_path.display());

        // Ensure parent directory exists (biomeOS requirement)
        if let Some(parent) = self.socket_path.parent() {
            if !parent.exists() {
                debug!("   Creating socket directory: {:?}", parent);
                std::fs::create_dir_all(parent)
                    .context(format!("Failed to create socket directory: {}", parent.display()))?;
            }
        }

        // Remove stale socket file (if exists)
        if self.socket_path.exists() {
            debug!("   Removing stale socket file");
            std::fs::remove_file(&self.socket_path)
                .context("Failed to remove stale socket file")?;
        }

        // Bind Unix socket (pure tokio - no jsonrpsee!)
        let listener = UnixListener::bind(&*self.socket_path)
            .context(format!("Failed to bind Unix socket: {}", self.socket_path.display()))?;

        // Mark server as ready and running atomically (lock-free!)
        self.is_running.store(true, Ordering::Release);
        self.is_ready.store(true, Ordering::Release);

        info!("✅ Unix socket JSON-RPC server listening: {}", self.socket_path.display());
        info!("   Protocol: JSON-RPC 2.0 (pure Rust)");
        info!("   APIs: 11 (3 P2P + 4 registry + 4 graph intelligence)");
        info!("   Status: READY ✅ (atomic flag set)");

        // Accept connections loop (checks is_running for graceful shutdown)
        while self.is_running() {
            // Use timeout to allow checking shutdown flag
            match tokio::time::timeout(Duration::from_millis(100), listener.accept()).await {
                Ok(Ok((stream, _addr))) => {
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("❌ Connection handler error: {}", e);
                        }
                    });
                }
                Ok(Err(e)) => {
                    error!("❌ Failed to accept connection: {}", e);
                }
                Err(_) => {
                    // Timeout - just continue loop and check is_running
                }
            }
        }

        info!("🛑 Unix socket server stopped gracefully");
        Ok(())
    }

    /// Handle a single client connection with JSON-RPC 2.0
    async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        debug!("📥 New IPC connection");

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("📤 Client disconnected");
                    break;
                }
                Ok(_) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    // Parse JSON-RPC request
                    let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                        Ok(request) => {
                            debug!("📨 JSON-RPC request: {}", request.method);
                            self.handle_jsonrpc_request(request).await
                        }
                        Err(e) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError::parse_error(format!(
                                "Failed to parse JSON-RPC request: {}",
                                e
                            ))),
                            id: serde_json::Value::Null,
                        },
                    };

                    // Send response
                    let response_json = serde_json::to_string(&response)?;
                    writer.write_all(response_json.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                }
                Err(e) => {
                    error!("❌ Failed to read from socket: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a JSON-RPC 2.0 request and route to appropriate API handler
    async fn handle_jsonrpc_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone().unwrap_or(serde_json::Value::Null);

        // Route to appropriate handler adapter based on method name
        let result = match request.method.as_str() {
            // P2P Discovery APIs (3)
            "discover_by_family" => self.handlers.discover_by_family_json(request.params).await,
            "create_genetic_tunnel" => {
                self.handlers.create_genetic_tunnel_json(request.params).await
            }
            "announce_capabilities" => {
                self.handlers.announce_capabilities_json(request.params).await
            }

            // Service Registry APIs (4)
            "register_service" => self.handlers.register_service_json(request.params).await,
            "discover_by_capability" => {
                self.handlers.discover_by_capability_json(request.params).await
            }
            "get_service_health" => self.handlers.get_service_health_json(request.params).await,
            "health_check" => self.handlers.health_check_json().await,

            // Graph Intelligence APIs (4)
            "graph.validate" => self.handlers.validate_graph_json(request.params).await,
            "graph.check_availability" => {
                self.handlers.check_availability_json(request.params).await
            }
            "graph.suggest_alternatives" => {
                self.handlers.suggest_alternatives_json(request.params).await
            }
            "coordination.validate_pattern" => {
                self.handlers.validate_coordination_pattern_json(request.params).await
            }

            // Unknown method
            _ => Err(JsonRpcError::method_not_found(&request.method)),
        };

        // Build response
        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(value),
                error: None,
                id,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(error),
                id,
            },
        }
    }

    /// Stop the server gracefully
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping Unix socket JSON-RPC server...");

        // Mark as not ready (atomic)
        self.is_ready.store(false, Ordering::Release);

        // Remove socket file
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).context("Failed to remove socket file")?;
            info!("🧹 Removed socket: {}", self.socket_path.display());
        }

        info!("✅ Unix socket server stopped");
        Ok(())
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
    fn test_socket_path_explicit_override() {
        // Test 1: SONGBIRD_SOCKET env var override (highest priority)
        std::env::set_var("SONGBIRD_SOCKET", "/tmp/test-socket.sock");
        std::env::set_var("SONGBIRD_FAMILY_ID", "nat0");
        std::env::set_var("UID", "1000");

        let path = UnixSocketServer::socket_path_from_env();
        assert_eq!(path.to_str().unwrap(), "/tmp/test-socket.sock");

        std::env::remove_var("SONGBIRD_SOCKET");
    }

    #[test]
    fn test_socket_path_xdg_runtime() {
        // Test 2: XDG runtime directory (if available)
        std::env::remove_var("SONGBIRD_SOCKET");
        std::env::remove_var("SONGBIRD_NODE_ID"); // Ensure no node_id pollution
        std::env::set_var("SONGBIRD_FAMILY_ID", "nat0");
        std::env::set_var("UID", "1000");

        let path = UnixSocketServer::socket_path_from_env();
        let path_str = path.to_str().unwrap();
        eprintln!("XDG path: {}", path_str);

        // Should contain family_id regardless of whether using XDG or /tmp
        assert!(
            path_str.contains("songbird") && path_str.contains("nat0"),
            "Path should contain 'songbird' and 'nat0', got: {}",
            path_str
        );

        // Path should be reasonable (XDG or /tmp)
        assert!(
            path_str.starts_with("/run/user/") || path_str.starts_with("/tmp/"),
            "Path should start with /run/user/ or /tmp/, got: {}",
            path_str
        );
    }

    #[test]
    fn test_socket_path_fallback_to_tmp() {
        // Test 3: Fallback to /tmp with node_id
        std::env::remove_var("SONGBIRD_SOCKET");
        std::env::set_var("SONGBIRD_FAMILY_ID", "test0");
        std::env::set_var("SONGBIRD_NODE_ID", "node1");
        std::env::set_var("UID", "99999"); // Non-existent UID to force /tmp

        let path = UnixSocketServer::socket_path_from_env();
        let path_str = path.to_str().unwrap();
        eprintln!("Fallback path: {}", path_str);

        // Should contain songbird and test0 regardless of path
        assert!(
            path_str.contains("songbird") && path_str.contains("test0"),
            "Path should contain 'songbird' and 'test0', got: {}",
            path_str
        );

        // If /run/user/99999 doesn't exist (likely), should fall back to /tmp
        if !std::path::Path::new("/run/user/99999").exists() {
            assert!(
                path_str.starts_with("/tmp/"),
                "Should use /tmp when XDG unavailable, got: {}",
                path_str
            );
            assert!(
                path_str.contains("node1"),
                "/tmp fallback should include node_id, got: {}",
                path_str
            );
        }
    }

    #[test]
    fn test_socket_path_default_family() {
        // Test 4: Default family_id and node_id
        // Clear all relevant env vars to ensure clean test
        std::env::remove_var("SONGBIRD_SOCKET");
        std::env::remove_var("SONGBIRD_FAMILY_ID");
        std::env::remove_var("SONGBIRD_NODE_ID");
        std::env::set_var("UID", "1000");

        let path = UnixSocketServer::socket_path_from_env();
        let path_str = path.to_str().unwrap();
        eprintln!("Default path: {}", path_str);
        assert!(
            path_str.contains("songbird") && path_str.contains("default"),
            "Path should contain 'songbird' and 'default', got: {}",
            path_str
        );
    }

    #[test]
    fn test_socket_path_no_hardcoding() {
        // Test 5: Different family IDs = different socket paths
        std::env::remove_var("SONGBIRD_SOCKET");
        std::env::set_var("UID", "1000");

        std::env::set_var("SONGBIRD_FAMILY_ID", "nat0");
        let path1 = UnixSocketServer::socket_path_from_env();
        eprintln!("Path 1 (nat0): {}", path1.to_str().unwrap());

        std::env::set_var("SONGBIRD_FAMILY_ID", "lan0");
        let path2 = UnixSocketServer::socket_path_from_env();
        eprintln!("Path 2 (lan0): {}", path2.to_str().unwrap());

        assert_ne!(path1, path2);
        assert!(
            path1.to_str().unwrap().contains("nat0"),
            "Path 1 should contain 'nat0', got: {}",
            path1.to_str().unwrap()
        );
        assert!(
            path2.to_str().unwrap().contains("lan0"),
            "Path 2 should contain 'lan0', got: {}",
            path2.to_str().unwrap()
        );
    }

    #[test]
    fn test_socket_path_node_id_differentiation() {
        // Test 6: Different node IDs = different socket paths (in /tmp)
        std::env::remove_var("SONGBIRD_SOCKET");
        std::env::set_var("SONGBIRD_FAMILY_ID", "nat0");
        std::env::set_var("UID", "99999"); // Force /tmp fallback

        std::env::set_var("SONGBIRD_NODE_ID", "alpha");
        let path1 = UnixSocketServer::socket_path_from_env();

        std::env::set_var("SONGBIRD_NODE_ID", "beta");
        let path2 = UnixSocketServer::socket_path_from_env();

        // Paths should be different if using /tmp (XDG doesn't include node_id)
        let path1_str = path1.to_str().unwrap();
        let path2_str = path2.to_str().unwrap();

        if path1_str.starts_with("/tmp/") && path2_str.starts_with("/tmp/") {
            assert_ne!(path1, path2);
            assert!(path1_str.contains("alpha"));
            assert!(path2_str.contains("beta"));
        }
    }
}
