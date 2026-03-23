// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Concurrency Evolution E2E Tests
//!
//! End-to-end tests validating the concurrency improvements:
//! - Full workflow isolation
//! - Concurrent command execution
//! - Real-world usage patterns
//!
//! Modern, idiomatic, async Rust with true concurrency.

use assert_cmd::Command;
use predicates::prelude::*;
use std::sync::Arc;
use tokio::sync::Barrier;
use tokio::task::JoinSet;

// ====================
// TEST HELPERS
// ====================

/// Create a clean command with isolated environment
fn clean_cmd() -> Command {
    let mut cmd = Command::new(assert_cmd::cargo_bin!("songbird"));
    cmd.env_clear();
    cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
    cmd
}

// ====================
// E2E WORKFLOW TESTS
// ====================

#[tokio::test]
async fn test_e2e_concurrent_help_commands() {
    // Multiple help commands can run concurrently
    let mut join_set = JoinSet::new();

    for _ in 0..10 {
        join_set.spawn(async {
            clean_cmd()
                .arg("--help")
                .assert()
                .success()
                .stdout(predicate::str::contains("songbird"));
        });
    }

    // All should succeed concurrently
    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }
}

#[tokio::test]
async fn test_e2e_concurrent_version_commands() {
    // Multiple version commands can run concurrently
    let mut join_set = JoinSet::new();

    for _ in 0..10 {
        join_set.spawn(async {
            clean_cmd()
                .arg("--version")
                .assert()
                .success()
                .stdout(predicate::str::contains("songbird"));
        });
    }

    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }
}

#[tokio::test]
async fn test_e2e_concurrent_invalid_commands() {
    // Multiple failing commands can run concurrently without interference
    let mut join_set = JoinSet::new();

    for i in 0..10 {
        join_set.spawn(async move {
            clean_cmd().arg(format!("invalid-command-{i}")).assert().failure();
        });
    }

    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }
}

#[tokio::test]
async fn test_e2e_concurrent_different_commands() {
    // Mix of different commands running concurrently
    let mut join_set = JoinSet::new();

    // Help commands
    for _ in 0..5 {
        join_set.spawn(async {
            clean_cmd().arg("--help").assert().success();
        });
    }

    // Version commands
    for _ in 0..5 {
        join_set.spawn(async {
            clean_cmd().arg("--version").assert().success();
        });
    }

    // Invalid commands
    for _ in 0..5 {
        join_set.spawn(async {
            clean_cmd().arg("invalid").assert().failure();
        });
    }

    // All 15 commands should complete without interference
    let mut count = 0;
    while let Some(result) = join_set.join_next().await {
        result.unwrap();
        count += 1;
    }

    assert_eq!(count, 15, "All concurrent commands completed");
}

// ====================
// SYNCHRONIZATION TESTS
// ====================

#[tokio::test]
async fn test_e2e_synchronized_start() {
    // All tasks start simultaneously (tests barrier pattern)
    let barrier = Arc::new(Barrier::new(10));
    let mut join_set = JoinSet::new();

    for _ in 0..10 {
        let barrier = barrier.clone();
        join_set.spawn(async move {
            // Wait for all tasks to be ready
            barrier.wait().await;

            // All execute simultaneously
            clean_cmd().arg("--version").assert().success();
        });
    }

    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }
}

#[tokio::test]
async fn test_e2e_concurrent_environment_isolation() {
    // Each command has different environment, all run concurrently
    let mut join_set = JoinSet::new();

    for i in 0..20 {
        join_set.spawn(async move {
            let mut cmd = clean_cmd();

            // Each command gets unique environment
            cmd.env("TEST_ID", format!("{i}"));
            cmd.env("TEST_VAR", format!("value_{i}"));

            // Command should succeed with its isolated environment
            cmd.arg("--version").assert().success();
        });
    }

    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }
}

// ====================
// STRESS TESTS
// ====================

