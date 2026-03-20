// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Fallback TCP localhost implementation
//!
//! For platforms that don't have Unix domain sockets or Windows named pipes,
//! we fall back to TCP localhost connections.

use crate::endpoint::NativeEndpoint;
use crate::error::{IpcError, IpcResult};
use crate::platform::{AsyncStream, PlatformIPC, PlatformListener};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, warn};

/// Fallback TCP localhost IPC implementation
pub struct FallbackIPC;

/// Port counter for automatic port assignment
static PORT_COUNTER: AtomicU16 = AtomicU16::new(50000);

#[async_trait]
impl PlatformIPC for FallbackIPC {
    async fn create_endpoint(&self, primal_name: &str) -> IpcResult<NativeEndpoint> {
        // Assign a unique port for this primal
        let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);

        debug!("Creating TCP localhost endpoint for '{}': port {}", primal_name, port);

        warn!("Using fallback TCP localhost for '{}' - platform lacks native IPC", primal_name);

        Ok(NativeEndpoint::TcpLocal(port))
    }

    async fn listen(&self, endpoint: &NativeEndpoint) -> IpcResult<Box<dyn PlatformListener>> {
        match endpoint {
            NativeEndpoint::TcpLocal(port) => {
                debug!("Creating TCP listener on: 127.0.0.1:{}", port);

                let listener =
                    TcpListener::bind(format!("127.0.0.1:{port}")).await.map_err(|e| {
                        IpcError::ListenerFailed(format!(
                            "Failed to bind TCP localhost at port {port}: {e}"
                        ))
                    })?;

                info!("TCP localhost listener created: 127.0.0.1:{}", port);

                Ok(Box::new(TcpListenerWrapper {
                    inner: listener,
                }))
            }
            _ => Err(IpcError::PlatformError("FallbackIPC requires TcpLocal endpoint".to_string())),
        }
    }

    async fn connect(&self, endpoint: &NativeEndpoint) -> IpcResult<Box<dyn AsyncStream>> {
        match endpoint {
            NativeEndpoint::TcpLocal(port) => {
                debug!("Connecting to TCP localhost: 127.0.0.1:{}", port);

                let stream =
                    TcpStream::connect(format!("127.0.0.1:{port}")).await.map_err(|e| {
                        IpcError::ConnectionFailed(format!(
                            "Failed to connect to TCP localhost at port {port}: {e}"
                        ))
                    })?;

                info!("Connected to TCP localhost: 127.0.0.1:{}", port);

                Ok(Box::new(stream))
            }
            _ => Err(IpcError::PlatformError("FallbackIPC requires TcpLocal endpoint".to_string())),
        }
    }

    async fn cleanup(&self, _endpoint: &NativeEndpoint) -> IpcResult<()> {
        // TCP sockets don't need cleanup
        Ok(())
    }
}

/// Wrapper for `TcpListener` to implement `PlatformListener`
struct TcpListenerWrapper {
    inner: TcpListener,
}

#[async_trait]
impl PlatformListener for TcpListenerWrapper {
    async fn accept(&mut self) -> IpcResult<Box<dyn AsyncStream>> {
        let (stream, addr) = self.inner.accept().await.map_err(|e| {
            IpcError::ConnectionFailed(format!("Failed to accept TCP connection: {e}"))
        })?;

        debug!("Accepted TCP connection from: {}", addr);

        Ok(Box::new(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_fallback_create_endpoint() {
        let ipc = FallbackIPC;
        let endpoint = ipc.create_endpoint("test-primal").await.unwrap();

        match endpoint {
            NativeEndpoint::TcpLocal(port) => {
                assert!(port >= 50000);
            }
            _ => panic!("Expected TcpLocal"),
        }
    }

    #[tokio::test]
    async fn test_fallback_listen_and_connect() {
        let ipc = FallbackIPC;
        let endpoint = ipc.create_endpoint("test-listen").await.unwrap();

        // Create listener
        let mut listener = ipc.listen(&endpoint).await.unwrap();

        // Connect in background task
        let endpoint_clone = endpoint.clone();
        let connect_handle = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let ipc = FallbackIPC;
            ipc.connect(&endpoint_clone).await
        });

        // Accept connection
        let mut server_stream = listener.accept().await.unwrap();

        // Get client stream
        let mut client_stream = connect_handle.await.unwrap().unwrap();

        // Test communication
        client_stream.write_all(b"hello").await.unwrap();
        let mut buf = [0u8; 5];
        server_stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello");
    }
}
