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
    /// Windows named pipe connection (Windows only).
    #[cfg(windows)]
    NamedPipe(tokio::net::windows::named_pipe::NamedPipeClient),
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
        #[cfg(windows)]
        {
            Self::connect_windows(path).await
        }
        #[cfg(not(any(unix, windows)))]
        {
            Self::connect_via_port_file(path).await
        }
    }

    /// Connect on Windows: try named pipe first, fall back to port file TCP.
    ///
    /// If `path` is already a pipe name (`\\.\pipe\...`), opens it directly.
    /// Otherwise derives a pipe name from the socket path basename and tries it.
    /// Falls back to reading `{path}.port` for TCP localhost.
    #[cfg(windows)]
    async fn connect_windows(path: &str) -> io::Result<Self> {
        let pipe_name = if path.starts_with(r"\\.\pipe\") || path.starts_with(r"\\.\Pipe\") {
            path.to_string()
        } else {
            Self::derive_pipe_name(path)
        };
        match tokio::net::windows::named_pipe::ClientOptions::new().open(&pipe_name) {
            Ok(client) => Ok(Self::NamedPipe(client)),
            Err(_) => Self::connect_via_port_file(path).await,
        }
    }

    /// Derive a Windows named pipe name from a Unix socket path.
    ///
    /// Extracts the basename without extension and prepends `\\.\pipe\biomeos_`.
    #[cfg(windows)]
    fn derive_pipe_name(path: &str) -> String {
        let basename =
            std::path::Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
        format!(r"\\.\pipe\biomeos_{basename}")
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
            crate::TransportEndpoint::NamedPipe {
                name,
            } => Self::connect_named_pipe(name).await,
            crate::TransportEndpoint::MeshRelay {
                ..
            } => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "MeshRelay requires Songbird mesh handler — cannot open raw stream",
            )),
        }
    }

    /// Connect to a Windows named pipe by name.
    ///
    /// On Windows, opens the pipe directly. On other platforms, returns an error.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the pipe doesn't exist or the platform doesn't
    /// support named pipes.
    #[allow(clippy::unused_async, reason = "async signature matches other IpcStream connect paths")]
    pub async fn connect_named_pipe(name: &str) -> io::Result<Self> {
        #[cfg(windows)]
        {
            let client =
                tokio::net::windows::named_pipe::ClientOptions::new().open(name).map_err(|e| {
                    io::Error::new(
                        e.kind(),
                        format!("failed to connect to named pipe '{name}': {e}"),
                    )
                })?;
            Ok(Self::NamedPipe(client))
        }
        #[cfg(not(windows))]
        {
            let _ = name;
            Err(io::Error::new(io::ErrorKind::Unsupported, "Named pipes require Windows platform"))
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
            #[cfg(windows)]
            Self::NamedPipe(_) => f.write_str("IpcStream::NamedPipe(..)"),
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
            #[cfg(windows)]
            Self::NamedPipe(s) => Pin::new(s).poll_read(cx, buf),
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
            #[cfg(windows)]
            Self::NamedPipe(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => Pin::new(s).poll_flush(cx),
            #[cfg(windows)]
            Self::NamedPipe(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => Pin::new(s).poll_shutdown(cx),
            #[cfg(windows)]
            Self::NamedPipe(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::TransportEndpoint;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn connect_tcp_roundtrip_payload() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let client = tokio::spawn(async move {
            let mut stream = IpcStream::connect_tcp("127.0.0.1", port).await.unwrap();
            stream.write_all(b"ping").await.unwrap();
            stream.flush().await.unwrap();
        });

        let (mut server, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 4];
        server.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ping");
        client.await.unwrap();
    }

    #[tokio::test]
    async fn connect_endpoint_tcp_delegates_to_connect_tcp() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let ep = TransportEndpoint::tcp("127.0.0.1", port);

        let client = tokio::spawn(async move {
            let mut stream = IpcStream::connect_endpoint(&ep).await.unwrap();
            stream.write_all(b"ok").await.unwrap();
        });

        let (mut server, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 2];
        server.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ok");
        client.await.unwrap();
    }

    #[tokio::test]
    async fn connect_endpoint_mesh_relay_is_unsupported() {
        let ep = TransportEndpoint::mesh_relay("peer-a", "security");
        let err = IpcStream::connect_endpoint(&ep).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Unsupported);
        assert!(err.to_string().contains("MeshRelay"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn connect_endpoint_uds_on_unix() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("ipc_stream_test.sock");
        let path = sock_path.to_str().unwrap().to_string();

        let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
        let path_for_client = path.clone();
        let client = tokio::spawn(async move {
            let mut stream = IpcStream::connect_endpoint(&TransportEndpoint::uds(path_for_client))
                .await
                .unwrap();
            stream.write_all(b"uds").await.unwrap();
        });

        let (mut server, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 3];
        server.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"uds");
        client.await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn connect_unix_direct_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("direct.sock");
        let path_owned = sock_path.to_str().unwrap().to_string();

        let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
        let client = tokio::spawn(async move {
            let mut stream = IpcStream::connect(&path_owned).await.unwrap();
            stream.write_all(b"hi").await.unwrap();
        });

        let (mut server, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 2];
        server.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hi");
        client.await.unwrap();
    }

    #[tokio::test]
    async fn connect_tcp_refused_on_closed_port() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let err = IpcStream::connect_tcp("127.0.0.1", port).await.unwrap_err();
        assert!(
            matches!(err.kind(), io::ErrorKind::ConnectionRefused | io::ErrorKind::ConnectionReset),
            "unexpected kind: {:?}",
            err.kind()
        );
    }

    #[tokio::test]
    async fn debug_format_shows_variant_name() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let stream = IpcStream::connect_tcp("127.0.0.1", port).await.unwrap();
        let debug = format!("{stream:?}");
        assert!(debug.contains("Tcp"));
    }

    #[cfg(not(windows))]
    #[tokio::test]
    async fn connect_named_pipe_unsupported_off_windows() {
        let err = IpcStream::connect_named_pipe(r"\\.\pipe\songbird_test").await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Unsupported);
        assert!(err.to_string().contains("Windows"));
    }

    #[cfg(not(windows))]
    #[tokio::test]
    async fn connect_endpoint_named_pipe_unsupported_off_windows() {
        let ep = TransportEndpoint::named_pipe(r"\\.\pipe\songbird_test");
        let err = IpcStream::connect_endpoint(&ep).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Unsupported);
    }

    #[tokio::test]
    async fn async_write_read_through_ipc_stream_wrapper() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let client = tokio::spawn(async move {
            let mut stream = IpcStream::connect_tcp("127.0.0.1", port).await.unwrap();
            stream.write_all(b"echo").await.unwrap();
            stream.flush().await.unwrap();
            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).await.unwrap();
            assert_eq!(&buf, b"pong");
        });

        let (mut server, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 4];
        server.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"echo");
        server.write_all(b"pong").await.unwrap();
        client.await.unwrap();
    }
}
