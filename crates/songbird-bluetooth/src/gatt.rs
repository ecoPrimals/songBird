//! GATT Client - Generic Attribute Profile operations
//!
//! Provides high-level API for GATT service discovery and characteristic access.
//! Uses ATT (Attribute Protocol) over L2CAP channel 0x0004.

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
/// Note: Constants awaiting hardware validation - will be used in Phase 3
#[allow(dead_code)]
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
/// Note: Constants awaiting hardware validation - will be used in Phase 3
#[allow(dead_code)]
mod att_uuid {
    /// Primary Service UUID (0x2800)
    pub const PRIMARY_SERVICE: u16 = 0x2800;

    /// Characteristic UUID (0x2803)
    pub const CHARACTERISTIC: u16 = 0x2803;

    /// Client Characteristic Configuration Descriptor (0x2902)
    pub const CLIENT_CHAR_CONFIG: u16 = 0x2902;
}

/// GATT (Generic Attribute Profile) client for Bluetooth LE devices
///
/// Provides a high-level interface for interacting with GATT services and characteristics
/// on Bluetooth Low Energy devices. Supports reading, writing, and subscribing to
/// characteristic notifications.
///
/// # Example
///
/// ```no_run
/// use songbird_bluetooth::gatt::GattClient;
/// use songbird_bluetooth::transport::Transport;
///
/// # async fn example<T: Transport>(transport: T) {
/// let mut client = GattClient::new(transport);
/// // Discover services
/// client.discover_services().await.unwrap();
/// // Read characteristic
/// let value = client.read_characteristic(uuid).await.unwrap();
/// # }
/// ```
pub struct GattClient<T: Transport> {
    device: Arc<Device>,
    services: Vec<Service>,
    l2cap_channel: L2capChannel,
    transport: Arc<Mutex<T>>,
    timeout_duration: Duration,
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
    async fn send_att_request(&mut self, request: &[u8]) -> Result<Vec<u8>> {
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
        let service_index =
            self.services.iter().position(|s| &s.uuid == service_uuid).ok_or_else(|| {
                BluetoothError::Gatt(format!("Service not found: {service_uuid}"))
            })?;

        let start_handle = self.services[service_index].start_handle;
        let end_handle = self.services[service_index].end_handle;

        // Build ATT Read By Type Request for Characteristic
        let _request =
            Self::build_read_by_type_request(start_handle, end_handle, att_uuid::CHARACTERISTIC);

        trace!(
            "Sending ATT Read By Type Request for characteristics: start=0x{:04X}, end=0x{:04X}",
            start_handle,
            end_handle
        );

        // TODO: Send request over L2CAP ATT channel (0x0004)
        // Parse response and add characteristics to service

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
    #[allow(dead_code)]
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

                // TODO: Send request over L2CAP ATT channel (0x0004)
                // Parse response and return value

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
    #[allow(dead_code)]
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

                // TODO: Send request over L2CAP ATT channel (0x0004)
                // If with_response, wait for Write Response

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
    #[allow(clippy::unused_self)]
    fn build_write_command(&self, handle: u16, data: &[u8]) -> Vec<u8> {
        let mut request = vec![att_opcode::WRITE_CMD];
        request.extend_from_slice(&handle.to_le_bytes());
        request.extend_from_slice(data);
        request
    }

    /// Parse ATT Write Response
    /// Note: Awaiting hardware validation - will be used in Phase 3
    #[allow(dead_code)]
    #[allow(clippy::unused_self)]
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
                if !characteristic.properties.notify() {
                    return Err(BluetoothError::gatt(format!(
                        "Characteristic {uuid} does not support notifications"
                    )));
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
    use crate::{
        device::{Address, DeviceInfo},
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
}
