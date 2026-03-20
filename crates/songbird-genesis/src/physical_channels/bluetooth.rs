// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Bluetooth LE pairing (legacy `btleplug`)
//!
//! **Deprecated:** use [`super::bluetooth_pure`] and the `pure-bluetooth` feature instead.

#![deprecated(note = "Use the `bluetooth_pure` module and `pure-bluetooth` feature instead")]

use crate::error::{GenesisError, Result};
use crate::types::{PhysicalChannelType, ProximityProof, TrustLevel};
use async_trait::async_trait;

use super::PhysicalChannelProvider;

/// Bluetooth LE channel (legacy `btleplug` stack)
#[derive(Debug)]
pub struct BluetoothChannel;

impl BluetoothChannel {
    /// Create new Bluetooth channel
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for BluetoothChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PhysicalChannelProvider for BluetoothChannel {
    async fn verify_proximity(&self) -> Result<ProximityProof> {
        Err(GenesisError::BluetoothError(
            "Legacy Bluetooth channel is not implemented; use `bluetooth_pure` (`pure-bluetooth` feature) for pairing and exchange"
                .to_string(),
        ))
    }

    async fn secure_exchange(&self) -> Result<Vec<u8>> {
        Err(GenesisError::BluetoothError(
            "Legacy Bluetooth secure exchange is not implemented; use `bluetooth_pure` (`pure-bluetooth` feature)"
                .to_string(),
        ))
    }

    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Medium
    }

    fn channel_type(&self) -> PhysicalChannelType {
        PhysicalChannelType::Bluetooth
    }
}
