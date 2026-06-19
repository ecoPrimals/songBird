// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! BTSP Tunnel Types and Structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Handle to an established tunnel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelHandle {
    /// Unique tunnel identifier
    pub id: String,
}

impl TunnelHandle {
    /// Create a new tunnel handle with generated ID
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
        }
    }

    /// Create tunnel handle with specific ID
    #[must_use]
    pub const fn with_id(id: String) -> Self {
        Self {
            id,
        }
    }
}

impl Default for TunnelHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Security context for encryption/decryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Tunnel ID this context belongs to
    pub tunnel_id: String,

    /// Peer ID
    pub peer_id: String,

    /// Optional nonce for this operation
    pub nonce: Option<Vec<u8>>,

    /// Additional authenticated data
    pub aad: Option<Vec<u8>>,
}

/// Tunnel status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    /// Tunnel handle
    pub handle: TunnelHandle,

    /// Current status
    pub status: TunnelState,

    /// Peer information
    pub peer_id: String,
    pub peer_endpoint: String,

    /// When tunnel was established
    pub established_at: DateTime<Utc>,

    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,

    /// Bytes sent through tunnel
    pub bytes_sent: u64,

    /// Bytes received through tunnel
    pub bytes_received: u64,

    /// Number of errors encountered
    pub error_count: u32,
}

/// Tunnel state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TunnelState {
    /// Tunnel is being established
    Connecting,

    /// Tunnel is active and ready
    Active,

    /// Tunnel is temporarily unavailable
    Degraded,

    /// Tunnel is closed
    Closed,

    /// Tunnel encountered an error
    Error,
}

/// Tunnel with associated data
#[derive(Debug, Clone)]
pub struct Tunnel {
    /// Tunnel handle
    pub handle: TunnelHandle,

    /// Peer information
    pub peer_id: String,
    pub peer_endpoint: String,

    /// Shared secret key for encryption
    pub shared_key: Vec<u8>,

    /// When tunnel was established
    pub established_at: DateTime<Utc>,

    /// Current state
    pub state: TunnelState,

    /// Statistics
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub error_count: u32,
}

impl Tunnel {
    /// Create a new tunnel
    #[must_use]
    pub fn new(peer_id: String, peer_endpoint: String, shared_key: Vec<u8>) -> Self {
        Self {
            handle: TunnelHandle::new(),
            peer_id,
            peer_endpoint,
            shared_key,
            established_at: Utc::now(),
            state: TunnelState::Active,
            bytes_sent: 0,
            bytes_received: 0,
            error_count: 0,
        }
    }

    /// Get tunnel status
    #[must_use]
    pub fn status(&self) -> TunnelStatus {
        TunnelStatus {
            handle: self.handle.clone(),
            status: self.state,
            peer_id: self.peer_id.clone(),
            peer_endpoint: self.peer_endpoint.clone(),
            established_at: self.established_at,
            last_activity: Utc::now(),
            bytes_sent: self.bytes_sent,
            bytes_received: self.bytes_received,
            error_count: self.error_count,
        }
    }

    /// Mark tunnel as closed
    pub const fn close(&mut self) {
        self.state = TunnelState::Closed;
    }

    /// Record bytes sent
    pub const fn record_sent(&mut self, bytes: usize) {
        self.bytes_sent += bytes as u64;
    }

    /// Record bytes received
    pub const fn record_received(&mut self, bytes: usize) {
        self.bytes_received += bytes as u64;
    }

    /// Record error
    pub const fn record_error(&mut self) {
        self.error_count += 1;
        if self.error_count > 10 {
            self.state = TunnelState::Degraded;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tunnel_handle_creation() {
        let handle1 = TunnelHandle::new();
        let handle2 = TunnelHandle::new();
        assert_ne!(handle1.id, handle2.id);
    }

    #[test]
    fn test_tunnel_creation() {
        let tunnel =
            Tunnel::new(String::from("peer-1"), String::from("http://peer:8080"), vec![1, 2, 3, 4]);

        assert_eq!(tunnel.state, TunnelState::Active);
        assert_eq!(tunnel.bytes_sent, 0);
        assert_eq!(tunnel.error_count, 0);
    }

    #[test]
    fn test_tunnel_error_degradation() {
        let mut tunnel =
            Tunnel::new(String::from("peer-1"), String::from("http://peer:8080"), vec![1, 2, 3, 4]);

        // Record multiple errors
        for _ in 0..11 {
            tunnel.record_error();
        }

        assert_eq!(tunnel.state, TunnelState::Degraded);
        assert_eq!(tunnel.error_count, 11);
    }
}
