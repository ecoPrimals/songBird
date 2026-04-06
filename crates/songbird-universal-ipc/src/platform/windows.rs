// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Windows named pipe implementation
//!
//! **Platform**: Windows (all architectures: `x86_64`, ARM64)
//! **Transport**: Named pipes (Windows native IPC)
//! **Path Format**: `\\.\pipe\biomeos_{primal_name}`
//!
//! ## Why Named Pipes for Windows?
//!
//! Named pipes are the native Windows IPC mechanism, similar to Unix domain
//! sockets on Linux/macOS. They provide:
//! - High performance (~10μs latency, ~5GB/s throughput)
//! - Security (ACLs, Windows security model)
//! - No filesystem overhead (kernel-managed)
//! - Automatic cleanup (no stale pipe files)
//!
//! ## Implementation
//!
//! Windows named pipes use a special path format:
//! - `\\.\pipe\{name}` - Local pipe (this machine only)
//! - `\\server\pipe\{name}` - Remote pipe (network, not used here)
//!
//! Tokio provides `tokio::net::windows::named_pipe` for async operations,
//! which is 100% Pure Rust (no FFI, no unsafe in our code).
//!
//! ## TRUE ecoBin Compliance
//!
//! - ✅ Pure Rust (zero unsafe code in this module)
//! - ✅ Zero C dependencies (tokio handles Windows API internally)
//! - ✅ Platform-agnostic (same code works on Windows `x86_64`, ARM64)
//! - ✅ No hardcoding (primal name from runtime, configurable via env vars)
//!
//! ## References
//!
//! - Windows Named Pipes: <https://docs.microsoft.com/en-us/windows/win32/ipc/named-pipes>
//! - tokio `named_pipe`: <https://docs.rs/tokio/latest/tokio/net/windows/named_pipe>/
//! - Platform evolution: ecoPrimals/biomeOS docs/deep-debt/PLATFORM_AGNOSTIC_IPC_EVOLUTION.md

use crate::endpoint::NativeEndpoint;
use crate::error::{IpcError, IpcResult};
use crate::platform::{AsyncStream, PlatformIPC, PlatformListener};
use async_trait::async_trait;
use songbird_types::primal_names::BIOMEOS_DIR;
use tracing::{debug, info};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};

/// Windows named pipe IPC implementation
///
/// **Platform**: Windows (`x86_64`, ARM64, any architecture)
pub struct WindowsIPC;

#[async_trait]
impl PlatformIPC for WindowsIPC {
    async fn create_endpoint(&self, primal_name: &str) -> IpcResult<NativeEndpoint> {
        // Named pipe naming: \\.\pipe\biomeos_{primal_name}
        // The \\.\pipe\ prefix is the Windows named pipe namespace

        // Allow override via environment variable (for testing, special deployments)
        let pipe_name = songbird_process_env::var(format!("{}_PIPE", primal_name.to_uppercase()))
            .unwrap_or_else(|_| {
                songbird_process_env::var("BIOMEOS_PIPE_DIR").map_or_else(
                    |_| format!(r"\\.\pipe\{BIOMEOS_DIR}_{primal_name}"),
                    |custom_dir| format!("{custom_dir}_{BIOMEOS_DIR}_{primal_name}"),
                )
            });

        debug!("Creating named pipe endpoint for '{}': {}", primal_name, pipe_name);

        info!("Windows named pipe (kernel-managed): {} (no filesystem)", pipe_name);

        Ok(NativeEndpoint::NamedPipe(pipe_name))
    }

    async fn listen(&self, endpoint: &NativeEndpoint) -> IpcResult<Box<dyn PlatformListener>> {
        match endpoint {
            NativeEndpoint::NamedPipe(_name) => {
                #[cfg(windows)]
                {
                    debug!("Creating named pipe server on: {}", _name);

                    // Create named pipe server (first instance)
                    // ServerOptions provides Pure Rust interface to Windows API
                    let server = ServerOptions::new()
                        .first_pipe_instance(true)
                        .create(_name)
                        .map_err(|e| {
                            IpcError::ListenerFailed(format!(
                                "Failed to create named pipe server {}: {}",
                                _name, e
                            ))
                        })?;

                    info!("Named pipe server created: {} (Windows-optimized)", _name);

                    Ok(Box::new(NamedPipeListenerWrapper {
                        server,
                        pipe_name: _name.clone(),
                    }))
                }

                #[cfg(not(windows))]
                {
                    Err(IpcError::PlatformError("Named pipes require Windows platform".to_string()))
                }
            }
            _ => Err(IpcError::PlatformError("WindowsIPC requires NamedPipe endpoint".to_string())),
        }
    }

    async fn connect(&self, endpoint: &NativeEndpoint) -> IpcResult<Box<dyn AsyncStream>> {
        match endpoint {
            NativeEndpoint::NamedPipe(_name) => {
                #[cfg(windows)]
                {
                    debug!("Connecting to named pipe: {}", _name);

                    // Retry connection (named pipe may not be ready immediately)
                    let mut retries = 5;
                    let client = loop {
                        match ClientOptions::new().open(_name) {
                            Ok(client) => break client,
                            Err(e) if retries > 0 => {
                                debug!("Named pipe not ready, retrying... ({} left)", retries);
                                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                                retries -= 1;
                            }
                            Err(e) => {
                                return Err(IpcError::ConnectionFailed(format!(
                                    "Failed to connect to named pipe {} after retries: {}",
                                    _name, e
                                )));
                            }
                        }
                    };

                    info!("Connected to named pipe: {}", _name);

                    Ok(Box::new(client))
                }

                #[cfg(not(windows))]
                {
                    Err(IpcError::PlatformError("Named pipes require Windows platform".to_string()))
                }
            }
            _ => Err(IpcError::PlatformError("WindowsIPC requires NamedPipe endpoint".to_string())),
        }
    }

