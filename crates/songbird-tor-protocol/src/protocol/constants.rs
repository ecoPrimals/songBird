// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tor protocol constants

/// Tor protocol version
pub const TOR_PROTOCOL_VERSION: u16 = 5;

/// Maximum cell payload size
pub const MAX_CELL_PAYLOAD: usize = 507;

/// Maximum relay cell payload size
pub const MAX_RELAY_PAYLOAD: usize = 498;

/// Circuit window size (for SENDME)
pub const CIRCUIT_WINDOW: u16 = 1000;

/// Stream window size (for SENDME)
pub const STREAM_WINDOW: u16 = 500;

/// Default timeout for operations
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::protocol::CELL_LEN;

    #[test]
    fn fixed_cell_payload_matches_link_layer_capacity() {
        assert_eq!(MAX_CELL_PAYLOAD, 507);
        assert_eq!(CELL_LEN, 512);
        assert_eq!(4 + 1 + MAX_CELL_PAYLOAD, CELL_LEN);
    }

    #[test]
    fn relay_payload_stays_below_cell_payload_for_relay_wrapping() {
        assert!(MAX_RELAY_PAYLOAD <= MAX_CELL_PAYLOAD);
        assert_eq!(MAX_RELAY_PAYLOAD, 498);
    }

    #[test]
    fn flow_control_windows_are_positive_and_ordered() {
        assert!(CIRCUIT_WINDOW > 0);
        assert!(STREAM_WINDOW > 0);
        assert!(CIRCUIT_WINDOW >= STREAM_WINDOW);
    }

    #[test]
    fn default_timeout_is_reasonable_for_client_ops() {
        assert!(DEFAULT_TIMEOUT_SECS >= 5 && DEFAULT_TIMEOUT_SECS <= 120);
    }
}
