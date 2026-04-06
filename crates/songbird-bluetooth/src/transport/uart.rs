// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! UART HCI Transport - Pure Rust implementation for embedded
//!
//! This module provides UART transport for Bluetooth HCI using `serialport`.
//! Perfect for embedded deployments and devices with UART Bluetooth modules.
//!
//! ## Hardware Support
//!
//! - ESP32 Bluetooth modules
//! - Nordic nRF52 modules
//! - Any UART Bluetooth controller
//! - Embedded Linux (Raspberry Pi, etc.)
//!
//! ## Example
//!
//! ```rust,no_run
//! use songbird_bluetooth::UartTransport;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let transport = UartTransport::new("/dev/ttyUSB0", 115200).await?;
//! // Transport ready for HCI communication
//! # Ok(())
//! # }
//! ```

use crate::error::{Result, TransportError};
use crate::transport::{Transport, TransportType};
use serialport::{SerialPort, SerialPortType};
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Default UART baud rate for HCI
const DEFAULT_BAUD_RATE: u32 = 115_200;

/// Default timeout for UART operations
const UART_TIMEOUT: Duration = Duration::from_secs(1);

/// HCI packet types
const HCI_COMMAND_PACKET: u8 = 0x01;
const HCI_ACL_DATA_PACKET: u8 = 0x02;
const HCI_EVENT_PACKET: u8 = 0x04;

/// UART HCI Transport
///
/// Provides UART access to Bluetooth controllers for embedded deployments.
/// Works with any UART Bluetooth module.
pub struct UartTransport {
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    port_name: String,
    baud_rate: u32,
    connected: bool,
}

impl UartTransport {
    /// Create new UART transport
    ///
    /// # Arguments
    ///
    /// * `port_name` - Serial port path (e.g., "/dev/ttyUSB0", "COM3")
    /// * `baud_rate` - Baud rate (typically 115200 for HCI)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Port cannot be opened
    /// - Invalid baud rate
    /// - Port configuration fails
    pub async fn new(port_name: impl Into<String>, baud_rate: u32) -> Result<Self> {
        tokio::task::yield_now().await; // Yield for async context
        let port_name = port_name.into();

        info!("Opening UART port: {} at {} baud", port_name, baud_rate);

        let port = serialport::new(&port_name, baud_rate)
            .timeout(UART_TIMEOUT)
            .open()
            .map_err(|e| TransportError::Uart(format!("Failed to open port {port_name}: {e}")))?;

        info!("UART transport initialized: {}", port_name);

        Ok(Self {
            port: Arc::new(Mutex::new(port)),
            port_name,
            baud_rate,
            connected: true,
        })
    }

    /// Create UART transport with default baud rate (115200)
    ///
    /// # Errors
    ///
    /// Returns error if port cannot be opened
    pub async fn with_default_baud(port_name: impl Into<String>) -> Result<Self> {
        Self::new(port_name, DEFAULT_BAUD_RATE).await
    }

    /// List available serial ports
    ///
    /// # Errors
    ///
    /// Returns error if port enumeration fails
    pub fn list_ports() -> Result<Vec<String>> {
        let ports = serialport::available_ports()
            .map_err(|e| TransportError::Uart(format!("Failed to enumerate ports: {e}")))?;

        let port_names: Vec<String> = ports
            .into_iter()
            .filter_map(|p| {
                // Filter for USB serial devices (likely Bluetooth modules)
                match p.port_type {
                    SerialPortType::UsbPort(_) => Some(p.port_name),
                    _ => None,
                }
            })
            .collect();

        Ok(port_names)
    }

    /// Write HCI packet with type indicator
    async fn write_packet(&self, packet_type: u8, data: &[u8]) -> Result<()> {
        let mut port = self.port.lock().await;

        // Write packet type indicator
        port.write_all(&[packet_type])
            .map_err(|e| TransportError::Uart(format!("Failed to write packet type: {e}")))?;

        // Write packet data
        port.write_all(data)
            .map_err(|e| TransportError::Uart(format!("Failed to write packet data: {e}")))?;

        port.flush().map_err(|e| TransportError::Uart(format!("Failed to flush: {e}")))?;
        drop(port);

        debug!("Wrote UART packet: type=0x{:02x}, len={}", packet_type, data.len());
        Ok(())
    }

