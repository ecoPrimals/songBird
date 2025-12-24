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
        Self([
            rng.gen(),
            rng.gen(),
            rng.gen(),
            rng.gen(),
            rng.gen(),
            rng.gen(),
        ])
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
            bytes[i] = u8::from_str_radix(part, 16)
                .map_err(|e| format!("Invalid hex byte: {e}"))?;
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
    pub fn new(address: Address) -> Self {
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
    pub fn with_rssi(mut self, rssi: i8) -> Self {
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
    pub(crate) fn new(info: DeviceInfo, handle: u16) -> Self {
        Self { info, handle }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_parsing() {
        let addr = "AA:BB:CC:DD:EE:FF".parse::<Address>().unwrap();
        assert_eq!(addr.0, [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        assert_eq!(addr.to_string(), "AA:BB:CC:DD:EE:FF");
    }

    #[test]
    fn test_device_info() {
        let addr = Address::from_bytes([1, 2, 3, 4, 5, 6]);
        let mut info = DeviceInfo::new(addr);
        info.name = Some("Test Device".to_string());
        info.rssi = -50;

        assert_eq!(info.address, addr);
        assert_eq!(info.name.as_deref(), Some("Test Device"));
        assert_eq!(info.rssi, -50);
    }
}

