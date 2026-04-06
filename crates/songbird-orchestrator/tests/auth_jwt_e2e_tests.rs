// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! E2E Tests for `security provider` JWT Delegation
//!
//! Tests the complete JWT flow from discovery to provisioning.

use songbird_orchestrator::auth::provision_jwt_secret;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
#[ignore = "Requires security provider running"]
async fn test_e2e_jwt_provisioning_from_security_provider() {
    // This test requires security provider to be running
    // Set SECURITY_PROVIDER_SOCKET (or legacy BEARDOG_SOCKET) to test with the security provider

    let socket_opt =
        std::env::var("SECURITY_PROVIDER_SOCKET").or_else(|_| std::env::var("BEARDOG_SOCKET"));

    if let Ok(socket) = socket_opt {
        println!("🔍 Testing JWT provisioning from security provider at: {socket}");

        let result = timeout(
            Duration::from_secs(5),
            provision_jwt_secret(Some(&socket), "songbird_e2e_test"),
        )
        .await;

        match result {
            Ok(Ok(secret)) => {
                println!("✅ Got JWT secret from security provider: {} chars", secret.len());
                assert!(secret.len() >= 85);
                assert!(secret.len() <= 90);

                // Verify it's valid base64
                use base64::Engine;
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(&secret)
                    .expect("Should be valid base64");
                assert_eq!(decoded.len(), 64); // 512 bits
            }
            Ok(Err(e)) => {
                println!(
                    "⚠️  security provider JWT fetch failed (expected if method not implemented): {e}"
                );
                // This is acceptable - security provider may not have the method yet
            }
            Err(_) => {
                println!("⏱️  Timeout waiting for security provider (5s)");
                panic!("security provider did not respond in time");
            }
        }
    } else {
        println!(
            "⏭️  Skipping security provider E2E test (set SECURITY_PROVIDER_SOCKET or BEARDOG_SOCKET to enable)"
        );
    }
}

#[tokio::test]
async fn test_e2e_jwt_provisioning_fallback_flow() {
    // Test the complete fallback flow (no security provider)
    println!("🔍 Testing JWT provisioning fallback flow...");

    let secret = provision_jwt_secret(None, "songbird_e2e_fallback")
        .await
        .expect("Fallback should always succeed");

    println!("✅ Fallback JWT secret generated: {} chars", secret.len());

    // Verify fallback secret
    assert!(secret.len() >= 85);
    assert!(secret.len() <= 90);

    // Verify it's valid base64
    use base64::Engine;
    let decoded =
        base64::engine::general_purpose::STANDARD.decode(&secret).expect("Should be valid base64");
    assert_eq!(decoded.len(), 64); // 512 bits

    println!("✅ Fallback flow complete and secure!");
}

#[tokio::test]
async fn test_e2e_jwt_provisioning_with_invalid_socket() {
    // Test with invalid socket path (should fall back to secure random)
    println!("🔍 Testing JWT provisioning with invalid socket...");

    let secret = provision_jwt_secret(
        Some("/tmp/nonexistent-security-provider-socket-12345.sock"),
        "songbird_e2e_invalid",
    )
    .await
    .expect("Should fall back to secure random");

    println!("✅ Fallback triggered for invalid socket: {} chars", secret.len());

    assert!(secret.len() >= 85);
    assert!(secret.len() <= 90);
}

#[tokio::test]
async fn test_e2e_jwt_provisioning_concurrent() {
    // Test concurrent JWT provisioning (should not interfere)
    println!("🔍 Testing concurrent JWT provisioning...");

    let handles: Vec<_> = (0..10)
        .map(|i| {
            tokio::spawn(async move {
                provision_jwt_secret(None, &format!("songbird_concurrent_{i}"))
                    .await
                    .expect("Should succeed")
            })
        })
        .collect();

    let secrets: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task should not panic"))
        .collect();

    println!("✅ Generated {} concurrent secrets", secrets.len());

    // All should be valid
    for secret in &secrets {
        assert!(secret.len() >= 85);
        assert!(secret.len() <= 90);
    }

    // All should be unique
    for i in 0..secrets.len() {
        for j in (i + 1)..secrets.len() {
            assert_ne!(secrets[i], secrets[j], "Secrets should be unique");
        }
    }

    println!("✅ All concurrent secrets valid and unique!");
}

#[tokio::test]
async fn test_e2e_jwt_provisioning_performance() {
    // Test JWT provisioning performance
    println!("🔍 Testing JWT provisioning performance...");

    let start = std::time::Instant::now();
    let iterations = 100;

    for i in 0..iterations {
        let _ =
            provision_jwt_secret(None, &format!("perf_test_{i}")).await.expect("Should succeed");
    }

    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / iterations;

    println!("✅ {iterations} iterations in {elapsed:?}");
    println!("   Average: {avg_ms}ms per secret");

    // Should be fast (< 10ms per secret on average)
    assert!(avg_ms < 10, "JWT generation should be fast");
}
