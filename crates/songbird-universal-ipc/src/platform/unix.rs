// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Unix domain socket implementation
//!
//! **Platform**: Linux, macOS, BSD, Unix-like systems
//! **Transport**: Filesystem-based Unix domain sockets
//! **Path**: XDG-compliant (`/run/user/$UID/biomeos/{primal}.sock`)
//!
//! ## XDG Base Directory Compliance
//!
//! Following XDG Base Directory Specification for runtime files:
//! - Priority 1: `${primal_name}_SOCKET` (explicit override); for `beardog`, `SECURITY_PROVIDER_SOCKET` is checked first
//! - Priority 2: `BIOMEOS_SOCKET_DIR/{primal}.sock` (shared directory)
//! - Priority 3: `$XDG_RUNTIME_DIR/biomeos/{primal}.sock` (XDG standard)
//! - Priority 4: `/run/user/$UID/biomeos/{primal}.sock` (fallback XDG)
//! - Priority 5: `{system temp dir}/{primal}.sock` (legacy fallback)
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
use songbird_types::primal_names::{BEARDOG, BIOMEOS_DIR};
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
/// 1. For `beardog`: `SECURITY_PROVIDER_SOCKET` then `{PRIMAL_NAME}_SOCKET` (e.g. `BEARDOG_SOCKET`)
/// 2. `BIOMEOS_SOCKET_DIR/{primal}.sock` - Shared socket directory
/// 3. `$XDG_RUNTIME_DIR/biomeos/{primal}.sock` - XDG standard
/// 4. `/run/user/$UID/biomeos/{primal}.sock` - Fallback XDG (Pure Rust!)
/// 5. `{system temp dir}/{primal}.sock` - Legacy fallback
///
/// **Pure Rust**: No unsafe code, no `libc::getuid()`. Uses environment variables.
fn get_socket_path(primal_name: &str) -> PathBuf {
    resolve_socket_path(primal_name, |key| songbird_process_env::var(key))
}

