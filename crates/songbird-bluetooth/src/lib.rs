//! # Songbird Pure Rust Bluetooth LE Stack
//!
//! **Universal Comms - Zero System Dependencies**
//!
//! This crate provides a complete, pure Rust Bluetooth Low Energy (BLE) stack
//! for Songbird, enabling universal deployment without OS-level Bluetooth dependencies.
//!
//! ## Architecture
//!
//! ```text
//! Songbird Application
//!     ↓
//! songbird-bluetooth (this crate)
//!     ↓
//! trouble-host (Pure Rust BLE stack)
//!     ↓
//! Transport (USB/UART)
//!     ↓
//! Bluetooth Hardware
//! ```
//!
//! ## Features
//!
//! - **Pure Rust**: Zero C dependencies, no system Bluetooth stack required
//! - **Universal**: Works on any platform with USB or UART
//! - **Concurrent**: Async/await with Tokio or Embassy
//! - **Safe**: `#![forbid(unsafe_code)]` - no unsafe blocks
//! - **Fast**: Zero-cost abstractions, optimized for performance
//! - **Embedded-Ready**: Same code on desktop and ARM
//!
//! ## Platform Support
//!
//! - ✅ Linux (any distro, no BlueZ needed)
//! - ✅ Windows (no WinRT needed)
//! - ✅ macOS (no CoreBluetooth needed)
//! - ✅ Embedded (ARM, RISC-V, etc.)
//! - ✅ Any platform with USB Bluetooth dongle
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_bluetooth::{BluetoothHost, UsbTransport};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create USB transport (works anywhere with USB dongle)
//! let transport = UsbTransport::new().await?;
//!
//! // Create BLE host
//! let mut host = BluetoothHost::new(transport)?;
//!
//! // Scan for devices
//! let devices = host.scan_devices(std::time::Duration::from_secs(5)).await?;
//!
//! // Connect to device
//! let connection = host.connect(devices[0].address).await?;
//!
//! // GATT operations
//! let data = connection.read_characteristic(uuid).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Hardware Requirements
//!
//! - USB Bluetooth dongle ($5-10, any CSR or Realtek chipset)
//! - Or UART Bluetooth module (for embedded)
//! - Or built-in Bluetooth controller

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod transport;
pub mod host;
pub mod gatt;
pub mod device;

// Re-exports for convenience
pub use error::{BluetoothError, Result};
pub use host::BluetoothHost;
pub use transport::{Transport, TransportType};
pub use device::{Device, DeviceInfo, Address};

#[cfg(feature = "usb")]
pub use transport::usb::UsbTransport;

#[cfg(feature = "uart")]
pub use transport::UartTransport;

/// Bluetooth LE stack version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum MTU size for BLE connections
pub const MAX_MTU: usize = 512;

/// Default scan duration
pub const DEFAULT_SCAN_DURATION: std::time::Duration = std::time::Duration::from_secs(5);