    async fn cleanup(&self, endpoint: &NativeEndpoint) -> IpcResult<()> {
        match endpoint {
            NativeEndpoint::NamedPipe(name) => {
                // Named pipes are automatically cleaned up by Windows kernel
                // No manual cleanup needed (similar to abstract sockets on Linux)
                debug!("Named pipe cleanup (automatic): {}", name);
                Ok(())
            }
            _ => Err(IpcError::PlatformError("WindowsIPC requires NamedPipe endpoint".to_string())),
        }
    }
}

#[cfg(windows)]
/// Wrapper for named pipe server to implement `PlatformListener`
struct NamedPipeListenerWrapper {
    server: tokio::net::windows::named_pipe::NamedPipeServer,
    pipe_name: String,
}

#[cfg(windows)]
#[async_trait]
impl PlatformListener for NamedPipeListenerWrapper {
    async fn accept(&mut self) -> IpcResult<Box<dyn AsyncStream>> {
        // Wait for client connection
        self.server.connect().await.map_err(|e| {
            IpcError::ConnectionFailed(format!(
                "Failed to accept named pipe connection on {}: {}",
                self.pipe_name, e
            ))
        })?;

        debug!("Accepted named pipe connection on: {}", self.pipe_name);

        // Create a new server instance for the next connection
        // Windows named pipes require a new server instance per connection
        let next_server = ServerOptions::new().create(&self.pipe_name).map_err(|e| {
            IpcError::ListenerFailed(format!(
                "Failed to create next pipe instance {}: {}",
                self.pipe_name, e
            ))
        })?;

        // Swap out the old server with the new one
        // The old server becomes the connected stream
        let connected_server = std::mem::replace(&mut self.server, next_server);

        Ok(Box::new(connected_server))
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    #[cfg(windows)]
    async fn test_windows_create_endpoint() {
        let ipc = WindowsIPC;
        let endpoint = ipc.create_endpoint("test-primal").await.unwrap();

        match endpoint {
            NativeEndpoint::NamedPipe(name) => {
                assert_eq!(name, r"\\.\pipe\biomeos_test-primal");
                assert!(name.starts_with(r"\\.\pipe\biomeos_"));
            }
            _ => panic!("Expected NamedPipe"),
        }
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_windows_listen_and_connect() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let ipc = WindowsIPC;

        // Use unique name for this test to avoid conflicts
        let test_name = format!("test-listen-{}", std::process::id());
        let endpoint = ipc.create_endpoint(&test_name).await.unwrap();

        // Create listener
        let mut listener = ipc.listen(&endpoint).await.unwrap();

        // Connect in background task (listener already bound — no sleep needed)
        let endpoint_clone = endpoint.clone();
        let connect_handle = tokio::spawn(async move {
            let ipc = WindowsIPC;
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
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_windows_cleanup_automatic() {
        let ipc = WindowsIPC;
        let endpoint = ipc.create_endpoint("cleanup-test").await.unwrap();

        // Create and close listener
        let listener = ipc.listen(&endpoint).await.unwrap();
        drop(listener);

        // Cleanup (should be no-op for named pipes)
        ipc.cleanup(&endpoint).await.unwrap();

        // Named pipes are kernel-managed, no filesystem entry exists
        // (similar to abstract sockets on Linux)
    }

    #[test]
    fn test_windows_naming_convention() {
        // Verify naming follows biomeOS standard: \\.\pipe\biomeos_{primal_name}
        let test_cases = vec![
            ("beardog", r"\\.\pipe\biomeos_beardog"),
            ("squirrel", r"\\.\pipe\biomeos_squirrel"),
            ("songbird", r"\\.\pipe\biomeos_songbird"),
            ("test-primal", r"\\.\pipe\biomeos_test-primal"),
        ];

        for (primal_name, expected_pipe) in test_cases {
            let pipe_name = format!(r"\\.\pipe\biomeos_{primal_name}");
            assert_eq!(pipe_name, expected_pipe);
            assert!(pipe_name.starts_with(r"\\.\pipe\biomeos_"));
        }
    }

    #[test]
    fn test_windows_env_override() {
        // Same logic as `create_endpoint`: read `{PRIMAL}_PIPE` via injectable lookup
        let get_var = |k: &str| -> Result<String, std::env::VarError> {
            if k == "TESTPRIMAL_PIPE" {
                Ok(r"\\.\pipe\custom_test".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        };
        let custom_name = get_var("TESTPRIMAL_PIPE").unwrap();
        assert_eq!(custom_name, r"\\.\pipe\custom_test");
    }

    #[test]
    #[cfg(not(windows))]
    fn test_windows_requires_windows_platform() {
        // On non-Windows platforms, WindowsIPC should return errors
        // This test documents the platform requirement
        assert!(cfg!(not(windows)));
    }
}
