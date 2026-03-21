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
    clippy::must_use_candidate,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive tests for canonical error handling
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
//! Tests the error context system and error handling patterns.

use songbird_canonical::errors::*;

// ============================================================================
// ERROR CONTEXT TESTS
// ============================================================================

#[test]
fn test_error_context_creation() {
    // Test basic ErrorContext construction
    let context = ErrorContext::new("Operation failed", "Database connection lost");

    assert_eq!(context.message(), "Operation failed");
    assert_eq!(context.context(), "Database connection lost");
    assert!(context.suggestions().is_empty());
}

#[test]
fn test_error_context_with_single_suggestion() {
    // Test error context with one recovery suggestion
    let context = ErrorContext::new("Connection timeout", "Network unreachable")
        .with_suggestion("Check network connectivity");

    assert_eq!(context.suggestions().len(), 1);
    assert_eq!(context.suggestions()[0], "Check network connectivity");
}

#[test]
fn test_error_context_with_multiple_suggestions() {
    // Test error context with multiple recovery suggestions
    let suggestions = vec!["Retry the operation", "Check service health", "Verify configuration"];

    let context = ErrorContext::new("Service unavailable", "HTTP 503 error")
        .with_suggestions(suggestions.clone());

    assert_eq!(context.suggestions().len(), 3);
    assert_eq!(context.suggestions()[0], "Retry the operation");
    assert_eq!(context.suggestions()[2], "Verify configuration");
}

#[test]
fn test_error_context_display() {
    // Test ErrorContext Display implementation
    let context =
        ErrorContext::new("Test error", "Test context").with_suggestion("Test suggestion");

    let display = format!("{context}");
    assert!(display.contains("Test error"));
    assert!(display.contains("Test context"));
    assert!(display.contains("Test suggestion"));
}

#[test]
fn test_error_context_clone() {
    // Test that ErrorContext can be cloned
    let context1 = ErrorContext::new("Error", "Context").with_suggestion("Suggestion");

    let context2 = context1.clone();
    assert_eq!(context1.message(), context2.message());
    assert_eq!(context1.context(), context2.context());
    assert_eq!(context1.suggestions().len(), context2.suggestions().len());
}

#[test]
fn test_error_context_empty_strings() {
    // Test error context with empty strings
    let context = ErrorContext::new("", "");

    assert_eq!(context.message(), "");
    assert_eq!(context.context(), "");
}

#[test]
fn test_error_context_very_long_message() {
    // Test with very long error message
    let long_message = "x".repeat(1000);
    let context = ErrorContext::new(long_message, "Context");

    assert_eq!(context.message().len(), 1000);
}

#[test]
fn test_error_context_special_characters() {
    // Test with special characters in message
    let context =
        ErrorContext::new("Error: !@#$%^&*()", "Context with UTF-8: 日本語, Ελληνικά, 🎵");

    assert!(context.message().contains("!@#$%"));
    assert!(context.context().contains("日本語"));
}

#[test]
fn test_error_context_chaining_suggestions() {
    // Test chaining multiple with_suggestion calls
    let context = ErrorContext::new("Error", "Context")
        .with_suggestion("First")
        .with_suggestion("Second")
        .with_suggestion("Third");

    assert_eq!(context.suggestions().len(), 3);
}

#[test]
fn test_error_context_display_no_suggestions() {
    // Test display format without suggestions
    let context = ErrorContext::new("Error message", "Error context");
    let display = format!("{context}");

    assert!(display.contains("Error message"));
    assert!(display.contains("Error context"));
    assert!(!display.contains("Suggestions:"));
}

#[test]
fn test_error_context_display_with_suggestions() {
    // Test display format with suggestions
    let context = ErrorContext::new("Error", "Context")
        .with_suggestions(vec!["Suggestion 1", "Suggestion 2"]);

    let display = format!("{context}");
    assert!(display.contains("Suggestions:"));
    assert!(display.contains("Suggestion 1"));
    assert!(display.contains("Suggestion 2"));
}

// ============================================================================
// HELPER FUNCTION TESTS
// ============================================================================

#[test]
fn test_success_result() {
    // Test success_result helper
    let value = success_result(42);
    assert_eq!(value, 42);
}

#[test]
fn test_success_result_string() {
    // Test success_result with String
    let value = success_result("test".to_string());
    assert_eq!(value, "test");
}