    /// Read HCI packet with type indicator
    async fn read_packet(&self, expected_type: u8) -> Result<Vec<u8>> {
        let mut port = self.port.lock().await;

        // Read packet type
        let mut packet_type = [0u8; 1];
        port.read_exact(&mut packet_type)
            .map_err(|e| TransportError::Uart(format!("Failed to read packet type: {e}")))?;

        if packet_type[0] != expected_type {
            return Err(TransportError::Communication(format!(
                "Unexpected packet type: expected 0x{:02x}, got 0x{:02x}",
                expected_type, packet_type[0]
            ))
            .into());
        }

        // Read packet length (HCI event: 2 bytes header + length byte)
        let mut header = [0u8; 2];
        port.read_exact(&mut header)
            .map_err(|e| TransportError::Uart(format!("Failed to read header: {e}")))?;

        let length = header[1] as usize;

        // Read packet data
        let mut data = vec![0u8; length];
        port.read_exact(&mut data)
            .map_err(|e| TransportError::Uart(format!("Failed to read data: {e}")))?;
        drop(port);

        // Reconstruct full packet (header + data)
        let mut packet = Vec::with_capacity(2 + length);
        packet.extend_from_slice(&header);
        packet.extend_from_slice(&data);

        debug!("Read UART packet: type=0x{:02x}, len={}", packet_type[0], packet.len());
        Ok(packet)
    }

    /// Get port name
    #[must_use]
    pub fn port_name(&self) -> &str {
        &self.port_name
    }

    /// Get baud rate
    #[must_use]
    pub const fn baud_rate(&self) -> u32 {
        self.baud_rate
    }
}

impl Transport for UartTransport {
    fn transport_type(&self) -> TransportType {
        TransportType::Uart
    }

    async fn send_command(&mut self, data: &[u8]) -> Result<()> {
        if !self.connected {
            return Err(TransportError::Communication("Transport not connected".into()).into());
        }

        self.write_packet(HCI_COMMAND_PACKET, data).await?;
        debug!("Sent HCI command: {} bytes", data.len());
        Ok(())
    }

    async fn receive_event(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(TransportError::Communication("Transport not connected".into()).into());
        }

        let packet = self.read_packet(HCI_EVENT_PACKET).await?;
        debug!("Received HCI event: {} bytes", packet.len());
        Ok(packet)
    }

    async fn send_acl(&mut self, data: &[u8]) -> Result<()> {
        if !self.connected {
            return Err(TransportError::Communication("Transport not connected".into()).into());
        }

        self.write_packet(HCI_ACL_DATA_PACKET, data).await?;
        debug!("Sent ACL data: {} bytes", data.len());
        Ok(())
    }

    async fn receive_acl(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(TransportError::Communication("Transport not connected".into()).into());
        }

        let packet = self.read_packet(HCI_ACL_DATA_PACKET).await?;
        debug!("Received ACL data: {} bytes", packet.len());
        Ok(packet)
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn close(&mut self) -> Result<()> {
        if self.connected {
            self.connected = false;
            info!("UART transport closed: {}", self.port_name);
        }
        Ok(())
    }
}

impl Drop for UartTransport {
    fn drop(&mut self) {
        self.connected = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_ports() {
        let result = UartTransport::list_ports();
        // May be empty if no serial ports, but shouldn't panic
        match result {
            Ok(ports) => {
                println!("Found {} serial ports", ports.len());
                for port in ports {
                    println!("  - {port}");
                }
            }
            Err(e) => {
                println!("Failed to list ports: {e}");
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires UART Bluetooth module"]
    async fn test_uart_transport_creation() {
        // This test requires actual hardware
        // Run with: cargo test --features uart -- --ignored
        let ports = UartTransport::list_ports().unwrap_or_default();
        if let Some(port) = ports.first() {
            let result = UartTransport::new(port, 115_200).await;
            if let Ok(transport) = result {
                assert!(transport.is_connected());
                assert_eq!(transport.transport_type(), TransportType::Uart);
                assert_eq!(transport.baud_rate(), 115_200);
            }
        }
    }
}
