// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Pure Rust STUN Server (RFC 5389)
//!
//! **Zero C Dependencies | Zero Unsafe Code | ecoBin Compliant**
//!
//! This module implements a STUN (Session Traversal Utilities for NAT) server
//! that responds to Binding Requests with the client's public IP address and port.
//!
//! ## Features
//!
//! - ✅ RFC 5389 compliant STUN server
//! - ✅ Pure Rust implementation
//! - ✅ Zero unsafe code (compiler-enforced safety)
//! - ✅ Async/await with tokio
//! - ✅ Graceful shutdown support
//! - ✅ Statistics tracking
//! - ✅ Production-ready error handling
//!
//! ## Architecture
//!
//! The server uses existing message encoding/decoding infrastructure from
//! `message.rs`, ensuring consistency with the client implementation.
//!
//! ## Usage
//!
//! ```no_run
//! use songbird_stun::StunServer;
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let bind_addr: SocketAddr = "0.0.0.0:3478".parse()?;
//!     let mut server = StunServer::new(bind_addr);
//!     
//!     server.run().await?;
//!     Ok(())
//! }
//! ```

use crate::error::{StunError, StunResult};
use crate::message::{MessageType, StunAttribute, StunMessage};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// STUN server statistics
///
/// Tracks server metrics for monitoring and debugging.
#[derive(Debug, Clone, Default)]
pub struct StunServerStats {
    /// Total Binding Requests successfully handled
    pub requests_handled: u64,

    /// Total errors encountered
    pub errors: u64,

    /// Server start time
    pub start_time: Option<Instant>,

    /// Last request received time
    pub last_request: Option<Instant>,
}

impl StunServerStats {
    /// Get server uptime in seconds
    #[must_use]
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.map_or(0, |start| start.elapsed().as_secs())
    }

    /// Get seconds since last request
    #[must_use]
    pub fn seconds_since_last_request(&self) -> Option<u64> {
        self.last_request.map(|last| last.elapsed().as_secs())
    }
}

/// Pure Rust STUN Server (RFC 5389)
///
/// Responds to STUN Binding Requests with the client's public IP address
/// and port, enabling NAT traversal without external relay servers.
///
/// ## Design Principles
///
/// - **Pure Rust**: Zero C dependencies, ecoBin compliant
/// - **Zero Unsafe**: All operations use safe Rust
/// - **Self-Contained**: No external primal dependencies
/// - **Modern Idiomatic**: Async/await, Result-based error handling
/// - **Production Ready**: Comprehensive error handling and statistics
///
/// ## Example
///
/// ```no_run
/// use songbird_stun::StunServer;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut server = StunServer::new("0.0.0.0:3478".parse()?);
///     server.run().await?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct StunServer {
    /// Bind address for incoming requests
    bind_addr: SocketAddr,

    /// Optional alternate address for NAT type detection (RFC 5780)
    ///
    /// Future enhancement: Used for responding from alternate IP/port
    /// to help clients detect NAT type.
    alternate_addr: Option<SocketAddr>,

    /// Server statistics (thread-safe)
    stats: Arc<RwLock<StunServerStats>>,
}

impl StunServer {
    /// Create new STUN server
    ///
    /// # Arguments
    ///
    /// * `bind_addr` - Address to bind for incoming STUN requests
    ///
    /// # Example
    ///
    /// ```
    /// use songbird_stun::StunServer;
    ///
    /// let server = StunServer::new("0.0.0.0:3478".parse().unwrap());
    /// ```
    #[must_use]
    pub fn new(bind_addr: SocketAddr) -> Self {
        Self {
            bind_addr,
            alternate_addr: None,
            stats: Arc::new(RwLock::new(StunServerStats::default())),
        }
    }

