// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Public API for universal IPC
//!
//! This module provides the main user-facing API for universal IPC.
//! All application primals should use this API instead of platform-specific sockets.

use crate::endpoint::VirtualEndpoint;
use crate::error::IpcResult;
use crate::platform::{AsyncStream, PlatformIPC, PlatformListener, get_platform_ipc};
use crate::registry::ServiceRegistry;
use std::sync::OnceLock;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::info;

/// Global universal IPC instance
static GLOBAL_IPC: OnceLock<UniversalIPC> = OnceLock::new();

/// Universal IPC instance
///
/// This is the main entry point for universal IPC functionality.
/// It manages service registration, discovery, and connection.
pub struct UniversalIPC {
    /// Service registry (in-memory)
    registry: ServiceRegistry,
    /// Platform-specific implementation
    platform: Box<dyn PlatformIPC>,
}

impl UniversalIPC {
    /// Create a new universal IPC instance
    ///
    /// This automatically detects the platform and selects the appropriate
    /// implementation (Unix sockets, Windows named pipes, or TCP fallback).
    pub fn new() -> IpcResult<Self> {
        let platform = get_platform_ipc();

        info!("Universal IPC initialized for platform: {}", std::env::consts::OS);

        Ok(Self {
            registry: ServiceRegistry::new(),
            platform,
        })
    }

    /// Register this primal
    ///
    /// # Arguments
    /// * `name` - Primal name (e.g., "beardog")
    /// * `capabilities` - List of capabilities this primal provides
    ///
    /// # Returns
    /// Virtual endpoint to listen on
    pub async fn register(
        &self,
        name: &str,
        capabilities: Vec<String>,
    ) -> IpcResult<VirtualEndpoint> {
        // Create platform-specific endpoint
        let native_endpoint = self.platform.create_endpoint(name).await?;

        // Register in service registry
        self.registry.register(name, native_endpoint, capabilities).await
    }

    /// Listen on a virtual endpoint
    ///
    /// # Arguments
    /// * `endpoint` - Virtual endpoint (from `register()`)
    ///
    /// # Returns
    /// Listener for accepting connections
    pub async fn listen(&self, endpoint: VirtualEndpoint) -> IpcResult<Listener> {
        // Resolve virtual endpoint to native endpoint
        let native_endpoint = self.registry.resolve(&endpoint.path).await?;

        // Create platform-specific listener
        let inner = self.platform.listen(&native_endpoint).await?;

        Ok(Listener {
            inner,
        })
    }

    /// Connect to a virtual endpoint
    ///
    /// # Arguments
    /// * `virtual_path` - Virtual path (e.g., "/primal/beardog")
    ///
    /// # Returns
    /// Connected stream (platform-agnostic!)
    pub async fn connect(&self, virtual_path: &str) -> IpcResult<Stream> {
        // Resolve virtual path to native endpoint
        let native_endpoint = self.registry.resolve(virtual_path).await?;

        // Connect using platform-specific implementation
        let inner = self.platform.connect(&native_endpoint).await?;

        Ok(Stream {
            inner,
        })
    }

    /// Find services by capability
    ///
    /// # Arguments
    /// * `capability` - Capability to search for
    ///
    /// # Returns
    /// List of virtual paths for services with this capability
    pub async fn find_by_capability(&self, capability: &str) -> Vec<String> {
        self.registry.find_by_capability(capability).await
    }

    /// List all registered services
    pub async fn list_services(&self) -> Vec<String> {
        self.registry.list_services().await
    }

    /// Unregister a service
    ///
    /// # Arguments
    /// * `name` - Service name to unregister
    pub async fn unregister(&self, name: &str) -> IpcResult<()> {
        // Unregister from registry
        self.registry.unregister(name).await?;

        // Registry-only teardown: native listener/socket cleanup needs a stored endpoint map first.

        Ok(())
    }
}

/// Platform-agnostic listener
///
/// Accepts incoming connections from other primals.
pub struct Listener {
    inner: Box<dyn PlatformListener>,
}

impl Listener {
    /// Accept an incoming connection
    ///
    /// # Returns
    /// Connected stream
    pub async fn accept(&mut self) -> IpcResult<Stream> {
        let inner = self.inner.accept().await?;
        Ok(Stream {
            inner,
        })
    }
}

/// Platform-agnostic stream
///
/// Provides `AsyncRead` + `AsyncWrite` for communication.
pub struct Stream {
    inner: Box<dyn AsyncStream>,
}

impl Stream {
    /// Wrap a boxed async stream (for example a raw [`tokio::net::UnixStream`]) for Tower Atomic JSON-RPC.
    #[must_use]
    pub fn from_boxed_async(inner: Box<dyn crate::platform::AsyncStream>) -> Self {
        Self {
            inner,
        }
    }
}

impl AsyncRead for Stream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API FUNCTIONS (For convenience)
// ═══════════════════════════════════════════════════════════════════════════

