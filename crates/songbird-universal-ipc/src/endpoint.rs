// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Endpoint types - Virtual and Native

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Virtual endpoint (platform-agnostic path)
///
/// Always uses Unix-style paths like `/primal/beardog`.
/// The universal IPC layer translates these to platform-specific endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VirtualEndpoint {
    /// Virtual path (always Unix-style)
    pub path: String,
}

impl VirtualEndpoint {
    /// Create a new virtual endpoint
    ///
    /// # Arguments
    /// * `primal_name` - Name of the primal (e.g., "beardog")
    ///
    /// # Returns
    /// Virtual endpoint with path `/primal/{name}`
    #[must_use]
    pub fn new(primal_name: &str) -> Self {
        Self {
            path: format!("/primal/{primal_name}"),
        }
    }

    /// Extract primal name from virtual path
    ///
    /// # Returns
    /// Primal name, or None if path format is invalid
    #[must_use]
    pub fn primal_name(&self) -> Option<&str> {
        self.path.strip_prefix("/primal/")
    }

    /// Validate virtual path format
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.primal_name().is_some()
    }
}

/// Native endpoint (platform-specific)
///
/// **Platform-Agnostic Design** (TRUE ecoBin v2.0):
/// All variants available on all platforms. Selection happens at runtime,
/// not compile-time. This eliminates platform guards and enables true portability.
///
/// Platform-to-transport mapping:
/// - Linux: `UnixSocket` (filesystem) or `AbstractSocket` (namespace)
/// - Android: `AbstractSocket` (SELinux-safe, no filesystem)
/// - Windows: `NamedPipe` (`\\.\pipe\biomeos_{name}`)
/// - macOS: `UnixSocket` (filesystem)
/// - iOS: `XPC` (Apple IPC) or `UnixSocket` fallback
/// - WASM: `InProcess` (same runtime, no real IPC)
/// - Embedded: `SharedMemory` (low-level IPC)
/// - Universal fallback: `TcpLocal` (works anywhere)
#[derive(Debug, Clone)]
pub enum NativeEndpoint {
    /// Unix domain socket (filesystem-based)
    /// - Linux, macOS, BSD: `/run/user/$UID/biomeos/{primal}.sock`
    /// - XDG-compliant, no hardcoded `/tmp/`
    UnixSocket(PathBuf),

    /// Abstract Unix socket (Linux namespace-based, Android-preferred)
    /// - Linux: `@biomeos_{primal}` (abstract namespace)
    /// - Android: `@biomeos_{primal}` (SELinux-safe, no filesystem)
    /// - No filesystem overhead, automatically cleaned up
    AbstractSocket(String),

    /// Windows named pipe
    /// - Windows: `\\.\pipe\biomeos_{primal}`
    /// - Requires tokio named pipe support
    NamedPipe(String),

    /// XPC service (iOS/macOS)
    /// - iOS: `org.biomeos.{primal}` (required for iOS)
    /// - macOS: Optional, can use `UnixSocket` instead
    /// - Requires platform-specific XPC bindings
    XPC(String),

    /// In-process channel (WASM, single-runtime)
    /// - WASM: All primals in same runtime, no real IPC needed
    /// - Uses async channels, zero overhead
    /// - Port is a logical identifier, not a real port
    InProcess(u16),

    /// Shared memory IPC (embedded, bare-metal)
    /// - Embedded: Direct memory access
    /// - Name identifies shared memory region
    /// - Requires platform-specific memory mapping
    SharedMemory(String),

    /// TCP localhost (universal fallback)
    /// - Works on ANY platform (ultimate fallback)
    /// - Port dynamically assigned (50000+)
    /// - Less performant but always available
    TcpLocal(u16),
}

impl NativeEndpoint {
    /// Get display string for logging
    ///
    /// **Platform-Agnostic**: Works on all platforms, no `#[cfg]` guards
    #[must_use]
    pub fn display(&self) -> String {
        match self {
            Self::UnixSocket(path) => {
                format!("unix://{}", path.display())
            }
            Self::AbstractSocket(name) => {
                format!("abstract://{name}")
            }
            Self::NamedPipe(name) => {
                format!("pipe://{name}")
            }
            Self::XPC(service) => {
                format!("xpc://{service}")
            }
            Self::InProcess(id) => {
                format!("inprocess://{id}")
            }
            Self::SharedMemory(region) => {
                format!("shmem://{region}")
            }
            Self::TcpLocal(port) => {
                format!("tcp://127.0.0.1:{port}")
            }
        }
    }

