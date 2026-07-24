// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! USB HCI Transport - C-based implementation using `rusb`
//!
//! This module provides USB transport for Bluetooth HCI using `rusb` (C bindings to libusb).
//! Works with any USB Bluetooth dongle without OS Bluetooth stack.
//!
//! ## Note
//!
//! This is the **legacy C-based** implementation. Consider using the pure Rust
//! implementation (`usb_nusb`) for better portability and ecoBin compliance.
//!
//! Enable with `--features usb-c` if you need maximum compatibility.
//!
//! ## Hardware Support
//!
//! - CSR chipsets (most common, ~$5-10)
//! - Realtek chipsets
//! - Broadcom chipsets
//! - Intel chipsets
//! - Any USB Bluetooth Class device (0xE0)
//!
//! ## Example
//!
//! ```rust,no_run
//! use songbird_bluetooth::UsbTransport;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let transport = UsbTransport::new().await?;
//! // Transport ready for HCI communication
//! # Ok(())
//! # }
//! ```

use crate::error::{Result, TransportError};
use crate::transport::{Transport, TransportType};
use rusb::{Context, Device, DeviceHandle, Direction, TransferType, UsbContext};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// USB Bluetooth Class code
const USB_CLASS_WIRELESS_CONTROLLER: u8 = 0xE0;

/// USB Bluetooth Subclass
const USB_SUBCLASS_RF_CONTROLLER: u8 = 0x01;

/// USB Bluetooth Protocol (Bluetooth Programming)
const USB_PROTOCOL_BLUETOOTH: u8 = 0x01;

/// HCI command endpoint (Control)
/// Note: Awaiting hardware validation - will be used in Phase 3
#[expect(dead_code, reason = "reserved for Phase 3 HCI command path")]
const HCI_COMMAND_ENDPOINT: u8 = 0x00;

/// HCI event endpoint (Interrupt IN)
const HCI_EVENT_ENDPOINT: u8 = 0x81;

/// HCI ACL data IN endpoint (Bulk IN)
const HCI_ACL_IN_ENDPOINT: u8 = 0x82;

/// HCI ACL data OUT endpoint (Bulk OUT)
const HCI_ACL_OUT_ENDPOINT: u8 = 0x02;

/// Default timeout for USB operations
const USB_TIMEOUT: Duration = Duration::from_secs(1);

/// USB HCI Transport
///
/// Provides direct USB access to Bluetooth controllers without
/// requiring OS Bluetooth stack. Works with any USB Bluetooth dongle.
pub struct UsbTransport {
    handle: Arc<Mutex<DeviceHandle<Context>>>,
    interface: u8,
    event_endpoint: u8,
    acl_in_endpoint: u8,
    acl_out_endpoint: u8,
    connected: bool,
}

impl UsbTransport {
    /// Create new USB transport
    ///
    /// Automatically finds and opens first available USB Bluetooth device.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No USB Bluetooth device found
    /// - Device cannot be opened
    /// - Interface cannot be claimed
    pub async fn new() -> Result<Self> {
        Self::with_filter(None, None).await
    }

    /// Create USB transport with vendor/product ID filter
    ///
    /// # Errors
    ///
    /// Returns error if no matching device found
    pub async fn with_filter(vendor_id: Option<u16>, product_id: Option<u16>) -> Result<Self> {
        tokio::task::yield_now().await; // Yield before blocking
        // Run USB operations in block_in_place - rusb types are not Send
        tokio::task::block_in_place(|| {
            let context = Context::new()
                .map_err(|e| TransportError::Usb(format!("Failed to create USB context: {e}")))?;

            let devices = context
                .devices()
                .map_err(|e| TransportError::Usb(format!("Failed to enumerate devices: {e}")))?;

            for device in devices.iter() {
                if let Ok(desc) = device.device_descriptor() {
                    if let Some(vid) = vendor_id
                        && desc.vendor_id() != vid
                    {
                        continue;
                    }
                    if let Some(pid) = product_id
                        && desc.product_id() != pid
                    {
                        continue;
                    }

                    if desc.class_code() == USB_CLASS_WIRELESS_CONTROLLER
                        || Self::is_bluetooth_interface(&device).unwrap_or(false)
                    {
                        info!(
                            "Found Bluetooth USB device: {:04x}:{:04x}",
                            desc.vendor_id(),
                            desc.product_id()
                        );

                        return Self::open_device(&device);
                    }
                }
            }

            Err(TransportError::NoAdapter.into())
        })
    }