/// Initialize universal IPC
///
/// This must be called once before using other IPC functions.
/// It's safe to call multiple times (subsequent calls are no-ops).
///
/// # Errors
/// Returns an error if initialization fails due to platform detection failure,
/// resource exhaustion, or permission issues.
pub fn init() -> IpcResult<()> {
    use crate::error::IpcError;
    use tracing::{debug, error};

    // Already initialized - return success
    if GLOBAL_IPC.get().is_some() {
        return Ok(());
    }

    debug!("Attempting to initialize Universal IPC");
    debug!("  Platform: {}", std::env::consts::OS);
    debug!("  Architecture: {}", std::env::consts::ARCH);
    debug!("Creating UniversalIPC instance");

    // Attempt initialization with proper error handling
    let ipc = UniversalIPC::new().map_err(|e| {
        error!("❌ Failed to create UniversalIPC: {}", e);
        error!("   This may indicate:");
        error!("     - Platform detection failure");
        error!("     - Resource exhaustion (file descriptors)");
        error!("     - Permission issues (socket creation)");
        IpcError::Other(format!("Universal IPC initialization failed: {e}"))
    })?;

    // Try to set the global instance (race-safe: another thread may have set it)
    match GLOBAL_IPC.set(ipc) {
        Ok(()) => {
            info!("✅ Universal IPC initialized successfully");
            Ok(())
        }
        Err(_already_set) => {
            // Another thread initialized it first - that's fine
            info!("✅ Universal IPC already initialized by another thread");
            Ok(())
        }
    }
}

/// Try to get global IPC instance
///
/// Returns `None` if `init()` hasn't been called yet.
/// Prefer this over `global()` when you can handle the uninitialized case.
pub fn try_global() -> Option<&'static UniversalIPC> {
    GLOBAL_IPC.get()
}

/// Get global IPC instance
///
/// # Panics
/// Panics if `init()` hasn't been called yet.
/// Use `try_global()` if you need to handle the uninitialized case gracefully.
pub fn global() -> &'static UniversalIPC {
    GLOBAL_IPC.get().expect("Universal IPC not initialized. Call ipc::init() first!")
}

/// Register this primal (convenience function)
///
/// # Arguments
/// * `name` - Primal name
/// * `capabilities` - List of capabilities
///
/// # Returns
/// Virtual endpoint to listen on
pub async fn register(name: &str, capabilities: Vec<String>) -> IpcResult<VirtualEndpoint> {
    global().register(name, capabilities).await
}

/// Listen on virtual endpoint (convenience function)
pub async fn listen(endpoint: VirtualEndpoint) -> IpcResult<Listener> {
    global().listen(endpoint).await
}

/// Connect to virtual path (convenience function)
pub async fn connect(virtual_path: &str) -> IpcResult<Stream> {
    global().connect(virtual_path).await
}

/// Find services by capability (convenience function)
pub async fn find_by_capability(capability: &str) -> Vec<String> {
    global().find_by_capability(capability).await
}

/// List all services (convenience function)
pub async fn list_services() -> Vec<String> {
    global().list_services().await
}

/// Unregister a service (convenience function)
///
/// # Arguments
/// * `name` - Service name to unregister
pub async fn unregister(name: &str) -> IpcResult<()> {
    global().unregister(name).await
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::endpoint::VirtualEndpoint;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_init() {
        let result = init();
        assert!(result.is_ok());

        // Second call should also succeed
        let result = init();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_listen_connect() {
        init().unwrap();

        // Register and listen
        let endpoint =
            register("test-primal-register-connect", vec!["test".to_string()]).await.unwrap();

        let mut listener = listen(endpoint).await.unwrap();

        // Connect in background (listener already bound — no sleep needed)
        let connect_handle =
            tokio::spawn(async move { connect("/primal/test-primal-register-connect").await });

        // Accept connection
        let mut server_stream = listener.accept().await.unwrap();

        // Get client stream
        let mut client_stream = connect_handle.await.unwrap().unwrap();

        // Test communication
        client_stream.write_all(b"hello from client").await.unwrap();

        let mut buf = [0u8; 17];
        server_stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello from client");

        // Reply
        server_stream.write_all(b"hello from server").await.unwrap();

        let mut buf = [0u8; 17];
        client_stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello from server");
    }

    #[tokio::test]
    async fn test_find_by_capability() {
        init().unwrap();

        // Register services with capabilities
        register("service1", vec!["crypto".to_string()]).await.unwrap();

        register("service2", vec!["crypto".to_string(), "storage".to_string()]).await.unwrap();

        register("service3", vec!["storage".to_string()]).await.unwrap();

        // Find by capability
        let crypto_services = find_by_capability("crypto").await;
        assert_eq!(crypto_services.len(), 2);

        let storage_services = find_by_capability("storage").await;
        assert_eq!(storage_services.len(), 2);
    }

    #[test]
    fn virtual_endpoint_new_and_primal_name() {
        let v = VirtualEndpoint::new("security");
        assert_eq!(v.path, "/primal/security");
        assert_eq!(v.primal_name().expect("name"), "security");
        assert!(v.is_valid());
    }

    #[test]
    fn virtual_endpoint_invalid_path_not_primal_prefixed() {
        let v = VirtualEndpoint {
            path: "/other/x".into(),
        };
        assert!(v.primal_name().is_none());
        assert!(!v.is_valid());
    }

    #[test]
    fn try_global_none_before_init_in_test_process() {
        // Other tests may have initialized global IPC; if so, we only assert try_global returns Some.
        let g = try_global();
        if g.is_none() {
            init().expect("init ipc for try_global probe");
            assert!(try_global().is_some(), "try_global after init");
        } else {
            assert!(g.is_some());
        }
    }
}
