// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Error types for onion relay

use thiserror::Error;

/// Convenient alias for operations that return [`OnionRelayError`].
pub type Result<T> = std::result::Result<T, OnionRelayError>;

/// Failures surfaced by STUN discovery, hole punch, signaling, or optional onion transport.
#[derive(Error, Debug)]
pub enum OnionRelayError {
    /// Public address discovery via STUN did not succeed.
    #[error("STUN discovery failed: {0}")]
    StunFailed(String),

    /// UDP hole punch exhausted retries without a working path.
    #[error("Hole punch failed after {attempts} attempts")]
    HolePunchFailed {
        /// Number of punch rounds attempted.
        attempts: u32,
    },

    /// Expected signaling reply never arrived.
    #[error("Signaling timeout: no response from peer")]
    SignalingTimeout,

    /// Rendezvous did not know about the requested peer.
    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    /// Underlying transport failure (UDP, relay, or onion connector).
    #[error("Transport error: {0}")]
    Transport(String),

    /// Cryptographic operation failed.
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Signaling payload could not be decoded or violated protocol rules.
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Onion service error (Sovereign Onion Service)
    #[cfg(feature = "onion")]
    #[error("Onion service error: {0}")]
    Onion(String),

    /// Low-level I/O failure.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Catch-all for unexpected failures.
    #[error("Other: {0}")]
    Other(String),
}
