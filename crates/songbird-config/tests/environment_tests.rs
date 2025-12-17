//! Environment Configuration Tests
//!
//! Tests for environment variable handling, defaults, and validation.
//!
//! Note: unwrap() is acceptable in test code for clarity

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::unnecessary_literal_unwrap)]

use std::env;

#[test]
fn test_env_var_parsing_valid_integer() {
    env::set_var("TEST_INT", "42");
    let parsed: Result<i32, _> = env::var("TEST_INT").expect("should parse valid input").parse();
    assert_eq!(parsed.expect("should parse valid input"), 42);
    env::remove_var("TEST_INT");
}

#[test]
fn test_env_var_parsing_invalid_integer() {
    env::set_var("TEST_INT", "not_a_number");
    let parsed: Result<i32, _> = env::var("TEST_INT").expect("should parse valid input").parse();
    assert!(parsed.is_err());
    env::remove_var("TEST_INT");
}

#[test]
fn test_env_var_parsing_empty_string() {
    env::set_var("TEST_VAR", "");
    let value = env::var("TEST_VAR").expect("test precondition");
    assert_eq!(value, "");
    env::remove_var("TEST_VAR");
}

#[test]
fn test_env_var_parsing_whitespace() {
    env::set_var("TEST_VAR", "  spaces  ");
    let value = env::var("TEST_VAR").expect("test precondition");
    assert_eq!(value, "  spaces  ");
    env::remove_var("TEST_VAR");
}

#[test]
fn test_env_var_parsing_boolean_true() {
    for true_val in ["true", "TRUE", "True", "1", "yes", "YES"] {
        env::set_var("TEST_BOOL", true_val);
        let value = env::var("TEST_BOOL").expect("test precondition");
        let is_truthy = matches!(value.to_lowercase().as_str(), "true" | "1" | "yes");
        assert!(is_truthy, "Failed for: {}", true_val);
        env::remove_var("TEST_BOOL");
    }
}

#[test]
fn test_env_var_parsing_boolean_false() {
    for false_val in ["false", "FALSE", "False", "0", "no", "NO"] {
        env::set_var("TEST_BOOL", false_val);
        let value = env::var("TEST_BOOL").expect("test precondition");
        let is_falsy = matches!(value.to_lowercase().as_str(), "false" | "0" | "no");
        assert!(is_falsy, "Failed for: {}", false_val);
        env::remove_var("TEST_BOOL");
    }
}

#[test]
fn test_env_var_missing_returns_error() {
    env::remove_var("NONEXISTENT_VAR");
    let result = env::var("NONEXISTENT_VAR");
    assert!(result.is_err());
}

#[test]
fn test_env_var_unicode_value() {
    env::set_var("TEST_UNICODE", "こんにちは世界");
    let value = env::var("TEST_UNICODE").expect("test precondition");
    assert_eq!(value, "こんにちは世界");
    env::remove_var("TEST_UNICODE");
}

#[test]
fn test_env_var_special_characters() {
    env::set_var("TEST_SPECIAL", "!@#$%^&*()");
    let value = env::var("TEST_SPECIAL").expect("test precondition");
    assert_eq!(value, "!@#$%^&*()");
    env::remove_var("TEST_SPECIAL");
}

#[test]
fn test_env_var_newline_characters() {
    env::set_var("TEST_NEWLINE", "line1\nline2\nline3");
    let value = env::var("TEST_NEWLINE").expect("test precondition");
    assert!(value.contains('\n'));
    env::remove_var("TEST_NEWLINE");
}

#[test]
fn test_env_var_tab_characters() {
    env::set_var("TEST_TAB", "col1\tcol2\tcol3");
    let value = env::var("TEST_TAB").expect("test precondition");
    assert!(value.contains('\t'));
    env::remove_var("TEST_TAB");
}

#[test]
fn test_env_var_very_long_value() {
    let long_value = "a".repeat(10000);
    env::set_var("TEST_LONG", &long_value);
    let value = env::var("TEST_LONG").expect("test precondition");
    assert_eq!(value.len(), 10000);
    env::remove_var("TEST_LONG");
}

#[test]
fn test_env_var_json_value() {
    let json = r#"{"key":"value","num":42}"#;
    env::set_var("TEST_JSON", json);
    let value = env::var("TEST_JSON").expect("test precondition");
    assert!(value.contains("key"));
    env::remove_var("TEST_JSON");
}

