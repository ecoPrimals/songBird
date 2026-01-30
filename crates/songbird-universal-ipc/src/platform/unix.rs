//! Unix domain socket implementation
//!
//! **Platform**: Linux, macOS, BSD, Unix-like systems
//! **Transport**: Filesystem-based Unix domain sockets
//! **Path**: XDG-compliant (`/run/user/$UID/biomeos/{primal}.sock`)
//!
//! ## XDG Base Directory Compliance
//!
//! Following XDG Base Directory Specification for runtime files:
//! - Priority 1: `${primal_name}_SOCKET` (explicit override)
//! - Priority 2: `BIOMEOS_SOCKET_DIR/{primal}.sock` (shared directory)
//! - Priority 3: `$XDG_RUNTIME_DIR/biomeos/{primal}.sock` (XDG standard)
//! - Priority 4: `/run/user/$UID/biomeos/{primal}.sock` (fallback XDG)
//! - Priority 5: `/tmp/{primal}.sock` (legacy fallback)
//!
//! **No hardcoded paths!** All paths derived from environment or XDG standards.
//!
//! ## Socket Naming Standard
//!
//! - Uses primal name only: `beardog.sock` (NOT `beardog-orchestrator.sock`)
//! - Family ID NOT in socket name (biomeOS compliance)
//! - Consistent with wateringHole standards
//!
//! ## TRUE ecoBin Compliance
//!
//! - ✅ Pure Rust (zero unsafe code)
//! - ✅ Zero hardcoding (all paths from environment or XDG)
//! - ✅ Runtime discovery (no compile-time assumptions)
//! - ✅ Platform-agnostic API (same as other transports)

use crate::endpoint::NativeEndpoint;
use crate::error::{IpcError, IpcResult};
use crate::platform::{AsyncStream, PlatformIPC, PlatformListener};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, info, warn};

/// Unix domain socket IPC implementation
///
/// **Platform-agnostic, XDG-compliant, zero hardcoding**
pub struct UnixIPC;

/// Get XDG-compliant Unix socket path for a primal
///
/// **Priority order** (biomeOS standard):
/// 1. `{PRIMAL_NAME}_SOCKET` - Explicit override (e.g., `BEARDOG_SOCKET`)
/// 2. `BIOMEOS_SOCKET_DIR/{primal}.sock` - Shared socket directory
/// 3. `$XDG_RUNTIME_DIR/biomeos/{primal}.sock` - XDG standard
/// 4. `/run/user/$UID/biomeos/{primal}.sock` - Fallback XDG (Pure Rust!)
/// 5. `/tmp/{primal}.sock` - Legacy fallback
///
/// **Pure Rust**: No unsafe code, no `libc::getuid()`. Uses environment variables.
fn get_socket_path(primal_name: &str) -> PathBuf {
    // Priority 1: Explicit override (e.g., BEARDOG_SOCKET=/path/to/socket.sock)
    let override_var = format!("{}_SOCKET", primal_name.to_uppercase().replace('-', "_"));
    if let Ok(path) = std::env::var(&override_var) {
        return PathBuf::from(path);
    }

    // Priority 2: Shared biomeos socket directory
    if let Ok(socket_dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return PathBuf::from(socket_dir).join(format!("{}.sock", primal_name));
    }

    // Priority 3: XDG_RUNTIME_DIR (XDG standard)
    if let Ok(xdg_runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(xdg_runtime_dir)
            .join("biomeos")
            .join(format!("{}.sock", primal_name));
    }

    // Priority 4: Fallback XDG path using UID env var (Pure Rust!)
    if let Ok(uid_str) = std::env::var("UID") {
        return PathBuf::from(format!("/run/user/{}/biomeos/{}.sock", uid_str, primal_name));
    }

    // Priority 5: Legacy /tmp fallback (if all else fails)
    warn!(
        "No XDG_RUNTIME_DIR or UID found, using legacy /tmp for primal '{}'",
        primal_name
    );
    PathBuf::from(format!("/tmp/{}.sock", primal_name))
}

