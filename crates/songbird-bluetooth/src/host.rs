//! BLE Host - Main entry point for Bluetooth operations
//!
//! Pure Rust BLE host built on trouble-host (Embassy project).
//! Provides high-level API for scanning, connecting, and GATT operations.

use crate::{
    controller::ControllerAdapter,
    device::{Address, Device, DeviceInfo},
    error::{BluetoothError, Result},
    gatt::GattClient,
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
            return Err(BluetoothError::InvalidOperation(
                "Transport not connected".into(),
            ));
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
        })
    }

    /// Scan for BLE devices
    ///
    /// # Errors
    ///
    /// Returns error if scan fails or timeout occurs
    pub async fn scan_devices(&mut self, duration: Duration) -> Result<Vec<DeviceInfo>> {
        let mut scanning = self.scanning.lock().await;
        if *scanning {
            return Err(BluetoothError::InvalidOperation("Scan already in progress".into()));
        }

        *scanning = true;
        info!("Starting BLE scan for {:?}", duration);

        // Phase 2: Actual BLE scanning with HCI commands
        let result = self.perform_scan(duration).await;

        *scanning = false;
        
        match result {
            Ok(devices) => {
                debug!("Scan complete, found {} devices", devices.len());
                Ok(devices)
            }
            Err(e) => {
                warn!("Scan failed: {}", e);
                Err(e)
            }
        }
    }

    /// Perform actual BLE scan using HCI commands
    async fn perform_scan(&self, duration: Duration) -> Result<Vec<DeviceInfo>> {
        // Step 1: Set scan parameters
        self.set_scan_parameters().await?;

        // Step 2: Enable scanning
        self.enable_scan(true).await?;

        // Step 3: Collect advertisements for the specified duration
        let devices = timeout(duration, self.collect_advertisements())
            .await
            .unwrap_or_else(|_| {
                debug!("Scan timeout reached");
                Ok(Vec::new())
            })?;

        // Step 4: Disable scanning
        self.enable_scan(false).await?;

        Ok(devices)
    }

    /// Set BLE scan parameters
    async fn set_scan_parameters(&self) -> Result<()> {
        debug!("Setting scan parameters");
        
        // HCI_LE_Set_Scan_Parameters command
        // Opcode: 0x200B
        // Parameters:
        // - Scan Type: 0x01 (Active)
        // - Scan Interval: 0x0010 (10ms)
        // - Scan Window: 0x0010 (10ms)
        // - Own Address Type: 0x00 (Public)
        // - Scanning Filter Policy: 0x00 (Accept all)
        
        let cmd = vec![
            0x01, // Command packet
            0x0B, 0x20, // Opcode: LE Set Scan Parameters
            0x07, // Parameter length
            0x01, // Scan Type: Active
            0x10, 0x00, // Scan Interval
            0x10, 0x00, // Scan Window
            0x00, // Own Address Type
            0x00, // Scanning Filter Policy
        ];

        self.controller.send_command(&cmd).await?;
        
        // Wait for command complete
        let _response = timeout(
            Duration::from_secs(1),
            self.controller.receive_event()
        ).await.map_err(|_| BluetoothError::Timeout {
            duration: Duration::from_secs(1)
        })??;

        debug!("Scan parameters set");
        Ok(())
    }

    /// Enable or disable BLE scanning
    async fn enable_scan(&self, enable: bool) -> Result<()> {
        debug!("{}abling scan", if enable { "En" } else { "Dis" });
        
        // HCI_LE_Set_Scan_Enable command
        // Opcode: 0x200C
        // Parameters:
        // - LE Scan Enable: 0x01 (enabled) or 0x00 (disabled)
        // - Filter Duplicates: 0x01 (enabled)
        
        let cmd = vec![
            0x01, // Command packet
            0x0C, 0x20, // Opcode: LE Set Scan Enable
            0x02, // Parameter length
            if enable { 0x01 } else { 0x00 }, // Scan enable
            0x01, // Filter duplicates
        ];

        self.controller.send_command(&cmd).await?;
        
        // Wait for command complete
        let _response = timeout(
            Duration::from_secs(1),
            self.controller.receive_event()
        ).await.map_err(|_| BluetoothError::Timeout {
            duration: Duration::from_secs(1)
        })??;

        debug!("Scan {}abled", if enable { "en" } else { "dis" });
        Ok(())
    }

    /// Collect BLE advertisements
    async fn collect_advertisements(&self) -> Result<Vec<DeviceInfo>> {
        let mut devices = HashMap::new();
        
        // Collect advertisements until timeout or max devices
        loop {
            // Try to receive advertisement with short timeout
            let event_result = timeout(
                Duration::from_millis(100),
                self.controller.receive_event()
            ).await;

            match event_result {
                Ok(Ok(event)) => {
                    if let Some(device) = self.parse_advertisement(&event) {
                        devices.entry(device.address)
                            .or_insert(device);
                    }
                }
                Ok(Err(e)) => {
                    warn!("Error receiving event: {}", e);
                    break;
                }
                Err(_) => {
                    // Timeout - no more advertisements
                    continue;
                }
            }
            
            // Limit to reasonable number of devices
            if devices.len() >= 100 {
                debug!("Reached max device limit");
                break;
            }
        }

        Ok(devices.into_values().collect())
    }

    /// Parse BLE advertisement event
    fn parse_advertisement(&self, event: &[u8]) -> Option<DeviceInfo> {
        // Check for LE Advertising Report event
        if event.len() < 12 || event[0] != 0x3E {
            return None;
        }

        // Subevent Code: 0x02 (LE Advertising Report)
        if event.get(2)? != &0x02 {
            return None;
        }

        // Parse address (6 bytes, reversed)
        let addr_start = 5;
        if event.len() < addr_start + 6 {
            return None;
        }

        let addr_bytes: [u8; 6] = event[addr_start..addr_start + 6]
            .try_into()
            .ok()?;
        
        let address = Address::from_bytes(addr_bytes);

        // Parse RSSI (last byte)
        let rssi = event.last().copied().map(|b| b as i8);

        // Parse device name from advertisement data (if present)
        let name = self.parse_device_name(event);

        let mut info = DeviceInfo::new(address);
        if let Some(n) = name {
            info = info.with_name(n);
        }
        if let Some(r) = rssi {
            info = info.with_rssi(r);
        }

        debug!("Found device: {}", address);
        Some(info)
    }

    /// Parse device name from advertisement data
    fn parse_device_name(&self, _event: &[u8]) -> Option<String> {
        // TODO: Implement AD type parsing
        // For now, return None
        None
    }

    /// Connect to a device
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Connection fails
    /// - Device not found
    /// - Timeout occurs
    pub async fn connect(&mut self, address: Address) -> Result<Arc<Device>> {
        info!("Connecting to device: {}", address);

        let connections = self.connections.read().await;
        if let Some(device) = connections.get(&address) {
            debug!("Device already connected");
            return Ok(Arc::clone(device));
        }
        drop(connections);

        // TODO: Implement actual connection using trouble-host
        // For now, create placeholder device

        let info = DeviceInfo::new(address);
        let device = Arc::new(Device::new(info, 0));

        let mut connections = self.connections.write().await;
        connections.insert(address, Arc::clone(&device));

        info!("Connected to {}", address);
        Ok(device)
    }

    /// Disconnect from a device
    ///
    /// # Errors
    ///
    /// Returns error if disconnection fails
    pub async fn disconnect(&mut self, address: Address) -> Result<()> {
        info!("Disconnecting from {}", address);

        let mut connections = self.connections.write().await;
        if connections.remove(&address).is_some() {
            // TODO: Implement actual disconnection
            info!("Disconnected from {}", address);
            Ok(())
        } else {
            Err(BluetoothError::device(format!("Device not connected: {address}")))
        }
    }

    /// Get GATT client for a connected device
    ///
    /// # Errors
    ///
    /// Returns error if device not connected
    pub async fn gatt_client(&self, address: Address) -> Result<GattClient> {
        let connections = self.connections.read().await;
        let device = connections
            .get(&address)
            .ok_or_else(|| BluetoothError::device(format!("Device not connected: {address}")))?;

        Ok(GattClient::new(Arc::clone(device)))
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
    use super::*;
    use crate::transport::{TransportType, Transport as TransportTrait};

    // Mock transport for testing
    struct MockTransport {
        connected: bool,
    }

    impl MockTransport {
        fn new() -> Self {
            Self { connected: true }
        }
    }

    #[async_trait::async_trait]
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
        let mut host = BluetoothHost::new(transport).unwrap();
        
        let result = host.scan_devices(Duration::from_millis(100)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_host_shutdown() {
        let transport = MockTransport::new();
        let host = BluetoothHost::new(transport).unwrap();
        
        let result = host.shutdown().await;
        assert!(result.is_ok());
    }
}

