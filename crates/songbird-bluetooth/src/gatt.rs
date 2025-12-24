//! GATT Client - Generic Attribute Profile operations
//!
//! Provides high-level API for GATT service discovery and characteristic access.
//! Uses ATT (Attribute Protocol) over L2CAP channel 0x0004.

use crate::{device::Device, error::{BluetoothError, Result}};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
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
#[derive(Debug, Clone, Copy, Default)]
pub struct CharacteristicProperties {
    /// Can be read
    pub read: bool,
    
    /// Can be written
    pub write: bool,
    
    /// Can be written without response
    pub write_without_response: bool,
    
    /// Supports notifications
    pub notify: bool,
    
    /// Supports indications
    pub indicate: bool,
}

/// GATT Client
///
/// Provides access to GATT services and characteristics on a connected device.
///
/// # Example
///
/// ```rust,no_run
/// use songbird_bluetooth::{BluetoothHost, UsbTransport};
/// use uuid::Uuid;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let transport = UsbTransport::new().await?;
/// # let mut host = BluetoothHost::new(transport)?;
/// # let address = todo!();
/// let connection = host.connect(address).await?;
/// let gatt = host.gatt_client(address).await?;
///
/// // Discover services
/// let services = gatt.discover_services().await?;
///
/// // Read characteristic
/// let char_uuid = Uuid::parse_str("00002a00-0000-1000-8000-00805f9b34fb")?;
/// let data = gatt.read_characteristic(&char_uuid).await?;
/// # Ok(())
/// # }
/// ```
/// ATT opcodes
mod att_opcode {
    pub const ERROR_RSP: u8 = 0x01;
    pub const READ_BY_GROUP_TYPE_REQ: u8 = 0x10;
    pub const READ_BY_GROUP_TYPE_RSP: u8 = 0x11;
    pub const READ_BY_TYPE_REQ: u8 = 0x08;
    pub const READ_BY_TYPE_RSP: u8 = 0x09;
    pub const READ_REQ: u8 = 0x0A;
    pub const READ_RSP: u8 = 0x0B;
    pub const WRITE_REQ: u8 = 0x12;
    pub const WRITE_RSP: u8 = 0x13;
    pub const WRITE_CMD: u8 = 0x52;
    pub const HANDLE_VALUE_NTF: u8 = 0x1B;
}

/// ATT UUIDs
mod att_uuid {
    /// Primary Service UUID (0x2800)
    pub const PRIMARY_SERVICE: u16 = 0x2800;
    
    /// Characteristic UUID (0x2803)
    pub const CHARACTERISTIC: u16 = 0x2803;
    
    /// Client Characteristic Configuration Descriptor (0x2902)
    pub const CLIENT_CHAR_CONFIG: u16 = 0x2902;
}

pub struct GattClient {
    device: Arc<Device>,
    services: Vec<Service>,
}

