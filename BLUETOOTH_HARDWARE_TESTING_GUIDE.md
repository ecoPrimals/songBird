# 🔧 Bluetooth Hardware Testing Guide

**Pure Rust Bluetooth Stack - Hardware Validation**

---

## 🎯 Overview

This guide covers testing the Songbird Pure Rust Bluetooth stack with real hardware. Use this for validating USB dongles, UART modules, and platform-specific configurations.

---

## 📋 Prerequisites

### Required Hardware

**For Desktop Testing (USB)**:
- USB Bluetooth dongle ($5-20)
- Recommended chipsets:
  - CSR (Cambridge Silicon Radio)
  - Realtek RTL8761
  - Intel AX200/AX210
  - Broadcom BCM20702

**For Embedded Testing (UART)**:
- UART Bluetooth module
- Options:
  - ESP32 with built-in Bluetooth
  - nRF52840 module
  - HC-05/HC-06 (basic testing)
  - Raspberry Pi built-in Bluetooth

### Software Requirements

```bash
# Rust toolchain
rustup install stable
rustup default stable

# Build tools
cargo install cargo-tarpaulin  # For coverage
cargo install cargo-audit      # For security audits

# Platform-specific tools (see sections below)
```

---

## 🖥️ Platform Setup

### Linux

#### Permissions

```bash
# Add user to plugdev group (USB access)
sudo usermod -a -G plugdev $USER

# Add user to dialout group (UART access)
sudo usermod -a -G dialout $USER

# Logout and login for changes to take effect
```

#### Verify USB Dongle

```bash
# List USB devices
lsusb | grep -i bluetooth

# Expected output:
# Bus 001 Device 003: ID 0a12:0001 Cambridge Silicon Radio, Ltd Bluetooth Dongle

# Check device permissions
ls -l /dev/bus/usb/001/003

# Should show read/write for plugdev group
```

#### Verify UART Port

```bash
# List serial ports
ls -l /dev/ttyUSB* /dev/ttyACM* /dev/ttyAMA*

# Check Bluetooth UART
sudo dmesg | grep -i bluetooth
sudo dmesg | grep -i tty
```

#### Stop System Bluetooth

```bash
# Stop BlueZ to free the hardware
sudo systemctl stop bluetooth

# Disable automatic start (optional)
sudo systemctl disable bluetooth

# Verify it's stopped
systemctl status bluetooth
```

### Windows

#### USB Drivers

Windows 10/11 handle USB Bluetooth dongles automatically:
- No driver installation needed
- Plug and play support
- Works out of the box

#### Testing

```powershell
# Run in PowerShell
cargo test -p songbird-bluetooth --features usb -- --ignored --nocapture

# Or in Command Prompt
cargo test -p songbird-bluetooth --features usb -- --ignored --nocapture
```

### macOS

#### Permissions

macOS handles USB devices automatically, but may need security approval:

1. First run may prompt for USB device access
2. Go to System Preferences → Security & Privacy
3. Allow the application to access USB devices

#### Testing

```bash
cargo test -p songbird-bluetooth --features usb -- --ignored --nocapture
```

### Raspberry Pi

#### Built-in Bluetooth (UART)

```bash
# Disable system Bluetooth to access UART
sudo systemctl stop bluetooth
sudo systemctl disable bluetooth

# Identify Bluetooth UART port
ls -l /dev/ttyAMA*
# Usually /dev/ttyAMA0

# Add user to dialout
sudo usermod -a -G dialout $USER
```

#### USB Dongle

Same as Linux instructions above.

---

## 🧪 Hardware Tests

### Test 1: USB Dongle Detection

**File**: `crates/songbird-bluetooth/src/transport/usb.rs`

```bash
# Run USB transport test
cargo test -p songbird-bluetooth test_usb_transport_creation --features usb -- --ignored --nocapture
```

**Expected Output**:
```
test transport::usb::tests::test_usb_transport_creation ... ok
```

**If it fails**:
- Check USB dongle is plugged in
- Verify permissions (see platform setup)
- Try different USB port
- Check `lsusb` shows device

### Test 2: HCI Reset Command

Create test file: `tests/hardware_hci_test.rs`

```rust
#[tokio::test]
#[ignore] // Run only with --ignored flag
async fn test_hci_reset_command() {
    use songbird_bluetooth::UsbTransport;
    
    let mut transport = UsbTransport::new().await.expect("USB dongle not found");
    
    // Send HCI Reset (0x03 0x0C)
    let reset_cmd = vec![0x01, 0x03, 0x0C, 0x00];
    transport.send_command(&reset_cmd).await.expect("Send failed");
    
    // Receive Command Complete
    let response = transport.receive_event().await.expect("Receive failed");
    
    // Verify Command Complete event
    assert_eq!(response[0], 0x04); // Event packet
    assert_eq!(response[1], 0x0E); // Command Complete
    
    println!("✅ HCI Reset successful: {:?}", response);
}
```

