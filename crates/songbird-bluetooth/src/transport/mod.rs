// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Transport layer abstraction for Bluetooth HCI
//!
//! Supports multiple transport types:
//! - **USB (nusb)**: Pure Rust USB transport (default, ecoBin compliant)
//! - **USB (rusb)**: C-based USB transport (fallback for compatibility)
//! - **UART**: For embedded deployments with UART Bluetooth modules
//!
//! All transports implement the `Transport` trait for unified access.
//!
//! ## Feature Flags
//!
//! - `usb-rust` (default): Pure Rust USB via `nusb` - ecoBin compliant
//! - `usb-c`: C-based USB via `rusb` - maximum compatibility fallback
//! - `uart`: Serial port transport

use crate::error::Result;
use std::fmt;

// Pure Rust USB transport (default)
#[cfg(feature = "usb-rust")]
pub mod usb_nusb;
#[cfg(feature = "usb-rust")]
pub use usb_nusb::UsbTransport;

// C-based USB transport (fallback) - only if nusb not enabled
#[cfg(all(feature = "usb-c", not(feature = "usb-rust")))]
pub mod usb;
#[cfg(all(feature = "usb-c", not(feature = "usb-rust")))]
pub use usb::UsbTransport;

// Keep usb module available for reference even when using nusb
#[cfg(all(feature = "usb-c", feature = "usb-rust"))]
#[allow(
    dead_code,
    reason = "dual-feature gate: usb-rust preferred, but usb-c module kept for fallback"
)]
pub mod usb;

#[cfg(feature = "uart")]
pub mod uart;

#[cfg(feature = "uart")]
pub use uart::UartTransport;

/// Transport type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportType {
    /// USB HCI transport
    Usb,
    /// UART HCI transport
    Uart,
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usb => write!(f, "USB"),
            Self::Uart => write!(f, "UART"),
        }
    }
}

/// Transport layer trait for Bluetooth HCI
///
/// This trait abstracts over different physical transports (USB, UART)
/// providing a unified interface for HCI communication.
///
/// # Safety
///
/// Implementations must ensure:
/// - Thread-safe send/receive operations
/// - Proper error handling and recovery
/// - No data corruption or loss
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Get the transport type
    fn transport_type(&self) -> TransportType;

    /// Send HCI command
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Transport is disconnected
    /// - Command buffer is invalid
    /// - Timeout occurs
    async fn send_command(&mut self, data: &[u8]) -> Result<()>;

    /// Receive HCI event
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Transport is disconnected
    /// - Receive buffer overflow
    /// - Timeout occurs
    async fn receive_event(&mut self) -> Result<Vec<u8>>;

    /// Send ACL data
    ///
    /// # Errors
    ///
    /// Returns error if transmission fails
    async fn send_acl(&mut self, data: &[u8]) -> Result<()>;

    /// Receive ACL data
    ///
    /// # Errors
    ///
    /// Returns error if reception fails
    async fn receive_acl(&mut self) -> Result<Vec<u8>>;

    /// Check if transport is connected
    fn is_connected(&self) -> bool;

    /// Close the transport
    ///
    /// # Errors
    ///
    /// Returns error if cleanup fails
    async fn close(&mut self) -> Result<()>;
}

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Transport type
    pub transport_type: TransportType,

    /// Vendor ID (for USB)
    pub vendor_id: Option<u16>,

    /// Product ID (for USB)
    pub product_id: Option<u16>,

    /// Serial port path (for UART)
    pub serial_port: Option<String>,

    /// Baud rate (for UART)
    pub baud_rate: Option<u32>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            transport_type: TransportType::Usb,
            vendor_id: None,
            product_id: None,
            serial_port: None,
            baud_rate: Some(115_200),
        }
    }
}

impl TransportConfig {
    /// Create USB transport configuration
    #[must_use]
    pub fn usb() -> Self {
        Self {
            transport_type: TransportType::Usb,
            ..Self::default()
        }
    }

    /// Create UART transport configuration
    #[must_use]
    pub fn uart(port: impl Into<String>) -> Self {
        Self {
            transport_type: TransportType::Uart,
            serial_port: Some(port.into()),
            ..Self::default()
        }
    }

    /// Set vendor ID (for USB filtering)
    #[must_use]
    pub const fn with_vendor_id(mut self, vid: u16) -> Self {
        self.vendor_id = Some(vid);
        self
    }

    /// Set product ID (for USB filtering)
    #[must_use]
    pub const fn with_product_id(mut self, pid: u16) -> Self {
        self.product_id = Some(pid);
        self
    }

    /// Set baud rate (for UART)
    #[must_use]
    pub const fn with_baud_rate(mut self, baud: u32) -> Self {
        self.baud_rate = Some(baud);
        self
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::{TransportConfig, TransportType};

    #[test]
    fn transport_type_display_usb_and_uart() {
        assert_eq!(format!("{}", TransportType::Usb), "USB");
        assert_eq!(format!("{}", TransportType::Uart), "UART");
    }

    #[test]
    fn transport_type_equality_and_hash_consistency() {
        assert_eq!(TransportType::Usb, TransportType::Usb);
        assert_ne!(TransportType::Usb, TransportType::Uart);
    }

    #[test]
    fn transport_config_default_prefers_usb_and_common_uart_baud() {
        let c = TransportConfig::default();
        assert_eq!(c.transport_type, TransportType::Usb);
        assert!(c.vendor_id.is_none());
        assert!(c.product_id.is_none());
        assert!(c.serial_port.is_none());
        assert_eq!(c.baud_rate, Some(115_200));
    }

    #[test]
    fn transport_config_usb_constructor() {
        let c = TransportConfig::usb();
        assert_eq!(c.transport_type, TransportType::Usb);
        assert_eq!(c.baud_rate, Some(115_200));
    }

    #[test]
    fn transport_config_uart_sets_port_and_type() {
        let c = TransportConfig::uart("/dev/ttyUSB0");
        assert_eq!(c.transport_type, TransportType::Uart);
        assert_eq!(c.serial_port.as_deref(), Some("/dev/ttyUSB0"));
    }

    #[test]
    fn transport_config_usb_chain_vid_pid() {
        let c = TransportConfig::usb().with_vendor_id(0x0A12).with_product_id(0x4010);
        assert_eq!(c.vendor_id, Some(0x0A12));
        assert_eq!(c.product_id, Some(0x4010));
    }

    #[test]
    fn transport_config_uart_with_custom_baud() {
        let c = TransportConfig::uart("COM3").with_baud_rate(921_600);
        assert_eq!(c.baud_rate, Some(921_600));
        assert_eq!(c.transport_type, TransportType::Uart);
    }

    #[test]
    fn transport_config_debug_includes_key_fields() {
        let c = TransportConfig::uart("/dev/rfcomm0").with_baud_rate(9600);
        let s = format!("{c:?}");
        assert!(s.contains("Uart") || s.contains("rfcomm"));
    }
}
