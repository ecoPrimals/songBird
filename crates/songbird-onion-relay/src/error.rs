// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn stun_failed_display_contains_message() {
        let e = OnionRelayError::StunFailed("no route".into());
        let s = e.to_string();
        assert!(s.contains("STUN"), "expected STUN in {s:?}");
        assert!(s.contains("no route"), "expected detail in {s:?}");
    }

    #[test]
    fn hole_punch_failed_includes_attempts() {
        let e = OnionRelayError::HolePunchFailed {
            attempts: 7,
        };
        assert!(e.to_string().contains('7'), "expected attempts in {}", e);
    }

    #[test]
    fn signaling_timeout_peer_not_found_transport_encryption_invalid_other() {
        assert!(OnionRelayError::SignalingTimeout.to_string().contains("Signaling"));
        assert!(OnionRelayError::PeerNotFound("n".into()).to_string().contains('n'));
        assert!(OnionRelayError::Transport("t".into()).to_string().contains("Transport"));
        assert!(OnionRelayError::Encryption("e".into()).to_string().contains("Encryption"));
        assert!(OnionRelayError::InvalidMessage("m".into()).to_string().contains("Invalid"));
        assert!(OnionRelayError::Other("o".into()).to_string().contains('o'));
    }

    #[cfg(feature = "onion")]
    #[test]
    fn onion_variant_display() {
        let e = OnionRelayError::Onion("svc".into());
        assert!(e.to_string().contains("Onion"));
    }

    #[test]
    fn io_error_converts_via_from() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let e: OnionRelayError = io.into();
        assert!(matches!(e, OnionRelayError::Io(_)), "expected Io, got {e:?}");
    }
}
