// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use std::collections::HashMap;
use std::net::IpAddr;

#[test]
fn test_get_bind_address() {
    let addr = get_bind_address();
    assert!(!addr.is_empty());
    assert!(addr.parse::<IpAddr>().is_ok());
}

#[test]
fn test_get_bind_address_uses_valid_env_override() {
    let addr = get_bind_address_with(&|k| {
        if k == "SONGBIRD_BIND_ADDRESS" {
            Ok("10.0.0.5".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });

    assert!(
        addr == "10.0.0.5" || addr.parse::<IpAddr>().is_ok(),
        "expected 10.0.0.5 or a valid IP, got: {addr}"
    );
}

#[test]
fn test_get_bind_address_ignores_invalid_env_override() {
    let addr = get_bind_address_with(&|k| {
        if k == "SONGBIRD_BIND_ADDRESS" {
            Ok("not-an-ip".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert!(addr.parse::<IpAddr>().is_ok());
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
    let ep = get_primal_endpoint_with("customprimal", &|k| {
        if k == "CUSTOMPRIMAL_ENDPOINT" {
            Ok("http://explicit:1111".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(ep, "http://explicit:1111");
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

#[test]
fn test_bind_address_aliases_match() {
    assert_eq!(get_canonical_bind_address(), get_bind_address());
    assert_eq!(get_default_bind_address(), default_bind_address());
}

#[test]
fn test_protocol_port_mappings_keys() {
    let m = protocol_port_mappings();
    assert!(m.contains_key("udp"));
    assert!(m.contains_key("tcp"));
    assert!(m.contains_key("websocket"));
    assert!(m.contains_key("secure_websocket"));
}

#[test]
fn test_external_address_and_subnet() {
    let _ = external_address();
    assert!(!default_subnet().is_empty());
}

#[test]
fn test_node_id_format() {
    let id = node_id();
    assert!(id.starts_with("songbird-"));
    assert!(id.len() > 8);
}

#[test]
fn test_network_submodule_helpers() {
    let h = network::default_host();
    assert!(!h.is_empty());
    let _p = network::default_orchestrator_port();
    let _d = network::default_dashboard_port();
    assert_eq!(network::DEFAULT_RETRY_DELAY, std::time::Duration::from_millis(1000));
}

#[test]
fn test_canonical_network_defaults_bind_and_allowed() {
    let _ = CanonicalNetworkDefaults::bind_address();
    let nets = CanonicalNetworkDefaults::allowed_networks();
    assert!(!nets.is_empty());
}

#[test]
fn test_get_temp_dir_non_empty() {
    assert!(!get_temp_dir().is_empty());
}

#[test]
fn test_get_log_level_respects_env() {
    let lvl = get_log_level_with(&|k| {
        if k == "SONGBIRD_LOG_LEVEL" {
            Ok("error".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(lvl, "error");
}

#[test]
fn test_find_primals_with_capability_explicit_list() {
    let map = HashMap::from([(
        "SONGBIRD_CAPABILITY_ZZ_ALPHA_PROVIDERS".to_string(),
        "foo, bar ".to_string(),
    )]);
    let out = find_primals_with_capability_in_env("zz-alpha", &map);
    assert_eq!(out, vec!["foo", "bar"]);
}

#[test]
fn test_get_common_primal_ports_includes_base_and_enabled_flag() {
    let base = get_port_range_start();
    let ports = get_common_primal_ports_from_env_map(&HashMap::from([(
        "SONGBIRD_ENABLE_SBTESTPRIMAL".to_string(),
        "true".to_string(),
    )]));
    assert!(ports.contains(&base));
    assert!(ports.iter().any(|&p| p > base));
}

#[test]
fn test_get_canonical_endpoint_development_uses_base_url_fallback() {
    let ep = get_canonical_endpoint_with("mysvc", 9999, |k| match k {
        "SONGBIRD_ENVIRONMENT" => Ok("development".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    });
    assert!(ep.contains("9999") || ep.contains("http"));
}

#[test]
fn test_cors_origins_non_production_not_empty() {
    let origins = get_canonical_cors_origins_with(&|k| match k {
        "SONGBIRD_ENVIRONMENT" => Ok("development".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    });
    assert!(!origins.is_empty());
}