**Run**:
```bash
cargo test -p songbird-bluetooth test_hci_reset_command --features usb -- --ignored --nocapture
```

### Test 3: BLE Scanning

```rust
#[tokio::test]
#[ignore]
async fn test_real_ble_scan() {
    use songbird_bluetooth::{BluetoothHost, UsbTransport};
    use std::time::Duration;
    
    let transport = UsbTransport::new().await.expect("USB dongle not found");
    let mut host = BluetoothHost::new(transport).expect("Host creation failed");
    
    println!("🔍 Starting BLE scan for 10 seconds...");
    let devices = host.scan_devices(Duration::from_secs(10))
        .await
        .expect("Scan failed");
    
    println!("✅ Found {} devices:", devices.len());
    for device in &devices {
        println!("  📱 {} (RSSI: {})", 
            device.address,
            device.rssi
        );
        if let Some(name) = &device.name {
            println!("      Name: {}", name);
        }
    }
    
    assert!(!devices.is_empty(), "Should find at least one device");
}
```

**Run**:
```bash
cargo test -p songbird-bluetooth test_real_ble_scan --features usb -- --ignored --nocapture
```

**Expected**: List of nearby BLE devices

### Test 4: Connection Test

```rust
#[tokio::test]
#[ignore]
async fn test_real_connection() {
    use songbird_bluetooth::{BluetoothHost, UsbTransport};
    use std::time::Duration;
    
    let transport = UsbTransport::new().await.unwrap();
    let mut host = BluetoothHost::new(transport).unwrap();
    
    // Scan for devices
    let devices = host.scan_devices(Duration::from_secs(5)).await.unwrap();
    assert!(!devices.is_empty(), "Need at least one device nearby");
    
    // Try to connect to first device
    let target = &devices[0];
    println!("🔗 Connecting to {} ...", target.address);
    
    let result = host.connect(target.address).await;
    
    match result {
        Ok(device) => {
            println!("✅ Connected! Handle: 0x{:04X}", device.handle());
            
            // Disconnect
            host.disconnect(target.address).await.unwrap();
            println!("✅ Disconnected");
        }
        Err(e) => {
            println!("⚠️  Connection failed (expected for most devices): {}", e);
        }
    }
}
```

**Note**: Most devices will reject unsolicited connections. This is expected.

---

## 📊 Performance Benchmarks

### Scan Performance

```bash
# Run 10 scans and average results
for i in {1..10}; do
    echo "Scan $i:"
    cargo test test_real_ble_scan --features usb -- --ignored --nocapture 2>&1 | grep "Found"
done
```

**Expected Metrics**:
- **Scan Duration**: 5-10 seconds
- **Devices Found**: 5-50 (varies by location)
- **CPU Usage**: 5-10% during scan
- **Memory**: ~5-10 MB

### Connection Performance

**Expected Metrics**:
- **Connection Time**: 100-500ms
- **Disconnect Time**: 50-100ms
- **Concurrent Connections**: Up to 4 (configurable)

---

## 🔍 Debugging

### Enable Trace Logging

```bash
# Set environment variable
export RUST_LOG=trace

# Run tests with full logging
cargo test test_real_ble_scan --features usb -- --ignored --nocapture
```

### HCI Packet Inspection

```bash
# Linux: Use btmon to see HCI packets
sudo btmon &

# Run your test
cargo test --features usb -- --ignored

# See HCI commands and events in btmon output
```

### USB Device Debugging

```bash
# Linux: Monitor USB events
sudo dmesg -w &

# Plug/unplug dongle and watch kernel messages
```

---

## ✅ Validation Checklist

### Basic Functionality

- [ ] USB dongle detected
- [ ] HCI Reset command works
- [ ] BLE scanning finds devices
- [ ] Advertisement parsing correct
- [ ] RSSI values reasonable (-100 to -30 dBm)
- [ ] Connection attempt completes (success or rejection)

### Platform Compatibility

- [ ] Linux: All tests pass
- [ ] Windows: USB tests pass
- [ ] macOS: USB tests pass
- [ ] Raspberry Pi: UART tests pass

### Performance

- [ ] Scan completes in 5-10 seconds
- [ ] CPU usage < 10% during scan
- [ ] Memory usage < 10 MB
- [ ] No memory leaks (run tests multiple times)

### Error Handling

- [ ] Graceful handling of missing dongle
- [ ] Proper timeout behavior
- [ ] Clear error messages
- [ ] No panics or crashes

---

## 🐛 Troubleshooting

### "USB dongle not found"

**Linux**:
```bash
# Check permissions
groups $USER  # Should include 'plugdev'

# Check USB device
lsusb | grep -i bluetooth

# Try with sudo (temporary test)
sudo -E cargo test --features usb -- --ignored
```

**Windows**:
- Ensure dongle is plugged in
- Try different USB port
- Check Device Manager

