// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Platform-abstracted transport listener — server-side accept loop primitive.
//!
//! G66 Transport Abstraction: the listener counterpart to [`IpcStream`](crate::IpcStream).
//! Business logic calls [`TransportListener::accept`] and gets a connected
//! [`IpcStream`](crate::IpcStream) without knowing which transport arrived.
//!
//! ## Platform Behavior
//!
//! - **Unix**: Binds a `UnixListener` at the given socket path
//! - **All platforms**: Binds a `TcpListener` on the specified address
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_types::{TransportEndpoint, TransportListener, IpcStream};
//!
//! let endpoint = TransportEndpoint::platform_default("songbird", 9100);
//! let listener = TransportListener::bind(&endpoint).await?;
//! loop {
//!     let stream: IpcStream = listener.accept().await?;
//!     tokio::spawn(handle_connection(stream));
//! }
//! ```

use std::io;

use crate::IpcStream;
use crate::TransportEndpoint;

/// A platform-abstracted transport listener.
///
/// Accepts incoming connections and returns [`IpcStream`] instances.
/// All `#[cfg]` conditionals are contained here — callers operate on
/// platform-neutral types.
pub enum TransportListener {
    /// Unix domain socket listener (Unix only).
    #[cfg(unix)]
    Unix(tokio::net::UnixListener),
    /// TCP listener (all platforms).
    Tcp(tokio::net::TcpListener),
}

impl TransportListener {
    /// Bind a listener to the given transport endpoint.
    ///
    /// For UDS endpoints on Unix, creates/removes the socket file and binds.
    /// For TCP endpoints, binds to the specified host:port.
    /// For NamedPipe/MeshRelay, returns `Unsupported`.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the bind fails.
    pub async fn bind(endpoint: &TransportEndpoint) -> io::Result<Self> {
        match endpoint {
            TransportEndpoint::Uds { path } => Self::bind_uds(path).await,
            TransportEndpoint::Tcp { host, port } => {
                let addr = format!("{host}:{port}");
                let listener = tokio::net::TcpListener::bind(&addr).await?;
                Ok(Self::Tcp(listener))
            }
            TransportEndpoint::NamedPipe { .. } => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Named pipe listener not yet implemented (Windows-only)",
            )),
            TransportEndpoint::MeshRelay { .. } => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "MeshRelay cannot be used as a listener endpoint",
            )),
        }
    }

    /// Bind a UDS listener, handling the socket path lifecycle.
    #[cfg(unix)]
    #[allow(clippy::unused_async)]
    async fn bind_uds(path: &str) -> io::Result<Self> {
        let sock_path = std::path::Path::new(path);
        if sock_path.exists() {
            std::fs::remove_file(sock_path)?;
        }
        if let Some(parent) = sock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let listener = tokio::net::UnixListener::bind(sock_path)?;
        Ok(Self::Unix(listener))
    }

    /// On non-Unix, UDS endpoints are not supported.
    #[cfg(not(unix))]
    #[allow(clippy::unused_async)]
    async fn bind_uds(path: &str) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("UDS not available on this platform for {path}"),
        ))
    }

    /// Accept an incoming connection.
    ///
    /// Returns a connected [`IpcStream`] that implements `AsyncRead + AsyncWrite`.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the accept fails.
    pub async fn accept(&self) -> io::Result<IpcStream> {
        match self {
            #[cfg(unix)]
            Self::Unix(l) => {
                let (stream, _addr) = l.accept().await?;
                Ok(IpcStream::Unix(stream))
            }
            Self::Tcp(l) => {
                let (stream, _addr) = l.accept().await?;
                Ok(IpcStream::Tcp(stream))
            }
        }
    }

    /// Returns the local address this listener is bound to (for TCP).
    ///
    /// For UDS, returns the socket path as a string.
    pub fn local_addr_string(&self) -> String {
        match self {
            #[cfg(unix)]
            Self::Unix(l) => l
                .local_addr()
                .ok()
                .and_then(|a| a.as_pathname().map(|p| p.display().to_string()))
                .unwrap_or_else(|| "<unnamed>".to_string()),
            Self::Tcp(l) => match l.local_addr() {
                Ok(a) => a.to_string(),
                Err(_) => "<unknown>".to_string(),
            },
        }
    }

    /// Whether this listener is a local (same-host) transport.
    #[must_use]
    pub const fn is_local(&self) -> bool {
        match self {
            #[cfg(unix)]
            Self::Unix(_) => true,
            Self::Tcp(_) => true, // TCP listener is always local to this host
        }
    }

    /// The transport type name.
    #[must_use]
    pub const fn transport_name(&self) -> &'static str {
        match self {
            #[cfg(unix)]
            Self::Unix(_) => "uds",
            Self::Tcp(_) => "tcp",
        }
    }
}

impl std::fmt::Debug for TransportListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(unix)]
            Self::Unix(_) => f.write_str("TransportListener::Unix(..)"),
            Self::Tcp(_) => f.write_str("TransportListener::Tcp(..)"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn bind_tcp_and_accept() {
        let ep = TransportEndpoint::tcp("127.0.0.1", 0);
        let listener = TransportListener::bind(&ep).await.unwrap();
        assert_eq!(listener.transport_name(), "tcp");

        let addr = listener.local_addr_string();
        let port: u16 = addr.rsplit(':').next().unwrap().parse().unwrap();

        let client_handle = tokio::spawn(async move {
            let mut stream = IpcStream::connect_tcp("127.0.0.1", port).await.unwrap();
            stream.write_all(b"hello").await.unwrap();
            stream.flush().await.unwrap();
        });

        let mut server_stream = listener.accept().await.unwrap();
        let mut buf = [0u8; 5];
        server_stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello");

        client_handle.await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn bind_uds_and_accept() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");
        let ep = TransportEndpoint::uds(sock_path.to_str().unwrap());
        let listener = TransportListener::bind(&ep).await.unwrap();
        assert_eq!(listener.transport_name(), "uds");

        let path_clone = sock_path.clone();
        let client_handle = tokio::spawn(async move {
            let mut stream = IpcStream::connect(path_clone.to_str().unwrap()).await.unwrap();
            stream.write_all(b"G66").await.unwrap();
            stream.flush().await.unwrap();
        });

        let mut server_stream = listener.accept().await.unwrap();
        let mut buf = [0u8; 3];
        server_stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"G66");

        client_handle.await.unwrap();
    }

    #[test]
    fn mesh_relay_cannot_bind() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let ep = TransportEndpoint::mesh_relay("peer", "cap");
            let result = TransportListener::bind(&ep).await;
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
        });
    }

    #[test]
    fn debug_format() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let ep = TransportEndpoint::tcp("127.0.0.1", 0);
            let listener = TransportListener::bind(&ep).await.unwrap();
            let debug = format!("{listener:?}");
            assert!(debug.contains("Tcp"));
        });
    }
}