    /// Bare filesystem path for Unix sockets, or equivalent connect target for other transports.
    ///
    /// Returns the path without any scheme prefix, suitable for direct
    /// `UnixStream::connect()` or `TcpStream::connect()`.
    #[must_use]
    pub fn socket_path(&self) -> Option<String> {
        match self {
            Self::UnixSocket(path) => Some(path.display().to_string()),
            Self::AbstractSocket(name) => Some(format!("@{name}")),
            Self::TcpLocal(port) => Some(format!("127.0.0.1:{port}")),
            Self::NamedPipe(_) | Self::XPC(_) | Self::InProcess(_) | Self::SharedMemory(_) => None,
        }
    }

    /// Get transport type name (for metrics/logging)
    #[must_use]
    pub const fn transport_type(&self) -> &'static str {
        match self {
            Self::UnixSocket(_) => "unix",
            Self::AbstractSocket(_) => "abstract",
            Self::NamedPipe(_) => "pipe",
            Self::XPC(_) => "xpc",
            Self::InProcess(_) => "inprocess",
            Self::SharedMemory(_) => "shmem",
            Self::TcpLocal(_) => "tcp",
        }
    }

    /// Check if endpoint exists/is accessible
    ///
    /// **Platform-Agnostic**: Returns best-effort result on all platforms
    #[must_use]
    pub fn exists(&self) -> bool {
        match self {
            Self::UnixSocket(path) => path.exists(),
            Self::AbstractSocket(_) => {
                // Abstract sockets don't have filesystem presence
                // Can't check without attempting connection
                true
            }
            Self::NamedPipe(_) => {
                // Named pipes don't have simple "exists" check
                // Would require platform-specific API call
                true
            }
            Self::XPC(_) => {
                // XPC services registered with launchd
                // Would require platform-specific query
                true
            }
            Self::InProcess(_) => {
                // In-process always "exists" in same runtime
                true
            }
            Self::SharedMemory(_) => {
                // Shared memory requires platform-specific check
                true
            }
            Self::TcpLocal(_) => {
                // TCP localhost always "exists"
                true
            }
        }
    }

    /// Get performance tier (for automatic transport selection)
    ///
    /// Lower is better (faster, lower latency, higher throughput)
    #[must_use]
    pub const fn performance_tier(&self) -> u8 {
        match self {
            Self::SharedMemory(_) => 0,   // ~1μs, 50GB/s
            Self::InProcess(_) => 1,      // ~0.1μs (same process)
            Self::UnixSocket(_) => 2,     // ~5μs, 10GB/s
            Self::AbstractSocket(_) => 2, // ~5μs, 10GB/s (same as Unix)
            Self::XPC(_) => 3,            // ~10μs
            Self::NamedPipe(_) => 3,      // ~10μs, 5GB/s
            Self::TcpLocal(_) => 4,       // ~50μs, 1GB/s
        }
    }

    /// Check if transport is native to current platform (optimal)
    #[must_use]
    pub const fn is_native(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            matches!(self, Self::UnixSocket(_) | Self::AbstractSocket(_))
        }

        #[cfg(target_os = "android")]
        {
            // Android prefers abstract sockets (SELinux-safe)
            matches!(self, Self::AbstractSocket(_))
        }

        #[cfg(target_os = "windows")]
        {
            matches!(self, Self::NamedPipe(_))
        }

        #[cfg(target_os = "macos")]
        {
            matches!(self, Self::UnixSocket(_) | Self::XPC(_))
        }

        #[cfg(target_os = "ios")]
        {
            matches!(self, Self::XPC(_))
        }

        #[cfg(target_family = "wasm")]
        {
            matches!(self, Self::InProcess(_))
        }

        #[cfg(not(any(
            target_os = "linux",
            target_os = "android",
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_family = "wasm"
        )))]
        {
            // Unknown platform, TCP is universal fallback
            matches!(self, Self::TcpLocal(_))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_endpoint_creation() {
        let endpoint = VirtualEndpoint::new("security");
        assert_eq!(endpoint.path, "/primal/security");
        assert_eq!(endpoint.primal_name(), Some("security"));
        assert!(endpoint.is_valid());
    }

    #[test]
    fn test_virtual_endpoint_invalid() {
        let endpoint = VirtualEndpoint {
            path: "/invalid/path".to_string(),
        };
        assert_eq!(endpoint.primal_name(), None);
        assert!(!endpoint.is_valid());
    }

    #[test]
    fn test_native_endpoint_display_unix() {
        let endpoint =
            NativeEndpoint::UnixSocket(PathBuf::from("/run/user/1000/biomeos/test.sock"));
        assert_eq!(endpoint.display(), "unix:///run/user/1000/biomeos/test.sock");
    }

    #[test]
    fn test_native_endpoint_display_abstract() {
        let endpoint = NativeEndpoint::AbstractSocket("@biomeos_test".to_string());
        assert_eq!(endpoint.display(), "abstract://@biomeos_test");
    }

    #[test]
    fn test_native_endpoint_display_pipe() {
        let endpoint = NativeEndpoint::NamedPipe(r"\\.\pipe\biomeos_test".to_string());
        assert_eq!(endpoint.display(), r"pipe://\\.\pipe\biomeos_test");
    }

    #[test]
    fn test_native_endpoint_display_xpc() {
        let endpoint = NativeEndpoint::XPC("org.biomeos.test".to_string());
        assert_eq!(endpoint.display(), "xpc://org.biomeos.test");
    }

    #[test]
    fn test_native_endpoint_display_inprocess() {
        let endpoint = NativeEndpoint::InProcess(12345);
        assert_eq!(endpoint.display(), "inprocess://12345");
    }

    #[test]
    fn test_native_endpoint_display_shmem() {
        let endpoint = NativeEndpoint::SharedMemory("biomeos_test_region".to_string());
        assert_eq!(endpoint.display(), "shmem://biomeos_test_region");
    }

    #[test]
    fn test_native_endpoint_display_tcp() {
        let endpoint = NativeEndpoint::TcpLocal(8080);
        assert_eq!(endpoint.display(), "tcp://127.0.0.1:8080");
    }

    #[test]
    fn test_native_endpoint_transport_type() {
        assert_eq!(
            NativeEndpoint::UnixSocket(PathBuf::from("/tmp/test.sock")).transport_type(),
            "unix"
        );
        assert_eq!(
            NativeEndpoint::AbstractSocket("@test".to_string()).transport_type(),
            "abstract"
        );
        assert_eq!(NativeEndpoint::NamedPipe("pipe".to_string()).transport_type(), "pipe");
        assert_eq!(NativeEndpoint::XPC("xpc".to_string()).transport_type(), "xpc");
        assert_eq!(NativeEndpoint::InProcess(1).transport_type(), "inprocess");
        assert_eq!(NativeEndpoint::SharedMemory("mem".to_string()).transport_type(), "shmem");
        assert_eq!(NativeEndpoint::TcpLocal(8080).transport_type(), "tcp");
    }

    #[test]
    fn test_native_endpoint_performance_tier() {
        // Verify performance ordering (lower = better)
        assert!(
            NativeEndpoint::SharedMemory("m".to_string()).performance_tier()
                < NativeEndpoint::InProcess(1).performance_tier()
        );
        assert!(
            NativeEndpoint::InProcess(1).performance_tier()
                < NativeEndpoint::UnixSocket(PathBuf::from("/tmp/test.sock")).performance_tier()
        );
        assert!(
            NativeEndpoint::UnixSocket(PathBuf::from("/tmp/test.sock")).performance_tier()
                < NativeEndpoint::NamedPipe("p".to_string()).performance_tier()
        );
        assert!(
            NativeEndpoint::NamedPipe("p".to_string()).performance_tier()
                < NativeEndpoint::TcpLocal(8080).performance_tier()
        );
    }
}