**macOS**:
- Check System Preferences → Security & Privacy
- Grant USB device access

### "Permission denied" on /dev/ttyUSB*

```bash
# Add to dialout group
sudo usermod -a -G dialout $USER

# Logout and login

# Or temporarily change permissions
sudo chmod 666 /dev/ttyUSB0
```

### Scan finds no devices

**Possible causes**:
1. No BLE devices nearby
2. Devices not advertising
3. System Bluetooth still running

**Solutions**:
```bash
# Ensure system Bluetooth stopped
sudo systemctl stop bluetooth

# Turn on some BLE devices (phone, smartwatch)

# Increase scan duration
# In test: Duration::from_secs(30)
```

### Connection always fails

**This is expected!** Most BLE devices:
- Don't accept unsolicited connections
- Require pairing first
- Have security requirements

To test actual connections:
- Use a development board (ESP32, nRF52)
- Configure it as peripheral
- Allow connections in its firmware

---

## 📈 Coverage Testing

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage with hardware tests
cargo tarpaulin -p songbird-bluetooth --features usb --ignored

# Generate HTML report
cargo tarpaulin -p songbird-bluetooth --features usb --out Html --ignored
```

---

## 🎯 Certified Hardware

### Tested USB Dongles

| Model | Chipset | Linux | Windows | macOS | Status |
|-------|---------|-------|---------|-------|--------|
| Generic CSR | CSR8510 | ✅ | ✅ | ✅ | Recommended |
| TP-Link UB400 | CSR8510 | ✅ | ✅ | ✅ | Recommended |
| ASUS USB-BT400 | BCM20702 | ✅ | ✅ | ✅ | Works well |
| Plugable USB-BT4LE | BCM20702 | ✅ | ✅ | ✅ | Works well |

**Note**: Most USB Bluetooth 4.0+ dongles should work. Above are specifically tested.

### Tested UART Modules

| Module | Interface | Tested On | Status |
|--------|-----------|-----------|--------|
| ESP32 DevKit | UART | Raspberry Pi | ✅ Works |
| nRF52840 DK | UART | Linux PC | ✅ Works |
| RPI Built-in | /dev/ttyAMA0 | Raspberry Pi 4 | 🚧 Testing |

---

## 📝 Test Report Template

When testing new hardware, use this template:

```markdown
## Hardware Test Report

**Date**: YYYY-MM-DD
**Tester**: Your Name
**Platform**: Linux/Windows/macOS/Raspberry Pi
**Rust Version**: `rustc --version`

### Hardware

- **Type**: USB Dongle / UART Module
- **Model**: [Model name/number]
- **Chipset**: [If known]
- **Purchase Link**: [Optional]

### Test Results

| Test | Result | Notes |
|------|--------|-------|
| USB Detection | ✅/❌ | |
| HCI Reset | ✅/❌ | |
| BLE Scan | ✅/❌ | Found X devices |
| Connection | ✅/❌ | |
| Disconnect | ✅/❌ | |

### Performance

- Scan Duration: X seconds
- Devices Found: X devices
- CPU Usage: X%
- Memory Usage: X MB

### Issues

[Describe any issues encountered]

### Recommendation

**Status**: ✅ Recommended / ⚠️ Works with issues / ❌ Not recommended

[Brief summary]
```

---

## 🚀 Continuous Integration

### GitHub Actions Example

```yaml
name: Hardware Tests

on:
  push:
    branches: [ main ]
  schedule:
    - cron: '0 0 * * 0'  # Weekly

jobs:
  hardware-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run hardware tests
        run: |
          cargo test -p songbird-bluetooth --features usb -- --ignored
        # Note: Requires self-hosted runner with USB dongle
```

**Note**: Hardware tests require physical devices, so need self-hosted runners.

---

## 📚 Additional Resources

### Bluetooth Core Specification

- Download: https://www.bluetooth.com/specifications/specs/
- Version: 5.3 or later
- Relevant sections: Vol 2 (BR/EDR), Vol 3 (Profiles), Vol 4 (HCI)

### USB Resources

- libusb documentation: https://libusb.info
- USB.org specifications: https://www.usb.org/documents

### Community

- Rust Embedded: https://github.com/rust-embedded
- Bluetooth LE on GitHub: Search "rust bluetooth"

---

## ✅ Summary

**For Quick Testing**:
1. Plug in USB Bluetooth dongle
2. Stop system Bluetooth: `sudo systemctl stop bluetooth`
3. Run: `cargo test -p songbird-bluetooth --features usb -- --ignored --nocapture`
4. Verify devices are found

**For Full Validation**:
1. Follow platform setup for your OS
2. Run all hardware tests
3. Document results
4. Report any issues

---

**Version**: 1.0  
**Last Updated**: December 24, 2025  
**Maintainer**: Songbird Team

🔧 **Hardware testing ensures universal compatibility!**

