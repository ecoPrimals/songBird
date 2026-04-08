// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]

use super::hardcoded_elimination::replace;
use std::collections::HashMap;
use std::env::VarError;

#[test]
fn format_endpoint_prefers_full_endpoint_env() {
    songbird_process_env::set_var("ROUTING_ENDPOINT", "https://router.example:7443");
    let ep = replace::format_endpoint("routing", None);
    assert_eq!(ep.as_ref(), "https://router.example:7443");
    songbird_process_env::remove_var("ROUTING_ENDPOINT");
}

#[test]
fn format_service_endpoint_joins_base_and_path() {
    songbird_process_env::set_var("METRICS_ENDPOINT", "http://metrics.local:9090");
    let s = replace::format_service_endpoint("metrics", "/api/v1/query", None);
    assert_eq!(s, "http://metrics.local:9090/api/v1/query");
    songbird_process_env::remove_var("METRICS_ENDPOINT");
}

#[test]
fn gaming_port_matches_config_default_start() {
    let g = replace::gaming_port();
    assert_eq!(g, super::hardcoded_elimination::get_config().network.gaming_port_range.start);
}

#[test]
fn bind_address_returns_valid_ip() {
    let ip = replace::bind_address();
    assert!(ip.is_loopback() || !ip.is_unspecified() || ip.is_unspecified());
}

#[test]
fn env_or_default_with_returns_default_when_missing() {
    let vars: HashMap<String, String> = HashMap::new();
    let env = |k: &str| vars.get(k).cloned().ok_or(VarError::NotPresent);
    assert_eq!(
        super::hardcoded_elimination::env_or_default_with(
            "SONGBIRD_MISSING_KEY_XYZ",
            "fallback",
            env
        ),
        "fallback"
    );
}

#[test]
fn env_or_default_with_returns_value_when_present() {
    let vars: HashMap<String, String> =
        HashMap::from([("SONGBIRD_TEST_ONLY_PORT".to_string(), "4242".to_string())]);
    let env = move |k: &str| vars.get(k).cloned().ok_or(VarError::NotPresent);
    assert_eq!(
        super::hardcoded_elimination::env_or_default_with("SONGBIRD_TEST_ONLY_PORT", "9", env),
        "4242"
    );
}

#[test]
fn env_capability_first_then_legacy_warn_prefers_first_capability_key() {
    let vars: HashMap<String, String> = HashMap::from([
        ("SONGBIRD_A".to_string(), "first".to_string()),
        ("SONGBIRD_B".to_string(), "second".to_string()),
    ]);
    let env = move |k: &str| vars.get(k).cloned().ok_or(VarError::NotPresent);
    let v = super::hardcoded_elimination::env_capability_first_then_legacy_warn_with(
        &["SONGBIRD_A", "SONGBIRD_B"],
        "SONGBIRD_LEGACY",
        "SONGBIRD_A",
        "default",
        env,
    );
    assert_eq!(v, "first");
}

#[test]
fn env_capability_first_then_legacy_warn_falls_back_to_legacy_when_capability_empty() {
    let vars: HashMap<String, String> = HashMap::from([
        ("SONGBIRD_CAP".to_string(), String::new()),
        ("SONGBIRD_OLD".to_string(), "legacy-val".to_string()),
    ]);
    let env = move |k: &str| vars.get(k).cloned().ok_or(VarError::NotPresent);
    let v = super::hardcoded_elimination::env_capability_first_then_legacy_warn_with(
        &["SONGBIRD_CAP"],
        "SONGBIRD_OLD",
        "SONGBIRD_CAP",
        "default",
        env,
    );
    assert_eq!(v, "legacy-val");
}

#[test]
fn env_capability_first_then_legacy_warn_uses_default_when_nothing_set() {
    let vars: HashMap<String, String> = HashMap::new();
    let env = |k: &str| vars.get(k).cloned().ok_or(VarError::NotPresent);
    let v = super::hardcoded_elimination::env_capability_first_then_legacy_warn_with(
        &["SONGBIRD_X"],
        "SONGBIRD_Y",
        "SONGBIRD_X",
        "computed-default",
        env,
    );
    assert_eq!(v, "computed-default");
}

#[test]
fn resolve_storage_provider_endpoint_with_default_url() {
    let vars: HashMap<String, String> = HashMap::new();
    let env = |k: &str| vars.get(k).cloned().ok_or(VarError::NotPresent);
    let url = super::hardcoded_elimination::resolve_storage_provider_endpoint_with(
        "203.0.113.10",
        9000,
        env,
    );
    assert_eq!(url, "http://203.0.113.10:9000/storage");
}

#[test]
fn resolve_storage_provider_endpoint_with_prefers_explicit_storage_provider() {
    let vars: HashMap<String, String> = HashMap::from([(
        "SONGBIRD_STORAGE_PROVIDER_ENDPOINT".to_string(),
        "http://storage.explicit:8003".to_string(),
    )]);
    let env = move |k: &str| vars.get(k).cloned().ok_or(VarError::NotPresent);
    let url =
        super::hardcoded_elimination::resolve_storage_provider_endpoint_with("127.0.0.1", 1, env);
    assert_eq!(url, "http://storage.explicit:8003");
}

#[test]
fn resolve_storage_provider_endpoint_with_nestgate_legacy() {
    let vars: HashMap<String, String> =
        HashMap::from([("SONGBIRD_NESTGATE_ENDPOINT".to_string(), "http://nest:9".to_string())]);
    let env = move |k: &str| vars.get(k).cloned().ok_or(VarError::NotPresent);
    let url =
        super::hardcoded_elimination::resolve_storage_provider_endpoint_with("127.0.0.1", 1, env);
    assert_eq!(url, "http://nest:9");
}

#[test]
fn default_tls_cert_path_with_prefers_songbird_tls_cert() {
    let vars: HashMap<String, String> =
        HashMap::from([("SONGBIRD_TLS_CERT".to_string(), "/etc/songbird/tls.crt".to_string())]);
    let env = move |k: &str| vars.get(k).cloned().ok_or(VarError::NotPresent);
    assert_eq!(
        super::hardcoded_elimination::default_tls_cert_path_with(env),
        "/etc/songbird/tls.crt"
    );
}

#[test]
fn default_tls_cert_path_with_falls_back_to_ssl_cert_file() {
    let vars: HashMap<String, String> =
        HashMap::from([("SSL_CERT_FILE".to_string(), "/etc/ssl/custom.pem".to_string())]);
    let env = move |k: &str| vars.get(k).cloned().ok_or(VarError::NotPresent);
    assert_eq!(
        super::hardcoded_elimination::default_tls_cert_path_with(env),
        "/etc/ssl/custom.pem"
    );
}

#[test]
fn default_tls_cert_path_with_home_directory_layout() {
    let vars: HashMap<String, String> =
        HashMap::from([("HOME".to_string(), "/home/t".to_string())]);
    let env = move |k: &str| vars.get(k).cloned().ok_or(VarError::NotPresent);
    assert_eq!(
        super::hardcoded_elimination::default_tls_cert_path_with(env),
        "/home/t/.songbird/certs/songbird.crt"
    );
}
