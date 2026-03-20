// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(
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

//! Sovereignty-aware adapter tests
//!
//! Note: This is a minimal test file as the comprehensive tests are in the source tree.
//! See: crates/songbird-universal/src/sovereignty/adapter_comprehensive_tests.rs (750+ lines)

// SongbirdResult not needed - using Result<(), SongbirdError> directly
#![expect(clippy::unwrap_used, reason = "test assertions and harness ergonomics")]
use songbird_universal::sovereignty::SovereigntyAwareAdapter;
// Basic struct creation test - using test mode with stub implementation
#[tokio::test]
async fn test_sovereignty_adapter_basic_creation() -> Result<(), Box<dyn std::error::Error>> {
    let _adapter = SovereigntyAwareAdapter::new().await?;
    // Basic smoke test - adapter creation works
    Ok(())
}
// Note: This test file is intentionally minimal because the comprehensive test suite
// (~750 lines) is maintained in the source tree for better organization.
// Extended cases live in `adapter_comprehensive_tests.rs` (~750 lines) instead of this file
// These tests cover:
// - Sovereignty-aware routing
// - Federation coordination
// - Network effects optimization
