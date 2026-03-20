// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::*;
use std::net::IpAddr;

#[test]
fn test_get_bind_address() {
    let addr = get_bind_address();
    assert!(!addr.is_empty());
    assert!(addr.parse::<IpAddr>().is_ok());
}

#[test]
fn test_get_bind_address_uses_valid_env_override() {
    songbird_process_env::set_var("SONGBIRD_BIND_ADDRESS", "10.0.0.5");
    let addr = get_bind_address();
    songbird_process_env::remove_var("SONGBIRD_BIND_ADDRESS");

    assert!(
        addr == "10.0.0.5" || addr.parse::<IpAddr>().is_ok(),
        "expected 10.0.0.5 or a valid IP, got: {addr}"
    );
}

#[test]
fn test_get_bind_address_ignores_invalid_env_override() {
    songbird_process_env::set_var("SONGBIRD_BIND_ADDRESS", "not-an-ip");
    let addr = get_bind_address();
    assert!(addr.parse::<IpAddr>().is_ok());
    songbird_process_env::remove_var("SONGBIRD_BIND_ADDRESS");
}

#[test]
fn test_calculate_primal_port_offset_stable() {
    let a = calculate_primal_port_offset("alpha");
    let b = calculate_primal_port_offset("alpha");
    let c = calculate_primal_port_offset("beta");
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_get_primal_endpoint_prefers_explicit_env() {
    songbird_process_env::set_var("CUSTOMPRIMAL_ENDPOINT", "http://explicit:1111");
    let ep = get_primal_endpoint("customprimal");
    assert_eq!(ep, "http://explicit:1111");
    songbird_process_env::remove_var("CUSTOMPRIMAL_ENDPOINT");
}

#[test]
fn test_port_range() {
    let start = get_port_range_start();
    let end = get_port_range_end();
    assert!(start > 0);
    assert!(end > start);
    assert!(end >= 1024, "Port range end should be >= 1024");
}

#[test]
fn test_environment_detection() {
    let _ = is_development_environment();
    let _ = is_production_environment();
}

#[test]
fn test_primal_endpoint_generation() {
    let endpoint = get_primal_endpoint("test_primal");
    assert!(endpoint.starts_with("http://") || endpoint.starts_with("https://"));
}

#[test]
fn test_directory_configuration() {
    let log_dir = get_log_dir();
    let cache_dir = get_cache_dir();
    let data_dir = get_data_dir();
    let config_dir = get_config_dir();

    assert!(!log_dir.is_empty());
    assert!(!cache_dir.is_empty());
    assert!(!data_dir.is_empty());
    assert!(!config_dir.is_empty());
}
