//! Unix domain socket implementation

use crate::endpoint::NativeEndpoint;
use crate::error::{IpcError, IpcResult};
use crate::platform::{AsyncStream, PlatformIPC, PlatformListener};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, info, warn};

/// Unix domain socket IPC implementation
pub struct UnixIPC;

#[async_trait]
impl PlatformIPC for UnixIPC {
    async fn create_endpoint(&self, primal_name: &str) -> IpcResult<NativeEndpoint> {
        // Use /tmp/primal-{name}.sock
        let path = PathBuf::from(format!("/tmp/primal-{primal_name}.sock"));

        debug!("Creating Unix socket endpoint for '{}': {}", primal_name, path.display());

        // Clean up old socket if it exists
        if path.exists() {
            warn!("Socket file already exists, removing: {}", path.display());
            tokio::fs::remove_file(&path).await.map_err(|e| {
                IpcError::PlatformError(format!("Failed to remove old socket: {e}"))
            })?;
        }

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
            NativeEndpoint::TcpLocal(_) => {
                Err(IpcError::PlatformError("Invalid endpoint for Unix platform".to_string()))
            }
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
            NativeEndpoint::TcpLocal(_) => {
                Err(IpcError::PlatformError("Invalid endpoint for Unix platform".to_string()))
            }
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
            NativeEndpoint::TcpLocal(_) => Ok(()), // Not a Unix socket, nothing to cleanup
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

    #[tokio::test]
    async fn test_unix_create_endpoint() {
        let ipc = UnixIPC;
        let endpoint = ipc.create_endpoint("test-primal").await.unwrap();

        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                assert!(path.to_str().unwrap().contains("test-primal"));
            }
            _ => panic!("Expected UnixSocket"),
        }
    }

    #[tokio::test]
    async fn test_unix_listen_and_connect() {
        let ipc = UnixIPC;
        let endpoint = ipc.create_endpoint("test-listen").await.unwrap();

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

        // Test communication
        client_stream.write_all(b"hello").await.unwrap();
        let mut buf = [0u8; 5];
        server_stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello");

        // Cleanup
        ipc.cleanup(&endpoint).await.unwrap();
    }

    #[tokio::test]
    async fn test_unix_cleanup() {
        let ipc = UnixIPC;
        let endpoint = ipc.create_endpoint("test-cleanup").await.unwrap();

        // Create listener (creates socket file)
        let _listener = ipc.listen(&endpoint).await.unwrap();

        // Verify file exists
        if let NativeEndpoint::UnixSocket(path) = &endpoint {
            assert!(path.exists());
        }

        // Cleanup
        ipc.cleanup(&endpoint).await.unwrap();

        // Verify file removed
        if let NativeEndpoint::UnixSocket(path) = &endpoint {
            assert!(!path.exists());
        }
    }
}
