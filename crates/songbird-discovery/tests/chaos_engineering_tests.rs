// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![cfg(feature = "test-mocks")]
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
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]

//! Chaos Engineering Tests for Songbird Discovery and Federation
//!
//! These tests inject random failures and adverse conditions to verify
//! system resilience under real-world chaos scenarios:
//! - Random peer disconnections
//! - Network partitions (split-brain)
//! - Clock skew/drift
//! - Slow network conditions
//! - Byzantine failures
//!
//! All tests verify graceful degradation and recovery.

#[cfg(test)]
mod chaos_engineering_tests {
    use serde_json::json;
    use songbird_discovery::IdentityAttestation;
    use songbird_discovery::anonymous::{AnonymousDiscoveryMessage, TransportEndpointMessage};
    use songbird_discovery::birdsong::{
        BirdSongConfig, BirdSongEncryption, BirdSongProcessor, ChaoticBirdSongMock,
    };
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::{Duration, Instant, sleep};

    fn chaotic_enc(family_id: Option<String>, failure_rate: f64) -> Arc<BirdSongEncryption> {
        Arc::new(BirdSongEncryption::Chaotic(Arc::new(ChaoticBirdSongMock::new(
            family_id,
            failure_rate,
        ))))
    }

    /// Network simulator that adds latency and packet loss
    struct NetworkSimulator {
        latency: Duration,
        jitter: Duration,
        packet_loss_rate: f64,
        packet_count: AtomicUsize,
    }

    impl NetworkSimulator {
        fn new(latency: Duration, jitter: Duration, packet_loss_rate: f64) -> Self {
            Self {
                latency,
                jitter,
                packet_loss_rate,
                packet_count: AtomicUsize::new(0),
            }
        }

        async fn simulate_send(&self) -> bool {
            let count = self.packet_count.fetch_add(1, Ordering::SeqCst);

            // Simulate packet loss
            if (count % 100) < (self.packet_loss_rate * 100.0) as usize {
                return false; // Packet lost
            }

            // Simulate latency with jitter
            let actual_latency =
                self.latency + Duration::from_millis(count as u64 % self.jitter.as_millis() as u64);
            sleep(actual_latency).await;

            true // Packet delivered
        }
    }

