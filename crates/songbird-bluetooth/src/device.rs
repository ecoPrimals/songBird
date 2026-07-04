// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Bluetooth device representation
//!
//! Modern, safe types for representing BLE devices and addresses.

use std::fmt;
use std::str::FromStr;

/// Bluetooth device address (MAC address)
///
/// 48-bit address in standard Bluetooth format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address([u8; 6]);

impl Address {
    /// Create address from bytes
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Get address as bytes
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 6] {
        &self.0
    }

    /// Create random address (for testing)
    #[cfg(test)]
    #[must_use]
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self(std::array::from_fn(|_| Rng::r#gen(&mut rng)))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

impl FromStr for Address {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 6 {
            return Err(format!("Invalid address format: {s}"));
        }

        let mut bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            bytes[i] =
                u8::from_str_radix(part, 16).map_err(|e| format!("Invalid hex byte: {e}"))?;
        }

        Ok(Self(bytes))
    }
}

/// Device information from scan
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device address
    pub address: Address,

    /// Device name (if available)
    pub name: Option<String>,

    /// RSSI (signal strength)
    pub rssi: i8,

    /// Advertised services
    pub services: Vec<uuid::Uuid>,

    /// Manufacturer data
    pub manufacturer_data: Option<Vec<u8>>,
}

impl DeviceInfo {
    /// Create new device info
    #[must_use]
    pub const fn new(address: Address) -> Self {
        Self {
            address,
            name: None,
            rssi: 0,
            services: Vec::new(),
            manufacturer_data: None,
        }
    }

    /// Set device name (builder pattern)
    #[must_use]
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set RSSI (builder pattern)
    #[must_use]
    pub const fn with_rssi(mut self, rssi: i8) -> Self {
        self.rssi = rssi;
        self
    }

    /// Add service UUID (builder pattern)
    #[must_use]
    pub fn with_service(mut self, service: uuid::Uuid) -> Self {
        self.services.push(service);
        self
    }

    /// Set manufacturer data (builder pattern)
    #[must_use]
    pub fn with_manufacturer_data(mut self, data: Vec<u8>) -> Self {
        self.manufacturer_data = Some(data);
        self
    }

    /// Check if device advertises a specific service
    #[must_use]
    pub fn has_service(&self, service_uuid: &uuid::Uuid) -> bool {
        self.services.contains(service_uuid)
    }
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Device({})", self.address)?;
        if let Some(ref name) = self.name {
            write!(f, " name=\"{name}\"")?;
        }
        write!(f, " rssi={}", self.rssi)?;
        if !self.services.is_empty() {
            write!(f, " services={}", self.services.len())?;
        }
        Ok(())
    }
}

/// Connected device handle
#[derive(Debug)]
pub struct Device {
    /// Device info
    pub info: DeviceInfo,

    /// Connection handle
    pub(crate) handle: u16,
}

impl Device {
    /// Create new device handle
    #[must_use]
    pub(crate) const fn new(info: DeviceInfo, handle: u16) -> Self {
        Self {
            info,
            handle,
        }
    }

    /// Get device address
    #[must_use]
    pub const fn address(&self) -> Address {
        self.info.address
    }

    /// Get device name
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.info.name.as_deref()
    }

    /// Get connection handle
    #[must_use]
    pub const fn handle(&self) -> u16 {
        self.handle
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_address_parsing() {
        let addr = match "AA:BB:CC:DD:EE:FF".parse::<Address>() {
            Ok(a) => a,
            Err(e) => panic!("parse address: {e:?}"),
        };
        assert_eq!(addr.0, [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        assert_eq!(addr.to_string(), "AA:BB:CC:DD:EE:FF");
    }

    #[test]
    fn test_device_info() {
        let addr = Address::from_bytes([1, 2, 3, 4, 5, 6]);
        let mut info = DeviceInfo::new(addr);
        info.name = Some(String::from("Test Device"));
        info.rssi = -50;

        assert_eq!(info.address, addr);
        assert_eq!(info.name.as_deref(), Some("Test Device"));
        assert_eq!(info.rssi, -50);
    }

    #[test]
    fn address_parse_rejects_wrong_field_count() {
        let err = "AA:BB:CC".parse::<Address>().expect_err("too few");
        assert!(err.contains("Invalid address"));
    }

    #[test]
    fn address_parse_rejects_non_hex() {
        let err = "GG:BB:CC:DD:EE:FF".parse::<Address>().expect_err("bad hex");
        assert!(err.contains("hex") || err.contains("Invalid"));
    }

    #[test]
    fn device_info_builder_and_has_service() {
        let addr = Address::from_bytes([9, 8, 7, 6, 5, 4]);
        let u = uuid::Uuid::from_u128(0xABCD);
        let info = DeviceInfo::new(addr)
            .with_name("N".into())
            .with_rssi(-30)
            .with_service(u)
            .with_manufacturer_data(vec![0x00, 0x01]);

        assert!(info.has_service(&u));
        assert!(!info.has_service(&uuid::Uuid::from_u128(1)));
        assert_eq!(info.manufacturer_data.as_ref().map(Vec::len), Some(2));
        let s = info.to_string();
        assert!(s.contains('N') && s.contains("rssi=-30"));
    }

    #[test]
    fn device_wrapper_exposes_address_name_handle() {
        let addr = Address::from_bytes([1, 2, 3, 4, 5, 6]);
        let info = DeviceInfo::new(addr).with_name("Wrap".into());
        let dev = Device::new(info, 0x00AB);
        assert_eq!(dev.address(), addr);
        assert_eq!(dev.name(), Some("Wrap"));
        assert_eq!(dev.handle(), 0x00AB);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn uuid_serde_json_round_trip_for_service_lists() {
        let u = uuid::Uuid::from_u128(0xF00D);
        let json = serde_json::to_string(&u).expect("serialize");
        let back: uuid::Uuid = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(u, back);
    }
}
