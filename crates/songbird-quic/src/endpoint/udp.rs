// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! UDP socket management for QUIC endpoints.
//!
//! Provides async send/recv over Tokio UDP sockets, managing the
//! mapping between QUIC connections and their UDP-level I/O.

use crate::error::{QuicError, Result};
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tracing::{debug, trace};

/// Maximum UDP datagram size for QUIC (matches `MAX_MTU` in lib.rs).
pub const MAX_DATAGRAM_SIZE: usize = 1200;

/// Received datagram with its source address.
#[derive(Debug)]
pub struct Datagram {
    /// Raw datagram bytes.
    #[expect(dead_code, reason = "QUIC datagram payload — used by connection/stream layer")]
    pub data: Vec<u8>,
    /// Source address of the sender.
    pub source: SocketAddr,
}

/// Async UDP endpoint for QUIC.
///
/// Wraps a Tokio `UdpSocket` with QUIC-aware send/recv operations.
#[derive(Debug)]
pub struct UdpEndpoint {
    socket: UdpSocket,
    local_addr: SocketAddr,
}

impl UdpEndpoint {
    /// Bind to the specified address.
    pub async fn bind(addr: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        let local_addr = socket.local_addr()?;
        debug!("UDP endpoint bound to {}", local_addr);
        Ok(Self {
            socket,
            local_addr,
        })
    }

    /// Bind to an ephemeral port on the given address (for clients).
    pub async fn bind_ephemeral(addr: std::net::IpAddr) -> Result<Self> {
        Self::bind(SocketAddr::new(addr, 0)).await
    }

    /// Local address this endpoint is bound to.
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Send a datagram to the specified address.
    #[expect(dead_code, reason = "QUIC send path — activated by connection manager")]
    pub async fn send_to(&self, data: &[u8], addr: SocketAddr) -> Result<usize> {
        if data.len() > MAX_DATAGRAM_SIZE {
            return Err(QuicError::Config(format!(
                "Datagram too large: {} > {MAX_DATAGRAM_SIZE}",
                data.len()
            )));
        }
        let n = self.socket.send_to(data, addr).await?;
        trace!("Sent {} bytes to {}", n, addr);
        Ok(n)
    }

    /// Receive a datagram. Returns the data and the source address.
    pub async fn recv_from(&self) -> Result<Datagram> {
        let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];
        let (n, source) = self.socket.recv_from(&mut buf).await?;
        buf.truncate(n);
        trace!("Received {} bytes from {}", n, source);
        Ok(Datagram {
            data: buf,
            source,
        })
    }

    /// Get a reference to the underlying socket (for advanced usage).
    #[must_use]
    #[expect(dead_code, reason = "public API for advanced socket configuration")]
    pub const fn socket(&self) -> &UdpSocket {
        &self.socket
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn bind_and_local_addr() {
        let ep = UdpEndpoint::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        assert_ne!(ep.local_addr().port(), 0);
        assert_eq!(ep.local_addr().ip(), std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    }

    #[tokio::test]
    async fn send_and_recv() {
        let ep1 = UdpEndpoint::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let ep2 = UdpEndpoint::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();

        let msg = b"quic-test-datagram";
        ep1.send_to(msg, ep2.local_addr()).await.unwrap();

        let dgram = ep2.recv_from().await.unwrap();
        assert_eq!(dgram.data, msg);
        assert_eq!(dgram.source, ep1.local_addr());
    }

    #[tokio::test]
    async fn oversized_datagram_rejected() {
        let ep = UdpEndpoint::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let big = vec![0u8; MAX_DATAGRAM_SIZE + 1];
        let result = ep.send_to(&big, "127.0.0.1:9999".parse().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn bind_ephemeral() {
        let ep = UdpEndpoint::bind_ephemeral(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
            .await
            .unwrap();
        assert_ne!(ep.local_addr().port(), 0);
    }

    /// Virtual time: `start_paused` avoids real sleeps while still exercising the timer driver.
    #[tokio::test(start_paused = true)]
    async fn sleep_completes_after_virtual_time_advance() {
        let start = tokio::time::Instant::now();
        let sleep = tokio::time::sleep(Duration::from_secs(10_000));
        tokio::time::advance(Duration::from_secs(10_000)).await;
        sleep.await;
        assert!(
            start.elapsed() >= Duration::from_secs(10_000),
            "paused timer should advance Instant::elapsed without wall-clock delay"
        );
    }
}
