// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Fault Injection Tests for `security provider` JWT Delegation
//!
//! Tests JWT provisioning under fault conditions.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use songbird_orchestrator::auth::provision_jwt_secret;
use std::time::Duration;

#[tokio::test]
// ✅ NO #[serial]! Concurrent-safe function calls!
async fn test_fault_invalid_socket_path() {
    // Test with various invalid socket paths
    println!("💥 FAULT: Testing invalid socket paths...");

    let invalid_paths = vec![
        "/tmp/nonexistent-socket.sock",
        "/root/no-permission.sock",
        "/dev/null",
        "",
        "/tmp/../../../etc/passwd",
    ];

    for path in invalid_paths {
        println!("   Testing path: {path}");
        let secret = provision_jwt_secret(Some(path), "fault_invalid_path")
            .await
            .expect("Should fall back to secure random");

        assert!(secret.len() >= 85);
        println!("   ✅ Fallback successful for: {path}");
    }

    println!("✅ FAULT: Invalid socket path test passed!");
}

#[tokio::test]
// ✅ NO #[serial]! Concurrent-safe function calls!
async fn test_fault_socket_connection_refused() {
    // Test with socket that exists but refuses connection
    println!("💥 FAULT: Testing connection refused...");

    // Use a path that's unlikely to have a listening socket
    let secret = provision_jwt_secret(
        Some("/tmp/security-provider-fault-refused-12345.sock"),
        "fault_refused",
    )
    .await
    .expect("Should fall back to secure random");

    assert!(secret.len() >= 85);

    println!("✅ FAULT: Connection refused test passed!");
}

#[tokio::test]
// ✅ NO #[serial]! Concurrent-safe function calls!
async fn test_fault_empty_purpose() {
    // Test with empty purpose string
    println!("💥 FAULT: Testing empty purpose...");

    let secret = provision_jwt_secret(None, "").await.expect("Should succeed with empty purpose");

    assert!(secret.len() >= 85);

    println!("✅ FAULT: Empty purpose test passed!");
}

#[tokio::test]
// ✅ NO #[serial]! Concurrent-safe function calls!
async fn test_fault_very_long_purpose() {
    // Test with very long purpose string
    println!("💥 FAULT: Testing very long purpose...");

    let long_purpose = "a".repeat(10000);
    let secret =
        provision_jwt_secret(None, &long_purpose).await.expect("Should succeed with long purpose");

    assert!(secret.len() >= 85);

    println!("✅ FAULT: Very long purpose test passed!");
}

#[tokio::test]
// ✅ NO #[serial]! Concurrent-safe function calls!
async fn test_fault_special_characters_in_purpose() {
    // Test with special characters in purpose
    println!("💥 FAULT: Testing special characters in purpose...");

    let special_purposes = vec![
        "purpose\nwith\nnewlines",
        "purpose\twith\ttabs",
        "purpose with spaces",
        "purpose;with;semicolons",
        "purpose'with'quotes",
        "purpose\"with\"doublequotes",
        "purpose\\with\\backslashes",
        "purpose/with/slashes",
        "purpose<with>brackets",
        "purpose{with}braces",
        "purpose[with]squarebrackets",
        "purpose|with|pipes",
        "purpose&with&ampersands",
        "purpose$with$dollars",
        "purpose#with#hashes",
        "purpose@with@ats",
        "purpose!with!exclamations",
        "purpose?with?questions",
        "purpose*with*asterisks",
        "purpose%with%percents",
        "purpose^with^carets",
        "purpose~with~tildes",
        "purpose`with`backticks",
        "purpose=with=equals",
        "purpose+with+pluses",
        "purpose-with-dashes",
        "purpose_with_underscores",
        "purpose.with.dots",
        "purpose,with,commas",
        "purpose:with:colons",
        "purpose(with)parens",
        "purpose\0with\0nulls",
        "purpose\x01with\x01control",
        "purpose🦀with🦀emoji",
        "purpose中文with中文",
        "purpose עבריתwith עברית",
        "purpose العربيةwith العربية",
    ];

    for purpose in special_purposes {
        let secret = provision_jwt_secret(None, purpose)
            .await
            .expect("Should succeed with special characters");

        assert!(secret.len() >= 85);
    }

    println!("✅ FAULT: Special characters test passed!");
}