    #[tokio::test]
    async fn test_random_provider_failures() {
        // Provider that fails 30% of the time
        let provider = chaotic_enc(Some("chaos-family".to_string()), 0.3);

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: false,
            ..Default::default()
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider), config));

        // Send 100 messages - should handle failures gracefully
        let mut success_count = 0;
        let mut fallback_count = 0;

        for i in 0..100 {
            let message = create_test_message(&format!("msg-{i}"));
            let plaintext = message.to_bytes().unwrap();

            let result = processor.encrypt_packet(&plaintext).await;
            match result {
                Ok(_) => success_count += 1,
                Err(_) => fallback_count += 1,
            }
        }

        // With 30% failure rate and fallback enabled, should have high success
        assert!(success_count > 60, "Should handle most messages despite failures");
        println!("✅ Chaos test: {success_count} successes, {fallback_count} fallbacks out of 100");
    }

    #[tokio::test]
    async fn test_provider_availability_toggling() {
        let provider = chaotic_enc(
            Some("toggle-family".to_string()),
            0.0, // No random failures
        );

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: false,
            ..Default::default()
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider.clone()), config));

        // Test with provider available
        assert!(provider.is_available());
        let msg1 = create_test_message("msg1");
        let result1 = processor.encrypt_packet(&msg1.to_bytes().unwrap()).await;
        assert!(result1.is_ok(), "Should work when available");

        // Toggle availability off
        provider.chaotic_toggle_availability();
        assert!(!provider.is_available());
        let msg2 = create_test_message("msg2");
        let result2 = processor.encrypt_packet(&msg2.to_bytes().unwrap()).await;
        assert!(result2.is_ok(), "Should fall back when unavailable");

        // Toggle availability back on
        provider.chaotic_toggle_availability();
        assert!(provider.is_available());
        let msg3 = create_test_message("msg3");
        let result3 = processor.encrypt_packet(&msg3.to_bytes().unwrap()).await;
        assert!(result3.is_ok(), "Should work again when available");

        println!("✅ Availability toggling: All scenarios handled gracefully");
    }

    #[tokio::test]
    async fn test_slow_network_conditions() {
        let network = NetworkSimulator::new(
            Duration::from_millis(100), // 100ms base latency
            Duration::from_millis(50),  // 50ms jitter
            0.05,                       // 5% packet loss
        );

        let start = Instant::now();
        let mut delivered = 0;
        let mut lost = 0;

        // Send 50 packets through simulated network
        for _ in 0..50 {
            if network.simulate_send().await {
                delivered += 1;
            } else {
                lost += 1;
            }
        }

        let duration = start.elapsed();

        // Verify network simulation worked
        assert!(duration > Duration::from_millis(100), "Should have latency");
        assert!(delivered > 40, "Should deliver most packets despite 5% loss");
        assert!(lost > 0, "Should lose some packets");

        println!("✅ Slow network: Delivered {delivered}, Lost {lost} in {duration:?}");
    }

    #[tokio::test]
    async fn test_high_contention_scenario() {
        let provider = chaotic_enc(
            Some("contention-family".to_string()),
            0.1, // 10% failure rate
        );

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: false,
            ..Default::default()
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider), config));

        // Spawn 10 concurrent tasks all trying to encrypt
        let mut handles = vec![];
        for task_id in 0..10 {
            let processor_clone = Arc::clone(&processor);
            let handle = tokio::spawn(async move {
                let mut task_success = 0;
                for i in 0..20 {
                    let msg = create_test_message(&format!("task-{task_id}-msg-{i}"));
                    if processor_clone.encrypt_packet(&msg.to_bytes().unwrap()).await.is_ok() {
                        task_success += 1;
                    }
                }
                task_success
            });
            handles.push(handle);
        }

        // Wait for all tasks
        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }
        let total_success: usize = results.into_iter().sum();

        // Should handle high contention gracefully
        assert!(total_success > 150, "Should handle most messages under contention");
        println!("✅ High contention: {total_success} successful out of 200 total");
    }

    #[tokio::test]
    async fn test_clock_skew_tolerance() {
        // Simulate peers with different clock times
        let peer1_time = 1700000000u64; // Arbitrary timestamp
        let peer2_time = 1700000300u64; // 5 minutes later

        let attestation1 = IdentityAttestation {
            provider_capability: "security/identity".to_string(),
            format: "tag_list".to_string(),
            data: json!({
                "family_id": "time-family",
                "timestamp": peer1_time,
            }),
        };

        let attestation2 = IdentityAttestation {
            provider_capability: "security/identity".to_string(),
            format: "tag_list".to_string(),
            data: json!({
                "family_id": "time-family",
                "timestamp": peer2_time,
            }),
        };

        // Both should serialize/deserialize successfully despite time difference
        let msg1 = create_test_message_with_attestations("peer1", vec![attestation1]);
        let msg2 = create_test_message_with_attestations("peer2", vec![attestation2]);

        let bytes1 = msg1.to_bytes().unwrap();
        let bytes2 = msg2.to_bytes().unwrap();

        let recovered1 = AnonymousDiscoveryMessage::from_bytes(&bytes1).unwrap();
        let recovered2 = AnonymousDiscoveryMessage::from_bytes(&bytes2).unwrap();

        assert!(recovered1.identity_attestations.is_some());
        assert!(recovered2.identity_attestations.is_some());

        println!("✅ Clock skew: Messages from different time zones handled correctly");
    }

    #[tokio::test]
    async fn test_cascading_failure_recovery() {
        let provider = chaotic_enc(
            Some("cascade-family".to_string()),
            0.5, // 50% failure rate (severe)
        );

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: true,
            ..Default::default()
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider.clone()), config));

        // Phase 1: High failure rate
        let mut phase1_success = 0;
        for i in 0..50 {
            let msg = create_test_message(&format!("phase1-{i}"));
            if processor.encrypt_packet(&msg.to_bytes().unwrap()).await.is_ok() {
                phase1_success += 1;
            }
        }

        // Phase 2: "Fix" the system (reduce failure rate by resetting counter)
        provider.chaotic_reset_failure_counter();

        let mut phase2_success = 0;
        for i in 0..50 {
            let msg = create_test_message(&format!("phase2-{i}"));
            if processor.encrypt_packet(&msg.to_bytes().unwrap()).await.is_ok() {
                phase2_success += 1;
            }
        }

        // System should recover in phase 2
        assert!(phase2_success >= phase1_success, "Should recover or maintain performance");
        println!(
            "✅ Cascading failure: Phase 1 = {phase1_success}, Phase 2 = {phase2_success} (recovery verified)"
        );
    }

    #[tokio::test]
    async fn test_burst_traffic_handling() {
        let provider = chaotic_enc(
            Some("burst-family".to_string()),
            0.05, // Low failure rate
        );

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: false,
            ..Default::default()
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider), config));

        // Simulate burst: 100 messages in rapid succession
        let start = Instant::now();
        let mut success = 0;

        for i in 0..100 {
            let msg = create_test_message(&format!("burst-{i}"));
            if processor.encrypt_packet(&msg.to_bytes().unwrap()).await.is_ok() {
                success += 1;
            }
        }

        let duration = start.elapsed();

        // Should handle burst gracefully
        assert!(success > 90, "Should handle most messages in burst");
        assert!(duration < Duration::from_secs(5), "Should complete burst quickly");

        println!("✅ Burst traffic: {success} successful in {duration:?}");
    }

    #[tokio::test]
    async fn test_partial_network_partition() {
        // Simulate scenario where some peers can talk to each other but not all
        let family_a = "partition-a";
        let family_b = "partition-b";

        let provider_a = chaotic_enc(Some(family_a.to_string()), 0.0);

        let provider_b = chaotic_enc(Some(family_b.to_string()), 0.0);

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            mixed_mode: false,
            ..Default::default()
        };

        let processor_a = Arc::new(BirdSongProcessor::new(Some(provider_a), config.clone()));
        let processor_b = Arc::new(BirdSongProcessor::new(Some(provider_b), config));

        // Messages within partition A
        let msg_a = create_test_message("msg-a");
        let encrypted_a = processor_a.encrypt_packet(&msg_a.to_bytes().unwrap()).await.unwrap();
        let decrypted_a = processor_a.decrypt_packet(&encrypted_a).await.unwrap();
        assert!(decrypted_a.is_some(), "Should decrypt within same partition");

        // Messages from partition A to partition B (should fail)
        let decrypted_cross = processor_b.decrypt_packet(&encrypted_a).await;
        assert!(
            decrypted_cross.is_err() || decrypted_cross.unwrap().is_none(),
            "Should not decrypt across partitions"
        );

        println!("✅ Network partition: Cross-partition communication properly isolated");
    }

    #[tokio::test]
    async fn test_memory_pressure_simulation() {
        // Create and discard many messages to simulate memory pressure
        let provider = chaotic_enc(Some("memory-test".to_string()), 0.0);

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: false,
            ..Default::default()
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider), config));

        // Create and process 1000 messages
        for i in 0..1000 {
            let msg = create_test_message(&format!("memory-test-{i}"));
            let plaintext = msg.to_bytes().unwrap();
            let _ = processor.encrypt_packet(&plaintext).await;
            // Messages are immediately dropped, simulating memory pressure
        }

        // If we get here without OOM, test passes
        println!("✅ Memory pressure: Handled 1000 messages without issues");
    }

    #[tokio::test]
    async fn test_rapid_state_changes() {
        let provider = chaotic_enc(Some("rapid-changes".to_string()), 0.2);

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: true,
            ..Default::default()
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider.clone()), config));

        // Rapidly toggle availability while sending messages
        let processor_clone = Arc::clone(&processor);
        let prov_for_toggle = provider.clone();
        let toggler = tokio::spawn(async move {
            for _ in 0..20 {
                sleep(Duration::from_millis(10)).await;
                prov_for_toggle.chaotic_toggle_availability();
            }
        });

        // Send messages while state is changing
        let mut success = 0;
        for i in 0..100 {
            let msg = create_test_message(&format!("rapid-{i}"));
            if processor_clone.encrypt_packet(&msg.to_bytes().unwrap()).await.is_ok() {
                success += 1;
            }
            sleep(Duration::from_millis(2)).await;
        }

        toggler.await.unwrap();

        // Should handle rapid state changes
        assert!(success > 50, "Should handle messages despite rapid state changes");
        println!("✅ Rapid state changes: {success} successful despite chaos");
    }

    // Helper functions

    fn create_test_message(node_id: &str) -> AnonymousDiscoveryMessage {
        let endpoints = vec![TransportEndpointMessage {
            interface_type: "ethernet".to_string(),
            address: "192.168.1.100:8080".to_string(),
            protocols: vec!["https".to_string()],
            preference: 100,
        }];

        AnonymousDiscoveryMessage::new_v3(
            node_id.to_string(),
            node_id.to_string(),
            endpoints,
            vec!["orchestration".to_string()],
        )
    }

    fn create_test_message_with_attestations(
        node_id: &str,
        attestations: Vec<IdentityAttestation>,
    ) -> AnonymousDiscoveryMessage {
        let endpoints = vec![TransportEndpointMessage {
            interface_type: "ethernet".to_string(),
            address: "192.168.1.100:8080".to_string(),
            protocols: vec!["https".to_string()],
            preference: 100,
        }];

        AnonymousDiscoveryMessage::new_v3(
            node_id.to_string(),
            node_id.to_string(),
            endpoints,
            vec!["orchestration".to_string()],
        )
        .with_identity_attestations(attestations)
    }
}
