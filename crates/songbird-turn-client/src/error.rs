// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Error types for TURN session operations.

use songbird_stun::StunError;

/// Errors from [`TurnSession`](crate::TurnSession) operations.
#[derive(Debug, thiserror::Error)]
pub enum TurnSessionError {
    /// STUN/TURN protocol error from the underlying client.
    #[error("TURN protocol error: {0}")]
    Protocol(#[from] StunError),

    /// I/O error on the UDP socket.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Session not connected (call `connect()` first).
    #[error("session not connected")]
    NotConnected,

    /// Received a STUN message that is not a `DataIndication`.
    #[error("unexpected STUN message type: 0x{0:04x}")]
    UnexpectedMessage(u16),

    /// Timeout waiting for data from the relay.
    #[error("receive timed out after {0:?}")]
    Timeout(std::time::Duration),

    /// Payload exceeds maximum TURN data size (< 64 KiB).
    #[error("payload too large: {0} bytes (max 65535)")]
    PayloadTooLarge(usize),
}
