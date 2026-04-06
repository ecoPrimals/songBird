// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for capability-based filtering and env-driven canonical URLs in `canonical::constants`.

use songbird_config::canonical::{
    find_primals_with_capability_in_env, get_canonical_endpoint_with,
};
use std::collections::HashMap;

#[test]
fn find_primals_respects_capability_providers_env() {
    let mut env: HashMap<String, String> = HashMap::new();
    env.insert(
        "SONGBIRD_CAPABILITY_ALPHA_TEST_PROVIDERS".to_string(),
        "node-one, node-two".to_string(),
    );
    let names = find_primals_with_capability_in_env("alpha-test", &env);

    assert_eq!(names, vec!["node-one", "node-two"]);
}

#[test]
fn find_primals_filters_by_per_primal_capabilities_env() {
    let mut env: HashMap<String, String> = HashMap::new();
    env.insert("ALICE_ENDPOINT".to_string(), "http://alice:1".to_string());
    env.insert("ALICE_CAPABILITIES".to_string(), "metrics, alpha-test , storage".to_string());
    env.insert("BOB_ENDPOINT".to_string(), "http://bob:2".to_string());
    env.insert("BOB_CAPABILITIES".to_string(), "other".to_string());

    let names = find_primals_with_capability_in_env("alpha-test", &env);

    assert_eq!(names, vec!["alice"]);
}

#[test]
fn find_primals_capability_providers_overrides_per_primal_scan() {
    let mut env: HashMap<String, String> = HashMap::new();
    env.insert("ALICE_ENDPOINT".to_string(), "http://alice:1".to_string());
    env.insert("ALICE_CAPABILITIES".to_string(), "ignored".to_string());
    env.insert("SONGBIRD_CAPABILITY_METRICS_PROVIDERS".to_string(), "z-only".to_string());

    let names = find_primals_with_capability_in_env("metrics", &env);

    assert_eq!(names, vec!["z-only"]);
}

#[test]
fn get_canonical_endpoint_production_uses_configurable_https_port() {
    let url = get_canonical_endpoint_with("svc", 9999, |key| match key {
        "SONGBIRD_ENVIRONMENT" => Ok("production".to_string()),
        "SONGBIRD_BIND_ADDRESS" => Ok("127.0.0.1".to_string()),
        "SONGBIRD_PRODUCTION_HTTPS_PORT" => Ok("9443".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    });

    assert_eq!(url, "https://127.0.0.1:9443");
}

#[test]
fn get_canonical_endpoint_staging_avoids_literal_internal_host_when_unset() {
    let url = get_canonical_endpoint_with("svc", 1111, |key| match key {
        "SONGBIRD_ENVIRONMENT" => Ok("staging".to_string()),
        "SONGBIRD_BIND_ADDRESS" => Ok("127.0.0.1".to_string()),
        "SONGBIRD_STAGING_HTTP_PORT" => Ok("9090".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    });

    assert_eq!(url, "http://127.0.0.1:9090");
}