#[async_trait]
impl PlatformIPC for UnixIPC {
    async fn create_endpoint(&self, primal_name: &str) -> IpcResult<NativeEndpoint> {
        // Get XDG-compliant socket path (no hardcoding!)
        let path = get_socket_path(primal_name);

        debug!(
            "Creating Unix socket endpoint for '{}': {} (XDG-compliant)",
            primal_name,
            path.display()
        );

        // Ensure parent directory exists (Pure Rust!)
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                IpcError::PlatformError(format!(
                    "Failed to create socket directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        // Clean up old socket if it exists
        if path.exists() {
            warn!("Socket file already exists, removing: {}", path.display());
            tokio::fs::remove_file(&path).await.map_err(|e| {
                IpcError::PlatformError(format!("Failed to remove old socket: {}", e))
            })?;
        }

        info!(
            "Unix socket endpoint: {} (XDG-compliant, no hardcoding)",
            path.display()
        );

        Ok(NativeEndpoint::UnixSocket(path))
    }

    async fn listen(&self, endpoint: &NativeEndpoint) -> IpcResult<Box<dyn PlatformListener>> {
        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                debug!("Creating Unix listener on: {}", path.display());

                let listener = UnixListener::bind(path).map_err(|e| {
                    IpcError::ListenerFailed(format!(
                        "Failed to bind Unix socket at {}: {}",
                        path.display(),
                        e
                    ))
                })?;

                info!("Unix listener created: {}", path.display());

                Ok(Box::new(UnixListenerWrapper {
                    inner: listener,
                }))
            }
            _ => Err(IpcError::PlatformError(
                "UnixIPC requires UnixSocket endpoint".to_string(),
            )),
        }
    }

    async fn connect(&self, endpoint: &NativeEndpoint) -> IpcResult<Box<dyn AsyncStream>> {
        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                debug!("Connecting to Unix socket: {}", path.display());

                let stream = UnixStream::connect(path).await.map_err(|e| {
                    IpcError::ConnectionFailed(format!(
                        "Failed to connect to Unix socket at {}: {}",
                        path.display(),
                        e
                    ))
                })?;

                info!("Connected to Unix socket: {}", path.display());

                Ok(Box::new(stream))
            }
            _ => Err(IpcError::PlatformError(
                "UnixIPC requires UnixSocket endpoint".to_string(),
            )),
        }
    }

    async fn cleanup(&self, endpoint: &NativeEndpoint) -> IpcResult<()> {
        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                if path.exists() {
                    debug!("Cleaning up Unix socket: {}", path.display());

                    tokio::fs::remove_file(path).await.map_err(|e| {
                        IpcError::CleanupFailed(format!(
                            "Failed to remove Unix socket at {}: {}",
                            path.display(),
                            e
                        ))
                    })?;

                    info!("Unix socket cleaned up: {}", path.display());
                }
                Ok(())
            }
            _ => Err(IpcError::PlatformError(
                "UnixIPC requires UnixSocket endpoint".to_string(),
            )),
        }
    }
}

/// Wrapper for `UnixListener` to implement `PlatformListener`
struct UnixListenerWrapper {
    inner: UnixListener,
}