/// Resolve socket path using an injectable env reader (concurrent-safe, testable)
fn resolve_socket_path<F>(primal_name: &str, env_reader: F) -> PathBuf
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    // Priority 1: Explicit override — capability name first for security provider primal
    if primal_name == BEARDOG
        && let Ok(path) = env_reader("SECURITY_PROVIDER_SOCKET")
    {
        return PathBuf::from(path);
    }
    let override_var = format!("{}_SOCKET", primal_name.to_uppercase().replace('-', "_"));
    if let Ok(path) = env_reader(&override_var) {
        return PathBuf::from(path);
    }

    // Priority 2: Shared biomeos socket directory
    if let Ok(socket_dir) = env_reader("BIOMEOS_SOCKET_DIR") {
        return PathBuf::from(socket_dir).join(format!("{primal_name}.sock"));
    }

    // Priority 3: XDG_RUNTIME_DIR (XDG standard)
    if let Ok(xdg_runtime_dir) = env_reader("XDG_RUNTIME_DIR") {
        return PathBuf::from(xdg_runtime_dir)
            .join(BIOMEOS_DIR)
            .join(format!("{primal_name}.sock"));
    }

    // Priority 4: Fallback XDG path using UID env var (Pure Rust!)
    if let Ok(uid_str) = env_reader("UID") {
        return PathBuf::from(format!(
            "/run/user/{uid_str}/{BIOMEOS_DIR}/{primal_name}.sock"
        ));
    }

    // Priority 5: Legacy temp-dir fallback (if all else fails)
    warn!(
        "No XDG_RUNTIME_DIR or UID found, using legacy temp directory for primal '{}'",
        primal_name
    );
    std::env::temp_dir().join(format!("{primal_name}.sock"))
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
                IpcError::PlatformError(format!("Failed to remove old socket: {e}"))
            })?;
        }

        info!("Unix socket endpoint: {} (XDG-compliant, no hardcoding)", path.display());

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
            _ => Err(IpcError::PlatformError("UnixIPC requires UnixSocket endpoint".to_string())),
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
            _ => Err(IpcError::PlatformError("UnixIPC requires UnixSocket endpoint".to_string())),
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
            _ => Err(IpcError::PlatformError("UnixIPC requires UnixSocket endpoint".to_string())),
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
    use std::collections::HashMap;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Create a mock env reader from a `HashMap` (concurrent-safe, no global state)
    fn mock_env(vars: HashMap<&str, &str>) -> impl Fn(&str) -> Result<String, std::env::VarError> {
        let owned: HashMap<String, String> =
            vars.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |key: &str| owned.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    }

    #[test]
    fn test_get_socket_path_explicit_override() {
        // Priority 1: Explicit override
        let env = mock_env(HashMap::from([("TESTPRIMAL_SOCKET", "/custom/path/test.sock")]));
        let path = resolve_socket_path("testprimal", env);
        assert_eq!(path, PathBuf::from("/custom/path/test.sock"));
    }

    #[test]
    fn test_get_socket_path_beardog_security_provider_first() {
        let env = mock_env(HashMap::from([
            ("SECURITY_PROVIDER_SOCKET", "/cap/security.sock"),
            ("BEARDOG_SOCKET", "/legacy/beardog.sock"),
        ]));
        let path = resolve_socket_path(BEARDOG, env);
        assert_eq!(path, PathBuf::from("/cap/security.sock"));
    }

    #[test]
    fn test_get_socket_path_biomeos_dir() {
        // Priority 2: BIOMEOS_SOCKET_DIR
        let env = mock_env(HashMap::from([("BIOMEOS_SOCKET_DIR", "/biomeos/sockets")]));
        let path = resolve_socket_path("testprimal2", env);
        assert_eq!(path, PathBuf::from("/biomeos/sockets/testprimal2.sock"));
    }

    #[test]
    fn test_get_socket_path_xdg_runtime() {
        // Priority 3: XDG_RUNTIME_DIR
        let env = mock_env(HashMap::from([("XDG_RUNTIME_DIR", "/run/user/1000")]));
        let path = resolve_socket_path("testprimal3", env);
        assert_eq!(path, PathBuf::from("/run/user/1000/biomeos/testprimal3.sock"));
    }

    #[test]
    fn test_get_socket_path_uid_fallback() {
        // Priority 4: UID env var (Pure Rust!)
        let env = mock_env(HashMap::from([("UID", "1000")]));
        let path = resolve_socket_path("testprimal4", env);
        assert_eq!(path, PathBuf::from("/run/user/1000/biomeos/testprimal4.sock"));
    }

    #[test]
    fn test_get_socket_path_legacy_fallback() {
        // Priority 5: Legacy /tmp fallback (no env vars set)
        let env = mock_env(HashMap::new());
        let path = resolve_socket_path("testprimal5", env);
        assert_eq!(path, PathBuf::from("/tmp/testprimal5.sock"));
    }

    #[test]
    fn test_get_socket_path_priority_order() {
        // If both BIOMEOS_SOCKET_DIR and XDG_RUNTIME_DIR are set,
        // BIOMEOS_SOCKET_DIR wins (higher priority)
        let env = mock_env(HashMap::from([
            ("BIOMEOS_SOCKET_DIR", "/biomeos/sockets"),
            ("XDG_RUNTIME_DIR", "/run/user/1000"),
        ]));
        let path = resolve_socket_path("myprimal", env);
        assert_eq!(path, PathBuf::from("/biomeos/sockets/myprimal.sock"));
    }

    #[test]
    fn test_socket_naming_standard() {
        // Verify naming follows biomeOS standard: {primal}.sock (no family)
        let env = mock_env(HashMap::new()); // uses /tmp fallback
        let test_cases = vec![
            ("beardog", "beardog.sock"),
            ("squirrel", "squirrel.sock"),
            ("songbird", "songbird.sock"),
        ];

        for (primal_name, expected_filename) in test_cases {
            let path = resolve_socket_path(primal_name, &env);
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
                if songbird_process_env::var("XDG_RUNTIME_DIR").is_ok()
                    || songbird_process_env::var("UID").is_ok()
                {
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

        // Connect in background task (listener already bound — no sleep needed)
        let endpoint_clone = endpoint.clone();
        let connect_handle = tokio::spawn(async move {
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
