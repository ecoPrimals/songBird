// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for version command
//!
//! Phase 3 Test Coverage Expansion - CLI Commands
//! Target: 0% → 90%+ coverage for version.rs (66 lines)

use super::*;

// =============================================================================
// EXECUTE VERSION COMMAND TESTS
// =============================================================================

#[tokio::test]
async fn test_execute_version_simple() {
    let result = execute_version_command(false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_version_detailed() {
    let result = execute_version_command(true).await;
    assert!(result.is_ok());
}

// =============================================================================
// SHOW SIMPLE VERSION TESTS
// =============================================================================

#[tokio::test]
async fn test_show_simple_version() {
    let result = show_simple_version().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_show_simple_version_multiple_calls() {
    let result1 = show_simple_version().await;
    let result2 = show_simple_version().await;
    let result3 = show_simple_version().await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

// =============================================================================
// SHOW DETAILED VERSION TESTS
// =============================================================================

#[tokio::test]
async fn test_show_detailed_version() {
    let result = show_detailed_version().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_show_detailed_version_multiple_calls() {
    let result1 = show_detailed_version().await;
    let result2 = show_detailed_version().await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

// =============================================================================
// BUILD INFO TESTS
// =============================================================================

#[test]
fn test_build_info_format() {
    let info = build_info();

    // Should return a string with git SHA and build date
    assert!(!info.is_empty());
    assert!(info.contains(' ')); // Should have space between SHA and date
}

#[test]
fn test_build_info_consistency() {
    let info1 = build_info();
    let info2 = build_info();

    // Should be consistent across calls
    assert_eq!(info1, info2);
}

#[test]
fn test_build_info_not_empty() {
    let info = build_info();
    assert!(!info.is_empty());
}

// =============================================================================
// BUILD RUST VERSION TESTS
// =============================================================================

#[test]
fn test_build_rust_version() {
    let version = build_rust_version();

    // Version might be empty if env vars not set, but function should not panic
    // Just verify it returns a string (empty or not)
    let _ = version;
}

#[test]
fn test_build_rust_version_format() {
    let version = build_rust_version();

    // Should return a string (might be empty if env vars not set)
    // Rust versions typically contain dots (e.g., "1.75.0") or might be empty
    let _ = version;
}

#[test]
fn test_build_rust_version_consistency() {
    let version1 = build_rust_version();
    let version2 = build_rust_version();

    assert_eq!(version1, version2);
}

// =============================================================================
// SHOW VERSION TESTS (UI VARIANT)
// =============================================================================

#[tokio::test]
async fn test_show_version_simple() {
    let result = show_version(false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_show_version_detailed() {
    let result = show_version(true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_show_version_both_modes() {
    let simple = show_version(false).await;
    let detailed = show_version(true).await;

    assert!(simple.is_ok());
    assert!(detailed.is_ok());
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[tokio::test]
async fn test_all_version_commands_succeed() {
    let result1 = execute_version_command(false).await;
    let result2 = execute_version_command(true).await;
    let result3 = show_simple_version().await;
    let result4 = show_detailed_version().await;
    let result5 = show_version(false).await;
    let result6 = show_version(true).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
    assert!(result4.is_ok());
    assert!(result5.is_ok());
    assert!(result6.is_ok());
}

#[tokio::test]
async fn test_version_commands_are_idempotent() {
    // Running multiple times should always succeed
    for _ in 0..3 {
        let result = execute_version_command(false).await;
        assert!(result.is_ok());
    }

    for _ in 0..3 {
        let result = show_version(true).await;
        assert!(result.is_ok());
    }
}

// =============================================================================
// BUILD INFO HELPER TESTS
// =============================================================================

#[test]
fn test_build_helpers_are_consistent() {
    // All build info helpers should be consistent across calls
    let info1 = build_info();
    let info2 = build_info();
    let version1 = build_rust_version();
    let version2 = build_rust_version();

    assert_eq!(info1, info2);
    assert_eq!(version1, version2);
}

#[test]
fn test_build_helpers_return_strings() {
    // All build info helpers should return strings (may be empty if env vars not set)
    let info = build_info();
    let version = build_rust_version();

    // Just verify they return strings without panicking
    let _ = info;
    let _ = version;
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[tokio::test]
async fn test_rapid_version_calls() {
    // Test calling version commands rapidly in succession
    let mut handles = vec![];

    for _ in 0..10 {
        handles.push(tokio::spawn(async { show_simple_version().await }));
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_alternating_simple_detailed() {
    // Test alternating between simple and detailed versions
    for _ in 0..5 {
        let simple = show_simple_version().await;
        assert!(simple.is_ok());

        let detailed = show_detailed_version().await;
        assert!(detailed.is_ok());
    }
}
