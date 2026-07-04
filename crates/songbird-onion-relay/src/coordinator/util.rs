// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared time helpers and nonces for punch coordination.

use crate::error::{OnionRelayError, Result};
use std::time::{SystemTime, UNIX_EPOCH};

/// Converts current wall-clock time to milliseconds since [`UNIX_EPOCH`].
pub fn unix_epoch_millis_u64() -> Result<u64> {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| OnionRelayError::Other(String::from("System time before UNIX epoch")))?
            .as_millis(),
    )
    .map_err(|_| OnionRelayError::Other(String::from("System time millis overflow")))
}

/// Generate random nonce
pub fn rand_nonce() -> [u8; 16] {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut nonce = [0u8; 16];
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    nonce[..8].copy_from_slice(&now.to_le_bytes()[..8]);
    // Add some randomness from memory address
    let ptr = std::ptr::from_ref(&nonce) as usize;
    nonce[8..16].copy_from_slice(&ptr.to_le_bytes());
    nonce
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn unix_epoch_millis_u64_returns_positive_epoch_ms() {
        let m = unix_epoch_millis_u64().expect("wall clock should be valid");
        assert!(m > 1_000_000_000, "expected ms since 2001, got {m}");
    }

    #[test]
    fn rand_nonce_length_and_varies() {
        let a = rand_nonce();
        let b = rand_nonce();
        assert_eq!(a.len(), 16, "nonce must be 16 bytes");
        assert_ne!(a, b, "successive nonces should differ with high probability");
    }
}
