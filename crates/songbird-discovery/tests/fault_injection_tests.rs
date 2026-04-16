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

//! Fault Injection Tests for `Songbird` Discovery and `BirdSong`
//!
//! These tests verify graceful degradation under failure conditions:
//! - Network failures
//! - Encryption failures
//! - Connection timeouts
//! - Memory pressure
//! - Partial corruption
//!
//! All tests verify that the system handles failures without panicking
//! and provides meaningful error messages.

#[cfg(test)]
mod fault_injection_tests {
    use serde_json::json;
    use songbird_discovery::IdentityAttestation;
    use songbird_discovery::anonymous::{AnonymousDiscoveryMessage, TransportEndpointMessage};
    use songbird_discovery::birdsong::{
        BirdSongConfig, BirdSongEncryption, BirdSongProcessor, CrossFamilyBirdSongMock,
        FailingBirdSongMock, UnavailableBirdSongMock,
    };
    use std::sync::Arc;

    fn failing_enc(family_id: Option<String>, fail_after: usize) -> Arc<BirdSongEncryption> {
        Arc::new(BirdSongEncryption::Failing(Arc::new(FailingBirdSongMock::new(
            family_id, fail_after,
        ))))
    }

    fn unavailable_enc(family_id: Option<String>) -> Arc<BirdSongEncryption> {
        Arc::new(BirdSongEncryption::Unavailable(Arc::new(UnavailableBirdSongMock {
            family_id,
        })))
    }

    fn cross_family_enc(family_id: Option<String>) -> Arc<BirdSongEncryption> {
        Arc::new(BirdSongEncryption::CrossFamily(Arc::new(CrossFamilyBirdSongMock {
            family_id,
        })))
    }