impl GattClient {
    /// Create new GATT client
    #[must_use]
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            services: Vec::new(),
        }
    }

    /// Discover all services
    ///
    /// # Errors
    ///
    /// Returns error if service discovery fails
    pub async fn discover_services(&mut self) -> Result<&[Service]> {
        debug!("Discovering services on {}", self.device.address());

        self.services.clear();

        // Discover all primary services using ATT Read By Group Type
        let mut start_handle: u16 = 0x0001;
        let end_handle: u16 = 0xFFFF;

        loop {
            // Send ATT Read By Group Type Request for Primary Service
            let request = self.build_read_by_group_type_request(
                start_handle,
                end_handle,
                att_uuid::PRIMARY_SERVICE,
            );

            trace!("Sending ATT Read By Group Type Request: start=0x{:04X}", start_handle);

            // TODO: Send request over L2CAP ATT channel (0x0004)
            // For now, simulate empty response to complete the implementation structure
            
            // Parse response would go here
            // let services = self.parse_read_by_group_type_response(&response)?;
            
            // For now, break after first attempt (will be fixed when L2CAP is implemented)
            break;
        }

        debug!("Discovered {} services", self.services.len());
        Ok(&self.services)
    }

    /// Build ATT Read By Group Type Request
    fn build_read_by_group_type_request(
        &self,
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
    fn parse_read_by_group_type_response(&mut self, response: &[u8]) -> Result<Vec<Service>> {
        if response.is_empty() {
            return Err(BluetoothError::Gatt("Empty response".into()));
        }

        let opcode = response[0];

        // Check for error response
        if opcode == att_opcode::ERROR_RSP {
            return self.handle_att_error(response);
        }

        // Check for correct response opcode
        if opcode != att_opcode::READ_BY_GROUP_TYPE_RSP {
            return Err(BluetoothError::Gatt(
                format!("Unexpected opcode: 0x{:02X}", opcode)
            ));
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
                Uuid::from_u128(0x0000_0000_1000_8000_8000_00805F9B34FB | ((uuid_16 as u128) << 96))
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
    fn handle_att_error(&self, response: &[u8]) -> Result<Vec<Service>> {
        if response.len() < 4 {
            return Err(BluetoothError::Gatt("Invalid error response".into()));
        }

        let error_code = response[4];
        
        // Error code 0x0A means "Attribute Not Found" - end of discovery
        if error_code == 0x0A {
            debug!("Service discovery complete (Attribute Not Found)");
            return Ok(Vec::new());
        }

        Err(BluetoothError::Gatt(
            format!("ATT error: 0x{:02X}", error_code)
        ))
    }

    /// Find service by UUID
    ///
    /// # Errors
    ///
    /// Returns error if service not found
    pub async fn find_service(&self, uuid: &Uuid) -> Result<&Service> {
        self.services
            .iter()
            .find(|s| &s.uuid == uuid)
            .ok_or_else(|| BluetoothError::gatt(format!("Service not found: {uuid}")))
    }

    /// Discover characteristics for a service
    ///
    /// # Errors
    ///
    /// Returns error if characteristic discovery fails
    pub async fn discover_characteristics(&mut self, service_uuid: &Uuid) -> Result<()> {
        debug!("Discovering characteristics for service: {}", service_uuid);

        // Find the service
        let service_index = self.services
            .iter()
            .position(|s| &s.uuid == service_uuid)
            .ok_or_else(|| BluetoothError::Gatt(format!("Service not found: {}", service_uuid)))?;

        let start_handle = self.services[service_index].start_handle;
        let end_handle = self.services[service_index].end_handle;

        // Build ATT Read By Type Request for Characteristic
        let request = self.build_read_by_type_request(
            start_handle,
            end_handle,
            att_uuid::CHARACTERISTIC,
        );

        trace!(
            "Sending ATT Read By Type Request for characteristics: start=0x{:04X}, end=0x{:04X}",
            start_handle, end_handle
        );

        // TODO: Send request over L2CAP ATT channel (0x0004)
        // Parse response and add characteristics to service

        Ok(())
    }

    /// Build ATT Read By Type Request
    fn build_read_by_type_request(
        &self,
        start_handle: u16,
        end_handle: u16,
        attr_type: u16,
    ) -> Vec<u8> {
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
    fn parse_read_by_type_response(&self, response: &[u8]) -> Result<Vec<Characteristic>> {
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
            return Err(BluetoothError::Gatt(
                format!("ATT error: 0x{:02X}", response.get(4).copied().unwrap_or(0))
            ));
        }

        // Check for correct response opcode
        if opcode != att_opcode::READ_BY_TYPE_RSP {
            return Err(BluetoothError::Gatt(
                format!("Unexpected opcode: 0x{:02X}", opcode)
            ));
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
                Uuid::from_u128(0x0000_0000_1000_8000_8000_00805F9B34FB | ((uuid_16 as u128) << 96))
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

            // Parse properties from byte
            let properties = CharacteristicProperties {
                read: (properties_byte & 0x02) != 0,
                write: (properties_byte & 0x08) != 0,
                write_without_response: (properties_byte & 0x04) != 0,
                notify: (properties_byte & 0x10) != 0,
                indicate: (properties_byte & 0x20) != 0,
            };

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
    pub async fn read_characteristic(&self, uuid: &Uuid) -> Result<Vec<u8>> {
        debug!("Reading characteristic: {}", uuid);

        // Find characteristic
        for service in &self.services {
            if let Some(characteristic) = service.characteristics.iter().find(|c| &c.uuid == uuid) {
                if !characteristic.properties.read {
                    return Err(BluetoothError::gatt(
                        format!("Characteristic {uuid} does not support read"),
                    ));
                }

                // Build ATT Read Request
                let request = self.build_read_request(characteristic.handle);

                trace!("Sending ATT Read Request for handle 0x{:04X}", characteristic.handle);

                // TODO: Send request over L2CAP ATT channel (0x0004)
                // Parse response and return value
                
                return Ok(Vec::new());
            }
        }

        Err(BluetoothError::gatt(format!("Characteristic not found: {uuid}")))
    }

    /// Build ATT Read Request
    fn build_read_request(&self, handle: u16) -> Vec<u8> {
        let mut request = vec![att_opcode::READ_REQ];
        request.extend_from_slice(&handle.to_le_bytes());
        request
    }

    /// Parse ATT Read Response
    fn parse_read_response(&self, response: &[u8]) -> Result<Vec<u8>> {
        if response.is_empty() {
            return Err(BluetoothError::Gatt("Empty response".into()));
        }

        let opcode = response[0];

        // Check for error response
        if opcode == att_opcode::ERROR_RSP {
            return Err(BluetoothError::Gatt(
                format!("ATT error: 0x{:02X}", response.get(4).copied().unwrap_or(0))
            ));
        }

        // Check for correct response opcode
        if opcode != att_opcode::READ_RSP {
            return Err(BluetoothError::Gatt(
                format!("Unexpected opcode: 0x{:02X}", opcode)
            ));
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
    pub async fn write_characteristic(&self, uuid: &Uuid, data: &[u8]) -> Result<()> {
        debug!("Writing {} bytes to characteristic: {}", data.len(), uuid);

        // Find characteristic
        for service in &self.services {
            if let Some(characteristic) = service.characteristics.iter().find(|c| &c.uuid == uuid) {
                if !characteristic.properties.write && !characteristic.properties.write_without_response {
                    return Err(BluetoothError::gatt(
                        format!("Characteristic {uuid} does not support write"),
                    ));
                }

                // Choose write type based on properties
                let with_response = characteristic.properties.write;

                // Build ATT Write Request or Command
                let _request = if with_response {
                    self.build_write_request(characteristic.handle, data)
                } else {
                    self.build_write_command(characteristic.handle, data)
                };

                trace!(
                    "Would send ATT Write {} for handle 0x{:04X}",
                    if with_response { "Request" } else { "Command" },
                    characteristic.handle
                );

                // TODO: Send request over L2CAP ATT channel (0x0004)
                // If with_response, wait for Write Response
                
                return Ok(());
            }
        }

        Err(BluetoothError::gatt(format!("Characteristic not found: {uuid}")))
    }

    /// Build ATT Write Request (with response)
    fn build_write_request(&self, handle: u16, data: &[u8]) -> Vec<u8> {
        let mut request = vec![att_opcode::WRITE_REQ];
        request.extend_from_slice(&handle.to_le_bytes());
        request.extend_from_slice(data);
        request
    }

    /// Build ATT Write Command (without response)
    fn build_write_command(&self, handle: u16, data: &[u8]) -> Vec<u8> {
        let mut request = vec![att_opcode::WRITE_CMD];
        request.extend_from_slice(&handle.to_le_bytes());
        request.extend_from_slice(data);
        request
    }

    /// Parse ATT Write Response
    fn parse_write_response(&self, response: &[u8]) -> Result<()> {
        if response.is_empty() {
            return Err(BluetoothError::Gatt("Empty response".into()));
        }

        let opcode = response[0];

        // Check for error response
        if opcode == att_opcode::ERROR_RSP {
            return Err(BluetoothError::Gatt(
                format!("ATT error: 0x{:02X}", response.get(4).copied().unwrap_or(0))
            ));
        }

        // Check for correct response opcode
        if opcode != att_opcode::WRITE_RSP {
            return Err(BluetoothError::Gatt(
                format!("Unexpected opcode: 0x{:02X}", opcode)
            ));
        }

        Ok(())
    }

    /// Subscribe to characteristic notifications
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Characteristic not found
    /// - Notifications not supported
    /// - Subscription fails
    pub async fn subscribe_notifications(
        &self,
        uuid: &Uuid,
        _callback: impl Fn(Vec<u8>) + Send + Sync + 'static,
    ) -> Result<()> {
        debug!("Subscribing to notifications: {}", uuid);

        // Find characteristic
        for service in &self.services {
            if let Some(characteristic) = service.characteristics.iter().find(|c| &c.uuid == uuid) {
                if !characteristic.properties.notify {
                    return Err(BluetoothError::gatt(
                        format!("Characteristic {uuid} does not support notifications"),
                    ));
                }

                // TODO: Implement actual subscription using trouble-host
                return Ok(());
            }
        }

        Err(BluetoothError::gatt(format!("Characteristic not found: {uuid}")))
    }

    /// Get device info
    #[must_use]
    pub fn device(&self) -> &Device {
        &self.device
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{Address, DeviceInfo};

    #[test]
    fn test_gatt_client_creation() {
        let info = DeviceInfo::new(Address::from_bytes([1, 2, 3, 4, 5, 6]));
        let device = Arc::new(Device::new(info, 0));
        let gatt = GattClient::new(device);
        
        assert_eq!(gatt.services.len(), 0);
    }

    #[test]
    fn test_characteristic_properties() {
        let props = CharacteristicProperties {
            read: true,
            write: true,
            write_without_response: false,
            notify: true,
            indicate: false,
        };

        assert!(props.read);
        assert!(props.write);
        assert!(!props.write_without_response);
        assert!(props.notify);
        assert!(!props.indicate);
    }
}

