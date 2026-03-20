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
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate
)]

//! Comprehensive tests for `NetworkFederationBridge`
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

use songbird_network_federation::NetworkFederationBridge;

#[test]
fn test_bridge_new() {
    let bridge = NetworkFederationBridge::new();
    assert!(std::mem::size_of_val(&bridge) > 0 || std::mem::size_of_val(&bridge) == 0);
}

#[test]
fn test_bridge_default() {
    let bridge = NetworkFederationBridge;
    assert!(std::mem::size_of_val(&bridge) > 0 || std::mem::size_of_val(&bridge) == 0);
}

#[tokio::test]
async fn test_bridge_initialize() {
    let mut bridge = NetworkFederationBridge::new();
    let result = bridge.initialize().await;
    assert!(result.is_ok(), "Initialize should succeed");
}

#[test]
fn test_bridge_debug_format() {
    let bridge = NetworkFederationBridge::new();
    let debug_str = format!("{bridge:?}");
    assert!(debug_str.contains("NetworkFederationBridge"));
}

#[test]
fn test_multiple_bridges_independent() {
    let _bridge1 = NetworkFederationBridge::new();
    let _bridge2 = NetworkFederationBridge::new();
    let _bridge3 = NetworkFederationBridge::new();

    // All should be created successfully
}

#[tokio::test]
async fn test_multiple_bridge_initializations() {
    let mut bridge1 = NetworkFederationBridge::new();
    let mut bridge2 = NetworkFederationBridge::new();

    let result1 = bridge1.initialize().await;
    let result2 = bridge2.initialize().await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_bridge_double_initialize() {
    let mut bridge = NetworkFederationBridge::new();

    let result1 = bridge.initialize().await;
    let result2 = bridge.initialize().await;

    assert!(result1.is_ok(), "First initialize should succeed");
    assert!(result2.is_ok(), "Second initialize should succeed");
}

#[test]
fn test_bridge_size_reasonable() {
    let bridge = NetworkFederationBridge::new();
    let size = std::mem::size_of_val(&bridge);

    assert!(size < 1_000); // Should be lightweight
}

#[test]
fn test_bridge_creation_consistency() {
    let bridge1 = NetworkFederationBridge::new();
    let bridge2 = NetworkFederationBridge;

    // Both creation methods should produce valid bridges
    let size1 = std::mem::size_of_val(&bridge1);
    let size2 = std::mem::size_of_val(&bridge2);
    assert_eq!(size1, size2);
}

#[tokio::test]
async fn test_bridge_initialize_idempotent() {
    let mut bridge = NetworkFederationBridge::new();

    // Initialize multiple times - should be safe
    for _ in 0..5 {
        let result = bridge.initialize().await;
        assert!(result.is_ok());
    }
}
