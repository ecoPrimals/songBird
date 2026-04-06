// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! BLE Host - Main entry point for Bluetooth operations
//!
//! Pure Rust BLE host built on trouble-host (Embassy project).
//! Provides high-level API for scanning, connecting, and GATT operations.

mod scan;

use crate::{
    controller::ControllerAdapter,
    device::{Address, Device, DeviceInfo},
    error::{BluetoothError, Result},
    gatt::GattClient,
    l2cap::L2capManager,
    transport::Transport,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// BLE Host configuration
#[derive(Debug, Clone)]
pub struct HostConfig {
    /// Device name
    pub device_name: String,

    /// Scan window (milliseconds)
    pub scan_window_ms: u16,

    /// Scan interval (milliseconds)
    pub scan_interval_ms: u16,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Max connections
    pub max_connections: usize,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            device_name: "Songbird".to_string(),
            scan_window_ms: 100,
            scan_interval_ms: 100,
            connection_timeout: Duration::from_secs(5),
            max_connections: 4,
        }
    }
}

/// Bluetooth Host
///
/// Main interface for BLE operations. Manages scanning, connections,
/// and GATT client operations.
///
/// # Example
///
/// ```rust,no_run
/// use songbird_bluetooth::{BluetoothHost, UsbTransport};
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let transport = UsbTransport::new().await?;
/// let mut host = BluetoothHost::new(transport)?;
///
/// // Scan for devices
/// let devices = host.scan_devices(Duration::from_secs(5)).await?;
/// println!("Found {} devices", devices.len());
///
/// // Connect to first device
/// if let Some(device) = devices.first() {
///     let connection = host.connect(device.address).await?;
///     println!("Connected to {}", device.address);
/// }
/// # Ok(())
/// # }
/// ```
pub struct BluetoothHost<T: Transport> {
    transport: Arc<Mutex<T>>,
    controller: Arc<ControllerAdapter<T>>,
    config: HostConfig,
    connections: Arc<RwLock<HashMap<Address, Arc<Device>>>>,
    scanning: Arc<Mutex<bool>>,
    l2cap_manager: Arc<L2capManager>,
}

