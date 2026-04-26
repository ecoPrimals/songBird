// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for the 7-stage startup orchestration pipeline.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::startup_orchestration::*;
use crate::app::core::SongbirdOrchestrator;
use anyhow::Result;
use songbird_types::config::CanonicalSongbirdConfig;

#[test]
fn startup_pipeline_stage_order_is_sequential_and_includes_sub_stages() {
    assert_eq!(STARTUP_PIPELINE_STAGE_ORDER.len(), 9);
    assert_eq!(
        STARTUP_PIPELINE_STAGE_ORDER,
        &[
            "stage_1_provision_security",
            "stage_2_start_servers",
            "stage_2b_igd_auto_configure",
            "stage_2c_socket_auto_discovery",
            "stage_3_register_self",
            "stage_4_start_discovery",
            "stage_5_start_federation",
            "stage_6_background_tasks",
            "stage_7_verify_connectivity",
        ]
    );
    let pos_2 =
        STARTUP_PIPELINE_STAGE_ORDER.iter().position(|s| *s == "stage_2_start_servers").unwrap();
    let pos_2b = STARTUP_PIPELINE_STAGE_ORDER
        .iter()
        .position(|s| *s == "stage_2b_igd_auto_configure")
        .unwrap();
    let pos_2c = STARTUP_PIPELINE_STAGE_ORDER
        .iter()
        .position(|s| *s == "stage_2c_socket_auto_discovery")
        .unwrap();
    let pos_3 =
        STARTUP_PIPELINE_STAGE_ORDER.iter().position(|s| *s == "stage_3_register_self").unwrap();
    assert!(pos_2 < pos_2b && pos_2b < pos_2c && pos_2c < pos_3);
}

#[test]
fn stage_3_federation_capabilities_match_expected_set() {
    assert_eq!(
        STAGE_3_FEDERATION_SELF_CAPABILITIES,
        &["orchestrator", "secure_http", "http.request", "tls.1.3"]
    );
}

#[test]
fn stage_4_discovery_capabilities_match_expected_set() {
    assert_eq!(
        STAGE_4_DISCOVERY_CAPABILITIES,
        &[
            "orchestration",
            "federation",
            "secure_http",
            "http.request",
            "http.get",
            "http.post",
            "tls.1.3",
        ]
    );
}

#[test]
fn trust_cleanup_interval_is_five_minutes() {
    assert_eq!(TRUST_CLEANUP_INTERVAL_SECS, 300);
}

#[test]
fn socket_rescan_interval_is_thirty_seconds() {
    assert_eq!(SOCKET_RESCAN_INTERVAL_SECS, 30);
}

#[test]
fn http_bind_socket_addr_accepts_loopback_defaults() {
    let addr = http_bind_socket_addr("127.0.0.1", 8080).unwrap();
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_eq!(addr.port(), 8080);
}

#[test]
fn http_bind_socket_addr_rejects_invalid_socket_form() {
    let err = http_bind_socket_addr("not-a-valid-socket-addr", 1).unwrap_err();
    assert!(err.to_string().contains("Invalid bind address"), "unexpected message: {err}");
}

#[test]
fn igd_enabled_from_env_value_parses_opt_in_cases() {
    assert!(igd_enabled_from_env_value("1"));
    assert!(igd_enabled_from_env_value("true"));
    assert!(igd_enabled_from_env_value("TRUE"));
    assert!(igd_enabled_from_env_value("TrUe"));
    assert!(!igd_enabled_from_env_value("0"));
    assert!(!igd_enabled_from_env_value("false"));
    assert!(!igd_enabled_from_env_value(""));
    assert!(!igd_enabled_from_env_value("yes"));
}

#[test]
fn http_bind_socket_addr_accepts_ipv4_wildcard() {
    // `host:port` concatenation matches IPv4; IPv6 literals need bracket form and are not supported here.
    let addr = http_bind_socket_addr("0.0.0.0", 9090).unwrap();
    assert_eq!(addr.ip().to_string(), "0.0.0.0");
    assert_eq!(addr.port(), 9090);
}

#[test]
fn default_config_network_matches_stage_2_bind_inputs() {
    let cfg = CanonicalSongbirdConfig::default();
    assert_eq!(cfg.network.bind_host, "127.0.0.1");
    assert_eq!(cfg.network.base_port, 8080);
    let addr = http_bind_socket_addr(&cfg.network.bind_host, cfg.network.base_port).unwrap();
    assert_eq!(addr.to_string(), "127.0.0.1:8080");
}

#[test]
fn http_bind_failure_propagates_as_error() {
    let res = http_bind_socket_addr("%%%invalid%%%", 80);
    assert!(res.is_err());
}

#[allow(dead_code, reason = "compile-time Send bound assertion — never called at runtime")]
fn _assert_start_returns_send_future(
    orch: &mut SongbirdOrchestrator,
) -> impl std::future::Future<Output = Result<()>> + Send {
    StartupOrchestrator::new(orch).start()
}

#[allow(dead_code, reason = "compile-time API usability assertion — never called at runtime")]
fn _assert_new_accepts_mutable_orchestrator(orch: &mut SongbirdOrchestrator) {
    let _ = StartupOrchestrator::new(orch);
}
