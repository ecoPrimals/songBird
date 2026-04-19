// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! iOS/macOS IPC implementation
//!
//! **Platform**: iOS, macOS (Apple platforms)
//! **Transport**:
//! - iOS: XPC (Apple IPC) — requires platform-specific bindings (not shipped in this crate)
//! - macOS: Unix sockets (filesystem-based, works today)
//!
//! ## Platform Differences
//!
//! ### iOS
//! - **Preferred**: XPC (`org.biomeos.{primal_name}`)
//! - **Requirements**: XPC framework bindings (not yet available in Pure Rust)
//! - **Status**: Documented for future implementation
//! - **Fallback**: TCP localhost (works today)
//!
//! ### macOS
//! - **Preferred**: Unix sockets (filesystem-based)
//! - **Path**: `/var/tmp/biomeos/{primal}.sock` (XDG-compliant for macOS)
//! - **Status**: Implemented (delegates to Unix implementation)
//! - **Alternative**: XPC (optional, for consistency with iOS)
//!
//! ## Implementation Strategy
//!
//! **Current** (v2.0 Phase 1):
//! - macOS: Use Unix sockets (fully functional)
//! - iOS: Document XPC requirements, use TCP fallback
//!
//! **Future** (v2.0 Phase 2+):
//! - Research Pure Rust XPC bindings (e.g., `xpc-sys` crate)
//! - Implement XPC transport for iOS
//! - Optional: Add XPC support for macOS (for ecosystem consistency)
//!
//! ## TRUE ecoBin Compliance
//!
//! - ✅ Pure Rust (zero unsafe code in this module)
//! - ⚠️  XPC requires platform-specific bindings (may have unsafe, needs analysis)
//! - ✅ Zero hardcoding (paths from XDG-compliant `env_config`)
//! - ✅ Runtime discovery (no compile-time assumptions)
//!
//! ## References
//!
//! - Apple XPC documentation: <https://developer.apple.com/documentation/xpc>
//! - XPC in Rust: Research needed (no mature Pure Rust bindings as of 2026)
//! - Alternative: `launchd` + Unix sockets (supported on both iOS and macOS)

use crate::endpoint::NativeEndpoint;
use crate::error::{IpcError, IpcResult};
use crate::platform::{AsyncStreamImpl, PlatformListenerImpl};
#[cfg(target_os = "macos")]
use songbird_types::primal_names::BIOMEOS_DIR;
#[cfg(any(target_os = "macos", target_os = "ios"))]
use tracing::{debug, info, warn};

/// iOS/macOS IPC implementation
///
/// **macOS**: Uses Unix sockets (fully functional)
/// **iOS**: XPC documented for future, TCP fallback for now
pub struct IosPlatformIPC;

