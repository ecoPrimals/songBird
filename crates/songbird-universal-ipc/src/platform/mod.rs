//! Platform-specific IPC implementations

use crate::endpoint::NativeEndpoint;
use crate::error::IpcResult;
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

// Platform-specific implementations
// 
// **TRUE ecoBin v2.0**: All modules available on all platforms.
// Selection happens at runtime via get_platform_ipc() and multi-transport support.

pub mod unix;      // Unix domain sockets (Linux, macOS, BSD)
pub mod android;   // Abstract sockets (Android, Linux)
pub mod windows;   // Named pipes (Windows)
pub mod ios;       // XPC (iOS, macOS)
pub mod wasm;      // In-process (WASM)
pub mod fallback;  // TCP localhost (universal)

/// Platform-specific IPC trait
///
/// This trait abstracts platform-specific IPC mechanisms.
/// Each platform (Unix, Windows, etc.) implements this trait
/// to provide native socket/pipe functionality.
#[async_trait]
pub trait PlatformIPC: Send + Sync {
    /// Create a native endpoint for the given primal name
    ///
    /// # Arguments
    /// * `primal_name` - Name of the primal (e.g., "beardog")
    ///
    /// # Returns
    /// Platform-specific native endpoint
    async fn create_endpoint(&self, primal_name: &str) -> IpcResult<NativeEndpoint>;

    /// Create a listener on the native endpoint
    ///
    /// # Arguments
    /// * `endpoint` - Native endpoint to listen on
    ///
    /// # Returns
    /// Platform-specific listener
    async fn listen(&self, endpoint: &NativeEndpoint) -> IpcResult<Box<dyn PlatformListener>>;

    /// Connect to a native endpoint
    ///
    /// # Arguments
    /// * `endpoint` - Native endpoint to connect to
    ///
    /// # Returns
    /// Platform-specific stream
    async fn connect(&self, endpoint: &NativeEndpoint) -> IpcResult<Box<dyn AsyncStream>>;

    /// Cleanup endpoint (remove socket file, close pipes, etc.)
    ///
    /// # Arguments
    /// * `endpoint` - Native endpoint to cleanup
    async fn cleanup(&self, endpoint: &NativeEndpoint) -> IpcResult<()>;
}

/// Platform-agnostic listener trait
#[async_trait]
pub trait PlatformListener: Send {
    /// Accept incoming connection
    ///
    /// # Returns
    /// Connected stream
    async fn accept(&mut self) -> IpcResult<Box<dyn AsyncStream>>;
}

/// Unified stream trait
///
/// This trait unifies all platform-specific streams (Unix sockets, named pipes, TCP)
/// into a single interface that implements `AsyncRead` + `AsyncWrite`.
pub trait AsyncStream: AsyncRead + AsyncWrite + Send + Unpin + 'static {}

// Blanket implementation: any type that implements the required traits is an AsyncStream
impl<T> AsyncStream for T where T: AsyncRead + AsyncWrite + Send + Unpin + 'static {}

/// Get the default platform IPC implementation
#[must_use]
pub fn get_platform_ipc() -> Box<dyn PlatformIPC> {
    #[cfg(unix)]
    {
        Box::new(unix::UnixIPC)
    }

    #[cfg(windows)]
    {
        Box::new(windows::WindowsIPC)
    }

    #[cfg(not(any(unix, windows)))]
    {
        Box::new(fallback::FallbackIPC)
    }
}
