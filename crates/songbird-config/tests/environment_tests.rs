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
    clippy::unnecessary_literal_unwrap,
    reason = "test assertions and harness ergonomics"
)]

//! Environment Configuration Tests
//!
//! Tests for environment variable handling, defaults, and validation.
//!
//! Note: `unwrap()` is acceptable in test code for clarity
//! Tests now use `ScopedEnv` for automatic cleanup - no more #[serial]!

mod test_utils;
use test_utils::ScopedEnv;

#[test]
fn test_env_var_parsing_valid_integer() {
    let _env = ScopedEnv::new().set("TEST_INT", "42");
    let parsed: Result<i32, _> =
        songbird_process_env::var("TEST_INT").expect("should parse valid input").parse();
    assert_eq!(parsed.expect("should parse valid input"), 42);
}

#[test]
fn test_env_var_parsing_invalid_integer() {
    let _env = ScopedEnv::new().set("TEST_INT", "not_a_number");
    let parsed: Result<i32, _> =
        songbird_process_env::var("TEST_INT").expect("should parse valid input").parse();
    assert!(parsed.is_err());
}

#[test]
fn test_env_var_parsing_empty_string() {
    let _env = ScopedEnv::new().set("TEST_VAR", "");
    let value = songbird_process_env::var("TEST_VAR").expect("test precondition");
    assert_eq!(value, "");
}

#[test]
fn test_env_var_parsing_whitespace() {
    let _env = ScopedEnv::new().set("TEST_VAR", "  spaces  ");
    let value = songbird_process_env::var("TEST_VAR").expect("test precondition");
    assert_eq!(value, "  spaces  ");
}

#[test]
fn test_env_var_parsing_boolean_true() {
    for true_val in ["true", "TRUE", "True", "1", "yes", "YES"] {
        let _env = ScopedEnv::new().set("TEST_BOOL", true_val);
        let value = songbird_process_env::var("TEST_BOOL").expect("test precondition");
        let is_truthy = matches!(value.to_lowercase().as_str(), "true" | "1" | "yes");
        assert!(is_truthy, "Failed for: {true_val}");
    }
}

#[test]
fn test_env_var_parsing_boolean_false() {
    for false_val in ["false", "FALSE", "False", "0", "no", "NO"] {
        let _env = ScopedEnv::new().set("TEST_BOOL", false_val);
        let value = songbird_process_env::var("TEST_BOOL").expect("test precondition");
        let is_falsy = matches!(value.to_lowercase().as_str(), "false" | "0" | "no");
        assert!(is_falsy, "Failed for: {false_val}");
    }
}

#[test]
fn test_env_var_missing_returns_error() {
    let _env = ScopedEnv::new().remove("NONEXISTENT_VAR");
    let result = songbird_process_env::var("NONEXISTENT_VAR");
    assert!(result.is_err());
}

#[test]
fn test_env_var_unicode_value() {
    let _env = ScopedEnv::new().set("TEST_UNICODE", "こんにちは世界");
    let value = songbird_process_env::var("TEST_UNICODE").expect("test precondition");
    assert_eq!(value, "こんにちは世界");
}

#[test]
fn test_env_var_special_characters() {
    let _env = ScopedEnv::new().set("TEST_SPECIAL", "!@#$%^&*()");
    let value = songbird_process_env::var("TEST_SPECIAL").expect("test precondition");
    assert_eq!(value, "!@#$%^&*()");
}

#[test]
fn test_env_var_newline_characters() {
    let _env = ScopedEnv::new().set("TEST_NEWLINE", "line1\nline2\nline3");
    let value = songbird_process_env::var("TEST_NEWLINE").expect("test precondition");
    assert!(value.contains('\n'));
}

#[test]
fn test_env_var_tab_characters() {
    let _env = ScopedEnv::new().set("TEST_TAB", "col1\tcol2\tcol3");
    let value = songbird_process_env::var("TEST_TAB").expect("test precondition");
    assert!(value.contains('\t'));
}

#[test]
fn test_env_var_very_long_value() {
    let long_value = "a".repeat(10000);
    let _env = ScopedEnv::new().set("TEST_LONG", &long_value);
    let value = songbird_process_env::var("TEST_LONG").expect("test precondition");
    assert_eq!(value.len(), 10000);
}

#[test]
fn test_env_var_json_value() {
    let json = r#"{"key":"value","num":42}"#;
    let _env = ScopedEnv::new().set("TEST_JSON", json);
    let value = songbird_process_env::var("TEST_JSON").expect("test precondition");
    assert!(value.contains("key"));
}

