// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Error types for onion relay

use thiserror::Error;

pub type Result<T> = std::result::Result<T, OnionRelayError>;

#[derive(Error, Debug)]
pub enum OnionRelayError {
    #[error("STUN discovery failed: {0}")]
    StunFailed(String),

    #[error("Hole punch failed after {attempts} attempts")]
    HolePunchFailed {
        attempts: u32,
    },

    #[error("Signaling timeout: no response from peer")]
    SignalingTimeout,

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Onion service error (Sovereign Onion Service)
    #[cfg(feature = "onion")]
    #[error("Onion service error: {0}")]
    Onion(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other: {0}")]
    Other(String),
}
