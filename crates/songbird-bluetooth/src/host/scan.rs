// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! BLE host scanning: HCI scan parameters, advertisement collection, and parsing.

use crate::{
    device::{Address, DeviceInfo},
    error::{BluetoothError, Result},
    transport::Transport,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn};

use super::BluetoothHost;

impl<T: Transport + 'static> BluetoothHost<T> {
    /// Scan for BLE devices
    ///
    /// # Complexity Note
    ///
    /// This function has high cognitive complexity (26/25) due to:
    /// - HCI command sequencing (enable scan, wait, disable scan)
    /// - Event parsing and filtering
    /// - Timeout handling and cleanup
    /// - Device deduplication logic
    ///
    /// The complexity is justified by the BLE specification requirements
    /// and the need for atomic scan operations. Splitting would reduce
    /// readability and introduce state management overhead.
    ///
    /// # Errors
    ///
    /// Returns error if scan fails or timeout occurs
    pub async fn scan_devices(&mut self, duration: Duration) -> Result<Vec<DeviceInfo>> {
        {
            let mut scanning = self.scanning.lock().await;
            if *scanning {
                return Err(BluetoothError::InvalidOperation("Scan already in progress".into()));
            }

            *scanning = true;
        }
        info!("Starting BLE scan for {:?}", duration);

        // Phase 2: Actual BLE scanning with HCI commands
        let result = self.perform_scan(duration).await;

        {
            let mut scanning = self.scanning.lock().await;
            *scanning = false;
        }

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
        let devices =
            timeout(duration, self.collect_advertisements()).await.unwrap_or_else(|_| {
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
        let _response = timeout(Duration::from_secs(1), self.controller.receive_event())
            .await
            .map_err(|_| BluetoothError::Timeout {
            duration: Duration::from_secs(1),
        })??;

        debug!("Scan parameters set");
        Ok(())
    }

    /// Enable or disable BLE scanning
    async fn enable_scan(&self, enable: bool) -> Result<()> {
        debug!(
            "{}abling scan",
            if enable {
                "En"
            } else {
                "Dis"
            }
        );

        // HCI_LE_Set_Scan_Enable command
        // Opcode: 0x200C
        // Parameters:
        // - LE Scan Enable: 0x01 (enabled) or 0x00 (disabled)
        // - Filter Duplicates: 0x01 (enabled)

        let cmd = vec![
            0x01, // Command packet
            0x0C,
            0x20,             // Opcode: LE Set Scan Enable
            0x02,             // Parameter length
            u8::from(enable), // Scan enable
            0x01,             // Filter duplicates
        ];

        self.controller.send_command(&cmd).await?;

        // Wait for command complete
        let _response = timeout(Duration::from_secs(1), self.controller.receive_event())
            .await
            .map_err(|_| BluetoothError::Timeout {
            duration: Duration::from_secs(1),
        })??;

        debug!(
            "Scan {}abled",
            if enable {
                "en"
            } else {
                "dis"
            }
        );
        Ok(())
    }

    /// Collect BLE advertisements
    async fn collect_advertisements(&self) -> Result<Vec<DeviceInfo>> {
        let mut devices = HashMap::new();

        // Collect advertisements until timeout or max devices
        loop {
            // Try to receive advertisement with short timeout
            let event_result =
                timeout(Duration::from_millis(100), self.controller.receive_event()).await;

            match event_result {
                Ok(Ok(event)) => {
                    if let Some(device) = Self::parse_advertisement(&event) {
                        devices.entry(device.address).or_insert(device);
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
    fn parse_advertisement(event: &[u8]) -> Option<DeviceInfo> {
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

        let addr_bytes: [u8; 6] = event[addr_start..addr_start + 6].try_into().ok()?;

        let address = Address::from_bytes(addr_bytes);

        // Parse RSSI (last byte)
        #[expect(
            clippy::cast_possible_wrap,
            reason = "intentional pattern; clippy false positive for this API"
        )]
        let rssi = event.last().copied().map(|b| b as i8);

        // Parse device name from advertisement data (if present)
        let name = Self::parse_device_name(event);

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
    ///
    /// Parses Bluetooth advertising data (AD) structures to extract the device name.
    /// Follows Bluetooth Core Specification Vol 3, Part C, Section 11.
    ///
    /// AD Structure format:
    /// - Length (1 byte): Length of Type + Data
    /// - Type (1 byte): AD Type
    /// - Data (variable): AD Data
    fn parse_device_name(event: &[u8]) -> Option<String> {
        // Minimum event size check
        if event.len() < 2 {
            return None;
        }

        let mut offset = 0;

        // Parse AD structures
        while offset + 1 < event.len() {
            let length = event[offset] as usize;

            // Check for end of data or invalid length
            if length == 0 || offset + length >= event.len() {
                break;
            }

            let ad_type = event[offset + 1];
            let data_start = offset + 2;
            let data_end = offset + 1 + length;

            // AD Type 0x08: Shortened Local Name
            // AD Type 0x09: Complete Local Name
            if (ad_type == 0x08 || ad_type == 0x09) && data_end <= event.len() {
                let name_bytes = &event[data_start..data_end];
                // Convert bytes to UTF-8 string, replacing invalid sequences
                return Some(String::from_utf8_lossy(name_bytes).to_string());
            }

            // Move to next AD structure
            offset += 1 + length;
        }

        None
    }
}