#[test]
fn test_env_var_url_value() {
    let _env = ScopedEnv::new().set("TEST_URL", "https://example.com:8080/path?query=value");
    let value = songbird_process_env::var("TEST_URL").expect("test precondition");
    assert!(value.starts_with("https://"));
}

#[test]
fn test_env_var_path_value() {
    let _env = ScopedEnv::new().set("TEST_PATH", "/usr/local/bin:/usr/bin:/bin");
    let value = songbird_process_env::var("TEST_PATH").expect("test precondition");
    assert!(value.contains('/'));
}

#[test]
fn test_env_var_comma_separated() {
    let _env = ScopedEnv::new().set("TEST_LIST", "item1,item2,item3");
    let value = songbird_process_env::var("TEST_LIST").expect("test precondition");
    assert_eq!(value.split(',').count(), 3);
}

#[test]
fn test_env_var_colon_separated() {
    let _env = ScopedEnv::new().set("TEST_PATH_LIST", "path1:path2:path3");
    let value = songbird_process_env::var("TEST_PATH_LIST").expect("test precondition");
    assert_eq!(value.split(':').count(), 3);
}

#[test]
fn test_env_var_equals_in_value() {
    let _env = ScopedEnv::new().set("TEST_EQUALS", "key=value");
    let value = songbird_process_env::var("TEST_EQUALS").expect("test precondition");
    assert!(value.contains('='));
}