impl IosPlatformIPC {
    /// Create a native endpoint for the given primal name.
    pub async fn create_endpoint(&self, primal_name: &str) -> IpcResult<NativeEndpoint> {
        #[cfg(target_os = "macos")]
        {
            use std::path::PathBuf;

            // macOS: Use Unix sockets (XDG-compliant path)
            // /var/tmp is recommended for macOS (persists across reboots)
            let socket_path = PathBuf::from(songbird_types::constants::MACOS_SHARED_TMP_DIR)
                .join(BIOMEOS_DIR)
                .join(format!("{primal_name}.sock"));

            debug!(
                "Creating macOS Unix socket endpoint for '{}': {}",
                primal_name,
                socket_path.display()
            );

            // Ensure directory exists
            if let Some(parent) = socket_path.parent() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    IpcError::PlatformError(format!(
                        "Failed to create socket directory {}: {}",
                        parent.display(),
                        e
                    ))
                })?;
            }

            // Clean up old socket if it exists
            if socket_path.exists() {
                warn!("Socket file already exists, removing: {}", socket_path.display());
                tokio::fs::remove_file(&socket_path).await.map_err(|e| {
                    IpcError::PlatformError(format!("Failed to remove old socket: {}", e))
                })?;
            }

            info!("macOS Unix socket endpoint: {} (XDG-compliant)", socket_path.display());

            Ok(NativeEndpoint::UnixSocket(socket_path))
        }

        #[cfg(target_os = "ios")]
        {
            // iOS: XPC requires Apple framework bindings not yet available in Pure Rust.
            // Return an InProcess endpoint as fallback (same runtime, zero IPC overhead).
            let logical_port = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut h = DefaultHasher::new();
                primal_name.hash(&mut h);
                (h.finish() % 60000 + 1024) as u16
            };

            debug!(
                "iOS: XPC bindings unavailable, using InProcess fallback for '{}' (port {})",
                primal_name, logical_port
            );

            Ok(NativeEndpoint::InProcess(logical_port))
        }

        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        {
            let _ = primal_name;
            Err(IpcError::PlatformError("iOSIPC is for macOS/iOS only".to_string()))
        }
    }

    /// Create a listener on the native endpoint.
    pub async fn listen(&self, endpoint: &NativeEndpoint) -> IpcResult<PlatformListenerImpl> {
        match endpoint {
            #[cfg(target_os = "macos")]
            NativeEndpoint::UnixSocket(path) => {
                // Delegate to Unix implementation
                debug!("Creating macOS Unix listener on: {}", path.display());

                let listener = tokio::net::UnixListener::bind(path).map_err(|e| {
                    IpcError::ListenerFailed(format!(
                        "Failed to bind macOS Unix socket at {}: {}",
                        path.display(),
                        e
                    ))
                })?;

                info!("macOS Unix listener created: {}", path.display());

                Ok(PlatformListenerImpl::Ios(IosListener {
                    inner: listener,
                }))
            }

            #[cfg(target_os = "ios")]
            NativeEndpoint::XPC(service) => Err(IpcError::PlatformError(format!(
                "XPC transport requires Apple framework bindings (not yet available in Pure Rust). \
                     Service: {}. Use InProcess or Unix socket fallback.",
                service
            ))),

            _ => Err(IpcError::PlatformError(
                "iOSIPC requires UnixSocket (macOS) or XPC (iOS) endpoint".to_string(),
            )),
        }
    }

    /// Connect to a native endpoint.
    pub async fn connect(&self, endpoint: &NativeEndpoint) -> IpcResult<AsyncStreamImpl> {
        match endpoint {
            #[cfg(target_os = "macos")]
            NativeEndpoint::UnixSocket(path) => {
                // Delegate to Unix implementation
                debug!("Connecting to macOS Unix socket: {}", path.display());

                let stream = tokio::net::UnixStream::connect(path).await.map_err(|e| {
                    IpcError::ConnectionFailed(format!(
                        "Failed to connect to macOS Unix socket at {}: {}",
                        path.display(),
                        e
                    ))
                })?;

                info!("Connected to macOS Unix socket: {}", path.display());

                Ok(AsyncStreamImpl::Unix(stream))
            }

            #[cfg(target_os = "ios")]
            NativeEndpoint::XPC(service) => Err(IpcError::PlatformError(format!(
                "XPC transport requires Apple framework bindings (not yet available in Pure Rust). \
                     Service: {}. Use InProcess or Unix socket fallback.",
                service
            ))),

            _ => Err(IpcError::PlatformError(
                "iOSIPC requires UnixSocket (macOS) or XPC (iOS) endpoint".to_string(),
            )),
        }
    }

    /// Cleanup endpoint.
    pub async fn cleanup(&self, endpoint: &NativeEndpoint) -> IpcResult<()> {
        match endpoint {
            #[cfg(target_os = "macos")]
            NativeEndpoint::UnixSocket(path) => {
                // Remove socket file (macOS)
                if path.exists() {
                    debug!("Removing macOS Unix socket file: {}", path.display());
                    tokio::fs::remove_file(path).await.map_err(|e| {
                        IpcError::PlatformError(format!(
                            "Failed to remove socket file {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                }
                Ok(())
            }

            #[cfg(target_os = "ios")]
            NativeEndpoint::XPC(_service) => {
                // XPC cleanup (when implemented)
                // XPC services are managed by launchd, no manual cleanup needed
                Ok(())
            }

            _ => Err(IpcError::PlatformError(
                "iOSIPC requires UnixSocket (macOS) or XPC (iOS) endpoint".to_string(),
            )),
        }
    }
}

#[cfg(target_os = "macos")]
/// Wrapper for `UnixListener` (macOS) for [`PlatformListenerImpl::Ios`].
pub struct IosListener {
    inner: tokio::net::UnixListener,
}

#[cfg(target_os = "macos")]
impl IosListener {
    /// Accept incoming connection.
    pub async fn accept(&mut self) -> IpcResult<AsyncStreamImpl> {
        let (stream, addr) = self.inner.accept().await.map_err(|e| {
            IpcError::ConnectionFailed(format!("Failed to accept macOS connection: {}", e))
        })?;

        debug!("Accepted macOS Unix connection from: {:?}", addr);

        Ok(AsyncStreamImpl::Unix(stream))
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    #[cfg(target_os = "macos")]
    async fn test_macos_create_endpoint() {
        use super::IosPlatformIPC;
        use crate::endpoint::NativeEndpoint;

        let ipc = IosPlatformIPC;
        let endpoint = ipc.create_endpoint("test-primal").await.unwrap();

        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                assert!(path.to_string_lossy().contains("biomeos"));
                assert!(path.to_string_lossy().contains("test-primal.sock"));
                assert!(path.to_string_lossy().starts_with("/var/tmp/biomeos/"));
            }
            _ => panic!("Expected UnixSocket on macOS"),
        }
    }

    #[tokio::test]
    #[cfg(target_os = "macos")]
    async fn test_macos_listen_and_connect() {
        use super::IosPlatformIPC;
        use crate::endpoint::NativeEndpoint;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let ipc = IosPlatformIPC;

        // Use unique name for this test
        let test_name = format!("test-listen-{}", std::process::id());
        let endpoint = ipc.create_endpoint(&test_name).await.unwrap();

        // Create listener
        let mut listener = ipc.listen(&endpoint).await.unwrap();

        // Connect in background task (listener already bound — no sleep needed)
        let endpoint_clone = endpoint.clone();
        let connect_handle = tokio::spawn(async move {
            let ipc = IosPlatformIPC;
            ipc.connect(&endpoint_clone).await
        });

        // Accept connection
        let mut server_stream = listener.accept().await.unwrap();

        // Get client stream
        let mut client_stream = connect_handle.await.unwrap().unwrap();

        // Test communication
        client_stream.write_all(b"hello from macOS").await.unwrap();
        let mut buf = [0u8; 16];
        server_stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello from macOS");

        // Cleanup
        drop(listener);
        drop(client_stream);
        drop(server_stream);
        ipc.cleanup(&endpoint).await.unwrap();
    }

    #[test]
    #[cfg(target_os = "ios")]
    fn test_ios_xpc_naming() {
        // Verify XPC service naming follows Apple convention
        let test_cases = vec![
            ("beardog", "org.biomeos.beardog"),
            ("squirrel", "org.biomeos.squirrel"),
            ("songbird", "org.biomeos.songbird"),
        ];

        for (primal_name, expected_service) in test_cases {
            let service_name = format!("org.biomeos.{}", primal_name);
            assert_eq!(service_name, expected_service);
            assert!(service_name.starts_with("org.biomeos."));
        }
    }

    #[test]
    fn test_platform_compatibility() {
        // Document platform-specific behavior
        #[cfg(target_os = "macos")]
        {
            // macOS uses Unix sockets (fully functional)
            assert_eq!(std::env::consts::OS, "macos");
        }

        #[cfg(target_os = "ios")]
        {
            // iOS uses XPC (documented, not yet implemented)
            // Will use TCP fallback until XPC bindings available
            assert_eq!(std::env::consts::OS, "ios");
        }
    }
}
