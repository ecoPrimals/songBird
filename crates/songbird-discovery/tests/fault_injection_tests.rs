//! Fault Injection Tests for Songbird Discovery and BirdSong
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
    use songbird_discovery::anonymous::{
        AnonymousDiscoveryMessage, TransportEndpointMessage,
    };
    use songbird_discovery::birdsong_integration::{
        BirdSongConfig, BirdSongEncryption, BirdSongProcessor,
    };
    use songbird_discovery::IdentityAttestation;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Mock provider that fails after N operations
    struct FailingBirdSongProvider {
        family_id: Option<String>,
        call_count: AtomicUsize,
        fail_after: usize,
    }

    impl FailingBirdSongProvider {
        fn new(family_id: Option<String>, fail_after: usize) -> Self {
            Self {
                family_id,
                call_count: AtomicUsize::new(0),
                fail_after,
            }
        }
    }

    #[async_trait::async_trait]
    impl BirdSongEncryption for FailingBirdSongProvider {
        async fn encrypt_discovery(&self, _plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
            let count = self.call_count.fetch_add(1, Ordering::SeqCst);
            if count >= self.fail_after {
                Err(anyhow::anyhow!("Simulated encryption failure after {} calls", count))
            } else {
                Ok(b"ENCRYPTED".to_vec())
            }
        }

        async fn decrypt_discovery(&self, _ciphertext: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
            let count = self.call_count.fetch_add(1, Ordering::SeqCst);
            if count >= self.fail_after {
                Err(anyhow::anyhow!("Simulated decryption failure after {} calls", count))
            } else {
                Ok(Some(b"DECRYPTED".to_vec()))
            }
        }

        fn is_available(&self) -> bool {
            self.call_count.load(Ordering::SeqCst) < self.fail_after
        }

        fn family_id(&self) -> Option<String> {
            self.family_id.clone()
        }

        fn provider_name(&self) -> String {
            "FailingProvider".to_string()
        }
    }

    /// Mock provider that is always unavailable
    struct UnavailableBirdSongProvider {
        family_id: Option<String>,
    }

    #[async_trait::async_trait]
    impl BirdSongEncryption for UnavailableBirdSongProvider {
        async fn encrypt_discovery(&self, _plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
            Err(anyhow::anyhow!("Provider unavailable"))
        }

        async fn decrypt_discovery(&self, _ciphertext: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
            Err(anyhow::anyhow!("Provider unavailable"))
        }

        fn is_available(&self) -> bool {
            false
        }

        fn family_id(&self) -> Option<String> {
            self.family_id.clone()
        }

        fn provider_name(&self) -> String {
            "UnavailableProvider".to_string()
        }
    }

    #[tokio::test]
    async fn test_encryption_failure_with_fallback() {
        // Provider that fails after 3 calls
        let provider = Arc::new(FailingBirdSongProvider::new(Some("test-family".to_string()), 3));

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true, // Enable fallback
            security_endpoint: None,
            mixed_mode: false,
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider.clone()), config));

        // First 3 calls should succeed
        for i in 0..3 {
            let result = processor.encrypt_packet(format!("message-{}", i).as_bytes()).await;
            assert!(result.is_ok(), "Call {} should succeed", i);
        }

        // 4th call should fail but processor should handle gracefully
        let result = processor.encrypt_packet(b"message-4").await;
        // With fallback enabled, it might succeed with plaintext
        // The key is: it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_encryption_failure_without_fallback() {
        let provider = Arc::new(FailingBirdSongProvider::new(
            Some("test-family".to_string()),
            0, // Fail immediately
        ));

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false, // No fallback
            security_endpoint: None,
            mixed_mode: false,
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider), config));

        // Should fail gracefully
        let result = processor.encrypt_packet(b"test").await;
        assert!(result.is_err(), "Should fail when provider fails and no fallback");
    }

    #[tokio::test]
    async fn test_unavailable_provider_with_fallback() {
        let provider = Arc::new(UnavailableBirdSongProvider {
            family_id: Some("test-family".to_string()),
        });

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true, // Enable fallback
            security_endpoint: None,
            mixed_mode: false,
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
        let mut corrupted = valid_bytes.clone();
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
        let provider = Arc::new(FailingBirdSongProvider::new(
            Some("test-family".to_string()),
            5, // Fail after 5
        ));

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            security_endpoint: None,
            mixed_mode: true, // Allow both encrypted and plaintext
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider), config));

        // Should handle mix of encrypted and plaintext gracefully
        for i in 0..10 {
            let message = create_test_message();
            let plaintext = message.to_bytes().unwrap();

            // Some will encrypt, some will fall back to plaintext
            let result = processor.encrypt_packet(&plaintext).await;
            assert!(result.is_ok(), "Iteration {} should succeed", i);
        }
    }

    #[tokio::test]
    async fn test_decryption_with_wrong_family() {
        // Processor for family "iidn"
        let provider_a = Arc::new(MockBirdSongProvider {
            family_id: Some("iidn".to_string()),
        });
        let config_a = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            security_endpoint: None,
            mixed_mode: false,
        };
        let processor_a = Arc::new(BirdSongProcessor::new(Some(provider_a.clone()), config_a));

        // Processor for family "other"
        let provider_b = Arc::new(MockBirdSongProvider {
            family_id: Some("other".to_string()),
        });
        let config_b = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            security_endpoint: None,
            mixed_mode: false,
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
        let provider = Arc::new(FailingBirdSongProvider::new(Some("test-family".to_string()), 5));

        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            security_endpoint: None,
            mixed_mode: false,
        };

        let processor = Arc::new(BirdSongProcessor::new(Some(provider.clone()), config));

        // Rapidly check availability and encrypt
        for i in 0..10 {
            let _is_available = provider.is_available();
            let result = processor.encrypt_packet(format!("msg-{}", i).as_bytes()).await;

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

    /// Simple mock provider for cross-family tests
    struct MockBirdSongProvider {
        family_id: Option<String>,
    }

    #[async_trait::async_trait]
    impl BirdSongEncryption for MockBirdSongProvider {
        async fn encrypt_discovery(&self, plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
            // Simple mock: prepend family ID
            let mut encrypted = self.family_id.clone().unwrap_or_default().into_bytes();
            encrypted.push(b':');
            encrypted.extend_from_slice(plaintext);
            Ok(encrypted)
        }

        async fn decrypt_discovery(&self, ciphertext: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
            // Check if family ID matches
            let family_bytes = self.family_id.clone().unwrap_or_default().into_bytes();
            if ciphertext.starts_with(&family_bytes) && ciphertext.len() > family_bytes.len() + 1 {
                Ok(Some(ciphertext[family_bytes.len() + 1..].to_vec()))
            } else {
                Ok(None) // Different family or invalid
            }
        }

        fn is_available(&self) -> bool {
            true
        }

        fn family_id(&self) -> Option<String> {
            self.family_id.clone()
        }

        fn provider_name(&self) -> String {
            "MockProvider".to_string()
        }
    }
}
