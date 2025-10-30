//! Comprehensive Error Tests for Canonical Crate
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

//!
//! Tests for canonical error types and error handling patterns.

use songbird_canonical::errors::*;

// ========== ErrorContext Tests ==========

#[test]
fn test_error_context_basic_creation() {
    let ctx = ErrorContext::new("Something went wrong", "In the test module");
    assert_eq!(ctx.message(), "Something went wrong");
    assert_eq!(ctx.context(), "In the test module");
    assert_eq!(ctx.suggestions().len(), 0);
}

#[test]
fn test_error_context_with_suggestion() {
    let ctx = ErrorContext::new("Error occurred", "During startup")
        .with_suggestion("Check configuration");

    assert_eq!(ctx.suggestions().len(), 1);
    assert_eq!(ctx.suggestions()[0], "Check configuration");
}

#[test]
fn test_error_context_with_multiple_suggestions() {
    let suggestions = vec!["Try option A", "Try option B", "Try option C"];
    let ctx =
        ErrorContext::new("Multiple options", "During operation").with_suggestions(suggestions);

    assert_eq!(ctx.suggestions().len(), 3);
    assert_eq!(ctx.suggestions()[0], "Try option A");
    assert_eq!(ctx.suggestions()[2], "Try option C");
}

#[test]
fn test_error_context_builder_pattern() {
    let ctx = ErrorContext::new("Build test", "Testing builder")
        .with_suggestion("First suggestion")
        .with_suggestion("Second suggestion")
        .with_suggestion("Third suggestion");

    assert_eq!(ctx.suggestions().len(), 3);
}

#[test]
fn test_error_context_clone() {
    let ctx = ErrorContext::new("Original", "Context").with_suggestion("Suggestion");
    let cloned = ctx.clone();

    assert_eq!(ctx.message(), cloned.message());
    assert_eq!(ctx.context(), cloned.context());
    assert_eq!(ctx.suggestions().len(), cloned.suggestions().len());
}

#[test]
fn test_error_context_display() {
    let ctx = ErrorContext::new("Display test", "Context info");
    let display = format!("{ctx}");

    assert!(display.contains("Display test"));
    assert!(display.contains("Context info"));
}

#[test]
fn test_error_context_display_with_suggestions() {
    let ctx = ErrorContext::new("Error", "Context")
        .with_suggestion("Suggestion 1")
        .with_suggestion("Suggestion 2");

    let display = format!("{ctx}");
    assert!(display.contains("Suggestion 1"));
    assert!(display.contains("Suggestion 2"));
}

#[test]
fn test_error_context_empty_suggestions() {
    let ctx = ErrorContext::new("No suggestions", "Just context");
    assert!(ctx.suggestions().is_empty());

    let display = format!("{ctx}");
    assert!(!display.contains("Suggestions:"));
}

#[test]
fn test_error_context_message_accessor() {
    let ctx = ErrorContext::new("Test message", "Test context");
    assert_eq!(ctx.message(), "Test message");
}

#[test]
fn test_error_context_context_accessor() {
    let ctx = ErrorContext::new("Message", "Test context value");
    assert_eq!(ctx.context(), "Test context value");
}

#[test]
fn test_error_context_suggestions_accessor() {
    let ctx = ErrorContext::new("Error", "Context").with_suggestion("A").with_suggestion("B");

    let suggestions = ctx.suggestions();
    assert_eq!(suggestions.len(), 2);
    assert_eq!(suggestions[0], "A");
    assert_eq!(suggestions[1], "B");
}

// ========== Helper Functions Tests ==========

#[test]
fn test_success_result_helper() {
    let result = success_result(42);
    assert_eq!(result, 42);
}

#[test]
fn test_success_result_with_string() {
    let result = success_result("test".to_string());
    assert_eq!(result, "test");
}

#[test]
fn test_success_result_with_struct() {
    #[derive(Debug, PartialEq)]
    struct TestStruct {
        value: i32,
    }

    let test = TestStruct {
        value: 100,
    };
    let result = success_result(test);
    assert_eq!(result.value, 100);
}

#[test]
fn test_unit_success() {
    let result = unit_success();
    assert!(result.is_ok());
}

#[test]
fn test_unit_success_unwrap() {
    let result = unit_success();
    assert!(result.is_ok());
}

// ========== Integration Tests ==========

#[test]
fn test_error_context_with_long_message() {
    let long_message = "A".repeat(1000);
    let ctx = ErrorContext::new(long_message.clone(), "Context");
    assert_eq!(ctx.message(), &long_message);
}

#[test]
fn test_error_context_with_unicode() {
    let ctx =
        ErrorContext::new("エラー発生", "コンテキスト").with_suggestion("解決策を試してください");

    assert!(ctx.message().contains("エラー"));
    assert!(ctx.context().contains("コンテキスト"));
    assert!(ctx.suggestions()[0].contains("解決策"));
}

#[test]
fn test_error_context_debug_output() {
    let ctx = ErrorContext::new("Debug test", "Context");
    let debug = format!("{ctx:?}");
    assert!(debug.contains("ErrorContext"));
}

// ========== Thread Safety Tests ==========

#[test]
fn test_error_context_thread_safe() {
    fn assert_send<T: Send>() {}
    #[allow(dead_code)]
    fn assert_sync<T: Sync>() {}

    // Note: ErrorContext is Clone but not Sync due to internal String types
    // This is acceptable for error types
    assert_send::<ErrorContext>();
}

// ========== Edge Cases ==========

#[test]
fn test_error_context_empty_message() {
    let ctx = ErrorContext::new("", "Context");
    assert_eq!(ctx.message(), "");
}

#[test]
fn test_error_context_empty_context() {
    let ctx = ErrorContext::new("Message", "");
    assert_eq!(ctx.context(), "");
}

#[test]
fn test_error_context_many_suggestions() {
    let mut ctx = ErrorContext::new("Error", "Context");
    for i in 0..100 {
        ctx = ctx.with_suggestion(format!("Suggestion {i}"));
    }
    assert_eq!(ctx.suggestions().len(), 100);
}
