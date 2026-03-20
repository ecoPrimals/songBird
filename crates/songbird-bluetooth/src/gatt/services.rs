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
    #[allow(clippy::cognitive_complexity)]
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
    #[allow(clippy::unused_async)] // Placeholder for future async GATT service resolution
    pub async fn find_service(&self, uuid: &Uuid) -> Result<&Service> {
        self.services
            .iter()
            .find(|s| &s.uuid == uuid)
            .ok_or_else(|| BluetoothError::gatt(format!("Service not found: {uuid}")))
    }
}