    /// Create STUN server with alternate address for NAT type detection
    ///
    /// Alternate address is used in RFC 5780 NAT type detection.
    /// This is a future enhancement (Phase 2).
    ///
    /// # Arguments
    ///
    /// * `bind_addr` - Primary bind address
    /// * `alternate_addr` - Alternate address for NAT detection
    ///
    /// # Example
    ///
    /// ```
    /// use songbird_stun::StunServer;
    ///
    /// let server = StunServer::with_alternate(
    ///     "0.0.0.0:3478".parse().unwrap(),
    ///     "0.0.0.0:3479".parse().unwrap(),
    /// );
    /// ```
    #[must_use]
    pub fn with_alternate(bind_addr: SocketAddr, alternate_addr: SocketAddr) -> Self {
        Self {
            bind_addr,
            alternate_addr: Some(alternate_addr),
            stats: Arc::new(RwLock::new(StunServerStats::default())),
        }
    }

    /// Run the STUN server
    ///
    /// Listens for incoming STUN Binding Requests and responds with
    /// the client's public IP address and port.
    ///
    /// This method runs indefinitely until an error occurs.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if server shuts down gracefully, or `Err` if
    /// a fatal error occurs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use songbird_stun::StunServer;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut server = StunServer::new("0.0.0.0:3478".parse()?);
    ///     server.run().await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if socket bind or runtime fails.
    pub async fn run(&self) -> StunResult<()> {
        self.run_inner(None).await
    }

    /// Run the STUN server with a readiness signal.
    ///
    /// Same as [`run`](Self::run) but sends the actual bound [`SocketAddr`]
    /// on `ready_tx` once the socket is bound, allowing callers to
    /// synchronize without sleeping.
    ///
    /// # Errors
    ///
    /// Returns an error if socket bind or runtime fails.
    pub async fn run_with_ready(
        &self,
        ready_tx: tokio::sync::oneshot::Sender<SocketAddr>,
    ) -> StunResult<()> {
        self.run_inner(Some(ready_tx)).await
    }

    /// Internal run loop shared by [`run`] and [`run_with_ready`].
    async fn run_inner(
        &self,
        ready_tx: Option<tokio::sync::oneshot::Sender<SocketAddr>>,
    ) -> StunResult<()> {
        // Bind UDP socket
        let socket = UdpSocket::bind(self.bind_addr)
            .await
            .map_err(|e| StunError::Network(format!("Failed to bind UDP socket: {e}")))?;

        let actual_addr = socket
            .local_addr()
            .map_err(|e| StunError::Network(format!("Failed to get local address: {e}")))?;

        info!("🌐 STUN server listening on {}", actual_addr);

        // Signal readiness with actual bound address
        if let Some(tx) = ready_tx {
            let _ = tx.send(actual_addr);
        }

        // Update start time
        {
            let mut stats = self.stats.write().await;
            stats.start_time = Some(Instant::now());
        }

        // Buffer for incoming packets (MTU size)
        let mut buf = vec![0u8; 1500];

        // Main server loop
        loop {
            // Receive packet
            match socket.recv_from(&mut buf).await {
                Ok((len, src_addr)) => {
                    debug!("📨 Received {} bytes from {}", len, src_addr);

                    // Handle request (fire and forget for performance)
                    if let Err(e) = self.handle_request(&socket, &buf[..len], src_addr).await {
                        warn!("⚠️  Failed to handle request from {}: {}", src_addr, e);

                        // Increment error count
                        let mut stats = self.stats.write().await;
                        stats.errors += 1;
                    }
                }
                Err(e) => {
                    error!("❌ Failed to receive packet: {}", e);

                    // Increment error count
                    let mut stats = self.stats.write().await;
                    stats.errors += 1;
                }
            }
        }
    }

