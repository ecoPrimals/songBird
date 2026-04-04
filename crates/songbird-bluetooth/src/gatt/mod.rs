// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! GATT Client - Generic Attribute Profile operations
//!
//! Provides high-level API for GATT service discovery and characteristic access.
//! Uses ATT (Attribute Protocol) over L2CAP channel 0x0004.
//!
//! Submodules: `att` (opcodes/UUIDs), `services` (discovery), `characteristics` (reads/writes),
//! `descriptors` (notifications / CCCD).

mod att;
mod characteristics;
mod descriptors;
mod services;

pub use characteristics::{Characteristic, CharacteristicProperties};
pub use services::Service;

use crate::{
    device::Device,
    error::{BluetoothError, Result},
    l2cap::L2capChannel,
    transport::Transport,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::trace;

/// GATT (Generic Attribute Profile) client for Bluetooth LE devices
///
/// Provides a high-level interface for interacting with GATT services and characteristics
/// on Bluetooth Low Energy devices. Supports reading, writing, and subscribing to
/// characteristic notifications.
///
/// # Example
///
/// ```rust,ignore
/// use songbird_bluetooth::gatt::GattClient;
/// use songbird_bluetooth::transport::Transport;
///
/// async fn example<T: Transport>(transport: T) {
///     let mut client = GattClient::new(transport);
///     // Discover services
///     client.discover_services().await.unwrap();
///     // Read characteristic by UUID
///     let value = client.read_characteristic(service_uuid, char_uuid).await.unwrap();
/// }
/// ```
pub struct GattClient<T: Transport> {
    pub(crate) device: Arc<Device>,
    pub(crate) services: Vec<Service>,
    pub(crate) l2cap_channel: L2capChannel,
    pub(crate) transport: Arc<Mutex<T>>,
    pub(crate) timeout_duration: Duration,
}

impl<T: Transport + 'static> GattClient<T> {
    /// Create new GATT client
    #[must_use]
    pub fn new(device: Arc<Device>, l2cap_channel: L2capChannel, transport: Arc<Mutex<T>>) -> Self {
        trace!("Creating GATT client for device {}", device.address());
        Self {
            device,
            services: Vec::new(),
            l2cap_channel,
            transport,
            timeout_duration: Duration::from_secs(5),
        }
    }

    /// Set GATT operation timeout
    #[must_use]
    pub const fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout_duration = duration;
        self
    }

    /// Send ATT request and receive response
    pub(crate) async fn send_att_request(&self, request: &[u8]) -> Result<Vec<u8>> {
        trace!("Sending ATT request: {} bytes", request.len());

        // Build L2CAP packet
        let acl_packet = self.l2cap_channel.build_acl_packet(request);

        // Send via transport (lock released immediately after send)
        self.transport.lock().await.send_acl(&acl_packet).await?;

        // Receive response with timeout (lock released immediately after receive)
        let response = timeout(self.timeout_duration, async {
            let acl_response = self.transport.lock().await.receive_acl().await?;
            // Parse L2CAP packet to extract ATT payload
            self.l2cap_channel.parse_acl_packet(&acl_response)
        })
        .await
        .map_err(|_| BluetoothError::Timeout {
            duration: self.timeout_duration,
        })??;

        trace!("Received ATT response: {} bytes", response.len());
        Ok(response)
    }

    /// Get device info
    #[must_use]
    pub fn device(&self) -> &Device {
        &self.device
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::{
        device::{Address, Device, DeviceInfo},
        l2cap::L2capChannel,
        transport::{Transport, TransportType},
    };
    use tokio::sync::Mutex;

    // Mock transport for testing
    struct MockTransport;

    #[async_trait::async_trait]
    impl Transport for MockTransport {
        fn transport_type(&self) -> TransportType {
            TransportType::Usb
        }

        async fn send_command(&mut self, _data: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn receive_event(&mut self) -> Result<Vec<u8>> {
            Ok(Vec::new())
        }

        async fn send_acl(&mut self, _data: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn receive_acl(&mut self) -> Result<Vec<u8>> {
            Ok(Vec::new())
        }

        fn is_connected(&self) -> bool {
            true
        }

        async fn close(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_gatt_client_creation() {
        let info = DeviceInfo::new(Address::from_bytes([1, 2, 3, 4, 5, 6]));
        let device = Arc::new(Device::new(info, 0x0040));
        let l2cap_channel = L2capChannel::new_att(0x0040);
        let transport = Arc::new(Mutex::new(MockTransport));

        let gatt = GattClient::new(device, l2cap_channel, transport);

        assert_eq!(gatt.services.len(), 0);
    }

    #[test]
    fn test_characteristic_properties() {
        let props = CharacteristicProperties::new().with_read().with_write().with_notify();

        assert!(props.read());
        assert!(props.write());
        assert!(!props.write_without_response());
        assert!(props.notify());
        assert!(!props.indicate());
    }

    /// Echoes the last ACL frame so [`GattClient::send_att_request`] can parse ATT payload.
    struct EchoAclTransport {
        last_acl: tokio::sync::Mutex<Option<Vec<u8>>>,
    }

    impl EchoAclTransport {
        fn new() -> Self {
            Self {
                last_acl: tokio::sync::Mutex::new(None),
            }
        }
    }

    #[async_trait::async_trait]
    impl Transport for EchoAclTransport {
        fn transport_type(&self) -> TransportType {
            TransportType::Usb
        }

        async fn send_command(&mut self, _data: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn receive_event(&mut self) -> Result<Vec<u8>> {
            Ok(Vec::new())
        }

        async fn send_acl(&mut self, data: &[u8]) -> Result<()> {
            *self.last_acl.lock().await = Some(data.to_vec());
            Ok(())
        }

        async fn receive_acl(&mut self) -> Result<Vec<u8>> {
            self.last_acl.lock().await.take().ok_or_else(|| {
                crate::error::BluetoothError::Transport(
                    crate::error::TransportError::Communication("no acl".into()),
                )
            })
        }

        fn is_connected(&self) -> bool {
            true
        }

        async fn close(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn send_att_request_round_trips_acl_framing() {
        let info = DeviceInfo::new(Address::from_bytes([1, 2, 3, 4, 5, 6]));
        let device = Arc::new(Device::new(info, 0x0040));
        let l2cap = L2capChannel::new_att(0x0040);
        let transport = Arc::new(Mutex::new(EchoAclTransport::new()));
        let client = GattClient::new(device, l2cap, transport);

        let att = [0x0Au8, 0x01, 0x00]; // Read Req, handle 1
        let out = client.send_att_request(&att).await.expect("att");
        assert_eq!(out, att);
    }

    #[test]
    fn gatt_client_with_timeout_updates_duration() {
        let info = DeviceInfo::new(Address::from_bytes([1, 2, 3, 4, 5, 6]));
        let device = Arc::new(Device::new(info, 0x0040));
        let l2cap = L2capChannel::new_att(0x0040);
        let transport = Arc::new(Mutex::new(MockTransport));
        let c = GattClient::new(device, l2cap, transport)
            .with_timeout(std::time::Duration::from_secs(9));
        assert_eq!(c.timeout_duration, std::time::Duration::from_secs(9));
    }
}
