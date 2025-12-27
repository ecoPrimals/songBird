//! Integration tests for Bluetooth stack
//!
//! These tests verify the full flow of operations using mock transports.

use songbird_bluetooth::{
    device::{Address, DeviceInfo},
    error::{BluetoothError, Result},
    transport::{Transport, TransportType},
    BluetoothHost,
};
use std::sync::{Arc, Mutex};

/// Mock transport for testing
struct MockTransport {
    connected: bool,
    commands_sent: Arc<Mutex<Vec<Vec<u8>>>>,
    event_responses: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl MockTransport {
    fn new() -> Self {
        Self {
            connected: true,
            commands_sent: Arc::new(Mutex::new(Vec::new())),
            event_responses: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn with_responses(responses: Vec<Vec<u8>>) -> Self {
        Self {
            connected: true,
            commands_sent: Arc::new(Mutex::new(Vec::new())),
            event_responses: Arc::new(Mutex::new(responses)),
        }
    }

    fn commands_sent(&self) -> Vec<Vec<u8>> {
        self.commands_sent.lock().unwrap().clone()
    }

    fn add_response(&self, response: Vec<u8>) {
        self.event_responses.lock().unwrap().push(response);
    }
}

#[async_trait::async_trait]
impl Transport for MockTransport {
    fn transport_type(&self) -> TransportType {
        TransportType::Usb
    }

    async fn send_command(&mut self, data: &[u8]) -> Result<()> {
        self.commands_sent.lock().unwrap().push(data.to_vec());
        Ok(())
    }

    async fn receive_event(&mut self) -> Result<Vec<u8>> {
        let mut responses = self.event_responses.lock().unwrap();
        if responses.is_empty() {
            return Err(BluetoothError::Timeout {
                duration: std::time::Duration::from_secs(1),
            });
        }
        Ok(responses.remove(0))
    }

    async fn send_acl(&mut self, _data: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn receive_acl(&mut self) -> Result<Vec<u8>> {
        Ok(Vec::new())
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
    let result = BluetoothHost::new(transport);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_host_creation_with_disconnected_transport() {
    let mut transport = MockTransport::new();
    transport.connected = false;
    let result = BluetoothHost::new(transport);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_scan_empty() {
    let transport = MockTransport::new();
    let mut host = BluetoothHost::new(transport).unwrap();

    // Scanning will timeout with no responses
    let result = host.scan_devices(std::time::Duration::from_millis(100)).await;

    // Should not error, just return empty results when timeout occurs
    assert!(result.is_ok() || matches!(result, Err(BluetoothError::Timeout { .. })));
}

#[tokio::test]
async fn test_connection_count() {
    let transport = MockTransport::new();
    let host = BluetoothHost::new(transport).unwrap();

    let count = host.connection_count().await;
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_is_scanning_initially_false() {
    let transport = MockTransport::new();
    let host = BluetoothHost::new(transport).unwrap();

    assert!(!host.is_scanning().await);
}

#[tokio::test]
async fn test_device_info_builder() {
    let address = Address::from_bytes([1, 2, 3, 4, 5, 6]);
    let info = DeviceInfo::new(address).with_name("Test Device".to_string()).with_rssi(-45);

    assert_eq!(info.address, address);
    assert_eq!(info.name, Some("Test Device".to_string()));
    assert_eq!(info.rssi, -45);
}

#[tokio::test]
async fn test_address_parsing() {
    let addr1 = Address::from_bytes([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    let addr2 = Address::from_bytes([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);

    assert_eq!(addr1, addr2);
}

#[tokio::test]
async fn test_address_display() {
    let addr = Address::from_bytes([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    let display = format!("{}", addr);

    assert!(display.contains("AA"));
    assert!(display.contains("FF"));
}

#[tokio::test]
async fn test_disconnect_not_connected() {
    let transport = MockTransport::new();
    let mut host = BluetoothHost::new(transport).unwrap();

    let address = Address::from_bytes([1, 2, 3, 4, 5, 6]);
    let result = host.disconnect(address).await;

    assert!(result.is_err());
    assert!(matches!(result, Err(BluetoothError::Device(_))));
}

#[tokio::test]
async fn test_gatt_client_for_non_connected_device() {
    let transport = MockTransport::new();
    let host = BluetoothHost::new(transport).unwrap();

    let address = Address::from_bytes([1, 2, 3, 4, 5, 6]);
    let result = host.gatt_client(address).await;

    assert!(result.is_err());
    assert!(matches!(result, Err(BluetoothError::Device(_))));
}

#[tokio::test]
async fn test_host_config_defaults() {
    use songbird_bluetooth::host::HostConfig;

    let config = HostConfig::default();
    assert_eq!(config.device_name, "Songbird");
    assert_eq!(config.scan_window_ms, 100);
    assert_eq!(config.scan_interval_ms, 100);
    assert_eq!(config.max_connections, 4);
}

#[tokio::test]
async fn test_scan_timeout() {
    let transport = MockTransport::new();
    let mut host = BluetoothHost::new(transport).unwrap();

    // Scan will timeout with no responses - this is expected behavior
    let result = host.scan_devices(std::time::Duration::from_millis(100)).await;

    // Should either succeed with empty list or timeout
    assert!(result.is_ok() || matches!(result, Err(BluetoothError::Timeout { .. })));
}

#[tokio::test]
async fn test_characteristic_properties() {
    use songbird_bluetooth::gatt::CharacteristicProperties;

    let props =
        CharacteristicProperties::new().with_read().with_write_without_response().with_notify();

    assert!(props.read());
    assert!(!props.write());
    assert!(props.write_without_response());
    assert!(props.notify());
    assert!(!props.indicate());
}

#[tokio::test]
async fn test_transport_type() {
    let transport = MockTransport::new();
    assert_eq!(transport.transport_type(), TransportType::Usb);
}

#[tokio::test]
async fn test_mock_transport_close() {
    let mut transport = MockTransport::new();
    assert!(transport.is_connected());

    transport.close().await.unwrap();
    assert!(!transport.is_connected());
}

#[tokio::test]
async fn test_mock_transport_send_receive() {
    let mut transport = MockTransport::with_responses(vec![
        vec![0x04, 0x0E, 0x04, 0x01, 0x03, 0x0C, 0x00], // Command Complete
    ]);

    // Send command
    transport.send_command(&[0x01, 0x03, 0x0C, 0x00]).await.unwrap();

    // Verify command was recorded
    let commands = transport.commands_sent();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], vec![0x01, 0x03, 0x0C, 0x00]);

    // Receive event
    let event = transport.receive_event().await.unwrap();
    assert_eq!(event, vec![0x04, 0x0E, 0x04, 0x01, 0x03, 0x0C, 0x00]);
}

#[tokio::test]
async fn test_mock_transport_empty_responses() {
    let mut transport = MockTransport::new();

    // Should timeout when no responses
    let result = transport.receive_event().await;
    assert!(result.is_err());
    assert!(matches!(result, Err(BluetoothError::Timeout { .. })));
}

#[tokio::test]
async fn test_multiple_hosts_with_mock_transport() {
    let transport1 = MockTransport::new();
    let transport2 = MockTransport::new();

    let host1 = BluetoothHost::new(transport1);
    let host2 = BluetoothHost::new(transport2);

    assert!(host1.is_ok());
    assert!(host2.is_ok());
}

#[tokio::test]
async fn test_host_shutdown() {
    let transport = MockTransport::new();
    let host = BluetoothHost::new(transport).unwrap();

    let result = host.shutdown().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_error_display() {
    let err = BluetoothError::Timeout {
        duration: std::time::Duration::from_secs(5),
    };
    let display = format!("{}", err);
    assert!(display.contains("5"));

    let err2 = BluetoothError::Device("test error".to_string());
    let display2 = format!("{}", err2);
    assert!(display2.contains("test error"));
}
