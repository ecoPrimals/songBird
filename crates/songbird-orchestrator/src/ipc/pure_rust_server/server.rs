//! Pure Rust Unix Socket Server Infrastructure
//!
//! v3.22.0: Evolved from jsonrpsee to pure Rust implementation (BearDog pattern)
//!
//! ## Design Principles
//!
//! 1. **Zero External RPC Libraries**: Pure `tokio::net::UnixListener` + JSON
//! 2. **Zero Hardcoding**: Socket path from env vars
//! 3. **Modern Async**: tokio + async/await
//! 4. **Thread-Safe**: Arc + atomic readiness flags
//! 5. **Observable**: Structured logging
//! 6. **Graceful Shutdown**: Cleanup on drop

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info, warn};

use super::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use super::squirrel_handlers;
use crate::app::connection_manager::ConnectionManager;
use crate::ipc::handlers::IpcHandlers;
use crate::ipc::registry::ServiceRegistry;
use songbird_discovery::anonymous::AnonymousDiscoveryListener;

/// Pure Rust Unix socket JSON-RPC server for inter-primal communication
///
/// ## Architecture
///
/// ```text
/// Primals → Unix Socket → JSON-RPC 2.0 → Songbird APIs
/// ```
///
/// ## APIs Exposed (14 total)
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
///
/// ### Squirrel Integration (3 APIs)
/// - `discover_capabilities`: Advertise Songbird's capabilities
/// - `http.request`: Delegate HTTP/HTTPS requests
/// - `health`: Simple health check
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
    /// **v5.27.0**: Added BearDog client for HTTP handler (Tower Atomic)
    ///
    /// ## Example
    ///
    /// ```rust
    /// let server = UnixSocketServer::new(
    ///     service_registry,
    ///     discovery_listener,
    ///     connection_manager,
    ///     beardog_client,
    /// );
    /// server.start().await?;
    /// ```
    pub fn new(
        service_registry: Arc<ServiceRegistry>,
        discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,
        connection_manager: Arc<ConnectionManager>,
        beardog_client: Arc<songbird_http_client::BearDogClient>,
    ) -> Self {
        let socket_path = Self::socket_path_from_env();
        let handlers = Arc::new(IpcHandlers::new(
            service_registry,
            discovery_listener,
            connection_manager,
            beardog_client,
        ));

        Self {
            socket_path,
            handlers,
            is_ready: Arc::new(AtomicBool::new(false)),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

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

        // Default: Use env_config for TRUE PRIMAL self-knowledge
        let socket_path = crate::env_config::socket_path();
        info!("📍 Using socket path (TRUE PRIMAL self-knowledge): {}", socket_path.display());
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
        info!("   APIs: 14 (3 P2P + 4 registry + 4 graph + 3 Squirrel)");
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
                    writer.flush().await?;

                    // BIOME OS FIX: Close connection after one request/response
                    // Squirrel's UniversalAiAdapter uses read_to_end() which waits for EOF
                    // Each RPC call should be independent (no persistent connections)
                    debug!("✅ Response sent, closing connection");
                    break;
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

            // HTTP/HTTPS APIs - Pure Rust Tower Atomic (v5.27.0)
            "http.request" => {
                self.handlers.http_request(request.params.unwrap_or(serde_json::json!({}))).await
            }
            "http.get" => {
                self.handlers.http_get(request.params.unwrap_or(serde_json::json!({}))).await
            }
            "http.post" => {
                self.handlers.http_post(request.params.unwrap_or(serde_json::json!({}))).await
            }
            "http.put" => {
                self.handlers.http_put(request.params.unwrap_or(serde_json::json!({}))).await
            }
            "http.delete" => {
                self.handlers.http_delete(request.params.unwrap_or(serde_json::json!({}))).await
            }

            // Squirrel Integration APIs (2) - Kept for backward compat
            "discover_capabilities" => squirrel_handlers::handle_discover_capabilities().await,
            "health" => squirrel_handlers::handle_health().await,

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
        // Test: SONGBIRD_SOCKET env var override
        // Clear all higher-priority env vars first to ensure test isolation
        std::env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");
        std::env::remove_var("BIOMEOS_SOCKET_PATH");
        std::env::set_var("SONGBIRD_SOCKET", "/tmp/test-socket.sock");
        std::env::set_var("SONGBIRD_FAMILY_ID", "nat0");

        let path = UnixSocketServer::socket_path_from_env();
        assert_eq!(path.to_str().unwrap(), "/tmp/test-socket.sock");

        std::env::remove_var("SONGBIRD_SOCKET");
        std::env::remove_var("SONGBIRD_FAMILY_ID");
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
        // Test 3: TRUE PRIMAL architecture - family-based sockets via env_config
        std::env::remove_var("SONGBIRD_SOCKET");
        std::env::set_var("SONGBIRD_FAMILY_ID", "test0");

        let path = UnixSocketServer::socket_path_from_env();
        let path_str = path.to_str().unwrap();
        eprintln!("Fallback path: {}", path_str);

        // TRUE PRIMAL: env_config returns /tmp/songbird-{family}.sock
        assert!(
            path_str.contains("songbird") && path_str.contains("test0"),
            "Path should contain 'songbird' and 'test0', got: {}",
            path_str
        );
        assert_eq!(path_str, "/tmp/songbird-test0.sock", "Should match env_config format");

        std::env::remove_var("SONGBIRD_FAMILY_ID");
    }

    #[test]
    fn test_socket_path_default_family() {
        // Test 4: TRUE PRIMAL self-knowledge - default family via env_config
        // Clear all relevant env vars to ensure clean test
        std::env::remove_var("SONGBIRD_SOCKET");
        std::env::remove_var("SONGBIRD_FAMILY_ID");

        let path = UnixSocketServer::socket_path_from_env();
        let path_str = path.to_str().unwrap();
        eprintln!("Default path: {}", path_str);

        // env_config::family_id() defaults to "nat0" when no env var set
        assert!(
            path_str.contains("songbird") && path_str.contains("nat0"),
            "Path should contain 'songbird' and 'nat0' (default family), got: {}",
            path_str
        );
        assert_eq!(path_str, "/tmp/songbird-nat0.sock", "Should match env_config default");
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
        // Test 6: TRUE PRIMAL architecture - family-based sockets, not node-based
        // Different families = different socket paths
        std::env::remove_var("SONGBIRD_SOCKET");

        std::env::set_var("SONGBIRD_FAMILY_ID", "nat0");
        let path1 = UnixSocketServer::socket_path_from_env();

        std::env::set_var("SONGBIRD_FAMILY_ID", "lan0");
        let path2 = UnixSocketServer::socket_path_from_env();

        // Paths should be different for different families
        let path1_str = path1.to_str().unwrap();
        let path2_str = path2.to_str().unwrap();

        assert_ne!(path1, path2, "Different families should have different socket paths");
        assert!(path1_str.contains("nat0"), "Path 1 should contain nat0");
        assert!(path2_str.contains("lan0"), "Path 2 should contain lan0");

        std::env::remove_var("SONGBIRD_FAMILY_ID");
    }
}