impl<T: Transport + 'static> BluetoothHost<T> {
    /// Create new Bluetooth host
    ///
    /// # Errors
    ///
    /// Returns error if transport initialization fails
    pub fn new(transport: T) -> Result<Self> {
        Self::with_config(transport, HostConfig::default())
    }

    /// Create host with custom configuration
    ///
    /// # Errors
    ///
    /// Returns error if configuration is invalid
    pub fn with_config(transport: T, config: HostConfig) -> Result<Self> {
        if !transport.is_connected() {
            return Err(BluetoothError::InvalidOperation("Transport not connected".into()));
        }

        info!("Initializing Bluetooth host: {}", config.device_name);

        let transport = Arc::new(Mutex::new(transport));
        let controller = Arc::new(ControllerAdapter::new(Arc::clone(&transport)));

        Ok(Self {
            transport,
            controller,
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            scanning: Arc::new(Mutex::new(false)),
            l2cap_manager: Arc::new(L2capManager::new()),
        })
    }

    /// Connect to a device
    ///
    /// # Complexity Note
    ///
    /// This function has high cognitive complexity (26/25) due to:
    /// - Connection state validation (already connected, limit checks)
    /// - HCI connection establishment sequence
    /// - Service discovery and characteristic enumeration
    /// - Error handling and cleanup across multiple async operations
    ///
    /// The complexity is justified by the BLE connection protocol requirements.
    /// Splitting would fragment the connection lifecycle and make debugging harder.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Connection fails
    /// - Device not found
    /// - Timeout occurs
    pub async fn connect(&mut self, address: Address) -> Result<Arc<Device>> {
        info!("Connecting to device: {}", address);

        // Check if already connected
        let connections = self.connections.read().await;
        if let Some(device) = connections.get(&address) {
            debug!("Device already connected");
            return Ok(Arc::clone(device));
        }
        drop(connections);

        // Check connection limit
        if self.connection_count().await >= self.config.max_connections {
            return Err(BluetoothError::InvalidOperation(format!(
                "Max connections ({}) reached",
                self.config.max_connections
            )));
        }

        // Perform actual BLE connection
        let handle = self.create_connection(address).await?;

        // Create device with connection handle
        let info = DeviceInfo::new(address);
        let device = Arc::new(Device::new(info, handle));

        // Store connection
        self.connections.write().await.insert(address, Arc::clone(&device));

        info!("Connected to {} (handle: 0x{:04X})", address, handle);
        Ok(device)
    }

    /// Create BLE connection using HCI commands
    async fn create_connection(&self, address: Address) -> Result<u16> {
        debug!("Creating BLE connection to {}", address);

        // Send HCI_LE_Create_Connection command
        self.send_create_connection_command(address).await?;

        // Wait for LE Connection Complete event
        let handle = self.wait_for_connection_complete().await?;

        debug!("Connection established with handle: 0x{:04X}", handle);
        Ok(handle)
    }

    /// Send HCI LE Create Connection command
    async fn send_create_connection_command(&self, address: Address) -> Result<()> {
        debug!("Sending LE Create Connection command");

        // HCI_LE_Create_Connection (0x200D)
        // Connection parameters optimized for low latency
        let addr_bytes = address.as_bytes();

        let mut cmd = vec![
            0x01, // Command packet
            0x0D, 0x20, // Opcode: LE Create Connection
            0x19, // Parameter length (25 bytes)
            // Scan parameters
            0x60, 0x00, // Scan Interval: 96 (60ms)
            0x30, 0x00, // Scan Window: 48 (30ms)
            // Connection parameters
            0x00, // Initiator Filter Policy: Use peer address
            0x00, // Peer Address Type: Public Device Address
        ];

        // Peer Address (6 bytes, little-endian)
        cmd.extend_from_slice(addr_bytes);

        cmd.extend_from_slice(&[
            0x00, // Own Address Type: Public Device Address
            // Connection interval
            0x18, 0x00, // Min: 24 (30ms)
            0x28, 0x00, // Max: 40 (50ms)
            // Connection parameters
            0x00, 0x00, // Connection Latency: 0
            0x80, 0x0C, // Supervision Timeout: 3200 (32s)
            // CE Length
            0x00, 0x00, // Min CE Length: 0
            0x00, 0x00, // Max CE Length: 0
        ]);

        self.controller.send_command(&cmd).await?;

        // Wait for Command Status event
        let status = timeout(Duration::from_secs(1), self.controller.receive_event())
            .await
            .map_err(|_| BluetoothError::Timeout {
                duration: Duration::from_secs(1),
            })??;

        // Check command status (should be 0x00 for success)
        if status.len() > 2 && status[2] != 0x00 {
            return Err(BluetoothError::Hci(format!(
                "Connection command failed with status: 0x{:02X}",
                status[2]
            )));
        }

        debug!("Create connection command sent successfully");
        Ok(())
    }

    /// Wait for LE Connection Complete event
    async fn wait_for_connection_complete(&self) -> Result<u16> {
        debug!("Waiting for connection complete event");

        // Wait for LE Meta Event with Connection Complete subevent
        let event = timeout(self.config.connection_timeout, self.wait_for_le_connection_event())
            .await
            .map_err(|_| BluetoothError::Timeout {
                duration: self.config.connection_timeout,
            })??;

        // Parse connection handle from event
        // LE Connection Complete event format:
        // [0]: Event Code (0x3E - LE Meta Event)
        // [1]: Parameter Length
        // [2]: Subevent Code (0x01 - LE Connection Complete)
        // [3]: Status
        // [4-5]: Connection Handle (little-endian)

        if event.len() < 6 {
            return Err(BluetoothError::Hci("Invalid connection complete event".into()));
        }

        // Check status
        if event[3] != 0x00 {
            return Err(BluetoothError::Hci(format!(
                "Connection failed with status: 0x{:02X}",
                event[3]
            )));
        }

        // Extract connection handle (bytes 4-5, little-endian)
        let handle = u16::from_le_bytes([event[4], event[5]]);

        debug!("Connection complete: handle 0x{:04X}", handle);
        Ok(handle)
    }

    /// Wait for specific LE connection event
    async fn wait_for_le_connection_event(&self) -> Result<Vec<u8>> {
        loop {
            let event = self.controller.receive_event().await?;

            // Check for LE Meta Event (0x3E)
            if event.is_empty() || event[0] != 0x3E {
                continue;
            }

            // Check for LE Connection Complete subevent (0x01)
            if event.len() > 2 && event[2] == 0x01 {
                return Ok(event);
            }

            // Also check for LE Enhanced Connection Complete (0x0A)
            if event.len() > 2 && event[2] == 0x0A {
                return Ok(event);
            }

            debug!("Ignoring non-connection event");
        }
    }

    /// Disconnect from a device
    ///
    /// # Errors
    ///
    /// Returns error if disconnection fails
    pub async fn disconnect(&mut self, address: Address) -> Result<()> {
        info!("Disconnecting from {}", address);

        let device = {
            let connections = self.connections.read().await;
            Arc::clone(connections.get(&address).ok_or_else(|| {
                BluetoothError::device(format!("Device not connected: {address}"))
            })?)
        };

        let handle = device.handle();

        // Send HCI Disconnect command
        self.send_disconnect_command(handle).await?;

        // Wait for disconnection complete
        self.wait_for_disconnection_complete(handle).await?;

        // Remove L2CAP channel
        self.l2cap_manager.remove_channel(handle).await;

        // Remove from connections map
        self.connections.write().await.remove(&address);

        info!("Disconnected from {} (handle: 0x{:04X})", address, handle);
        Ok(())
    }

    /// Send HCI Disconnect command
    async fn send_disconnect_command(&self, handle: u16) -> Result<()> {
        debug!("Sending disconnect command for handle 0x{:04X}", handle);

        // HCI_Disconnect (0x0406)
        let handle_bytes = handle.to_le_bytes();
        let cmd = vec![
            0x01, // Command packet
            0x06,
            0x04, // Opcode: Disconnect
            0x03, // Parameter length
            handle_bytes[0],
            handle_bytes[1], // Connection Handle
            0x13,            // Reason: Remote User Terminated Connection
        ];

        self.controller.send_command(&cmd).await?;

        // Wait for Command Status event
        let _status = timeout(Duration::from_secs(1), self.controller.receive_event())
            .await
            .map_err(|_| BluetoothError::Timeout {
                duration: Duration::from_secs(1),
            })??;

        debug!("Disconnect command sent");
        Ok(())
    }

    /// Wait for Disconnection Complete event
    async fn wait_for_disconnection_complete(&self, handle: u16) -> Result<()> {
        debug!("Waiting for disconnection complete");

        let event = timeout(Duration::from_secs(5), self.wait_for_disconnect_event(handle))
            .await
            .map_err(|_| BluetoothError::Timeout {
                duration: Duration::from_secs(5),
            })??;

        // Check status (byte 2)
        if event.len() > 2 && event[2] != 0x00 {
            warn!("Disconnection completed with status: 0x{:02X}", event[2]);
        }

        debug!("Disconnection complete");
        Ok(())
    }

    /// Wait for disconnect event
    async fn wait_for_disconnect_event(&self, expected_handle: u16) -> Result<Vec<u8>> {
        loop {
            let event = self.controller.receive_event().await?;

            // Check for Disconnection Complete event (0x05)
            if event.is_empty() || event[0] != 0x05 {
                continue;
            }

            // Verify handle matches (bytes 3-4, little-endian)
            if event.len() >= 5 {
                let handle = u16::from_le_bytes([event[3], event[4]]);
                if handle == expected_handle {
                    return Ok(event);
                }
            }

            debug!("Ignoring disconnect event for different handle");
        }
    }

    /// Get GATT client for a connected device
    ///
    /// # Errors
    ///
    /// Returns error if device not connected
    pub async fn gatt_client(&self, address: Address) -> Result<GattClient<T>> {
        let device =
            Arc::clone(self.connections.read().await.get(&address).ok_or_else(|| {
                BluetoothError::device(format!("Device not connected: {address}"))
            })?);

        // Create or get L2CAP ATT channel for this connection
        let l2cap_channel = match self.l2cap_manager.get_att_channel(device.handle()).await {
            Ok(channel) => channel,
            Err(_) => self.l2cap_manager.create_att_channel(device.handle()).await?,
        };

        Ok(GattClient::new(device, l2cap_channel, Arc::clone(&self.transport)))
    }

    /// Get number of active connections
    #[must_use]
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Check if scanning
    #[must_use]
    pub async fn is_scanning(&self) -> bool {
        *self.scanning.lock().await
    }

    /// Get host configuration
    #[must_use]
    pub const fn config(&self) -> &HostConfig {
        &self.config
    }

    /// Shutdown host and close all connections
    ///
    /// # Errors
    ///
    /// Returns error if shutdown fails
    pub async fn shutdown(mut self) -> Result<()> {
        info!("Shutting down Bluetooth host");

        // Disconnect all devices
        let addresses: Vec<Address> = self.connections.read().await.keys().copied().collect();
        for address in addresses {
            if let Err(e) = self.disconnect(address).await {
                warn!("Failed to disconnect {}: {}", address, e);
            }
        }

        // Close transport
        self.transport.lock().await.close().await?;

        info!("Bluetooth host shut down");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::transport::{Transport as TransportTrait, TransportType};

    // Mock transport for testing
    struct MockTransport {
        connected: bool,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                connected: true,
            }
        }
    }

    impl TransportTrait for MockTransport {
        fn transport_type(&self) -> TransportType {
            TransportType::Usb
        }

        async fn send_command(&mut self, _data: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn receive_event(&mut self) -> Result<Vec<u8>> {
            Ok(vec![])
        }

        async fn send_acl(&mut self, _data: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn receive_acl(&mut self) -> Result<Vec<u8>> {
            Ok(vec![])
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        async fn close(&mut self) -> Result<()> {
            self.connected = false;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_host_creation() {
        let transport = MockTransport::new();
        let host = BluetoothHost::new(transport);
        assert!(host.is_ok());
    }

    #[tokio::test]
    async fn test_host_scanning() {
        let transport = MockTransport::new();
        let mut host = match BluetoothHost::new(transport) {
            Ok(h) => h,
            Err(e) => panic!("BluetoothHost::new: {e:?}"),
        };

        let result = host.scan_devices(Duration::from_millis(100)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_host_shutdown() {
        let transport = MockTransport::new();
        let host = match BluetoothHost::new(transport) {
            Ok(h) => h,
            Err(e) => panic!("BluetoothHost::new: {e:?}"),
        };

        let result = host.shutdown().await;
        assert!(result.is_ok());
    }
}
