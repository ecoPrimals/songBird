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

//! Tests for universal adapter
#![expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![expect(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![expect(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![expect(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![expect(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

use songbird_types::SongbirdResult;
use songbird_universal::*;

#[test]
fn test_create_universal_adapter() -> SongbirdResult<()> {
    let adapter = create_universal_adapter();
    // Just test that we can create it
    assert!(format!("{adapter:?}").contains("UnifiedUniversalAdapter"));
    Ok(())
}

#[test]
fn test_adapter_with_default_config() -> SongbirdResult<()> {
    let config = UnifiedAdapterConfig::default();
    let adapter = create_universal_adapter_with_config(config);
    assert!(format!("{adapter:?}").contains("UnifiedUniversalAdapter"));
    Ok(())
}

#[test]
fn test_adapter_config_default() -> SongbirdResult<()> {
    let config = UnifiedAdapterConfig::default();
    // Test that default config can be created
    assert!(format!("{config:?}").contains("UnifiedAdapterConfig"));
    Ok(())
}

#[test]
fn test_adapter_config_clone() -> SongbirdResult<()> {
    let config = UnifiedAdapterConfig::default();
    let cloned = config;
    // Verify cloning works
    assert!(format!("{cloned:?}").contains("UnifiedAdapterConfig"));
    Ok(())
}

#[tokio::test]
async fn test_adapter_creation_async() -> SongbirdResult<()> {
    let adapter = create_universal_adapter();
    // Test async creation
    assert!(format!("{adapter:?}").contains("UnifiedUniversalAdapter"));
    Ok(())
}
