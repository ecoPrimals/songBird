// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! GATT characteristic types and characteristic read/write operations.

use super::GattClient;
use super::att::att_opcode;
use crate::error::{BluetoothError, Result};
use crate::transport::Transport;
use tracing::{debug, trace, warn};
use uuid::Uuid;

/// GATT Characteristic
#[derive(Debug, Clone)]
pub struct Characteristic {
    /// Characteristic UUID
    pub uuid: Uuid,

    /// Characteristic handle
    pub handle: u16,

    /// Properties (read, write, notify, etc.)
    pub properties: CharacteristicProperties,
}

/// Characteristic properties
///
/// Uses bitflags for efficient representation and operations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CharacteristicProperties {
    flags: u8,
}

impl CharacteristicProperties {
    /// Bit flag for read property
    pub const READ: u8 = 1 << 0;

    /// Bit flag for write property
    pub const WRITE: u8 = 1 << 1;

    /// Bit flag for write without response property
    pub const WRITE_WITHOUT_RESPONSE: u8 = 1 << 2;

    /// Bit flag for notify property
    pub const NOTIFY: u8 = 1 << 3;

    /// Bit flag for indicate property
    pub const INDICATE: u8 = 1 << 4;

    /// Create empty properties
    #[must_use]
    pub const fn new() -> Self {
        Self {
            flags: 0,
        }
    }

    /// Create from raw flags
    #[must_use]
    pub const fn from_flags(flags: u8) -> Self {
        Self {
            flags,
        }
    }

    /// Check if readable
    #[must_use]
    pub const fn read(&self) -> bool {
        (self.flags & Self::READ) != 0
    }

    /// Check if writable
    #[must_use]
    pub const fn write(&self) -> bool {
        (self.flags & Self::WRITE) != 0
    }

    /// Check if writable without response
    #[must_use]
    pub const fn write_without_response(&self) -> bool {
        (self.flags & Self::WRITE_WITHOUT_RESPONSE) != 0
    }

    /// Check if notifications supported
    #[must_use]
    pub const fn notify(&self) -> bool {
        (self.flags & Self::NOTIFY) != 0
    }

    /// Check if indications supported
    #[must_use]
    pub const fn indicate(&self) -> bool {
        (self.flags & Self::INDICATE) != 0
    }

    /// Set readable
    #[must_use]
    pub const fn with_read(mut self) -> Self {
        self.flags |= Self::READ;
        self
    }

    /// Set writable
    #[must_use]
    pub const fn with_write(mut self) -> Self {
        self.flags |= Self::WRITE;
        self
    }

    /// Set writable without response
    #[must_use]
    pub const fn with_write_without_response(mut self) -> Self {
        self.flags |= Self::WRITE_WITHOUT_RESPONSE;
        self
    }

    /// Set notifications
    #[must_use]
    pub const fn with_notify(mut self) -> Self {
        self.flags |= Self::NOTIFY;
        self
    }

    /// Set indications
    #[must_use]
    pub const fn with_indicate(mut self) -> Self {
        self.flags |= Self::INDICATE;
        self
    }
}