#[test]
fn test_env_var_url_value() {
    env::set_var("TEST_URL", "https://example.com:8080/path?query=value");
    let value = env::var("TEST_URL").expect("test precondition");
    assert!(value.starts_with("https://"));
    env::remove_var("TEST_URL");
}

#[test]
fn test_env_var_path_value() {
    env::set_var("TEST_PATH", "/usr/local/bin:/usr/bin:/bin");
    let value = env::var("TEST_PATH").expect("test precondition");
    assert!(value.contains('/'));
    env::remove_var("TEST_PATH");
}

#[test]
fn test_env_var_comma_separated() {
    env::set_var("TEST_LIST", "item1,item2,item3");
    let value = env::var("TEST_LIST").expect("test precondition");
    assert_eq!(value.split(',').count(), 3);
    env::remove_var("TEST_LIST");
}

#[test]
fn test_env_var_colon_separated() {
    env::set_var("TEST_PATH_LIST", "path1:path2:path3");
    let value = env::var("TEST_PATH_LIST").expect("test precondition");
    assert_eq!(value.split(':').count(), 3);
    env::remove_var("TEST_PATH_LIST");
}

#[test]
fn test_env_var_equals_in_value() {
    env::set_var("TEST_EQUALS", "key=value");
    let value = env::var("TEST_EQUALS").expect("test precondition");
    assert!(value.contains('='));
    env::remove_var("TEST_EQUALS");
}