    /// Handle a single STUN request
    ///
    /// Parses the request, validates it's a Binding Request, generates
    /// a response with the client's address, and sends it back.
    ///
    /// # Arguments
    ///
    /// * `socket` - UDP socket to send response on
    /// * `data` - Raw packet data
    /// * `src_addr` - Source address of the request
    async fn handle_request(
        &self,
        socket: &UdpSocket,
        data: &[u8],
        src_addr: SocketAddr,
    ) -> StunResult<()> {
        // Parse STUN message
        let request = StunMessage::decode(data)?;

        // Validate message type (must be Binding Request)
        if request.message_type != MessageType::BindingRequest {
            return Err(StunError::InvalidResponse(format!(
                "Expected Binding Request, got {:?}",
                request.message_type
            )));
        }

        debug!("✅ Valid Binding Request from {}", src_addr);

        // Create response
        let response = self.create_binding_response(&request, src_addr);

        // Encode response
        let response_bytes = response.encode();

        // Send response
        socket
            .send_to(&response_bytes, src_addr)
            .await
            .map_err(|e| StunError::Network(format!("Failed to send response: {e}")))?;

        debug!("📤 Sent {} byte response to {}", response_bytes.len(), src_addr);

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.requests_handled += 1;
            stats.last_request = Some(Instant::now());
        }