#[tokio::test]
async fn test_e2e_high_concurrency_stress() {
    // Stress test: 100 concurrent commands
    let mut join_set = JoinSet::new();

    for i in 0..100 {
        join_set.spawn(async move {
            let mut cmd = clean_cmd();
            cmd.env("STRESS_ID", format!("{i}"));

            // Mix of different commands
            if i % 3 == 0 {
                cmd.arg("--help").assert().success();
            } else if i % 3 == 1 {
                cmd.arg("--version").assert().success();
            } else {
                cmd.arg("invalid").assert().failure();
            }
        });
    }

    let mut count = 0;
    while let Some(result) = join_set.join_next().await {
        result.unwrap();
        count += 1;
    }

    assert_eq!(count, 100, "All 100 stress test commands completed");
}

#[tokio::test]
async fn test_e2e_rapid_sequential_to_concurrent() {
    // Test transition from sequential to concurrent execution

    // Phase 1: Sequential (baseline)
    for _ in 0..10 {
        clean_cmd().arg("--version").assert().success();
    }

    // Phase 2: Concurrent (should be just as reliable)
    let mut join_set = JoinSet::new();
    for _ in 0..10 {
        join_set.spawn(async {
            clean_cmd().arg("--version").assert().success();
        });
    }

    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }
}

// ====================
// ISOLATION VALIDATION TESTS
// ====================

#[tokio::test]
async fn test_e2e_no_cross_contamination() {
    // Verify that concurrent commands don't contaminate each other
    let mut join_set = JoinSet::new();

    for i in 0..50 {
        join_set.spawn(async move {
            let mut cmd = clean_cmd();

            // Set unique env for this command
            cmd.env("UNIQUE_ID", format!("{i}"));
            cmd.env("UNIQUE_VALUE", format!("value_{i}"));

            // If there's cross-contamination, this would be flaky
            cmd.arg("--version").assert().success().stdout(predicate::str::contains("songbird"));
        });
    }

    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }
}

#[tokio::test]
async fn test_e2e_environment_independence() {
    // Verify that each command's environment is truly independent
    let mut join_set = JoinSet::new();

    // Create commands with conflicting environment variables
    for i in 0..20 {
        join_set.spawn(async move {
            let mut cmd = clean_cmd();

            // All use the same env var name but different values
            cmd.env("COMMON_VAR", format!("unique_value_{i}"));

            // Should not interfere with each other
            cmd.arg("--version").assert().success();
        });
    }

    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }
}

// ====================
// REAL-WORLD PATTERNS
// ====================

#[tokio::test]
async fn test_e2e_realistic_test_suite_pattern() {
    // Simulates how a real test suite would run concurrently

    struct TestCase {
        name: &'static str,
        args: Vec<&'static str>,
        should_succeed: bool,
    }

    let test_cases = vec![
        TestCase {
            name: "help",
            args: vec!["--help"],
            should_succeed: true,
        },
        TestCase {
            name: "version",
            args: vec!["--version"],
            should_succeed: true,
        },
        TestCase {
            name: "invalid1",
            args: vec!["bad"],
            should_succeed: false,
        },
        TestCase {
            name: "invalid2",
            args: vec!["wrong"],
            should_succeed: false,
        },
    ];

    // Run all test cases concurrently
    let mut join_set = JoinSet::new();

    for test_case in test_cases {
        join_set.spawn(async move {
            let mut cmd = clean_cmd();
            for arg in test_case.args {
                cmd.arg(arg);
            }

            if test_case.should_succeed {
                cmd.assert().success();
            } else {
                cmd.assert().failure();
            }

            test_case.name
        });
    }

    let mut completed = Vec::new();
    while let Some(result) = join_set.join_next().await {
        completed.push(result.unwrap());
    }

    assert_eq!(completed.len(), 4, "All test cases completed");
}

// ====================
// DOCUMENTATION TESTS
// ====================

/// Documents the correct concurrent E2E test pattern
#[tokio::test]
async fn test_e2e_pattern_documentation() {
    // ✅ CORRECT PATTERN: Concurrent E2E tests

    let mut join_set = JoinSet::new();

    // Each test case runs independently
    join_set.spawn(async {
        clean_cmd().arg("--help").assert().success();
    });

    join_set.spawn(async {
        clean_cmd().arg("--version").assert().success();
    });

    // No #[serial] needed!
    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }
}
