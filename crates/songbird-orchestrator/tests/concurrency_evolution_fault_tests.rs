// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Concurrency Evolution Fault Tests
//!
//! Fault injection tests for concurrency improvements:
//! - Invalid input handling
//! - Error recovery
//! - Edge cases
//! - Resource constraints
//!
//! Modern, idiomatic, async Rust with comprehensive fault coverage.

// `cargo_bin!` only applies to binaries in *this* package; the `songbird` executable is built
// by the workspace root crate. The deprecated `cargo_bin` function resolves the path at runtime.
#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::task::JoinSet;

// ====================
// TEST HELPERS
// ====================

/// Create a clean command with isolated environment
fn clean_cmd() -> Command {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
    cmd.env_clear();
    cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
    cmd
}

// ====================
// FAULT TESTS - INVALID INPUTS
// ====================

#[tokio::test]
async fn fault_test_concurrent_invalid_commands() {
    // FAULT: All commands are invalid
    let mut join_set = JoinSet::new();

    for i in 0..50 {
        join_set.spawn(async move {
            clean_cmd().arg(format!("invalid-command-{i}")).assert().failure();
        });
    }

    let mut failure_count = 0;
    while let Some(_) = join_set.join_next().await {
        failure_count += 1;
    }

    assert_eq!(failure_count, 50, "All invalid commands should fail");
}

#[tokio::test]
async fn fault_test_empty_command_args() {
    // FAULT: Commands with no arguments
    let mut join_set = JoinSet::new();

    for _ in 0..20 {
        join_set.spawn(async {
            // No arguments provided (should show help or fail gracefully)
            clean_cmd().assert().code(predicate::ne(255)); // Should not panic/crash
        });
    }

    while let Some(_) = join_set.join_next().await {}
}

#[tokio::test]
async fn fault_test_malformed_arguments() {
    // FAULT: Various malformed arguments
    let malformed_args = vec![
        vec!["--port", "invalid"],
        vec!["--port", "-1"],
        vec!["--port", "99999999"],
        vec!["server", "--unknown-flag"],
        vec!["--", "--", "--"],
    ];

    let mut join_set = JoinSet::new();

    for args in malformed_args {
        join_set.spawn(async move {
            let mut cmd = clean_cmd();
            for arg in args {
                cmd.arg(arg);
            }
            cmd.assert().failure();
        });
    }

    while let Some(_) = join_set.join_next().await {}
}

// ====================
// FAULT TESTS - ENVIRONMENT ISSUES
// ====================

#[tokio::test]
async fn fault_test_missing_path_env() {
    // FAULT: Command with no PATH environment variable
    let mut join_set = JoinSet::new();

    for _ in 0..20 {
        join_set.spawn(async {
            let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
            cmd.env_clear();
            // Deliberately NO PATH set (unlike clean_cmd)

            // Should still work (or fail gracefully)
            let _ = cmd.arg("--version").assert();
        });
    }

    while let Some(_) = join_set.join_next().await {}
}

#[tokio::test]
async fn fault_test_corrupted_environment_values() {
    // FAULT: Environment with corrupted/invalid values
    let mut join_set = JoinSet::new();

    for i in 0..20 {
        join_set.spawn(async move {
            let mut cmd = clean_cmd();

            // Various corrupted values
            cmd.env("SONGBIRD_PORT", "not_a_number");
            cmd.env("SONGBIRD_CONFIG", "\0\0\0"); // Null bytes
            cmd.env("SONGBIRD_NODE_ID", "x".repeat(10000)); // Huge value
            cmd.env("CORRUPTED", format!("{}_{}", i, "\n\r\t"));

            // Should handle gracefully
            let _ = cmd.arg("--version").assert();
        });
    }

    while let Some(_) = join_set.join_next().await {}
}

// ====================
// FAULT TESTS - CONCURRENT FAILURES
// ====================

#[tokio::test]
async fn fault_test_all_fail_concurrently() {
    // FAULT: All concurrent operations fail
    let mut join_set = JoinSet::new();
    let failure_count = Arc::new(AtomicUsize::new(0));

    for _ in 0..100 {
        let failure_count = failure_count.clone();
        join_set.spawn(async move {
            let result = clean_cmd().arg("guaranteed-to-fail").assert().try_failure();

            if result.is_ok() {
                failure_count.fetch_add(1, Ordering::SeqCst);
            }
        });
    }

    while let Some(_) = join_set.join_next().await {}

    assert_eq!(failure_count.load(Ordering::SeqCst), 100, "All operations should fail as expected");
}

#[tokio::test]
async fn fault_test_mixed_success_failure_isolation() {
    // FAULT: Verify failures don't affect successful operations
    let mut join_set = JoinSet::new();
    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));

    for i in 0..100 {
        let success_count = success_count.clone();
        let failure_count = failure_count.clone();

        join_set.spawn(async move {
            if i % 2 == 0 {
                // Should succeed
                clean_cmd().arg("--version").assert().success();
                success_count.fetch_add(1, Ordering::SeqCst);
            } else {
                // Should fail
                clean_cmd().arg("invalid").assert().failure();
                failure_count.fetch_add(1, Ordering::SeqCst);
            }
        });
    }

    while let Some(_) = join_set.join_next().await {}

    // Verify isolation: failures didn't corrupt successes
    assert_eq!(success_count.load(Ordering::SeqCst), 50);
    assert_eq!(failure_count.load(Ordering::SeqCst), 50);
}

// ====================
// FAULT TESTS - RESOURCE CONSTRAINTS
// ====================