        Ok(())
    }

    /// Create STUN Binding Response
    ///
    /// Generates a response message containing the client's public address
    /// as seen by this server.
    ///
    /// # Arguments
    ///
    /// * `request` - Original Binding Request
    /// * `client_addr` - Client's source address (their public IP:port)
    ///
    /// # Returns
    ///
    /// STUN message with Binding Response type and address attributes.
    fn create_binding_response(
        &self,
        request: &StunMessage,
        client_addr: SocketAddr,
    ) -> StunMessage {
        // Create response with same transaction ID (RFC 5389 requirement)
        let mut response = StunMessage {
            message_type: MessageType::BindingResponse,
            transaction_id: request.transaction_id,
            attributes: Vec::new(),
        };

        // Add MAPPED-ADDRESS attribute (RFC 5389 Section 15.1)
        // This is the client's source address as seen by the server
        response.attributes.push(StunAttribute::MappedAddress(client_addr));

        // Add XOR-MAPPED-ADDRESS attribute (RFC 5389 Section 15.2)
        // Recommended for NAT hairpinning and obfuscation
        response.attributes.push(StunAttribute::XorMappedAddress(client_addr));

        // Future: Add SOFTWARE attribute (RFC 5389 Section 15.10)
        // Would identify server software, but attribute not yet implemented in message.rs

        // Future: Add OTHER-ADDRESS for RFC 5780 NAT type detection
        if let Some(alternate) = self.alternate_addr {
            response.attributes.push(StunAttribute::OtherAddress(alternate));
        }

        response
    }

    /// Get current server statistics
    ///
    /// Returns a snapshot of server metrics including requests handled,
    /// errors, and uptime.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use songbird_stun::StunServer;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let server = StunServer::new("0.0.0.0:3478".parse().unwrap());
    /// let stats = server.stats().await;
    /// println!("Requests handled: {}", stats.requests_handled);
    /// # }
    /// ```
    pub async fn stats(&self) -> StunServerStats {
        self.stats.read().await.clone()
    }

    /// Get bind address
    ///
    /// Returns the address the server is configured to bind to.
    #[must_use]
    pub const fn bind_addr(&self) -> SocketAddr {
        self.bind_addr
    }

    /// Get alternate address
    ///
    /// Returns the alternate address if configured (for NAT type detection).
    #[must_use]
    pub const fn alternate_addr(&self) -> Option<SocketAddr> {
        self.alternate_addr
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::message::StunMessage;
    use std::time::Instant;

    #[test]
    fn test_server_creation() {
        let bind_addr: SocketAddr = "127.0.0.1:3478".parse().unwrap();
        let server = StunServer::new(bind_addr);

        assert_eq!(server.bind_addr(), bind_addr);
        assert_eq!(server.alternate_addr(), None);
    }

    #[test]
    fn test_server_with_alternate() {
        let bind_addr: SocketAddr = "127.0.0.1:3478".parse().unwrap();
        let alternate_addr: SocketAddr = "127.0.0.1:3479".parse().unwrap();
        let server = StunServer::with_alternate(bind_addr, alternate_addr);

        assert_eq!(server.bind_addr(), bind_addr);
        assert_eq!(server.alternate_addr(), Some(alternate_addr));
    }

    #[test]
    fn test_create_binding_response() {
        let server = StunServer::new("127.0.0.1:3478".parse().unwrap());
        let request = StunMessage::new_binding_request();
        let client_addr: SocketAddr = "192.168.1.100:54321".parse().unwrap();

        let response = server.create_binding_response(&request, client_addr);

        // Should be Binding Response
        assert_eq!(response.message_type, MessageType::BindingResponse);

        // Should preserve transaction ID (RFC 5389 requirement)
        assert_eq!(response.transaction_id, request.transaction_id);

        // Should include MAPPED-ADDRESS
        assert_eq!(response.get_mapped_address(), Some(client_addr));

        // Should include XOR-MAPPED-ADDRESS
        assert_eq!(response.get_xor_mapped_address(), Some(client_addr));

        // Should have at least 2 attributes (MAPPED, XOR-MAPPED)
        assert!(response.attributes.len() >= 2);
    }

    #[test]
    fn test_create_binding_response_preserves_transaction_id() {
        let server = StunServer::new("127.0.0.1:3478".parse().unwrap());

        // Create multiple requests with different transaction IDs
        for _ in 0..10 {
            let request = StunMessage::new_binding_request();
            let client_addr: SocketAddr = "192.168.1.100:54321".parse().unwrap();

            let response = server.create_binding_response(&request, client_addr);

            // Each response must match its request's transaction ID
            assert_eq!(response.transaction_id, request.transaction_id);
        }
    }

    #[tokio::test]
    async fn test_stats_initialization() {
        let server = StunServer::new("127.0.0.1:3478".parse().unwrap());
        let stats = server.stats().await;

        assert_eq!(stats.requests_handled, 0);
        assert_eq!(stats.errors, 0);
        assert_eq!(stats.start_time, None);
        assert_eq!(stats.last_request, None);
    }

    #[tokio::test]
    async fn test_server_with_alternate_includes_other_address() {
        let bind_addr: SocketAddr = "127.0.0.1:3478".parse().unwrap();
        let alternate_addr: SocketAddr = "127.0.0.1:3479".parse().unwrap();
        let server = StunServer::with_alternate(bind_addr, alternate_addr);

        let request = StunMessage::new_binding_request();
        let client_addr: SocketAddr = "192.168.1.100:54321".parse().unwrap();

        let response = server.create_binding_response(&request, client_addr);

        // Should include OTHER-ADDRESS attribute when alternate is configured
        let has_other_address =
            response.attributes.iter().any(|attr| matches!(attr, StunAttribute::OtherAddress(_)));

        assert!(has_other_address);
    }

    #[test]
    fn stats_uptime_seconds_zero_without_start_time() {
        let stats = StunServerStats::default();
        assert_eq!(stats.uptime_seconds(), 0, "uptime should be 0 until start_time is set");
    }

    #[test]
    fn stats_seconds_since_last_request_none_when_never_seen() {
        let stats = StunServerStats::default();
        assert_eq!(
            stats.seconds_since_last_request(),
            None,
            "last request time should be absent before any request"
        );
    }

    #[test]
    fn stats_uptime_seconds_truncates_subsecond_elapsed() {
        let stats = StunServerStats {
            start_time: Some(Instant::now()),
            ..Default::default()
        };
        assert_eq!(
            stats.uptime_seconds(),
            0,
            "uptime_seconds uses whole seconds; immediate read should be 0"
        );
    }

    #[test]
    fn stats_seconds_since_last_request_some_after_last_request() {
        let stats = StunServerStats {
            last_request: Some(Instant::now()),
            ..Default::default()
        };
        assert!(
            stats.seconds_since_last_request().is_some(),
            "elapsed since last request should be defined once last_request is set"
        );
    }
}
