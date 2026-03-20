// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Client Characteristic Configuration and notification subscription (descriptor-level).

use super::GattClient;
use crate::error::{BluetoothError, Result};
use crate::transport::Transport;
use tracing::debug;
use uuid::Uuid;

impl<T: Transport + 'static> GattClient<T> {
    /// Subscribe to characteristic notifications
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Characteristic not found
    /// - Notifications not supported
    /// - Subscription fails
    #[allow(clippy::unused_async)] // TODO: Will be async when implementing actual GATT operations
    pub async fn subscribe_notifications(
        &self,
        uuid: &Uuid,
        _callback: impl Fn(Vec<u8>) + Send + Sync + 'static,
    ) -> Result<()> {
        debug!("Subscribing to notifications: {}", uuid);

        // Find characteristic
        for service in &self.services {
            if let Some(characteristic) = service.characteristics.iter().find(|c| &c.uuid == uuid) {
                if !characteristic.properties.notify() {
                    return Err(BluetoothError::gatt(format!(
                        "Characteristic {uuid} does not support notifications"
                    )));
                }

                // TODO: Implement actual subscription using trouble-host (CCCD 0x2902)
                return Ok(());
            }
        }

        Err(BluetoothError::gatt(format!("Characteristic not found: {uuid}")))
    }
}
