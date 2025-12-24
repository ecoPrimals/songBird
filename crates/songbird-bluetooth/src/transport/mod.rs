//! Transport layer abstraction for Bluetooth HCI
//!
//! Supports multiple transport types:
//! - USB: For desktop deployments with USB Bluetooth dongles
//! - UART: For embedded deployments with UART Bluetooth modules
//!
//! All transports implement the `Transport` trait for unified access.

use crate::error::Result;
use std::fmt;

#[cfg(feature = "usb")]
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
    pub fn with_vendor_id(mut self, vid: u16) -> Self {
        self.vendor_id = Some(vid);
        self
    }

    /// Set product ID (for USB filtering)
    #[must_use]
    pub fn with_product_id(mut self, pid: u16) -> Self {
        self.product_id = Some(pid);
        self
    }

    /// Set baud rate (for UART)
    #[must_use]
    pub fn with_baud_rate(mut self, baud: u32) -> Self {
        self.baud_rate = Some(baud);
        self
    }
}
