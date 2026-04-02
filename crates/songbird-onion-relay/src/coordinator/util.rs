// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Shared time helpers and nonces for punch coordination.

use crate::error::{OnionRelayError, Result};
use std::time::{SystemTime, UNIX_EPOCH};

/// Converts current wall-clock time to milliseconds since [`UNIX_EPOCH`].
pub fn unix_epoch_millis_u64() -> Result<u64> {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| OnionRelayError::Other("System time before UNIX epoch".to_string()))?
            .as_millis(),
    )
    .map_err(|_| OnionRelayError::Other("System time millis overflow".to_string()))
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