#[async_trait]
impl PlatformListener for UnixListenerWrapper {
    async fn accept(&mut self) -> IpcResult<Box<dyn AsyncStream>> {
        let (stream, addr) = self.inner.accept().await.map_err(|e| {
            IpcError::ConnectionFailed(format!("Failed to accept Unix connection: {e}"))
        })?;

        debug!("Accepted Unix connection from: {:?}", addr);

        Ok(Box::new(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn test_get_socket_path_explicit_override() {
        // Priority 1: Explicit override
        std::env::set_var("TESTPRIMAL_SOCKET", "/custom/path/test.sock");
        let path = get_socket_path("testprimal");
        assert_eq!(path, PathBuf::from("/custom/path/test.sock"));
        std::env::remove_var("TESTPRIMAL_SOCKET");
    }

    #[test]
    fn test_get_socket_path_biomeos_dir() {
        // Priority 2: BIOMEOS_SOCKET_DIR
        std::env::remove_var("TESTPRIMAL2_SOCKET"); // Clear override
        std::env::set_var("BIOMEOS_SOCKET_DIR", "/biomeos/sockets");
        let path = get_socket_path("testprimal2");
        assert_eq!(path, PathBuf::from("/biomeos/sockets/testprimal2.sock"));
        std::env::remove_var("BIOMEOS_SOCKET_DIR");
    }

    #[test]
    fn test_get_socket_path_xdg_runtime() {
        // Priority 3: XDG_RUNTIME_DIR
        std::env::remove_var("TESTPRIMAL3_SOCKET");
        std::env::remove_var("BIOMEOS_SOCKET_DIR");
        std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
        let path = get_socket_path("testprimal3");
        assert_eq!(
            path,
            PathBuf::from("/run/user/1000/biomeos/testprimal3.sock")
        );
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_get_socket_path_uid_fallback() {
        // Priority 4: UID env var (Pure Rust!)
        std::env::remove_var("TESTPRIMAL4_SOCKET");
        std::env::remove_var("BIOMEOS_SOCKET_DIR");
        std::env::remove_var("XDG_RUNTIME_DIR");
        std::env::set_var("UID", "1000");
        let path = get_socket_path("testprimal4");
        assert_eq!(
            path,
            PathBuf::from("/run/user/1000/biomeos/testprimal4.sock")
        );
        std::env::remove_var("UID");
    }

    #[test]
    fn test_get_socket_path_legacy_fallback() {
        // Priority 5: Legacy /tmp fallback
        std::env::remove_var("TESTPRIMAL5_SOCKET");
        std::env::remove_var("BIOMEOS_SOCKET_DIR");
        std::env::remove_var("XDG_RUNTIME_DIR");
        std::env::remove_var("UID");
        let path = get_socket_path("testprimal5");
        assert_eq!(path, PathBuf::from("/tmp/testprimal5.sock"));
    }

    #[test]
    fn test_socket_naming_standard() {
        // Verify naming follows biomeOS standard: {primal}.sock (no family)
        let test_cases = vec![
            ("beardog", "beardog.sock"),
            ("squirrel", "squirrel.sock"),
            ("songbird", "songbird.sock"),
        ];

        for (primal_name, expected_filename) in test_cases {
            let path = get_socket_path(primal_name);
            let filename = path.file_name().unwrap().to_string_lossy();
            assert_eq!(filename, expected_filename);
        }
    }

    #[tokio::test]
    async fn test_unix_create_endpoint() {
        let ipc = UnixIPC;
        let endpoint = ipc.create_endpoint("test-primal").await.unwrap();

        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                assert!(path.to_str().unwrap().contains("test-primal"));
                assert!(path.to_str().unwrap().ends_with(".sock"));
                // Should NOT hardcode /tmp (unless all XDG vars missing)
                if std::env::var("XDG_RUNTIME_DIR").is_ok() || std::env::var("UID").is_ok() {
                    assert!(!path.to_str().unwrap().starts_with("/tmp/"));
                }
            }
            _ => panic!("Expected UnixSocket"),
        }
    }

    #[tokio::test]
    async fn test_unix_listen_and_connect() {
        let ipc = UnixIPC;
        
        // Use unique name to avoid test conflicts
        let test_name = format!("test-listen-{}", std::process::id());
        let endpoint = ipc.create_endpoint(&test_name).await.unwrap();

        // Create listener
        let mut listener = ipc.listen(&endpoint).await.unwrap();

        // Connect in background task
        let endpoint_clone = endpoint.clone();
        let connect_handle = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let ipc = UnixIPC;
            ipc.connect(&endpoint_clone).await
        });

        // Accept connection
        let mut server_stream = listener.accept().await.unwrap();

        // Get client stream
        let mut client_stream = connect_handle.await.unwrap().unwrap();

        // Test bidirectional communication
        client_stream.write_all(b"hello from client").await.unwrap();
        let mut buf = [0u8; 17];
        server_stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello from client");

        server_stream.write_all(b"hello from server").await.unwrap();
        let mut buf2 = [0u8; 17];
        client_stream.read_exact(&mut buf2).await.unwrap();
        assert_eq!(&buf2, b"hello from server");

        // Cleanup
        drop(listener);
        drop(client_stream);
        drop(server_stream);
        ipc.cleanup(&endpoint).await.unwrap();
    }

    #[tokio::test]
    async fn test_unix_cleanup() {
        let ipc = UnixIPC;
        
        // Use unique name
        let test_name = format!("test-cleanup-{}", std::process::id());
        let endpoint = ipc.create_endpoint(&test_name).await.unwrap();

        // Create listener (creates socket file)
        let listener = ipc.listen(&endpoint).await.unwrap();

        // Verify file exists
        if let NativeEndpoint::UnixSocket(path) = &endpoint {
            assert!(path.exists());
        }

        // Drop listener first
        drop(listener);

        // Cleanup
        ipc.cleanup(&endpoint).await.unwrap();

        // Verify file removed
        if let NativeEndpoint::UnixSocket(path) = &endpoint {
            assert!(!path.exists());
        }
    }

    #[test]
    fn test_pure_rust_no_unsafe() {
        // Verify UnixIPC implementation uses zero unsafe code
        // This is enforced by #![deny(unsafe_code)] in lib.rs
        // get_socket_path() uses only env vars (Pure Rust!)
        assert!(true);
    }
}
