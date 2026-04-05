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
    #[expect(
        clippy::unused_async,
        reason = "placeholder for future GATT operations (notifications/CCCD)"
    )]
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

                // No CCCD (0x2902) write yet: subscription is a no-op until the ATT stack wires notify.
                return Ok(());
            }
        }

        Err(BluetoothError::gatt(format!("Characteristic not found: {uuid}")))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::GattClient;
    use crate::device::{Address, DeviceInfo};
    use crate::gatt::{Characteristic, CharacteristicProperties, Service};
    use crate::l2cap::L2capChannel;
    use crate::transport::{Transport, TransportType};
    use tokio::sync::Mutex;
    use uuid::Uuid;

    struct MockTransport;

    impl Transport for MockTransport {
        fn transport_type(&self) -> TransportType {
            TransportType::Usb
        }

        async fn send_command(&mut self, _data: &[u8]) -> crate::error::Result<()> {
            Ok(())
        }

        async fn receive_event(&mut self) -> crate::error::Result<Vec<u8>> {
            Ok(Vec::new())
        }

        async fn send_acl(&mut self, _data: &[u8]) -> crate::error::Result<()> {
            Ok(())
        }

        async fn receive_acl(&mut self) -> crate::error::Result<Vec<u8>> {
            Ok(Vec::new())
        }

        fn is_connected(&self) -> bool {
            true
        }

        async fn close(&mut self) -> crate::error::Result<()> {
            Ok(())
        }
    }

    fn client_with_characteristic(
        char_uuid: Uuid,
        props: CharacteristicProperties,
    ) -> GattClient<MockTransport> {
        let info = DeviceInfo::new(Address::from_bytes([1, 2, 3, 4, 5, 6]));
        let device = std::sync::Arc::new(crate::device::Device::new(info, 0x0040));
        let l2cap_channel = L2capChannel::new_att(0x0040);
        let transport = std::sync::Arc::new(Mutex::new(MockTransport));
        let mut client = GattClient::new(device, l2cap_channel, transport);
        let svc_uuid = Uuid::from_u128(0x1111);
        let mut svc = Service::new(svc_uuid, 1, 10);
        svc.characteristics.push(Characteristic {
            uuid: char_uuid,
            handle: 3,
            properties: props,
        });
        client.services.push(svc);
        client
    }

    #[tokio::test]
    async fn subscribe_notifications_ok_when_notify_supported() {
        let u = Uuid::from_u128(0x180F);
        let client = client_with_characteristic(u, CharacteristicProperties::new().with_notify());
        client.subscribe_notifications(&u, |_v| {}).await.expect("subscribe");
    }

    #[tokio::test]
    async fn subscribe_notifications_fails_without_notify_property() {
        let u = Uuid::from_u128(0x1810);
        let client = client_with_characteristic(u, CharacteristicProperties::new().with_read());
        let err = client.subscribe_notifications(&u, |_v| {}).await.expect_err("no notify");
        assert!(err.to_string().contains("notifications") || err.to_string().contains("notify"));
    }

    #[tokio::test]
    async fn subscribe_notifications_fails_when_uuid_missing() {
        let u = Uuid::from_u128(0xDEAD);
        let client = client_with_characteristic(
            Uuid::from_u128(0xBEEF),
            CharacteristicProperties::new().with_notify(),
        );
        let err = client.subscribe_notifications(&u, |_v| {}).await.expect_err("missing");
        assert!(
            err.to_string().contains("not found") || err.to_string().contains("Characteristic")
        );
    }

    #[tokio::test]
    async fn subscribe_notifications_empty_services_list_fails() {
        let info = DeviceInfo::new(Address::from_bytes([1, 2, 3, 4, 5, 6]));
        let device = std::sync::Arc::new(crate::device::Device::new(info, 0x0040));
        let l2cap_channel = L2capChannel::new_att(0x0040);
        let transport = std::sync::Arc::new(Mutex::new(MockTransport));
        let client = GattClient::new(device, l2cap_channel, transport);
        let err =
            client.subscribe_notifications(&Uuid::from_u128(1), |_v| {}).await.expect_err("empty");
        assert!(err.to_string().contains("not found"));
    }
}
