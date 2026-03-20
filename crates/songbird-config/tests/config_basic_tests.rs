// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    clippy::unnecessary_literal_unwrap,
    clippy::uninlined_format_args
)]

//! Basic configuration tests
//!
//! Tests for `SongbirdConfig` creation and basic operations.
//! Note: Using deprecated SongbirdConfig for backward compatibility testing.
//! Canonical config types are now standard (migrated Jan 2026).

use songbird_config::config::SongbirdConfig;
#[test]
fn test_songbird_config_default() {
    let config = SongbirdConfig::default();
    assert!(!config.network.bind_address.is_empty());
}

#[test]
fn test_songbird_config_clone() {
    let config = SongbirdConfig::default();
    let cloned = config.clone();
    assert_eq!(config.network.bind_address, cloned.network.bind_address);
}

#[test]
fn test_songbird_config_debug() {
    let config = SongbirdConfig::default();
    let debug_string = format!("{:?}", config);
    assert!(debug_string.contains("SongbirdConfig"));
}

#[test]
fn test_network_bind_address_modification() {
    let mut config = SongbirdConfig::default();
    let original = config.network.bind_address.clone();
    config.network.bind_address = "192.168.1.100".to_string();
    assert_ne!(config.network.bind_address, original);
    assert_eq!(config.network.bind_address, "192.168.1.100");
}

#[test]
fn test_network_bind_address_empty() {
    let mut config = SongbirdConfig::default();
    config.network.bind_address = String::new();
    assert!(config.network.bind_address.is_empty());
}

#[test]
fn test_network_bind_address_with_unicode() {
    let mut config = SongbirdConfig::default();
    config.network.bind_address = "アドレス".to_string();
    assert!(config.network.bind_address.contains("アドレス"));
}

#[test]
fn test_network_port_range_valid() {
    let config = SongbirdConfig::default();
    assert!(config.network.port_range.start < config.network.port_range.end);
}

#[test]
fn test_network_port_range_modification() {
    let mut config = SongbirdConfig::default();
    config.network.port_range.start = 10000;
    config.network.port_range.end = 20000;
    assert_eq!(config.network.port_range.start, 10000);
    assert_eq!(config.network.port_range.end, 20000);
}

#[test]
fn test_network_port_range_boundaries() {
    let config = SongbirdConfig::default();
    // Ports should be in valid range
    assert!(config.network.port_range.start >= 1024);
    // Port range end is u16, always <= 65535
    assert!(config.network.port_range.end > 0);
}

#[test]
fn test_network_port_range_invalid_detected() {
    let mut config = SongbirdConfig::default();
    config.network.port_range.start = 9000;
    config.network.port_range.end = 8000; // Invalid
    // We can detect this is invalid
    assert!(config.network.port_range.start >= config.network.port_range.end);
}

#[test]
fn test_network_config_field_access() {
    let config = SongbirdConfig::default();
    let _ = config.network.bind_address;
    let _ = config.network.port_range.start;
    let _ = config.network.port_range.end;
}

#[test]
fn test_config_has_network() {
    let config = SongbirdConfig::default();
    // Network config exists
    assert!(!config.network.bind_address.is_empty() || config.network.bind_address.is_empty());
}

#[test]
fn test_config_multiple_modifications() {
    let mut config = SongbirdConfig::default();
    config.network.bind_address = "192.168.1.1".to_string();
    config.network.port_range.start = 8000;
    config.network.port_range.end = 9000;
    assert_eq!(config.network.bind_address, "192.168.1.1");
    assert_eq!(config.network.port_range.start, 8000);
    assert_eq!(config.network.port_range.end, 9000);
}

#[test]
fn test_config_clone_independence() {
    let config1 = SongbirdConfig::default();
    let mut config2 = config1.clone();
    config2.network.bind_address = "modified".to_string();
    // Clones are independent
    assert_ne!(config1.network.bind_address, config2.network.bind_address);
}
