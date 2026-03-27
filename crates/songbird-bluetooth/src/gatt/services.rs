// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! GATT service type and primary service discovery.

use super::GattClient;
use super::att::{att_opcode, att_uuid};
use super::characteristics::Characteristic;
use crate::error::{BluetoothError, Result};
use crate::transport::Transport;
use tracing::{debug, trace, warn};
use uuid::Uuid;

/// GATT Service
#[derive(Debug, Clone)]
pub struct Service {
    /// Service UUID
    pub uuid: Uuid,

    /// Start handle
    pub start_handle: u16,

    /// End handle
    pub end_handle: u16,

    /// Characteristics in this service
    pub characteristics: Vec<Characteristic>,
}

impl Service {
    /// Create new service
    #[must_use]
    pub const fn new(uuid: Uuid, start_handle: u16, end_handle: u16) -> Self {
        Self {
            uuid,
            start_handle,
            end_handle,
            characteristics: Vec::new(),
        }
    }
}

impl<T: Transport + 'static> GattClient<T> {
    /// Discover all services
    ///
    /// # Errors
    ///
    /// Returns error if service discovery fails
    #[allow(
        clippy::cognitive_complexity,
        reason = "ATT primary service discovery kept as one loop for clarity"
    )]
    pub async fn discover_services(&mut self) -> Result<&[Service]> {
        debug!("Discovering services on {}", self.device.address());

        self.services.clear();

        // Discover all primary services using ATT Read By Group Type
        let mut start_handle: u16 = 0x0001;
        let end_handle: u16 = 0xFFFF;

        loop {
            // Send ATT Read By Group Type Request for Primary Service
            let request = Self::build_read_by_group_type_request(
                start_handle,
                end_handle,
                att_uuid::PRIMARY_SERVICE,
            );

            trace!("Sending ATT Read By Group Type Request: start=0x{:04X}", start_handle);

            // Send request and get response
            let response = self.send_att_request(&request).await?;

            // Parse response
            let discovered_services = Self::parse_read_by_group_type_response(&response)?;

            if discovered_services.is_empty() {
                // No more services
                break;
            }

            // Add services to list
            self.services.extend(discovered_services.clone());

            // Update start handle for next iteration
            if let Some(last_service) = discovered_services.last() {
                start_handle = last_service.end_handle + 1;
                if start_handle == 0 {
                    // Wrapped around
                    break;
                }
            } else {
                break;
            }
        }

        debug!("Discovered {} services", self.services.len());
        Ok(&self.services)
    }

    /// Build ATT Read By Group Type Request
    fn build_read_by_group_type_request(
        start_handle: u16,
        end_handle: u16,
        group_type: u16,
    ) -> Vec<u8> {
        let mut request = vec![att_opcode::READ_BY_GROUP_TYPE_REQ];

        // Start handle (little-endian)
        request.extend_from_slice(&start_handle.to_le_bytes());

        // End handle (little-endian)
        request.extend_from_slice(&end_handle.to_le_bytes());

        // Attribute Group Type (16-bit UUID, little-endian)
        request.extend_from_slice(&group_type.to_le_bytes());

        request
    }

    /// Parse ATT Read By Group Type Response
    fn parse_read_by_group_type_response(response: &[u8]) -> Result<Vec<Service>> {
        if response.is_empty() {
            return Err(BluetoothError::Gatt("Empty response".into()));
        }

        let opcode = response[0];

        // Check for error response
        if opcode == att_opcode::ERROR_RSP {
            return Self::handle_att_error(response);
        }

        // Check for correct response opcode
        if opcode != att_opcode::READ_BY_GROUP_TYPE_RSP {
            return Err(BluetoothError::Gatt(format!("Unexpected opcode: 0x{opcode:02X}")));
        }

        if response.len() < 2 {
            return Err(BluetoothError::Gatt("Response too short".into()));
        }

        let length = response[1] as usize;
        let mut services = Vec::new();
        let mut offset = 2;

        // Parse attribute data list
        while offset + length <= response.len() {
            let start_handle = u16::from_le_bytes([response[offset], response[offset + 1]]);
            let end_handle = u16::from_le_bytes([response[offset + 2], response[offset + 3]]);

            // Parse UUID (can be 16-bit or 128-bit)
            let uuid = if length == 6 {
                // 16-bit UUID
                let uuid_16 = u16::from_le_bytes([response[offset + 4], response[offset + 5]]);
                Uuid::from_u128(
                    0x0000_0000_1000_8000_8000_0080_5F9B_34FB | (u128::from(uuid_16) << 96),
                )
            } else if length == 20 {
                // 128-bit UUID
                let mut uuid_bytes = [0u8; 16];
                uuid_bytes.copy_from_slice(&response[offset + 4..offset + 20]);
                Uuid::from_bytes_le(uuid_bytes)
            } else {
                warn!("Unknown UUID length: {}", length - 4);
                offset += length;
                continue;
            };

            debug!(
                "Found service: UUID={}, handles=0x{:04X}-0x{:04X}",
                uuid, start_handle, end_handle
            );

            let service = Service::new(uuid, start_handle, end_handle);
            services.push(service);

            offset += length;
        }

        Ok(services)
    }

    /// Handle ATT error response
    fn handle_att_error(response: &[u8]) -> Result<Vec<Service>> {
        if response.len() < 4 {
            return Err(BluetoothError::Gatt("Invalid error response".into()));
        }

        let error_code = response[4];

        // Error code 0x0A means "Attribute Not Found" - end of discovery
        if error_code == 0x0A {
            debug!("Service discovery complete (Attribute Not Found)");
            return Ok(Vec::new());
        }

        Err(BluetoothError::Gatt(format!("ATT error: 0x{error_code:02X}")))
    }

    /// Find service by UUID
    ///
    /// # Errors
    ///
    /// Returns error if service not found
    #[expect(clippy::unused_async, reason = "placeholder for future async GATT service resolution")]
    pub async fn find_service(&self, uuid: &Uuid) -> Result<&Service> {
        self.services
            .iter()
            .find(|s| &s.uuid == uuid)
            .ok_or_else(|| BluetoothError::gatt(format!("Service not found: {uuid}")))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::Service;
    use crate::device::{Address, DeviceInfo};
    use crate::gatt::GattClient;
    use crate::gatt::att::{att_opcode, att_uuid};
    use crate::l2cap::L2capChannel;
    use crate::transport::{Transport, TransportType};
    use tokio::sync::Mutex;
    use uuid::Uuid;

    struct MockTransport;

    #[async_trait::async_trait]
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

    fn sample_gatt_client() -> GattClient<MockTransport> {
        let info = DeviceInfo::new(Address::from_bytes([1, 2, 3, 4, 5, 6]));
        let device = std::sync::Arc::new(crate::device::Device::new(info, 0x0040));
        let l2cap_channel = L2capChannel::new_att(0x0040);
        let transport = std::sync::Arc::new(Mutex::new(MockTransport));
        GattClient::new(device, l2cap_channel, transport)
    }

    #[test]
    fn service_new_initializes_fields_and_empty_characteristics() {
        let u = Uuid::from_u128(0x1234);
        let s = Service::new(u, 0x0001, 0x000A);
        assert_eq!(s.uuid, u);
        assert_eq!(s.start_handle, 0x0001);
        assert_eq!(s.end_handle, 0x000A);
        assert!(s.characteristics.is_empty());
    }

    #[test]
    fn primary_service_uuid_constant_matches_bluetooth_base() {
        assert_eq!(att_uuid::PRIMARY_SERVICE, 0x2800);
    }

    #[test]
    fn build_read_by_group_type_request_encodes_handles_and_group_type_le() {
        let req = GattClient::<MockTransport>::build_read_by_group_type_request(
            0x0001,
            0xFFFF,
            att_uuid::PRIMARY_SERVICE,
        );
        assert_eq!(req[0], att_opcode::READ_BY_GROUP_TYPE_REQ);
        assert_eq!(&req[1..3], &[0x01, 0x00]);
        assert_eq!(&req[3..5], &[0xFF, 0xFF]);
        assert_eq!(&req[5..7], &[0x00, 0x28]);
    }

    #[test]
    fn parse_read_by_group_type_response_rejects_empty() {
        let err =
            GattClient::<MockTransport>::parse_read_by_group_type_response(&[]).expect_err("empty");
        assert!(err.to_string().contains("Empty") || format!("{err:?}").contains("Gatt"));
    }

    #[test]
    fn parse_read_by_group_type_response_rejects_unexpected_opcode() {
        let rsp = [0xFF, 6];
        let err = GattClient::<MockTransport>::parse_read_by_group_type_response(&rsp)
            .expect_err("bad opcode");
        assert!(err.to_string().contains("Unexpected opcode") || err.to_string().contains("0xFF"));
    }

    #[test]
    fn parse_read_by_group_type_response_error_rsp_attribute_not_found_yields_empty() {
        // ERROR_RSP: opcode, request opcode, handle LE, error code 0x0A at index 4
        let rsp = [att_opcode::ERROR_RSP, att_opcode::READ_BY_GROUP_TYPE_REQ, 0x01, 0x00, 0x0A];
        let services =
            GattClient::<MockTransport>::parse_read_by_group_type_response(&rsp).expect("ok");
        assert!(services.is_empty());
    }

    #[test]
    fn parse_read_by_group_type_response_error_rsp_other_code_fails() {
        let rsp = [att_opcode::ERROR_RSP, 0, 0, 0, 0x03];
        let err = GattClient::<MockTransport>::parse_read_by_group_type_response(&rsp)
            .expect_err("att error");
        assert!(err.to_string().contains("ATT error") || err.to_string().contains("0x03"));
    }

    #[test]
    fn parse_read_by_group_type_response_parses_16_bit_service_uuid() {
        // READ_BY_GROUP_TYPE_RSP, length=6 per record, one primary service 0x1800 (Battery)
        let rsp = [att_opcode::READ_BY_GROUP_TYPE_RSP, 6, 0x01, 0x00, 0x05, 0x00, 0x00, 0x18];
        let services =
            GattClient::<MockTransport>::parse_read_by_group_type_response(&rsp).expect("parse");
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].start_handle, 0x0001);
        assert_eq!(services[0].end_handle, 0x0005);
        let expected = Uuid::from_u128(
            0x0000_0000_1000_8000_8000_0080_5F9B_34FB | (u128::from(0x1800u16) << 96),
        );
        assert_eq!(services[0].uuid, expected);
    }

    #[test]
    fn parse_read_by_group_type_response_parses_128_bit_service_uuid() {
        let uuid_bytes = [
            0x12u8, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88,
        ];
        let mut rsp = vec![att_opcode::READ_BY_GROUP_TYPE_RSP, 20];
        rsp.extend_from_slice(&[0x10, 0x00, 0x20, 0x00]);
        rsp.extend_from_slice(&uuid_bytes);
        let expected = Uuid::from_bytes_le(uuid_bytes);
        let services =
            GattClient::<MockTransport>::parse_read_by_group_type_response(&rsp).expect("parse");
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].uuid, expected);
        assert_eq!(services[0].start_handle, 0x0010);
        assert_eq!(services[0].end_handle, 0x0020);
    }

    #[test]
    fn parse_read_by_group_type_response_truncated_after_opcode_fails() {
        let rsp = [att_opcode::READ_BY_GROUP_TYPE_RSP];
        let err = GattClient::<MockTransport>::parse_read_by_group_type_response(&rsp)
            .expect_err("short");
        assert!(err.to_string().contains("short") || err.to_string().contains("too"));
    }

    #[tokio::test]
    async fn find_service_returns_matching_service() {
        let mut client = sample_gatt_client();
        let u = Uuid::from_u128(0xAAA);
        client.services.push(Service::new(u, 1, 2));
        let found = client.find_service(&u).await.expect("found");
        assert_eq!(found.uuid, u);
    }

    #[tokio::test]
    async fn find_service_missing_returns_error() {
        let client = sample_gatt_client();
        let u = Uuid::from_u128(0xBBB);
        let err = client.find_service(&u).await.expect_err("missing");
        assert!(err.to_string().contains("not found") || err.to_string().contains("Service"));
    }
}
