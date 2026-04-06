// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for Bluetooth error types

use songbird_bluetooth::error::{BluetoothError, TransportError};
use std::time::Duration;

#[test]
fn test_bluetooth_error_device() {
    let err = BluetoothError::device("Device not found");
    let msg = format!("{}", err);
    assert!(msg.contains("Device error"));
    assert!(msg.contains("Device not found"));
}

#[test]
fn test_bluetooth_error_hci() {
    let err = BluetoothError::hci("HCI command failed");
    let msg = format!("{}", err);
    assert!(msg.contains("HCI error"));
    assert!(msg.contains("HCI command failed"));
}

#[test]
fn test_bluetooth_error_gatt() {
    let err = BluetoothError::gatt("GATT attribute not found");
    let msg = format!("{}", err);
    assert!(msg.contains("GATT error"));
    assert!(msg.contains("GATT attribute not found"));
}

#[test]
fn test_bluetooth_error_timeout() {
    let duration = Duration::from_secs(30);
    let err = BluetoothError::timeout(duration);
    let msg = format!("{}", err);
    assert!(msg.contains("Operation timed out"));
    assert!(msg.contains("30s"));
}

#[test]
fn test_bluetooth_error_is_timeout() {
    let timeout_err = BluetoothError::timeout(Duration::from_secs(5));
    assert!(timeout_err.is_timeout());

    let device_err = BluetoothError::device("test");
    assert!(!device_err.is_timeout());
}

#[test]
fn test_bluetooth_error_is_recoverable() {
    let timeout_err = BluetoothError::timeout(Duration::from_secs(5));
    assert!(timeout_err.is_recoverable());

    let device_err = BluetoothError::device("test");
    assert!(device_err.is_recoverable());

    let transport_err = BluetoothError::Transport(TransportError::NoAdapter);
    assert!(transport_err.is_recoverable());

    let config_err = BluetoothError::Configuration("test".to_string());
    assert!(!config_err.is_recoverable());

    let not_supported = BluetoothError::NotSupported("test".to_string());
    assert!(!not_supported.is_recoverable());
}

#[test]
fn test_bluetooth_error_invalid_operation() {
    let err = BluetoothError::InvalidOperation("Cannot scan while disconnected".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Invalid operation"));
}

#[test]
fn test_bluetooth_error_configuration() {
    let err = BluetoothError::Configuration("Missing adapter address".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Configuration error"));
}

#[test]
fn test_bluetooth_error_not_supported() {
    let err = BluetoothError::NotSupported("BLE 5.0 features".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Not supported"));
}

#[test]
fn test_bluetooth_error_invalid_data() {
    let err = BluetoothError::InvalidData {
        context: "Malformed PDU header".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Invalid data"));
    assert!(msg.contains("Malformed PDU header"));
}

#[test]
fn test_transport_error_no_adapter() {
    let err = TransportError::NoAdapter;
    let msg = format!("{}", err);
    assert!(msg.contains("No Bluetooth adapter found"));
}

#[test]
fn test_transport_error_initialization_failed() {
    let err = TransportError::InitializationFailed("Driver not loaded".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Failed to initialize adapter"));
    assert!(msg.contains("Driver not loaded"));
}

#[test]
fn test_transport_error_communication() {
    let err = TransportError::Communication("Connection reset".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Communication error"));
    assert!(msg.contains("Connection reset"));
}

#[test]
fn test_bluetooth_error_from_transport() {
    let transport_err = TransportError::NoAdapter;
    let bt_err: BluetoothError = transport_err.into();
    let msg = format!("{}", bt_err);
    assert!(msg.contains("Transport error"));
}

#[test]
fn test_bluetooth_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let bt_err: BluetoothError = io_err.into();
    let msg = format!("{}", bt_err);
    assert!(msg.contains("I/O error"));
}

#[test]
fn test_bluetooth_error_debug() {
    let err = BluetoothError::device("test");
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("Device"));
}

#[test]
fn test_transport_error_debug() {
    let err = TransportError::NoAdapter;
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("NoAdapter"));
}