#[test]
fn test_env_var_quotes_in_value() {
    env::set_var("TEST_QUOTES", r#""quoted value""#);
    let value = env::var("TEST_QUOTES").expect("test precondition");
    assert!(value.contains('"'));
    env::remove_var("TEST_QUOTES");
}

#[test]
fn test_env_var_parsing_u64_max() {
    env::set_var("TEST_U64", u64::MAX.to_string());
    let parsed: Result<u64, _> = env::var("TEST_U64").expect("should parse valid input").parse();
    assert_eq!(parsed.expect("should parse valid input"), u64::MAX);
    env::remove_var("TEST_U64");
}

#[test]
fn test_env_var_parsing_negative_number() {
    env::set_var("TEST_NEG", "-42");
    let parsed: Result<i32, _> = env::var("TEST_NEG").expect("should parse valid input").parse();
    assert_eq!(parsed.expect("should parse valid input"), -42);
    env::remove_var("TEST_NEG");
}

#[test]
fn test_env_var_parsing_float() {
    env::set_var("TEST_FLOAT", "3.14159");
    let parsed: Result<f64, _> = env::var("TEST_FLOAT").expect("should parse valid input").parse();
    assert!((parsed.expect("should parse valid input") - std::f64::consts::PI).abs() < 0.001);
    env::remove_var("TEST_FLOAT");
}

#[test]
fn test_env_var_parsing_scientific_notation() {
    env::set_var("TEST_SCI", "1.23e10");
    let parsed: Result<f64, _> = env::var("TEST_SCI").expect("should parse valid input").parse();
    assert!(parsed.expect("should parse valid input") > 1e10);
    env::remove_var("TEST_SCI");
}

#[test]
fn test_env_var_case_sensitivity() {
    env::set_var("TEST_CASE", "value");

    // Most systems are case-sensitive for env vars
    let lower = env::var("TEST_CASE");
    assert!(lower.is_ok());

    env::remove_var("TEST_CASE");
}

#[test]
fn test_env_var_trimming_not_automatic() {
    env::set_var("TEST_TRIM", "  value  ");
    let value = env::var("TEST_TRIM").expect("test precondition");
    // Env vars don't auto-trim
    assert_eq!(value, "  value  ");

    // Manual trim works
    let trimmed = value.trim();
    assert_eq!(trimmed, "value");
    env::remove_var("TEST_TRIM");
}

#[test]
fn test_env_var_zero_value() {
    env::set_var("TEST_ZERO", "0");
    let parsed: Result<i32, _> = env::var("TEST_ZERO").expect("should parse valid input").parse();
    assert_eq!(parsed.expect("should parse valid input"), 0);
    env::remove_var("TEST_ZERO");
}

#[test]
fn test_env_var_default_with_or() {
    env::remove_var("MISSING_VAR");
    let value = env::var("MISSING_VAR").unwrap_or_else(|_| "default".to_string());
    assert_eq!(value, "default");
}

#[test]
fn test_env_var_default_with_or_else() {
    env::remove_var("MISSING_VAR");
    let value = env::var("MISSING_VAR").unwrap_or_else(|_| {
        // Complex default logic
        format!("computed_default_{}", 42)
    });
    assert_eq!(value, "computed_default_42");
}

#[test]
fn test_env_var_chaining_with_and_then() {
    env::set_var("TEST_CHAIN", "123");
    let result = env::var("TEST_CHAIN").ok().and_then(|s| s.parse::<i32>().ok());
    assert_eq!(result, Some(123));
    env::remove_var("TEST_CHAIN");
}

#[test]
fn test_env_var_mapping_with_map() {
    env::set_var("TEST_MAP", "hello");
    let result = env::var("TEST_MAP").ok().map(|s| s.to_uppercase());
    assert_eq!(result, Some("HELLO".to_string()));
    env::remove_var("TEST_MAP");
}

#[test]
fn test_env_var_set_and_read_immediately() {
    env::set_var("IMMEDIATE_VAR", "immediate_value");
    let value = env::var("IMMEDIATE_VAR").expect("test precondition");
    assert_eq!(value, "immediate_value");
    env::remove_var("IMMEDIATE_VAR");
}

#[test]
fn test_env_var_remove_and_verify() {
    env::set_var("TEMP_VAR", "temp");
    assert!(env::var("TEMP_VAR").is_ok());

    env::remove_var("TEMP_VAR");
    assert!(env::var("TEMP_VAR").is_err());
}

#[test]
fn test_env_var_overwrite() {
    env::set_var("OVERWRITE_VAR", "first");
    assert_eq!(env::var("OVERWRITE_VAR").expect("test precondition"), "first");

    env::set_var("OVERWRITE_VAR", "second");
    assert_eq!(env::var("OVERWRITE_VAR").expect("test precondition"), "second");

    env::remove_var("OVERWRITE_VAR");
}

#[test]
fn test_env_var_underscore_naming() {
    env::set_var("TEST_WITH_UNDERSCORES", "value");
    let value = env::var("TEST_WITH_UNDERSCORES").expect("test precondition");
    assert_eq!(value, "value");
    env::remove_var("TEST_WITH_UNDERSCORES");
}

#[test]
fn test_env_var_numeric_suffix() {
    env::set_var("TEST_VAR_123", "numeric_suffix");
    let value = env::var("TEST_VAR_123").expect("test precondition");
    assert_eq!(value, "numeric_suffix");
    env::remove_var("TEST_VAR_123");
}

#[test]
fn test_env_var_parsing_hex_number() {
    env::set_var("TEST_HEX", "0xFF");
    let value = env::var("TEST_HEX").expect("test precondition");
    // Would need custom parsing for hex
    assert_eq!(value, "0xFF");
    env::remove_var("TEST_HEX");
}

#[test]
fn test_env_var_multiline_value() {
    let multiline = "line1\nline2\nline3";
    env::set_var("TEST_MULTILINE", multiline);
    let value = env::var("TEST_MULTILINE").expect("test precondition");
    let lines: Vec<&str> = value.lines().collect();
    assert_eq!(lines.len(), 3);
    env::remove_var("TEST_MULTILINE");
}

#[test]
fn test_env_var_base64_value() {
    let base64 = "SGVsbG8gV29ybGQh"; // "Hello World!" in base64
    env::set_var("TEST_BASE64", base64);
    let value = env::var("TEST_BASE64").expect("test precondition");
    assert_eq!(value, base64);
    env::remove_var("TEST_BASE64");
}

#[test]
fn test_env_var_iso_date() {
    env::set_var("TEST_DATE", "2025-12-11T10:30:00Z");
    let value = env::var("TEST_DATE").expect("test precondition");
    assert!(value.contains('T'));
    env::remove_var("TEST_DATE");
}

#[test]
fn test_env_var_duration_format() {
    env::set_var("TEST_DURATION", "1h30m45s");
    let value = env::var("TEST_DURATION").expect("test precondition");
    assert!(value.contains('h'));
    env::remove_var("TEST_DURATION");
}
