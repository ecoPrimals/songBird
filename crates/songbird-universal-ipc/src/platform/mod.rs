// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Platform-specific IPC implementations

use crate::endpoint::NativeEndpoint;
use crate::error::IpcResult;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

// Platform-specific implementations
//
// **TRUE ecoBin v2.0**: All modules available for multi-transport support.
// Runtime selection via get_platform_ipc() and capability-based discovery.
//
// **NOTE:** Modules compile on all platforms for maximum flexibility.
// The actual platform selection happens at runtime.

pub mod android; // Abstract sockets (Android, Linux)
pub mod fallback;
pub mod ios; // XPC (iOS, macOS)
#[cfg(all(unix, not(target_os = "android")))]
pub mod unix; // Unix domain sockets (Linux, macOS, BSD)
pub mod wasm; // In-process (WASM)
pub mod windows; // Named pipes (Windows) // TCP localhost (universal fallback)

/// Concrete async stream for all platform transports (no trait objects).
#[derive(Debug)]
pub enum AsyncStreamImpl {
    /// Unix domain socket stream (filesystem or abstract).
    #[cfg(unix)]
    Unix(tokio::net::UnixStream),
    /// TCP localhost (fallback and cross-platform).
    Tcp(tokio::net::TcpStream),
    /// Windows named pipe client (`connect`).
    #[cfg(windows)]
    WindowsPipeClient(tokio::net::windows::named_pipe::NamedPipeClient),
    /// Windows named pipe server side after accept (connected instance).
    #[cfg(windows)]
    WindowsPipeServer(tokio::net::windows::named_pipe::NamedPipeServer),
}