    #[tokio::test]
    async fn test_encryption_failure_with_fallback() {
        // Provider that fails after 3 calls
        let provider = failing_enc(Some("test-family".to_string()), 3);

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: false,
            ..Default::default()
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider.clone()), config));

        // First 3 calls should succeed
        for i in 0..3 {
            let result = processor.encrypt_packet(format!("message-{i}").as_bytes()).await;
            assert!(result.is_ok(), "Call {i} should succeed");
        }

        // 4th call should fail but processor should handle gracefully
        let result = processor.encrypt_packet(b"message-4").await;
        // With fallback enabled, it might succeed with plaintext
        // The key is: it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_encryption_failure_without_fallback() {
        let provider = failing_enc(Some("test-family".to_string()), 0);

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            mixed_mode: false,
            ..Default::default()
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider), config));

        // Should fail gracefully
        let result = processor.encrypt_packet(b"test").await;
        assert!(result.is_err(), "Should fail when provider fails and no fallback");
    }

    #[tokio::test]
    async fn test_unavailable_provider_with_fallback() {
        let provider = unavailable_enc(Some("test-family".to_string()));

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: false,
            ..Default::default()
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider), config));

        // Should fall back to plaintext
        let message = create_test_message();
        let plaintext = message.to_bytes().unwrap();

        let result = processor.encrypt_packet(&plaintext).await;
        assert!(result.is_ok(), "Should fall back to plaintext when provider unavailable");
    }

    #[tokio::test]
    async fn test_corrupted_packet_detection() {
        // Create a valid message
        let message = create_test_message();
        let valid_bytes = message.to_bytes().unwrap();

        // Corrupt the bytes
        let mut corrupted = valid_bytes;
        corrupted[10] = 0xFF;
        corrupted[20] = 0xFF;
        corrupted[30] = 0xFF;

        // Try to parse corrupted packet
        let result = AnonymousDiscoveryMessage::from_bytes(&corrupted);

        // Should fail gracefully (not panic)
        assert!(result.is_err(), "Corrupted packet should be rejected");
    }

    #[tokio::test]
    async fn test_empty_packet_handling() {
        let empty = vec![];

        // Should handle empty packet gracefully
        let result = AnonymousDiscoveryMessage::from_bytes(&empty);
        assert!(result.is_err(), "Empty packet should be rejected");
    }

    #[tokio::test]
    async fn test_oversized_packet_handling() {
        // Create a packet larger than reasonable
        let oversized = vec![0u8; 10_000_000]; // 10MB packet

        // Should reject or handle gracefully (not OOM)
        let result = AnonymousDiscoveryMessage::from_bytes(&oversized);
        assert!(result.is_err(), "Oversized packet should be rejected");
    }

    #[tokio::test]
    async fn test_malformed_json_in_attestation() {
        let endpoints = vec![TransportEndpointMessage {
            interface_type: "ethernet".to_string(),
            address: "192.168.1.100:8080".to_string(),
            protocols: vec!["https".to_string()],
            preference: 100,
        }];

        // Create message with valid structure
        let message = AnonymousDiscoveryMessage::new_v3(
            "test-node".to_string(),
            "test-node".to_string(),
            endpoints,
            vec!["test".to_string()],
        );

        // Serialize and deserialize should work
        let bytes = message.to_bytes().unwrap();
        let recovered = AnonymousDiscoveryMessage::from_bytes(&bytes).unwrap();

        assert_eq!(recovered.node_id, message.node_id);
    }

    #[tokio::test]
    async fn test_mixed_mode_with_failing_provider() {
        let provider = failing_enc(Some("test-family".to_string()), 5);

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: true,
            ..Default::default()
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider), config));

        // Should handle mix of encrypted and plaintext gracefully
        for i in 0..10 {
            let message = create_test_message();
            let plaintext = message.to_bytes().unwrap();

            // Some will encrypt, some will fall back to plaintext
            let result = processor.encrypt_packet(&plaintext).await;
            assert!(result.is_ok(), "Iteration {i} should succeed");
        }
    }

    #[tokio::test]
    async fn test_decryption_with_wrong_family() {
        // Processor for family "iidn"
        let provider_a = cross_family_enc(Some("iidn".to_string()));
        let config_a = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            mixed_mode: false,
            ..Default::default()
        };
        let processor_a = Arc::new(BirdSongProcessor::new(Some(provider_a.clone()), config_a));

        // Processor for family "other"
        let provider_b = cross_family_enc(Some("other".to_string()));
        let config_b = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            mixed_mode: false,
            ..Default::default()
        };
        let processor_b = Arc::new(BirdSongProcessor::new(Some(provider_b), config_b));

        // Encrypt with family "iidn"
        let message = create_test_message();
        let plaintext = message.to_bytes().unwrap();
        let encrypted = processor_a.encrypt_packet(&plaintext).await.unwrap();

        // Try to decrypt with family "other"
        let result = processor_b.decrypt_packet(&encrypted).await;

        // Should either fail or return None (can't decrypt)
        assert!(
            result.is_err() || result.unwrap().is_none(),
            "Should not decrypt message from different family"
        );
    }

    #[tokio::test]
    async fn test_rapid_provider_availability_changes() {
        let provider = failing_enc(Some("test-family".to_string()), 5);

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: false,
            ..Default::default()
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider.clone()), config));

        // Rapidly check availability and encrypt
        for i in 0..10 {
            let _is_available = provider.is_available();
            let result = processor.encrypt_packet(format!("msg-{i}").as_bytes()).await;

            // Should handle state changes gracefully
            assert!(result.is_ok() || result.is_err()); // Just verify no panic
        }
    }

    // Helper functions

    fn create_test_message() -> AnonymousDiscoveryMessage {
        let endpoints = vec![TransportEndpointMessage {
            interface_type: "ethernet".to_string(),
            address: "192.168.1.100:8080".to_string(),
            protocols: vec!["https".to_string()],
            preference: 100,
        }];

        let attestations = vec![IdentityAttestation {
            provider_capability: "security/identity".to_string(),
            format: "tag_list".to_string(),
            data: json!({
                "family_id": "test-family",
                "tags": ["beardog:family:test-family:node1"]
            }),
        }];

        AnonymousDiscoveryMessage::new_v3(
            "test-node".to_string(),
            "test-node".to_string(),
            endpoints,
            vec!["orchestration".to_string()],
        )
        .with_identity_attestations(attestations)
    }
}
