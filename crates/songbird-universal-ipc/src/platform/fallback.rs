// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Fallback TCP localhost implementation
//!
//! For platforms that don't have Unix domain sockets or Windows named pipes,
//! we fall back to TCP localhost connections.

use crate::endpoint::NativeEndpoint;
use crate::error::{IpcError, IpcResult};
use crate::platform::{AsyncStreamImpl, PlatformListenerImpl};
use songbird_types::constants::LOCALHOST;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, warn};

/// Fallback TCP localhost IPC implementation
pub struct FallbackPlatformIPC;

/// Port counter for automatic port assignment
static PORT_COUNTER: AtomicU16 = AtomicU16::new(50000);

impl FallbackPlatformIPC {
    /// Create a native endpoint for the given primal name.
    pub async fn create_endpoint(&self, primal_name: &str) -> IpcResult<NativeEndpoint> {
        // Assign a unique port for this primal
        let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);

        debug!("Creating TCP localhost endpoint for '{}': port {}", primal_name, port);

        warn!("Using fallback TCP localhost for '{}' - platform lacks native IPC", primal_name);

        Ok(NativeEndpoint::TcpLocal(port))
    }

    /// Create a listener on the native endpoint.
    pub async fn listen(&self, endpoint: &NativeEndpoint) -> IpcResult<PlatformListenerImpl> {
        match endpoint {
            NativeEndpoint::TcpLocal(port) => {
                debug!("Creating TCP listener on: {LOCALHOST}:{port}");

                let listener =
                    TcpListener::bind(format!("{LOCALHOST}:{port}")).await.map_err(|e| {
                        IpcError::ListenerFailed(format!(
                            "Failed to bind TCP localhost at port {port}: {e}"
                        ))
                    })?;

                info!("TCP localhost listener created: {LOCALHOST}:{port}");

                Ok(PlatformListenerImpl::Fallback(FallbackListener {
                    inner: listener,
                }))
            }
            _ => {
                Err(IpcError::PlatformError(String::from("FallbackIPC requires TcpLocal endpoint")))
            }
        }
    }

    /// Connect to a native endpoint.
    pub async fn connect(&self, endpoint: &NativeEndpoint) -> IpcResult<AsyncStreamImpl> {
        match endpoint {
            NativeEndpoint::TcpLocal(port) => {
                debug!("Connecting to TCP localhost: {LOCALHOST}:{port}");

                let stream =
                    TcpStream::connect(format!("{LOCALHOST}:{port}")).await.map_err(|e| {
                        IpcError::ConnectionFailed(format!(
                            "Failed to connect to TCP localhost at port {port}: {e}"
                        ))
                    })?;

                info!("Connected to TCP localhost: {LOCALHOST}:{port}");

                Ok(AsyncStreamImpl::Tcp(stream))
            }
            _ => {
                Err(IpcError::PlatformError(String::from("FallbackIPC requires TcpLocal endpoint")))
            }
        }
    }

    /// Cleanup endpoint.
    pub async fn cleanup(&self, _endpoint: &NativeEndpoint) -> IpcResult<()> {
        // TCP sockets don't need cleanup
        Ok(())
    }
}

/// Wrapper for `TcpListener` for [`PlatformListenerImpl::Fallback`].
pub struct FallbackListener {
    inner: TcpListener,
}

impl FallbackListener {
    /// Accept incoming connection.
    pub async fn accept(&mut self) -> IpcResult<AsyncStreamImpl> {
        let (stream, addr) = self.inner.accept().await.map_err(|e| {
            IpcError::ConnectionFailed(format!("Failed to accept TCP connection: {e}"))
        })?;

        debug!("Accepted TCP connection from: {}", addr);

        Ok(AsyncStreamImpl::Tcp(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_fallback_create_endpoint() {
        let ipc = FallbackPlatformIPC;
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
        let ipc = FallbackPlatformIPC;
        let endpoint = ipc.create_endpoint("test-listen").await.unwrap();

        // Create listener
        let mut listener = ipc.listen(&endpoint).await.unwrap();

        // Connect in background task
        let endpoint_clone = endpoint.clone();
        let connect_handle = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let ipc = FallbackPlatformIPC;
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