#[test]
fn test_env_var_quotes_in_value() {
    let _env = ScopedEnv::new().set("TEST_QUOTES", r#""quoted value""#);
    let value = songbird_process_env::var("TEST_QUOTES").expect("test precondition");
    assert!(value.contains('"'));
}

#[test]
fn test_env_var_parsing_u64_max() {
    let value_str = u64::MAX.to_string();
    let _env = ScopedEnv::new().set("TEST_U64", &value_str);
    let parsed: Result<u64, _> =
        songbird_process_env::var("TEST_U64").expect("should parse valid input").parse();
    assert_eq!(parsed.expect("should parse valid input"), u64::MAX);
}

#[test]
fn test_env_var_parsing_negative_number() {
    let _env = ScopedEnv::new().set("TEST_NEG", "-42");
    let parsed: Result<i32, _> =
        songbird_process_env::var("TEST_NEG").expect("should parse valid input").parse();
    assert_eq!(parsed.expect("should parse valid input"), -42);
}

#[test]
fn test_env_var_parsing_float() {
    let _env = ScopedEnv::new().set("TEST_FLOAT", "3.14159");
    let parsed: Result<f64, _> =
        songbird_process_env::var("TEST_FLOAT").expect("should parse valid input").parse();
    assert!((parsed.expect("should parse valid input") - std::f64::consts::PI).abs() < 0.001);
}

#[test]
fn test_env_var_parsing_scientific_notation() {
    let _env = ScopedEnv::new().set("TEST_SCI", "1.23e10");
    let parsed: Result<f64, _> =
        songbird_process_env::var("TEST_SCI").expect("should parse valid input").parse();
    assert!(parsed.expect("should parse valid input") > 1e10);
}

#[test]
fn test_env_var_case_sensitivity() {
    let _env = ScopedEnv::new().set("TEST_CASE", "value");

    // Most systems are case-sensitive for env vars
    let lower = songbird_process_env::var("TEST_CASE");
    assert!(lower.is_ok());
}

#[test]
fn test_env_var_trimming_not_automatic() {
    let _env = ScopedEnv::new().set("TEST_TRIM", "  value  ");
    let value = songbird_process_env::var("TEST_TRIM").expect("test precondition");
    // Env vars don't auto-trim
    assert_eq!(value, "  value  ");

    // Manual trim works
    let trimmed = value.trim();
    assert_eq!(trimmed, "value");
}

#[test]
fn test_env_var_zero_value() {
    let _env = ScopedEnv::new().set("TEST_ZERO", "0");
    let parsed: Result<i32, _> =
        songbird_process_env::var("TEST_ZERO").expect("should parse valid input").parse();
    assert_eq!(parsed.expect("should parse valid input"), 0);
}

#[test]
fn test_env_var_default_with_or() {
    let _env = ScopedEnv::new().remove("MISSING_VAR");
    let value = songbird_process_env::var("MISSING_VAR").unwrap_or_else(|_| "default".to_string());
    assert_eq!(value, "default");
}

#[test]
fn test_env_var_default_with_or_else() {
    let _env = ScopedEnv::new().remove("MISSING_VAR");
    let value = songbird_process_env::var("MISSING_VAR").unwrap_or_else(|_| {
        // Complex default logic
        format!("computed_default_{}", 42)
    });
    assert_eq!(value, "computed_default_42");
}

#[test]
fn test_env_var_chaining_with_and_then() {
    let _env = ScopedEnv::new().set("TEST_CHAIN", "123");
    let result = songbird_process_env::var("TEST_CHAIN").ok().and_then(|s| s.parse::<i32>().ok());
    assert_eq!(result, Some(123));
}

#[test]
fn test_env_var_mapping_with_map() {
    let _env = ScopedEnv::new().set("TEST_MAP", "hello");
    let result = songbird_process_env::var("TEST_MAP").ok().map(|s| s.to_uppercase());
    assert_eq!(result, Some("HELLO".to_string()));
}

#[test]
fn test_env_var_set_and_read_immediately() {
    let _env = ScopedEnv::new().set("IMMEDIATE_VAR", "immediate_value");
    let value = songbird_process_env::var("IMMEDIATE_VAR").expect("test precondition");
    assert_eq!(value, "immediate_value");
}

#[test]
fn test_env_var_remove_and_verify() {
    // First, create a scoped var to set it
    {
        let _env = ScopedEnv::new().set("TEMP_VAR", "temp");
        assert!(songbird_process_env::var("TEMP_VAR").is_ok());
    } // _env drops here, TEMP_VAR is restored (removed)

    // Now verify it's gone
    assert!(songbird_process_env::var("TEMP_VAR").is_err());
}

#[test]
fn test_env_var_overwrite() {
    let _env = ScopedEnv::new().set("OVERWRITE_VAR", "first");
    assert_eq!(songbird_process_env::var("OVERWRITE_VAR").expect("test precondition"), "first");

    // Test overwrite within the same scope
    // Note: Overwrite within scope; ScopedEnv restores original on drop
    songbird_process_env::set_var("OVERWRITE_VAR", "second");
    assert_eq!(songbird_process_env::var("OVERWRITE_VAR").expect("test precondition"), "second");
    // When _env drops, it will restore to "first" (or None if didn't exist before)
}

#[test]
fn test_env_var_underscore_naming() {
    let _env = ScopedEnv::new().set("TEST_WITH_UNDERSCORES", "value");
    let value = songbird_process_env::var("TEST_WITH_UNDERSCORES").expect("test precondition");
    assert_eq!(value, "value");
}

#[test]
fn test_env_var_numeric_suffix() {
    let _env = ScopedEnv::new().set("TEST_VAR_123", "numeric_suffix");
    let value = songbird_process_env::var("TEST_VAR_123").expect("test precondition");
    assert_eq!(value, "numeric_suffix");
}

#[test]
fn test_env_var_parsing_hex_number() {
    let _env = ScopedEnv::new().set("TEST_HEX", "0xFF");
    let value = songbird_process_env::var("TEST_HEX").expect("test precondition");
    // Would need custom parsing for hex
    assert_eq!(value, "0xFF");
}

#[test]
fn test_env_var_multiline_value() {
    let multiline = "line1\nline2\nline3";
    let _env = ScopedEnv::new().set("TEST_MULTILINE", multiline);
    let value = songbird_process_env::var("TEST_MULTILINE").expect("test precondition");
    let lines: Vec<&str> = value.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn test_env_var_base64_value() {
    let base64 = "SGVsbG8gV29ybGQh"; // "Hello World!" in base64
    let _env = ScopedEnv::new().set("TEST_BASE64", base64);
    let value = songbird_process_env::var("TEST_BASE64").expect("test precondition");
    assert_eq!(value, base64);
}

#[test]
fn test_env_var_iso_date() {
    let _env = ScopedEnv::new().set("TEST_DATE", "2025-12-11T10:30:00Z");
    let value = songbird_process_env::var("TEST_DATE").expect("test precondition");
    assert!(value.contains('T'));
}

#[test]
fn test_env_var_duration_format() {
    let _env = ScopedEnv::new().set("TEST_DURATION", "1h30m45s");
    let value = songbird_process_env::var("TEST_DURATION").expect("test precondition");
    assert!(value.contains('h'));
}