impl<T: Transport + 'static> GattClient<T> {
    /// Discover characteristics for a service
    ///
    /// # Errors
    ///
    /// Returns error if characteristic discovery fails
    #[expect(
        clippy::unused_async,
        reason = "placeholder for future GATT operations (ATT over L2CAP)"
    )]
    pub async fn discover_characteristics(&mut self, service_uuid: &Uuid) -> Result<()> {
        debug!("Discovering characteristics for service: {}", service_uuid);

        // Find the service
        let service_index =
            self.services.iter().position(|s| &s.uuid == service_uuid).ok_or_else(|| {
                BluetoothError::Gatt(format!("Service not found: {service_uuid}"))
            })?;

        let start_handle = self.services[service_index].start_handle;
        let end_handle = self.services[service_index].end_handle;

        // Build ATT Read By Type Request for Characteristic
        let _request = Self::build_read_by_type_request(
            start_handle,
            end_handle,
            super::att::att_uuid::CHARACTERISTIC,
        );

        trace!(
            "Sending ATT Read By Type Request for characteristics: start=0x{:04X}, end=0x{:04X}",
            start_handle, end_handle
        );

        // ATT request bytes are assembled above; sending on L2CAP ATT (CID 0x0004) and parsing the
        // response into `self.services` is not wired on this code path yet.

        Ok(())
    }

    /// Build ATT Read By Type Request
    fn build_read_by_type_request(start_handle: u16, end_handle: u16, attr_type: u16) -> Vec<u8> {
        let mut request = vec![att_opcode::READ_BY_TYPE_REQ];

        // Start handle (little-endian)
        request.extend_from_slice(&start_handle.to_le_bytes());

        // End handle (little-endian)
        request.extend_from_slice(&end_handle.to_le_bytes());

        // Attribute Type (16-bit UUID, little-endian)
        request.extend_from_slice(&attr_type.to_le_bytes());

        request
    }

    /// Parse ATT Read By Type Response for characteristics
    /// Note: Awaiting hardware validation - will be used in Phase 3
    #[expect(
        dead_code,
        reason = "ATT response parser for Phase 3 L2CAP; covered by unit tests only"
    )]
    fn parse_read_by_type_response(response: &[u8]) -> Result<Vec<Characteristic>> {
        if response.is_empty() {
            return Err(BluetoothError::Gatt("Empty response".into()));
        }

        let opcode = response[0];

        // Check for error response
        if opcode == att_opcode::ERROR_RSP {
            if response.len() >= 5 && response[4] == 0x0A {
                // Attribute Not Found - normal end of discovery
                return Ok(Vec::new());
            }
            return Err(BluetoothError::Gatt(format!(
                "ATT error: 0x{:02X}",
                response.get(4).copied().unwrap_or(0)
            )));
        }

        // Check for correct response opcode
        if opcode != att_opcode::READ_BY_TYPE_RSP {
            return Err(BluetoothError::Gatt(format!("Unexpected opcode: 0x{opcode:02X}")));
        }

        if response.len() < 2 {
            return Err(BluetoothError::Gatt("Response too short".into()));
        }

        let length = response[1] as usize;
        let mut characteristics = Vec::new();
        let mut offset = 2;

        // Parse characteristic declarations
        while offset + length <= response.len() {
            let handle = u16::from_le_bytes([response[offset], response[offset + 1]]);

            if offset + 2 >= response.len() {
                break;
            }

            let properties_byte = response[offset + 2];
            let value_handle = if offset + 4 < response.len() {
                u16::from_le_bytes([response[offset + 3], response[offset + 4]])
            } else {
                handle + 1
            };

            // Parse UUID
            let uuid = if length == 7 {
                // 16-bit UUID
                let uuid_16 = u16::from_le_bytes([response[offset + 5], response[offset + 6]]);
                Uuid::from_u128(
                    0x0000_0000_1000_8000_8000_0080_5F9B_34FB | (u128::from(uuid_16) << 96),
                )
            } else if length == 21 {
                // 128-bit UUID
                let mut uuid_bytes = [0u8; 16];
                uuid_bytes.copy_from_slice(&response[offset + 5..offset + 21]);
                Uuid::from_bytes_le(uuid_bytes)
            } else {
                warn!("Unknown characteristic format length: {}", length);
                offset += length;
                continue;
            };

            // Parse properties from byte (BT spec bit positions)
            let mut properties = CharacteristicProperties::new();
            if (properties_byte & 0x02) != 0 {
                properties = properties.with_read();
            }
            if (properties_byte & 0x08) != 0 {
                properties = properties.with_write();
            }
            if (properties_byte & 0x04) != 0 {
                properties = properties.with_write_without_response();
            }
            if (properties_byte & 0x10) != 0 {
                properties = properties.with_notify();
            }
            if (properties_byte & 0x20) != 0 {
                properties = properties.with_indicate();
            }

            debug!(
                "Found characteristic: UUID={}, handle=0x{:04X}, props={:?}",
                uuid, value_handle, properties
            );

            characteristics.push(Characteristic {
                uuid,
                handle: value_handle,
                properties,
            });

            offset += length;
        }

        Ok(characteristics)
    }

    /// Read characteristic value
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Characteristic not found
    /// - Read not supported
    /// - Read fails
    #[expect(
        clippy::unused_async,
        reason = "placeholder for future GATT operations (ATT over L2CAP)"
    )]
    pub async fn read_characteristic(&self, uuid: &Uuid) -> Result<Vec<u8>> {
        debug!("Reading characteristic: {}", uuid);

        // Find characteristic
        for service in &self.services {
            if let Some(characteristic) = service.characteristics.iter().find(|c| &c.uuid == uuid) {
                if !characteristic.properties.read() {
                    return Err(BluetoothError::gatt(format!(
                        "Characteristic {uuid} does not support read"
                    )));
                }

                // Build ATT Read Request
                let _request = Self::build_read_request(characteristic.handle);

                trace!("Sending ATT Read Request for handle 0x{:04X}", characteristic.handle);

                // Returns empty payload: ATT read is not sent on L2CAP until the transport hook exists.

                return Ok(Vec::new());
            }
        }

        Err(BluetoothError::gatt(format!("Characteristic not found: {uuid}")))
    }

    /// Build ATT Read Request
    fn build_read_request(handle: u16) -> Vec<u8> {
        let mut request = vec![att_opcode::READ_REQ];
        request.extend_from_slice(&handle.to_le_bytes());
        request
    }

    /// Parse ATT Read Response
    /// Note: Awaiting hardware validation - will be used in Phase 3
    #[expect(
        dead_code,
        reason = "ATT response parser for Phase 3 L2CAP; covered by unit tests only"
    )]
    fn parse_read_response(response: &[u8]) -> Result<Vec<u8>> {
        if response.is_empty() {
            return Err(BluetoothError::Gatt("Empty response".into()));
        }

        let opcode = response[0];

        // Check for error response
        if opcode == att_opcode::ERROR_RSP {
            return Err(BluetoothError::Gatt(format!(
                "ATT error: 0x{:02X}",
                response.get(4).copied().unwrap_or(0)
            )));
        }

        // Check for correct response opcode
        if opcode != att_opcode::READ_RSP {
            return Err(BluetoothError::Gatt(format!("Unexpected opcode: 0x{opcode:02X}")));
        }

        // Value starts at byte 1
        Ok(response[1..].to_vec())
    }

    /// Write characteristic value
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Characteristic not found
    /// - Write not supported
    /// - Write fails
    #[expect(
        clippy::unused_async,
        reason = "placeholder for future GATT operations (ATT over L2CAP)"
    )]
    pub async fn write_characteristic(&self, uuid: &Uuid, data: &[u8]) -> Result<()> {
        debug!("Writing {} bytes to characteristic: {}", data.len(), uuid);

        // Find characteristic
        for service in &self.services {
            if let Some(characteristic) = service.characteristics.iter().find(|c| &c.uuid == uuid) {
                if !characteristic.properties.write()
                    && !characteristic.properties.write_without_response()
                {
                    return Err(BluetoothError::gatt(format!(
                        "Characteristic {uuid} does not support write"
                    )));
                }

                // Choose write type based on properties
                let with_response = characteristic.properties.write();

                // Build ATT Write Request or Command
                let _request = if with_response {
                    Self::build_write_request(characteristic.handle, data)
                } else {
                    self.build_write_command(characteristic.handle, data)
                };

                trace!(
                    "Would send ATT Write {} for handle 0x{:04X}",
                    if with_response {
                        "Request"
                    } else {
                        "Command"
                    },
                    characteristic.handle
                );

                // Succeeds without I/O: write request/command bytes are not sent on L2CAP in this build.

                return Ok(());
            }
        }

        Err(BluetoothError::gatt(format!("Characteristic not found: {uuid}")))
    }

    /// Build ATT Write Request (with response)
    fn build_write_request(handle: u16, data: &[u8]) -> Vec<u8> {
        let mut request = vec![att_opcode::WRITE_REQ];
        request.extend_from_slice(&handle.to_le_bytes());
        request.extend_from_slice(data);
        request
    }

    /// Build ATT Write Command (without response)
    #[expect(clippy::unused_self, reason = "instance method for symmetry with build_write_request")]
    fn build_write_command(&self, handle: u16, data: &[u8]) -> Vec<u8> {
        let mut request = vec![att_opcode::WRITE_CMD];
        request.extend_from_slice(&handle.to_le_bytes());
        request.extend_from_slice(data);
        request
    }

    /// Parse ATT Write Response
    /// Note: Awaiting hardware validation - will be used in Phase 3
    #[expect(
        dead_code,
        reason = "ATT response parser for Phase 3 L2CAP; covered by unit tests only"
    )]
    #[expect(clippy::unused_self, reason = "instance method for symmetry with other parse helpers")]
    fn parse_write_response(&self, response: &[u8]) -> Result<()> {
        if response.is_empty() {
            return Err(BluetoothError::Gatt("Empty response".into()));
        }

        let opcode = response[0];

        // Check for error response
        if opcode == att_opcode::ERROR_RSP {
            return Err(BluetoothError::Gatt(format!(
                "ATT error: 0x{:02X}",
                response.get(4).copied().unwrap_or(0)
            )));
        }

        // Check for correct response opcode
        if opcode != att_opcode::WRITE_RSP {
            return Err(BluetoothError::Gatt(format!("Unexpected opcode: 0x{opcode:02X}")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::att_opcode;
    use crate::error::BluetoothError;
    use crate::gatt::GattClient;
    use crate::transport::{Transport, TransportType};
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

    #[test]
    fn build_read_by_type_request_encodes_opcode_handles_and_type_le() {
        let req = GattClient::<MockTransport>::build_read_by_type_request(0x0001, 0xFFFF, 0x2803);
        assert_eq!(req[0], att_opcode::READ_BY_TYPE_REQ);
        assert_eq!(&req[1..3], &[0x01, 0x00]);
        assert_eq!(&req[3..5], &[0xFF, 0xFF]);
        assert_eq!(&req[5..7], &[0x03, 0x28]);
    }

    #[test]
    fn build_read_request_encodes_handle_little_endian() {
        let req = GattClient::<MockTransport>::build_read_request(0x1234);
        assert_eq!(req, vec![att_opcode::READ_REQ, 0x34, 0x12]);
    }

    #[test]
    fn parse_read_response_extracts_value_after_opcode() {
        let mut rsp = vec![att_opcode::READ_RSP, 0xDE, 0xAD];
        let v = GattClient::<MockTransport>::parse_read_response(&rsp).expect("ok");
        assert_eq!(v, vec![0xDE, 0xAD]);
        rsp[0] = att_opcode::ERROR_RSP;
        rsp.resize(5, 0);
        let e = GattClient::<MockTransport>::parse_read_response(&rsp).expect_err("err");
        assert!(matches!(e, BluetoothError::Gatt(_)));
    }

    #[test]
    fn build_write_request_and_command_include_payload() {
        let data = [1u8, 2, 3];
        let req = GattClient::<MockTransport>::build_write_request(0x0002, &data);
        assert_eq!(req[0], att_opcode::WRITE_REQ);
        assert_eq!(&req[1..3], &[0x02, 0x00]);
        assert_eq!(&req[3..], &data);

        let client = sample_client();
        let cmd = client.build_write_command(0x0003, &data);
        assert_eq!(cmd[0], att_opcode::WRITE_CMD);
        assert_eq!(&cmd[1..3], &[0x03, 0x00]);
        assert_eq!(&cmd[3..], &data);
    }

    #[test]
    fn parse_write_response_accepts_write_rsp() {
        let client = sample_client();
        client.parse_write_response(&[att_opcode::WRITE_RSP]).expect("ok");
        let e = client.parse_write_response(&[]).expect_err("empty");
        assert!(matches!(e, BluetoothError::Gatt(_)));
    }

    fn sample_client() -> GattClient<MockTransport> {
        use crate::device::{Address, Device, DeviceInfo};
        use crate::l2cap::L2capChannel;
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let info = DeviceInfo::new(Address::from_bytes([1, 2, 3, 4, 5, 6]));
        let device = Arc::new(Device::new(info, 0x0040));
        let l2cap = L2capChannel::new_att(0x0040);
        GattClient::new(device, l2cap, Arc::new(Mutex::new(MockTransport)))
    }

    #[test]
    fn parse_read_by_type_response_empty_yields_error() {
        let e = GattClient::<MockTransport>::parse_read_by_type_response(&[]).expect_err("empty");
        assert!(matches!(e, BluetoothError::Gatt(_)));
    }

    #[test]
    fn parse_read_by_type_response_error_not_found_returns_empty_list() {
        let rsp = [att_opcode::ERROR_RSP, att_opcode::READ_BY_TYPE_REQ, 0x01, 0x00, 0x0A];
        let v = GattClient::<MockTransport>::parse_read_by_type_response(&rsp).expect("ok");
        assert!(v.is_empty());
    }

    #[test]
    fn parse_read_by_type_response_parses_16_bit_characteristic_declaration() {
        // length 7: handle(2) props(1) value_handle(2) uuid16(2)
        let rsp = [att_opcode::READ_BY_TYPE_RSP, 7, 0x05, 0x00, 0x02, 0x06, 0x00, 0x2A, 0x18];
        let chars = GattClient::<MockTransport>::parse_read_by_type_response(&rsp).expect("parse");
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].handle, 0x0006);
        let expected = Uuid::from_u128(
            0x0000_0000_1000_8000_8000_0080_5F9B_34FB | (u128::from(0x182Au16) << 96),
        );
        assert_eq!(chars[0].uuid, expected);
        assert!(chars[0].properties.read());
    }
}