    /// Check if device has Bluetooth interface
    fn is_bluetooth_interface<T: UsbContext>(device: &Device<T>) -> rusb::Result<bool> {
        let config_desc = device.active_config_descriptor()?;

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                if interface_desc.class_code() == USB_CLASS_WIRELESS_CONTROLLER
                    && interface_desc.sub_class_code() == USB_SUBCLASS_RF_CONTROLLER
                    && interface_desc.protocol_code() == USB_PROTOCOL_BLUETOOTH
                {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Open and configure Bluetooth device
    fn open_device(device: &Device<Context>) -> Result<Self> {
        let handle = device
            .open()
            .map_err(|e| TransportError::Usb(format!("Failed to open device: {e}")))?;

        // Find Bluetooth interface
        let config_desc = device
            .active_config_descriptor()
            .map_err(|e| TransportError::Usb(format!("Failed to get config descriptor: {e}")))?;

        let mut bt_interface = None;
        let mut event_endpoint = HCI_EVENT_ENDPOINT;
        let mut acl_in_endpoint = HCI_ACL_IN_ENDPOINT;
        let mut acl_out_endpoint = HCI_ACL_OUT_ENDPOINT;

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                if interface_desc.class_code() == USB_CLASS_WIRELESS_CONTROLLER {
                    bt_interface = Some(interface.number());

                    // Find endpoints
                    for endpoint in interface_desc.endpoint_descriptors() {
                        match (endpoint.transfer_type(), endpoint.direction()) {
                            (TransferType::Interrupt, Direction::In) => {
                                event_endpoint = endpoint.address();
                                debug!("HCI Event endpoint: 0x{:02x}", event_endpoint);
                            }
                            (TransferType::Bulk, Direction::In) => {
                                acl_in_endpoint = endpoint.address();
                                debug!("HCI ACL IN endpoint: 0x{:02x}", acl_in_endpoint);
                            }
                            (TransferType::Bulk, Direction::Out) => {
                                acl_out_endpoint = endpoint.address();
                                debug!("HCI ACL OUT endpoint: 0x{:02x}", acl_out_endpoint);
                            }
                            _ => {}
                        }
                    }
                    break;
                }
            }
            if bt_interface.is_some() {
                break;
            }
        }

        let interface_num = bt_interface
            .ok_or_else(|| TransportError::Usb("No Bluetooth interface found".into()))?;

        // Claim interface
        handle
            .claim_interface(interface_num)
            .map_err(|e| TransportError::Usb(format!("Failed to claim interface: {e}")))?;

        info!("USB Bluetooth transport initialized on interface {}", interface_num);

        Ok(Self {
            handle: Arc::new(Mutex::new(handle)),
            interface: interface_num,
            event_endpoint,
            acl_in_endpoint,
            acl_out_endpoint,
            connected: true,
        })
    }
}

impl Transport for UsbTransport {
    fn transport_type(&self) -> TransportType {
        TransportType::Usb
    }

    async fn send_command(&mut self, data: &[u8]) -> Result<()> {
        if !self.connected {
            return Err(TransportError::Communication("Transport not connected".into()).into());
        }

        // HCI commands sent via control transfer
        self.handle
            .lock()
            .await
            .write_control(
                0x20, // bmRequestType: Class request, host to device
                0x00, // bRequest: HCI command
                0,    // wValue
                0,    // wIndex
                data,
                USB_TIMEOUT,
            )
            .map_err(|e| TransportError::Usb(format!("Failed to send command: {e}")))?;

        debug!("Sent HCI command: {} bytes", data.len());
        Ok(())
    }

    async fn receive_event(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(TransportError::Communication("Transport not connected".into()).into());
        }

        let mut buf = vec![0u8; 256];

        let len = self
            .handle
            .lock()
            .await
            .read_interrupt(self.event_endpoint, &mut buf, USB_TIMEOUT)
            .map_err(|e| TransportError::Usb(format!("Failed to receive event: {e}")))?;

        buf.truncate(len);
        debug!("Received HCI event: {} bytes", len);
        Ok(buf)
    }

    async fn send_acl(&mut self, data: &[u8]) -> Result<()> {
        if !self.connected {
            return Err(TransportError::Communication("Transport not connected".into()).into());
        }

        self.handle
            .lock()
            .await
            .write_bulk(self.acl_out_endpoint, data, USB_TIMEOUT)
            .map_err(|e| TransportError::Usb(format!("Failed to send ACL data: {e}")))?;

        debug!("Sent ACL data: {} bytes", data.len());
        Ok(())
    }

    async fn receive_acl(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(TransportError::Communication("Transport not connected".into()).into());
        }

        let mut buf = vec![0u8; 1024];

        let len = self
            .handle
            .lock()
            .await
            .read_bulk(self.acl_in_endpoint, &mut buf, USB_TIMEOUT)
            .map_err(|e| TransportError::Usb(format!("Failed to receive ACL data: {e}")))?;

        buf.truncate(len);
        debug!("Received ACL data: {} bytes", len);
        Ok(buf)
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn close(&mut self) -> Result<()> {
        if self.connected {
            let result = self.handle.lock().await.release_interface(self.interface);
            if let Err(e) = result {
                warn!("Failed to release interface: {}", e);
            }
            self.connected = false;
            info!("USB transport closed");
        }
        Ok(())
    }
}

impl Drop for UsbTransport {
    fn drop(&mut self) {
        // Best effort cleanup
        // Can't be async in Drop, but connection will close when handle drops
        self.connected = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires USB Bluetooth dongle"]
    async fn test_usb_transport_creation() {
        let result = UsbTransport::new().await;
        // May fail if no dongle present, but shouldn't panic
        if let Ok(transport) = result {
            assert!(transport.is_connected());
            assert_eq!(transport.transport_type(), TransportType::Usb);
        }
    }
}