impl AsyncRead for AsyncStreamImpl {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self.as_mut() {
            #[cfg(unix)]
            Self::Unix(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            #[cfg(windows)]
            Self::WindowsPipeClient(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            #[cfg(windows)]
            Self::WindowsPipeServer(s) => std::pin::Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for AsyncStreamImpl {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match &mut *self.as_mut() {
            #[cfg(unix)]
            Self::Unix(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            #[cfg(windows)]
            Self::WindowsPipeClient(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            #[cfg(windows)]
            Self::WindowsPipeServer(s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match &mut *self.as_mut() {
            #[cfg(unix)]
            Self::Unix(s) => std::pin::Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_flush(cx),
            #[cfg(windows)]
            Self::WindowsPipeClient(s) => std::pin::Pin::new(s).poll_flush(cx),
            #[cfg(windows)]
            Self::WindowsPipeServer(s) => std::pin::Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match &mut *self.as_mut() {
            #[cfg(unix)]
            Self::Unix(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            #[cfg(windows)]
            Self::WindowsPipeClient(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            #[cfg(windows)]
            Self::WindowsPipeServer(s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }
    }
}

/// Platform listener (enum dispatch, no trait objects).
pub enum PlatformListenerImpl {
    #[cfg(all(unix, not(target_os = "android")))]
    Unix(unix::UnixListener),
    /// Abstract Unix sockets (Linux + Android; also compiled on other Unix for a uniform enum surface).
    #[cfg(unix)]
    Android(android::AndroidListener),
    #[cfg(windows)]
    Windows(windows::WindowsListener),
    #[cfg(target_os = "macos")]
    Ios(ios::IosListener),
    Fallback(fallback::FallbackListener),
}

impl PlatformListenerImpl {
    /// Accept incoming connection.
    pub async fn accept(&mut self) -> IpcResult<AsyncStreamImpl> {
        match self {
            #[cfg(all(unix, not(target_os = "android")))]
            Self::Unix(l) => l.accept().await,
            #[cfg(unix)]
            Self::Android(l) => l.accept().await,
            #[cfg(windows)]
            Self::Windows(l) => l.accept().await,
            #[cfg(target_os = "macos")]
            Self::Ios(l) => l.accept().await,
            Self::Fallback(l) => l.accept().await,
        }
    }
}

/// Platform IPC implementation (enum dispatch, no trait objects).
pub enum PlatformIpcImpl {
    #[cfg(all(unix, not(target_os = "android")))]
    Unix(unix::UnixPlatformIPC),
    #[cfg(unix)]
    Android(android::AndroidPlatformIPC),
    #[cfg(windows)]
    Windows(windows::WindowsPlatformIPC),
    #[cfg(target_os = "ios")]
    Ios(ios::IosPlatformIPC),
    #[cfg(target_family = "wasm")]
    Wasm(wasm::WasmPlatformIPC),
    Fallback(fallback::FallbackPlatformIPC),
}

impl PlatformIpcImpl {
    /// Create a native endpoint for the given primal name.
    pub async fn create_endpoint(&self, primal_name: &str) -> IpcResult<NativeEndpoint> {
        match self {
            #[cfg(all(unix, not(target_os = "android")))]
            Self::Unix(p) => p.create_endpoint(primal_name).await,
            #[cfg(unix)]
            Self::Android(p) => p.create_endpoint(primal_name).await,
            #[cfg(windows)]
            Self::Windows(p) => p.create_endpoint(primal_name).await,
            #[cfg(target_os = "ios")]
            Self::Ios(p) => p.create_endpoint(primal_name).await,
            #[cfg(target_family = "wasm")]
            Self::Wasm(p) => p.create_endpoint(primal_name).await,
            Self::Fallback(p) => p.create_endpoint(primal_name).await,
        }
    }

    /// Create a listener on the native endpoint.
    pub async fn listen(&self, endpoint: &NativeEndpoint) -> IpcResult<PlatformListenerImpl> {
        match self {
            #[cfg(all(unix, not(target_os = "android")))]
            Self::Unix(p) => p.listen(endpoint).await,
            #[cfg(unix)]
            Self::Android(p) => p.listen(endpoint).await,
            #[cfg(windows)]
            Self::Windows(p) => p.listen(endpoint).await,
            #[cfg(target_os = "ios")]
            Self::Ios(p) => p.listen(endpoint).await,
            #[cfg(target_family = "wasm")]
            Self::Wasm(p) => p.listen(endpoint).await,
            Self::Fallback(p) => p.listen(endpoint).await,
        }
    }

    /// Connect to a native endpoint.
    pub async fn connect(&self, endpoint: &NativeEndpoint) -> IpcResult<AsyncStreamImpl> {
        match self {
            #[cfg(all(unix, not(target_os = "android")))]
            Self::Unix(p) => p.connect(endpoint).await,
            #[cfg(unix)]
            Self::Android(p) => p.connect(endpoint).await,
            #[cfg(windows)]
            Self::Windows(p) => p.connect(endpoint).await,
            #[cfg(target_os = "ios")]
            Self::Ios(p) => p.connect(endpoint).await,
            #[cfg(target_family = "wasm")]
            Self::Wasm(p) => p.connect(endpoint).await,
            Self::Fallback(p) => p.connect(endpoint).await,
        }
    }

    /// Cleanup endpoint (remove socket file, close pipes, etc.)
    pub async fn cleanup(&self, endpoint: &NativeEndpoint) -> IpcResult<()> {
        match self {
            #[cfg(all(unix, not(target_os = "android")))]
            Self::Unix(p) => p.cleanup(endpoint).await,
            #[cfg(unix)]
            Self::Android(p) => p.cleanup(endpoint).await,
            #[cfg(windows)]
            Self::Windows(p) => p.cleanup(endpoint).await,
            #[cfg(target_os = "ios")]
            Self::Ios(p) => p.cleanup(endpoint).await,
            #[cfg(target_family = "wasm")]
            Self::Wasm(p) => p.cleanup(endpoint).await,
            Self::Fallback(p) => p.cleanup(endpoint).await,
        }
    }
}

/// Unified stream trait
///
/// This trait unifies all platform-specific streams (Unix sockets, named pipes, TCP)
/// into a single interface that implements `AsyncRead` + `AsyncWrite`.
pub trait AsyncStream: AsyncRead + AsyncWrite + Send + Unpin + 'static {}

// Blanket implementation: any type that implements the required traits is an AsyncStream
impl<T> AsyncStream for T where T: AsyncRead + AsyncWrite + Send + Unpin + 'static {}

/// Get the default platform IPC implementation
///
/// **Legacy function** - Returns single "best guess" implementation.
/// For multi-transport support, use `get_platform_transports()` instead.
#[must_use]
pub fn get_platform_ipc() -> PlatformIpcImpl {
    #[cfg(target_os = "android")]
    {
        // Android: Prefer abstract sockets (SELinux-safe)
        PlatformIpcImpl::Android(android::AndroidPlatformIPC)
    }

    #[cfg(all(unix, not(target_os = "android")))]
    {
        // Unix: Filesystem sockets (Linux, macOS, BSD)
        PlatformIpcImpl::Unix(unix::UnixPlatformIPC)
    }

    #[cfg(windows)]
    {
        // Windows: Named pipes (Pure Rust implementation!)
        PlatformIpcImpl::Windows(windows::WindowsPlatformIPC)
    }

    #[cfg(target_family = "wasm")]
    {
        // WASM: In-process channels
        PlatformIpcImpl::Wasm(wasm::WasmPlatformIPC)
    }

    #[cfg(not(any(unix, windows, target_family = "wasm")))]
    {
        // Unknown platform: Universal TCP fallback
        PlatformIpcImpl::Fallback(fallback::FallbackPlatformIPC)
    }
}

/// Get ordered list of platform transports to try
///
/// **Multi-Transport Support** (TRUE ecoBin v2.0):
/// Returns transports in priority order (native → fallback).
/// Caller should try each until one succeeds.
///
/// # Returns
/// Vec of (name, implementation) pairs in priority order
#[must_use]
pub fn get_platform_transports() -> Vec<(&'static str, PlatformIpcImpl)> {
    let mut transports = Vec::new();

    // Platform-specific native transports (highest priority)
    #[cfg(target_os = "android")]
    {
        transports
            .push(("android-abstract", PlatformIpcImpl::Android(android::AndroidPlatformIPC)));
    }

    #[cfg(target_os = "linux")]
    {
        // Linux supports both abstract and filesystem
        #[cfg(not(target_os = "android"))]
        transports.push(("linux-abstract", PlatformIpcImpl::Android(android::AndroidPlatformIPC)));

        transports.push(("linux-unix", PlatformIpcImpl::Unix(unix::UnixPlatformIPC)));
    }

    #[cfg(target_os = "macos")]
    {
        transports.push(("macos-unix", PlatformIpcImpl::Unix(unix::UnixPlatformIPC)));
        // Note: Could also add ios::IosPlatformIPC for XPC support on macOS
    }

    #[cfg(target_os = "ios")]
    {
        transports.push(("ios-xpc", PlatformIpcImpl::Ios(ios::IosPlatformIPC)));
    }

    #[cfg(all(
        unix,
        not(any(
            target_os = "linux",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        ))
    ))]
    {
        // Other Unix (BSD, etc.)
        transports.push(("unix", PlatformIpcImpl::Unix(unix::UnixPlatformIPC)));
    }

    #[cfg(windows)]
    {
        // Windows: Named pipes (Pure Rust, tokio-based!)
        transports.push(("windows-pipe", PlatformIpcImpl::Windows(windows::WindowsPlatformIPC)));
    }

    #[cfg(target_family = "wasm")]
    {
        transports.push(("wasm-inprocess", PlatformIpcImpl::Wasm(wasm::WasmPlatformIPC)));
    }

    // Universal TCP fallback (lowest priority, always works)
    transports.push(("tcp-fallback", PlatformIpcImpl::Fallback(fallback::FallbackPlatformIPC)));

    transports
}

/// Get platform-specific transport name (for logging/metrics)
#[must_use]
pub const fn get_platform_name() -> &'static str {
    #[cfg(target_os = "android")]
    {
        "android"
    }

    #[cfg(all(target_os = "linux", not(target_os = "android")))]
    {
        "linux"
    }

    #[cfg(target_os = "macos")]
    {
        "macos"
    }

    #[cfg(target_os = "ios")]
    {
        "ios"
    }

    #[cfg(target_os = "windows")]
    {
        "windows"
    }

    #[cfg(target_family = "wasm")]
    {
        "wasm"
    }

    #[cfg(not(any(
        target_os = "android",
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "windows",
        target_family = "wasm"
    )))]
    {
        "unknown"
    }
}

/// Try multiple transports in priority order
///
/// **Multi-Transport Strategy** (TRUE ecoBin v2.0):
/// 1. Try native platform transport (fastest, most secure)
/// 2. Try alternative transports (if available)
/// 3. Fall back to TCP localhost (always works)
///
/// Returns the first transport that successfully creates an endpoint.
///
/// # Arguments
/// * `primal_name` - Name of the primal
///
/// # Returns
/// `(transport_name, endpoint)` for the successful transport
pub async fn try_multi_transport(primal_name: &str) -> IpcResult<(&'static str, NativeEndpoint)> {
    use crate::error::IpcError;
    use tracing::{debug, info, warn};

    let transports = get_platform_transports();

    debug!(
        "Trying {} transports for primal '{}' (platform: {})",
        transports.len(),
        primal_name,
        get_platform_name()
    );

    let mut last_error = None;

    for (name, implementation) in transports {
        debug!("Attempting transport: {}", name);

        match implementation.create_endpoint(primal_name).await {
            Ok(endpoint) => {
                info!(
                    "✅ Successfully created endpoint using '{}' transport: {}",
                    name,
                    endpoint.display()
                );

                if endpoint.is_native() {
                    info!("   Native transport (optimal performance)");
                } else {
                    warn!("   Non-native transport (acceptable fallback)");
                }

                return Ok((name, endpoint));
            }
            Err(e) => {
                warn!("❌ Transport '{}' failed: {}", name, e);
                last_error = Some(e);
                continue;
            }
        }
    }

    // All transports failed
    Err(last_error.unwrap_or_else(|| IpcError::Other("No transports available".to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform_transports() {
        let transports = get_platform_transports();

        // Should always have at least TCP fallback
        assert!(!transports.is_empty());

        // Last transport should always be TCP fallback
        let (last_name, _) = transports.last().unwrap();
        assert_eq!(*last_name, "tcp-fallback");
    }

    #[test]
    fn test_get_platform_name() {
        let name = get_platform_name();

        // Should return a valid platform name
        assert!(!name.is_empty());

        // Should be one of the known platforms
        let known_platforms = ["android", "linux", "macos", "ios", "windows", "wasm", "unknown"];
        assert!(known_platforms.contains(&name));
    }

    #[tokio::test]
    async fn test_try_multi_transport() {
        // Should successfully create an endpoint using available transports
        let result = try_multi_transport("test-primal").await;

        assert!(result.is_ok());

        let (transport_name, endpoint) = result.unwrap();

        // Should have selected a transport
        assert!(!transport_name.is_empty());

        // Endpoint should be valid
        assert!(!endpoint.display().is_empty());
    }
}
