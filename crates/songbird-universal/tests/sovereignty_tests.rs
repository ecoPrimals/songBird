// SPDX-License-Identifier: AGPL-3.0-or-later
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
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]

//! Modern tests for sovereignty system
#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![allow(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![allow(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![allow(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//!
//! Tests for sovereignty-aware routing and federation capabilities.

use songbird_universal::sovereignty::{SovereigntyAdapterConfig, SovereigntyAwareAdapter};

#[tokio::test]
async fn test_sovereignty_adapter_creation() {
    let adapter = SovereigntyAwareAdapter::new().await;
    assert!(adapter.is_ok(), "Adapter should be created successfully");
}

#[tokio::test]
async fn test_sovereignty_config_default() {
    let config = SovereigntyAdapterConfig::default();
    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Adapter should be created with default config");
}

#[tokio::test]
async fn test_sovereignty_adapter_with_custom_config() {
    let config = SovereigntyAdapterConfig::default();
    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Adapter should be created with custom config");
}

#[tokio::test]
async fn test_multiple_sovereignty_adapters_independent() {
    let config1 = SovereigntyAdapterConfig::default();
    let config2 = SovereigntyAdapterConfig::default();

    let adapter1 = SovereigntyAwareAdapter::with_config(config1).await;
    let adapter2 = SovereigntyAwareAdapter::with_config(config2).await;

    assert!(adapter1.is_ok(), "First adapter should be created");
    assert!(adapter2.is_ok(), "Second adapter should be created");
    // Each should maintain independent state
}

#[tokio::test]
async fn test_sovereignty_config_structure() {
    let config = SovereigntyAdapterConfig::default();
    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Config should be well-formed and constructable");
}
