// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Error types for Songbird Bluetooth
//!
//! Modern, idiomatic error handling using `thiserror`.
//! Zero-cost abstractions with proper error propagation.

/// Result type for Bluetooth operations
pub type Result<T> = std::result::Result<T, BluetoothError>;

/// Comprehensive error type for Bluetooth operations
///
/// All errors are:
/// - Actionable: Include context for debugging
/// - Composable: Can be converted from lower-level errors
/// - Traceable: Preserve error chains
#[derive(Debug, thiserror::Error)]
pub enum BluetoothError {
    /// Transport layer error (USB, UART, etc.)
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    /// HCI protocol error
    #[error("HCI error: {0}")]
    Hci(String),

    /// GATT protocol error
    #[error("GATT error: {0}")]
    Gatt(String),

    /// Device not found or connection error
    #[error("Device error: {0}")]
    Device(String),

    /// Timeout during operation
    #[error("Operation timed out after {duration:?}")]
    Timeout {
        /// Duration before timeout
        duration: std::time::Duration,
    },

    /// Invalid parameter or state
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Not supported on this platform or hardware
    #[error("Not supported: {0}")]
    NotSupported(String),

    /// Invalid data format
    #[error("Invalid data: {context}")]
    InvalidData {
        /// Error context
        context: String,
    },
}

/// Transport-specific errors
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// USB transport error
    #[cfg(any(feature = "usb-rust", feature = "usb-c"))]
    #[error("USB error: {0}")]
    Usb(String),

    /// UART transport error
    #[cfg(feature = "uart")]
    #[error("UART error: {0}")]
    Uart(String),

    /// No Bluetooth adapter found
    #[error("No Bluetooth adapter found")]
    NoAdapter,

    /// Adapter initialization failed
    #[error("Failed to initialize adapter: {0}")]
    InitializationFailed(String),

    /// Communication error
    #[error("Communication error: {0}")]
    Communication(String),
}

impl BluetoothError {
    /// Create a device error
    #[must_use]
    pub fn device(msg: impl Into<String>) -> Self {
        Self::Device(msg.into())
    }

    /// Create an HCI error
    #[must_use]
    pub fn hci(msg: impl Into<String>) -> Self {
        Self::Hci(msg.into())
    }

    /// Create a GATT error
    #[must_use]
    pub fn gatt(msg: impl Into<String>) -> Self {
        Self::Gatt(msg.into())
    }

    /// Create a timeout error
    #[must_use]
    pub const fn timeout(duration: std::time::Duration) -> Self {
        Self::Timeout {
            duration,
        }
    }

    /// Check if error is a timeout
    #[must_use]
    pub const fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }

    /// Check if error is recoverable
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::Timeout { .. } | Self::Device(_) | Self::Transport(_))
    }
}

impl TransportError {
    /// Create a USB error
    #[cfg(any(feature = "usb-rust", feature = "usb-c"))]
    #[must_use]
    pub fn usb(msg: impl Into<String>) -> Self {
        Self::Usb(msg.into())
    }

    /// Create a UART error
    #[cfg(feature = "uart")]
    #[must_use]
    pub fn uart(msg: impl Into<String>) -> Self {
        Self::Uart(msg.into())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::{BluetoothError, Result, TransportError};
    use std::time::Duration;

    #[test]
    fn bluetooth_error_display_covers_invalid_data_variant() {
        let e = BluetoothError::InvalidData {
            context: "bad acl".to_string(),
        };
        let s = e.to_string();
        assert!(s.contains("Invalid data"));
        assert!(s.contains("bad acl"));
    }

    #[test]
    fn bluetooth_error_from_io_preserves_chain() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let e: BluetoothError = io.into();
        assert!(matches!(e, BluetoothError::Io(_)));
    }

    #[test]
    fn transport_error_into_bluetooth_error_round_trips_message() {
        let e: BluetoothError = TransportError::Communication("reset".into()).into();
        assert!(e.to_string().contains("reset"));
    }

    #[test]
    fn is_recoverable_matches_transport_and_timeout() {
        assert!(BluetoothError::timeout(Duration::from_millis(1)).is_recoverable());
        assert!(BluetoothError::Transport(TransportError::NoAdapter).is_recoverable());
        assert!(!BluetoothError::gatt("x").is_recoverable());
    }

    #[test]
    fn result_type_alias_compiles_for_ok() {
        let r: Result<u32> = Ok(42);
        assert!(matches!(r, Ok(42)));
    }

    #[cfg(any(feature = "usb-rust", feature = "usb-c"))]
    #[test]
    fn transport_error_usb_factory() {
        let e = TransportError::usb("descriptor");
        assert!(e.to_string().contains("USB"));
    }
}