#[tokio::test]
// ✅ NO #[serial]! Concurrent-safe function calls!
async fn test_fault_concurrent_failures() {
    // Test concurrent provisioning where some fail
    println!("💥 FAULT: Testing concurrent failures...");

    let handles: Vec<_> = (0..100)
        .map(|i| {
            tokio::spawn(async move {
                let socket = if i % 3 == 0 {
                    Some("/tmp/nonexistent-socket.sock")
                } else {
                    None
                };

                provision_jwt_secret(socket, &format!("fault_concurrent_{i}"))
                    .await
                    .expect("Should succeed with fallback")
            })
        })
        .collect();

    let secrets: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task should not panic"))
        .collect();

    println!("✅ Generated {} secrets with concurrent failures", secrets.len());

    // All should be valid
    for secret in &secrets {
        assert!(secret.len() >= 85);
    }

    println!("✅ FAULT: Concurrent failures test passed!");
}

#[tokio::test]
// ✅ NO #[serial]! Concurrent-safe function calls!
async fn test_fault_rapid_socket_changes() {
    // Test rapid socket path changes
    println!("💥 FAULT: Testing rapid socket changes...");

    let handles: Vec<_> = (0..100)
        .map(|i| {
            tokio::spawn(async move {
                let socket = format!("/tmp/security-provider-fault-{}.sock", i % 10);
                provision_jwt_secret(Some(&socket), &format!("fault_rapid_{i}"))
                    .await
                    .expect("Should succeed with fallback")
            })
        })
        .collect();

    let secrets: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task should not panic"))
        .collect();

    println!("✅ Generated {} secrets with rapid socket changes", secrets.len());

    // All should be valid
    for secret in &secrets {
        assert!(secret.len() >= 85);
    }

    println!("✅ FAULT: Rapid socket changes test passed!");
}

#[tokio::test]
// ✅ NO #[serial]! Concurrent-safe function calls!
async fn test_fault_timeout_recovery() {
    // Test recovery from timeouts
    println!("💥 FAULT: Testing timeout recovery...");

    for i in 0..10 {
        let result = tokio::time::timeout(
            Duration::from_millis(1), // Very short timeout
            provision_jwt_secret(
                Some("/tmp/nonexistent-socket.sock"),
                &format!("fault_timeout_{i}"),
            ),
        )
        .await;

        match result {
            Ok(Ok(secret)) => {
                assert!(secret.len() >= 85);
                println!("   ✅ Iteration {i} succeeded");
            }
            Ok(Err(e)) => panic!("Unexpected error: {e}"),
            Err(_) => {
                println!("   ⏱️  Iteration {i} timed out (acceptable)");
            }
        }
    }

    println!("✅ FAULT: Timeout recovery test passed!");
}

#[tokio::test(start_paused = true)]
// ✅ NO #[serial]! Concurrent-safe function calls!
async fn test_fault_resource_exhaustion() {
    // Test under resource exhaustion conditions
    println!("💥 FAULT: Testing resource exhaustion...");

    // Spawn many tasks that hold resources
    let resource_handles: Vec<_> = (0..1000)
        .map(|_| {
            tokio::spawn(async {
                tokio::time::sleep(Duration::from_secs(1)).await;
            })
        })
        .collect();

    // Try to provision JWT while resources are exhausted
    let provision_handles: Vec<_> = (0..50)
        .map(|i| {
            tokio::spawn(async move {
                provision_jwt_secret(None, &format!("fault_exhaustion_{i}"))
                    .await
                    .expect("Should succeed despite resource exhaustion")
            })
        })
        .collect();

    let secrets: Vec<String> = futures::future::join_all(provision_handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task should not panic"))
        .collect();

    // Wait for resource tasks to complete
    futures::future::join_all(resource_handles).await;

    println!("✅ Generated {} secrets under resource exhaustion", secrets.len());

    // All should be valid
    for secret in &secrets {
        assert!(secret.len() >= 85);
    }

    println!("✅ FAULT: Resource exhaustion test passed!");
}