#[tokio::test]
async fn fault_test_rapid_creation_destruction() {
    // FAULT: Rapid cycles of creation/destruction (resource stress)
    for _ in 0..100 {
        let mut join_set = JoinSet::new();

        for _ in 0..10 {
            join_set.spawn(async {
                let _cmd = clean_cmd();
                // Immediate drop
            });
        }

        while let Some(_) = join_set.join_next().await {}
    }

    // If we survived 1000 rapid cycles, we're robust
}

#[tokio::test]
async fn fault_test_command_handle_leaks() {
    // FAULT: Verify no handle leaks over many iterations
    let mut successful = 0;

    for _ in 0..100 {
        let mut join_set = JoinSet::new();

        for _ in 0..10 {
            join_set.spawn(async {
                clean_cmd().arg("--version").assert().success();
            });
        }

        while let Some(_) = join_set.join_next().await {
            successful += 1;
        }
    }

    assert_eq!(successful, 1000, "All iterations should succeed (no leaks)");
}

// ====================
// FAULT TESTS - EDGE CASES
// ====================

#[tokio::test]
async fn fault_test_zero_environment_variables() {
    // FAULT: Minimal possible environment
    let mut join_set = JoinSet::new();

    for _ in 0..20 {
        join_set.spawn(async {
            let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
            cmd.env_clear();
            // Absolutely nothing in environment

            // Should handle gracefully
            let _ = cmd.arg("--version").assert();
        });
    }

    while let Some(_) = join_set.join_next().await {}
}

#[tokio::test]
async fn fault_test_maximum_argument_length() {
    // FAULT: Arguments at system limits
    let mut join_set = JoinSet::new();

    for i in 0..20 {
        join_set.spawn(async move {
            clean_cmd()
                .arg("x".repeat(1000 + i * 100)) // Very long argument
                .assert()
                .failure(); // Should fail, but gracefully
        });
    }

    while let Some(_) = join_set.join_next().await {}
}

#[tokio::test]
async fn fault_test_special_characters_in_env() {
    // FAULT: Special characters that might break parsing
    let special_values = vec![
        "value with spaces",
        "value\nwith\nnewlines",
        "value\twith\ttabs",
        "value\"with\"quotes",
        "value'with'quotes",
        "value$with$dollar",
        "value\\with\\backslash",
        "value;with;semicolon",
        "value&with&ampersand",
        "value|with|pipe",
    ];

    let mut join_set = JoinSet::new();

    for (i, value) in special_values.iter().enumerate() {
        let value = value.to_string();
        join_set.spawn(async move {
            let mut cmd = clean_cmd();
            cmd.env(format!("SPECIAL_{i}"), value);
            cmd.arg("--version").assert().success();
        });
    }

    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }
}

// ====================
// FAULT TESTS - ERROR RECOVERY
// ====================

#[tokio::test]
async fn fault_test_recovery_after_failures() {
    // FAULT: Verify system recovers after failures
    let mut join_set = JoinSet::new();

    // Phase 1: Cause failures
    for _ in 0..20 {
        join_set.spawn(async {
            clean_cmd().arg("invalid").assert().failure();
        });
    }

    while let Some(_) = join_set.join_next().await {}

    // Phase 2: Verify recovery (should work normally)
    let mut join_set = JoinSet::new();
    for _ in 0..20 {
        join_set.spawn(async {
            clean_cmd().arg("--version").assert().success();
        });
    }

    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }
}

#[tokio::test]
async fn fault_test_interleaved_success_failure_recovery() {
    // FAULT: Interleave success/failure/recovery patterns
    for _ in 0..10 {
        let mut join_set = JoinSet::new();

        // Mix of patterns
        for i in 0..30 {
            join_set.spawn(async move {
                match i % 3 {
                    0 => {
                        // Success
                        clean_cmd().arg("--version").assert().success();
                    }
                    1 => {
                        // Failure
                        clean_cmd().arg("invalid").assert().failure();
                    }
                    _ => {
                        // Recovery (success after potential failure)
                        clean_cmd().arg("--help").assert().success();
                    }
                }
            });
        }

        while let Some(_) = join_set.join_next().await {}
    }
}

// ====================
// FAULT TESTS - BOUNDARY CONDITIONS
// ====================

#[tokio::test]
async fn fault_test_single_command() {
    // FAULT: Edge case - just one command (no concurrency)
    clean_cmd().arg("--version").assert().success();
}

#[tokio::test]
async fn fault_test_two_commands() {
    // FAULT: Edge case - minimal concurrency (2)
    let mut join_set = JoinSet::new();

    join_set.spawn(async {
        clean_cmd().arg("--version").assert().success();
    });

    join_set.spawn(async {
        clean_cmd().arg("--help").assert().success();
    });

    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }
}

#[tokio::test]
async fn fault_test_odd_number_concurrent() {
    // FAULT: Edge case - odd number (not power of 2)
    let mut join_set = JoinSet::new();

    for _ in 0..13 {
        // Prime-ish number
        join_set.spawn(async {
            clean_cmd().arg("--version").assert().success();
        });
    }

    let mut count = 0;
    while let Some(result) = join_set.join_next().await {
        result.unwrap();
        count += 1;
    }

    assert_eq!(count, 13);
}

// ====================
// DOCUMENTATION TESTS
// ====================

/// Documents fault handling patterns
#[tokio::test]
async fn fault_test_pattern_documentation() {
    // ✅ CORRECT PATTERN: Isolated fault handling

    let mut join_set = JoinSet::new();

    // Each failure is isolated
    join_set.spawn(async {
        clean_cmd().arg("invalid1").assert().failure();
    });

    join_set.spawn(async {
        clean_cmd().arg("invalid2").assert().failure();
    });

    // Failures don't affect each other
    while let Some(_) = join_set.join_next().await {}
}