#[test]
fn test_success_result_vec() {
    // Test success_result with Vec
    let value = success_result(vec![1, 2, 3]);
    assert_eq!(value, vec![1, 2, 3]);
}

#[test]
fn test_unit_success() {
    // Test unit_success helper
    let result = unit_success();
    assert!(result.is_ok());
}

// ============================================================================
// ERROR CONTEXT USAGE PATTERNS
// ============================================================================

#[test]
fn test_error_context_for_network_error() {
    // Realistic usage: network error
    let context =
        ErrorContext::new("Failed to connect to service", "Connection refused on port 8080")
            .with_suggestion("Check if the service is running")
            .with_suggestion("Verify firewall rules")
            .with_suggestion("Check network connectivity");

    assert_eq!(context.suggestions().len(), 3);
    assert!(context.message().contains("connect"));
}

#[test]
fn test_error_context_for_authentication_error() {
    // Realistic usage: authentication error
    let context = ErrorContext::new("Authentication failed", "Invalid credentials provided")
        .with_suggestion("Verify username and password")
        .with_suggestion("Check if account is locked")
        .with_suggestion("Reset password if needed");

    assert!(context.message().contains("Authentication"));
}

#[test]
fn test_error_context_for_timeout_error() {
    // Realistic usage: timeout error
    let context = ErrorContext::new("Operation timed out", "Request exceeded 30 second limit")
        .with_suggestion("Increase timeout value")
        .with_suggestion("Check service responsiveness")
        .with_suggestion("Retry with exponential backoff");

    assert!(context.context().contains("30 second"));
}

#[test]
fn test_error_context_for_validation_error() {
    // Realistic usage: validation error
    let context = ErrorContext::new("Validation failed", "Required field 'email' is missing")
        .with_suggestion("Provide a valid email address")
        .with_suggestion("Check API documentation for required fields");

    assert!(context.context().contains("email"));
}

#[test]
fn test_error_context_for_resource_error() {
    // Realistic usage: resource exhaustion
    let context = ErrorContext::new("Resource limit exceeded", "Maximum 1000 connections reached")
        .with_suggestion("Close unused connections")
        .with_suggestion("Increase connection pool size")
        .with_suggestion("Scale up infrastructure");

    assert_eq!(context.suggestions().len(), 3);
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn test_error_context_with_newlines() {
    // Test multiline messages
    let context =
        ErrorContext::new("Error on line 1\nError on line 2", "Context line 1\nContext line 2");

    assert!(context.message().contains('\n'));
    assert!(context.context().contains('\n'));
}

#[test]
fn test_error_context_with_tabs() {
    // Test messages with tabs
    let context = ErrorContext::new("Error:\tdetails\there", "Context:\tmore\tinfo");

    assert!(context.message().contains('\t'));
}

#[test]
fn test_error_context_very_many_suggestions() {
    // Test with many suggestions
    let suggestions: Vec<String> = (0..100).map(|i| format!("Suggestion {i}")).collect();
    let context = ErrorContext::new("Error", "Context").with_suggestions(suggestions);

    assert_eq!(context.suggestions().len(), 100);
}

#[test]
fn test_error_context_empty_suggestion() {
    // Test with empty suggestion string
    let context = ErrorContext::new("Error", "Context").with_suggestion("");

    assert_eq!(context.suggestions().len(), 1);
    assert_eq!(context.suggestions()[0], "");
}

#[test]
fn test_error_context_unicode_suggestions() {
    // Test Unicode in suggestions
    let context = ErrorContext::new("Error", "Context")
        .with_suggestion("試してください (Please try)")
        .with_suggestion("Δοκιμάστε (Try it)")
        .with_suggestion("🔧 Fix the issue");

    assert_eq!(context.suggestions().len(), 3);
    assert!(context.suggestions()[2].contains("🔧"));
}

#[test]
fn test_error_context_builder_pattern() {
    // Test fluent builder pattern
    let context = ErrorContext::new("Error", "Context")
        .with_suggestion("First")
        .with_suggestions(vec!["Second", "Third"])
        .with_suggestion("Fourth");

    assert_eq!(context.suggestions().len(), 4);
    assert_eq!(context.suggestions()[0], "First");
    assert_eq!(context.suggestions()[3], "Fourth");
}
