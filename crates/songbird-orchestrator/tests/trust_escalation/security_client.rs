// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

use super::common::lock_env;
use songbird_orchestrator::trust::escalation::SecurityTrustClient;

// ═══════════════════════════════════════════════════════════════════════════
// SecurityTrustClient tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_security_trust_client_default_no_endpoint() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_SECURITY_PROVIDER");
    songbird_process_env::remove_var("SECURITY_ENDPOINT");
    songbird_process_env::remove_var("BEARDOG_URL");
    let _client = SecurityTrustClient::default();
}

#[test]
fn test_security_trust_client_new_with_songbird_security_provider_env() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_SECURITY_PROVIDER", "http://localhost:9090");
    let _client = SecurityTrustClient::new();
    songbird_process_env::remove_var("SONGBIRD_SECURITY_PROVIDER");
}

#[test]
fn test_security_trust_client_new_with_security_endpoint() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_SECURITY_PROVIDER");
    songbird_process_env::set_var("SECURITY_ENDPOINT", "http://localhost:9091");
    let _client = SecurityTrustClient::new();
    songbird_process_env::remove_var("SECURITY_ENDPOINT");
}

#[test]
fn test_security_trust_client_new_with_deprecated_url() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_SECURITY_PROVIDER");
    songbird_process_env::remove_var("SECURITY_ENDPOINT");
    songbird_process_env::set_var("BEARDOG_URL", "http://localhost:9092");
    let _client = SecurityTrustClient::new();
    songbird_process_env::remove_var("BEARDOG_URL");
}
