// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for capability-based filtering and env-driven canonical URLs in `canonical::constants`.

use songbird_config::canonical::{find_primals_with_capability, get_canonical_endpoint};

#[test]
#[serial_test::serial]
fn find_primals_respects_capability_providers_env() {
    songbird_process_env::set_var("SONGBIRD_CAPABILITY_ALPHA_TEST_PROVIDERS", "node-one, node-two");
    let names = find_primals_with_capability("alpha-test");
    songbird_process_env::remove_var("SONGBIRD_CAPABILITY_ALPHA_TEST_PROVIDERS");

    assert_eq!(names, vec!["node-one", "node-two"]);
}

#[test]
#[serial_test::serial]
fn find_primals_filters_by_per_primal_capabilities_env() {
    songbird_process_env::set_var("ALICE_ENDPOINT", "http://alice:1");
    songbird_process_env::set_var("ALICE_CAPABILITIES", "metrics, alpha-test , storage");
    songbird_process_env::set_var("BOB_ENDPOINT", "http://bob:2");
    songbird_process_env::set_var("BOB_CAPABILITIES", "other");

    let names = find_primals_with_capability("alpha-test");
    songbird_process_env::remove_var("ALICE_ENDPOINT");
    songbird_process_env::remove_var("ALICE_CAPABILITIES");
    songbird_process_env::remove_var("BOB_ENDPOINT");
    songbird_process_env::remove_var("BOB_CAPABILITIES");

    assert_eq!(names, vec!["alice"]);
}

#[test]
#[serial_test::serial]
fn find_primals_capability_providers_overrides_per_primal_scan() {
    songbird_process_env::set_var("ALICE_ENDPOINT", "http://alice:1");
    songbird_process_env::set_var("ALICE_CAPABILITIES", "ignored");
    songbird_process_env::set_var("SONGBIRD_CAPABILITY_METRICS_PROVIDERS", "z-only");

    let names = find_primals_with_capability("metrics");
    songbird_process_env::remove_var("ALICE_ENDPOINT");
    songbird_process_env::remove_var("ALICE_CAPABILITIES");
    songbird_process_env::remove_var("SONGBIRD_CAPABILITY_METRICS_PROVIDERS");

    assert_eq!(names, vec!["z-only"]);
}

#[test]
#[serial_test::serial]
fn get_canonical_endpoint_production_uses_configurable_https_port() {
    songbird_process_env::set_var("SONGBIRD_ENVIRONMENT", "production");
    songbird_process_env::set_var("SONGBIRD_BIND_ADDRESS", "127.0.0.1");
    songbird_process_env::set_var("SONGBIRD_PRODUCTION_HTTPS_PORT", "9443");
    songbird_process_env::remove_var("SONGBIRD_BASE_URL");

    let url = get_canonical_endpoint("svc", 9999);
    songbird_process_env::remove_var("SONGBIRD_ENVIRONMENT");
    songbird_process_env::remove_var("SONGBIRD_BIND_ADDRESS");
    songbird_process_env::remove_var("SONGBIRD_PRODUCTION_HTTPS_PORT");

    assert_eq!(url, "https://127.0.0.1:9443");
}

#[test]
#[serial_test::serial]
fn get_canonical_endpoint_staging_avoids_literal_internal_host_when_unset() {
    songbird_process_env::set_var("SONGBIRD_ENVIRONMENT", "staging");
    songbird_process_env::set_var("SONGBIRD_BIND_ADDRESS", "127.0.0.1");
    songbird_process_env::set_var("SONGBIRD_STAGING_HTTP_PORT", "9090");
    songbird_process_env::remove_var("SONGBIRD_BASE_URL");
    songbird_process_env::remove_var("SONGBIRD_STAGING_BASE_URL");

    let url = get_canonical_endpoint("svc", 1111);
    songbird_process_env::remove_var("SONGBIRD_ENVIRONMENT");
    songbird_process_env::remove_var("SONGBIRD_BIND_ADDRESS");
    songbird_process_env::remove_var("SONGBIRD_STAGING_HTTP_PORT");

    assert_eq!(url, "http://127.0.0.1:9090");
}
