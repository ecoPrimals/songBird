// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Platform-abstracted IPC stream — the shared connection primitive.
//!
//! Eliminates the repeated `#[cfg(unix)]` / `#[cfg(not(unix))]` connect pattern
//! found across 15+ crates. Consumers call [`IpcStream::connect`] with a path
//! and get a connected async stream regardless of platform.
//!
//! # Platform behavior
//!
//! - **Unix**: Connects via Unix domain socket (`tokio::net::UnixStream`)
//! - **Windows/other**: Reads a port file at `{path}.port` and connects via TCP
//!   to `127.0.0.1:{port}` (standard Windows IPC-over-TCP pattern)
//!
//! # Example
//!
//! ```rust,ignore
//! use songbird_types::IpcStream;
//!
//! let stream = IpcStream::connect("/run/biomeos/security.sock").await?;
//! // stream implements AsyncRead + AsyncWrite
//! ```

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// A platform-abstracted IPC connection.
///
/// On Unix this wraps a `UnixStream`; on other platforms it wraps a `TcpStream`
/// connected to localhost on a port read from a sidecar `.port` file.
pub enum IpcStream {
    /// Unix domain socket connection (Unix only).
    #[cfg(unix)]
    Unix(tokio::net::UnixStream),
    /// TCP connection to localhost (cross-platform fallback).
    Tcp(tokio::net::TcpStream),
}

impl IpcStream {
    /// Connect to a local IPC endpoint by socket path.
    ///
    /// On Unix, connects directly to the socket at `path`.
    /// On other platforms, reads `{path}.port` to get a TCP port and connects
    /// to `127.0.0.1:{port}`.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the connection fails (socket not found, port file
    /// missing or unreadable, TCP connection refused, etc.).
    pub async fn connect(path: &str) -> io::Result<Self> {
        #[cfg(unix)]
        {
            let stream = tokio::net::UnixStream::connect(path).await?;
            Ok(Self::Unix(stream))
        }
        #[cfg(not(unix))]
        {
            Self::connect_via_port_file(path).await
        }
    }

    /// Connect to a TCP endpoint directly (any platform).
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the TCP connection fails.
    pub async fn connect_tcp(host: &str, port: u16) -> io::Result<Self> {
        let addr = format!("{host}:{port}");
        let stream = tokio::net::TcpStream::connect(&addr).await?;
        Ok(Self::Tcp(stream))
    }

    /// Connect to a [`TransportEndpoint`](crate::TransportEndpoint).
    ///
    /// Handles `Uds` and `Tcp` variants. `MeshRelay` returns `Unsupported`
    /// (requires the full Songbird mesh handler, not a raw stream).
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the connection fails or the endpoint type is unsupported.
    pub async fn connect_endpoint(endpoint: &crate::TransportEndpoint) -> io::Result<Self> {
        match endpoint {
            crate::TransportEndpoint::Uds {
                path,
            } => Self::connect(path).await,
            crate::TransportEndpoint::Tcp {
                host,
                port,
            } => Self::connect_tcp(host, *port).await,
            crate::TransportEndpoint::MeshRelay {
                ..
            } => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "MeshRelay requires Songbird mesh handler — cannot open raw stream",
            )),
        }
    }

    /// Read a `.port` file and connect via TCP to localhost on that port.
    #[cfg(not(unix))]
    async fn connect_via_port_file(path: &str) -> io::Result<Self> {
        let port_path = format!("{path}.port");
        let port_str = tokio::fs::read_to_string(&port_path).await.map_err(|e| {
            io::Error::new(e.kind(), format!("failed to read IPC port file '{port_path}': {e}"))
        })?;
        let port: u16 = port_str.trim().parse().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid port in '{port_path}': {e}"),
            )
        })?;
        Self::connect_tcp("127.0.0.1", port).await
    }
}

impl std::fmt::Debug for IpcStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(unix)]
            Self::Unix(_) => f.write_str("IpcStream::Unix(..)"),
            Self::Tcp(_) => f.write_str("IpcStream::Tcp(..)"),
        }
    }
}

impl AsyncRead for IpcStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_read(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for IpcStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}
