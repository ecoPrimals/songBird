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
///
/// **Legacy function** - Returns single "best guess" implementation.
/// For multi-transport support, use `get_platform_transports()` instead.
#[must_use]
pub fn get_platform_ipc() -> Box<dyn PlatformIPC> {
    #[cfg(target_os = "android")]
    {
        // Android: Prefer abstract sockets (SELinux-safe)
        Box::new(android::AndroidIPC)
    }

    #[cfg(all(unix, not(target_os = "android")))]
    {
        // Unix: Filesystem sockets (Linux, macOS, BSD)
        Box::new(unix::UnixIPC)
    }

    #[cfg(windows)]
    {
        // Windows: Named pipes (Pure Rust implementation!)
        Box::new(windows::WindowsIPC)
    }

    #[cfg(target_family = "wasm")]
    {
        // WASM: In-process channels
        Box::new(wasm::WasmIPC)
    }

    #[cfg(not(any(unix, windows, target_family = "wasm")))]
    {
        // Unknown platform: Universal TCP fallback
        Box::new(fallback::FallbackIPC)
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
pub fn get_platform_transports() -> Vec<(&'static str, Box<dyn PlatformIPC>)> {
    let mut transports = Vec::new();

    // Platform-specific native transports (highest priority)
    #[cfg(target_os = "android")]
    {
        transports.push(("android-abstract", Box::new(android::AndroidIPC) as Box<dyn PlatformIPC>));
    }

    #[cfg(target_os = "linux")]
    {
        // Linux supports both abstract and filesystem
        #[cfg(not(target_os = "android"))]
        transports.push(("linux-abstract", Box::new(android::AndroidIPC) as Box<dyn PlatformIPC>));
        
        transports.push(("linux-unix", Box::new(unix::UnixIPC) as Box<dyn PlatformIPC>));
    }

    #[cfg(target_os = "macos")]
    {
        transports.push(("macos-unix", Box::new(unix::UnixIPC) as Box<dyn PlatformIPC>));
        // Note: Could also add ios::iOSIPC for XPC support on macOS
    }

    #[cfg(target_os = "ios")]
    {
        transports.push(("ios-xpc", Box::new(ios::iOSIPC) as Box<dyn PlatformIPC>));
    }

    #[cfg(all(unix, not(any(target_os = "linux", target_os = "macos", target_os = "ios", target_os = "android"))))]
    {
        // Other Unix (BSD, etc.)
        transports.push(("unix", Box::new(unix::UnixIPC) as Box<dyn PlatformIPC>));
    }

    #[cfg(windows)]
    {
        // Windows: Named pipes (Pure Rust, tokio-based!)
        transports.push(("windows-pipe", Box::new(windows::WindowsIPC) as Box<dyn PlatformIPC>));
    }

    #[cfg(target_family = "wasm")]
    {
        transports.push(("wasm-inprocess", Box::new(wasm::WasmIPC) as Box<dyn PlatformIPC>));
    }

    // Universal TCP fallback (lowest priority, always works)
    transports.push(("tcp-fallback", Box::new(fallback::FallbackIPC) as Box<dyn PlatformIPC>));

    transports
}

/// Get platform-specific transport name (for logging/metrics)
#[must_use]
pub fn get_platform_name() -> &'static str {
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
    Err(last_error.unwrap_or_else(|| {
        IpcError::Other("No transports available".to_string())
    }))
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
        let known_platforms = vec![
            "android", "linux", "macos", "ios", "windows", "wasm", "unknown"
        ];
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
