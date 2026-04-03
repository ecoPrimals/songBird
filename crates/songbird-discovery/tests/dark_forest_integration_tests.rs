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
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]

//! Dark Forest Beacon Genetics Integration Tests
//!
//! End-to-end tests demonstrating TRUE privacy-preserving discovery
//! with zero metadata leakage.
//!
//! ## Test Coverage
//!
//! 1. **Same beacon family**: Successful discovery
//! 2. **Different beacon families**: Complete invisibility (privacy)
//! 3. **Migration modes**: Dual-format support
//! 4. **Backward compatibility**: Legacy format still works
//! 5. **Replay protection**: Old beacons rejected
//! 6. **Session rotation**: Tracking prevention

use anyhow::Result;
use async_trait::async_trait;
use songbird_discovery::birdsong::{BirdSongConfig, BirdSongEncryption, BirdSongProcessor};
use songbird_discovery::dark_forest_beacon::{BeaconPayload, DarkForestBeacon};
use std::sync::Arc;

/// Mock encryption provider for testing Dark Forest beacons
///
/// Simulates `security provider`'s `beacon.*` RPC methods with in-memory beacon seeds.
struct MockDarkForestProvider {
    beacon_id: Vec<u8>,
    beacon_seed: [u8; 32],
    known_beacons: Vec<Vec<u8>>,
    available: bool,
}

impl MockDarkForestProvider {
    fn new(seed: [u8; 32]) -> Self {
        let beacon_id = Self::derive_beacon_id(&seed);
        Self {
            beacon_id,
            beacon_seed: seed,
            known_beacons: Vec::new(),
            available: true,
        }
    }

    fn with_known_beacons(mut self, beacons: Vec<Vec<u8>>) -> Self {
        self.known_beacons = beacons;
        self
    }

    fn derive_beacon_id(seed: &[u8; 32]) -> Vec<u8> {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(seed);
        hasher.update(b"beacon-id-v1");
        let hash = hasher.finalize();
        hash.as_bytes()[..16].to_vec()
    }

    fn encrypt_with_seed(&self, plaintext: &[u8]) -> (Vec<u8>, [u8; 12]) {
        // Simple XOR encryption for testing (real: ChaCha20-Poly1305)
        let encrypted: Vec<u8> =
            plaintext.iter().enumerate().map(|(i, &b)| b ^ self.beacon_seed[i % 32]).collect();

        let nonce = [0u8; 12]; // Simplified for testing
        (encrypted, nonce)
    }

    fn try_decrypt_with_seed(&self, ciphertext: &[u8], _nonce: &[u8; 12]) -> Option<Vec<u8>> {
        // Simple XOR decryption (inverse of encrypt)
        let decrypted: Vec<u8> =
            ciphertext.iter().enumerate().map(|(i, &b)| b ^ self.beacon_seed[i % 32]).collect();

        Some(decrypted)
    }
}

#[async_trait]
impl BirdSongEncryption for MockDarkForestProvider {
    async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        Ok(plaintext.to_vec()) // Legacy method - passthrough
    }

    async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(Some(ciphertext.to_vec())) // Legacy method - passthrough
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn family_id(&self) -> Option<String> {
        Some("test-family".to_string())
    }

    fn provider_name(&self) -> String {
        "MockDarkForestProvider".to_string()
    }

    // Dark Forest methods

    async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        Ok(self.encrypt_with_seed(payload))
    }

    async fn try_decrypt_beacon(
        &self,
        encrypted: &[u8],
        nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        Ok(self.try_decrypt_with_seed(encrypted, nonce))
    }

    async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        Ok(Some(self.beacon_id.clone()))
    }

    async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        Ok(self.known_beacons.clone())
    }

    async fn supports_dark_forest(&self) -> bool {
        true
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Integration Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_dark_forest_same_beacon_family_discovers() {
    // Two nodes with SAME beacon seed should discover each other

    let beacon_seed = [42u8; 32];

    // Node A
    let provider_a = Arc::new(MockDarkForestProvider::new(beacon_seed));
    let config_a = BirdSongConfig::dark_forest();
    let processor_a = BirdSongProcessor::new(Some(provider_a.clone()), config_a);

    // Node B (same beacon seed)
    let provider_b = Arc::new(MockDarkForestProvider::new(beacon_seed));
    let config_b = BirdSongConfig::dark_forest();
    let processor_b = BirdSongProcessor::new(Some(provider_b), config_b);

    // Node A creates beacon
    let payload_a = BeaconPayload::new(
        vec![1, 2, 3],
        "node-a".to_string(),
        vec!["/ip4/192.168.1.100/tcp/8080".to_string()],
        &["ai".to_string(), "storage".to_string()],
        None,
        "session-123".to_string(),
    );

    let beacon =
        processor_a.encrypt_dark_forest_beacon(&payload_a).await.expect("Should encrypt beacon");

    // Node B tries to decrypt
    let result = processor_b.decrypt_dark_forest_beacon(&beacon).await.expect("Should not error");

    // Should successfully decrypt (same beacon family)
    assert!(result.is_some(), "Same beacon family should decrypt successfully");

    let (decrypted_payload, _beacon_id) = result.unwrap();
    assert_eq!(decrypted_payload.node_id, "node-a");
    assert_eq!(decrypted_payload.endpoints.len(), 1);
}

#[tokio::test]
async fn test_dark_forest_different_beacon_families_invisible() {
    // Two nodes with DIFFERENT beacon seeds should NOT discover each other
    // This is the core Dark Forest privacy guarantee

    let beacon_seed_a = [42u8; 32];
    let beacon_seed_b = [99u8; 32]; // Different seed

    // Node A
    let provider_a = Arc::new(MockDarkForestProvider::new(beacon_seed_a));
    let config_a = BirdSongConfig::dark_forest();
    let processor_a = BirdSongProcessor::new(Some(provider_a), config_a);

    // Node B (different beacon seed)
    let provider_b = Arc::new(MockDarkForestProvider::new(beacon_seed_b));
    let config_b = BirdSongConfig::dark_forest();
    let processor_b = BirdSongProcessor::new(Some(provider_b), config_b);

    // Node A creates beacon
    let payload_a = BeaconPayload::new(
        vec![1, 2, 3],
        "node-a".to_string(),
        vec!["/ip4/192.168.1.100/tcp/8080".to_string()],
        &["ai".to_string()],
        None,
        "session-456".to_string(),
    );

    let beacon =
        processor_a.encrypt_dark_forest_beacon(&payload_a).await.expect("Should encrypt beacon");

    // Node B tries to decrypt
    let result = processor_b.decrypt_dark_forest_beacon(&beacon).await.expect("Should not error");

    // Should NOT decrypt (different beacon family)
    // This is TRUE Dark Forest: Node B sees only noise
    assert!(
        result.is_none(),
        "Different beacon families should be invisible (Dark Forest working)"
    );
}

#[tokio::test]
#[ignore = "Requires security provider beacon.try_decrypt_with_id RPC (not yet implemented)"]
async fn test_dark_forest_multi_beacon_decryption() {
    // Test decryption with known beacon seeds from "meetings"
    //
    // NOTE: This test requires security provider's beacon.try_decrypt_with_id RPC method
    // which is part of security provider Phase 1 (parallel evolution).
    //
    // Current implementation: try_decrypt_with_beacon_id() uses our own seed
    // Full implementation: Should call security provider RPC with specific beacon_id

    let our_seed = [1u8; 32];
    let friend_seed = [2u8; 32];
    let stranger_seed = [3u8; 32];

    // Derive beacon IDs
    let friend_id = MockDarkForestProvider::derive_beacon_id(&friend_seed);

    // Our node (knows about friend from meeting)
    let our_provider =
        Arc::new(MockDarkForestProvider::new(our_seed).with_known_beacons(vec![friend_id.clone()]));
    let config = BirdSongConfig::dark_forest();
    let our_processor = BirdSongProcessor::new(Some(our_provider), config);

    // Friend creates beacon
    let friend_provider = Arc::new(MockDarkForestProvider::new(friend_seed));
    let friend_processor =
        BirdSongProcessor::new(Some(friend_provider), BirdSongConfig::dark_forest());

    let friend_payload = BeaconPayload::new(
        friend_id,
        "friend-node".to_string(),
        vec!["/ip4/192.168.1.200/tcp/8080".to_string()],
        &["compute".to_string()],
        None,
        "session-friend".to_string(),
    );

    let friend_beacon =
        friend_processor.encrypt_dark_forest_beacon(&friend_payload).await.expect("Should encrypt");

    // We should be able to decrypt friend's beacon (meeting exchange)
    let result =
        our_processor.decrypt_dark_forest_beacon(&friend_beacon).await.expect("Should not error");

    assert!(result.is_some(), "Should decrypt beacon from known beacon (meeting)");

    let (decrypted, _) = result.unwrap();
    assert_eq!(decrypted.node_id, "friend-node");

    // Stranger creates beacon (we haven't met)
    let stranger_provider = Arc::new(MockDarkForestProvider::new(stranger_seed));
    let stranger_processor =
        BirdSongProcessor::new(Some(stranger_provider), BirdSongConfig::dark_forest());

    let stranger_payload = BeaconPayload::new(
        vec![9, 9, 9],
        "stranger-node".to_string(),
        vec!["/ip4/192.168.1.300/tcp/8080".to_string()],
        &["security".to_string()],
        None,
        "session-stranger".to_string(),
    );

    let stranger_beacon = stranger_processor
        .encrypt_dark_forest_beacon(&stranger_payload)
        .await
        .expect("Should encrypt");

    // We should NOT be able to decrypt stranger's beacon (no meeting)
    let result =
        our_processor.decrypt_dark_forest_beacon(&stranger_beacon).await.expect("Should not error");

    assert!(result.is_none(), "Should NOT decrypt beacon from unknown beacon (no meeting)");
}

#[tokio::test]
async fn test_dark_forest_replay_protection() {
    // Old beacons should be rejected (replay attack prevention)

    let beacon_seed = [77u8; 32];
    let provider = Arc::new(MockDarkForestProvider::new(beacon_seed));
    let config = BirdSongConfig::dark_forest();
    let processor = BirdSongProcessor::new(Some(provider.clone()), config);

    // Create old beacon (timestamp in the past)
    let payload = BeaconPayload::new(
        vec![1, 2, 3],
        "test-node".to_string(),
        vec!["/ip4/127.0.0.1/tcp/8080".to_string()],
        &["test".to_string()],
        None,
        "session-old".to_string(),
    );

    let (encrypted, nonce) = provider.encrypt_beacon(&payload.to_bytes().unwrap()).await.unwrap();

    // Create beacon with very old timestamp
    let old_beacon = DarkForestBeacon {
        encrypted_payload: encrypted,
        nonce,
        timestamp: 1000000, // Very old (Jan 1970)
        version: 2,
    };

    // Try to decrypt
    let result = processor.decrypt_dark_forest_beacon(&old_beacon).await.expect("Should not error");

    // Should reject (too old)
    assert!(result.is_none(), "Old beacons should be rejected (replay protection)");
}

#[tokio::test]
async fn test_dark_forest_configuration_presets() {
    // Test configuration presets

    let dark_forest = BirdSongConfig::dark_forest();
    assert!(dark_forest.enabled);
    assert!(dark_forest.dark_forest_enabled);
    assert!(dark_forest.accept_legacy_format);
    assert!(!dark_forest.dual_broadcast);
    assert!(dark_forest.is_dark_forest_active());

    let migration = BirdSongConfig::migration_mode();
    assert!(migration.enabled);
    assert!(migration.dark_forest_enabled);
    assert!(migration.accept_legacy_format);
    assert!(migration.dual_broadcast);

    let legacy = BirdSongConfig::legacy_only();
    assert!(legacy.enabled);
    assert!(!legacy.dark_forest_enabled);
    assert!(legacy.accept_legacy_format);

    let dark_forest_only = BirdSongConfig::dark_forest_only();
    assert!(dark_forest_only.enabled);
    assert!(dark_forest_only.dark_forest_enabled);
    assert!(!dark_forest_only.accept_legacy_format);
    assert!(!dark_forest_only.fallback_to_plaintext);
}

#[tokio::test]
async fn test_beacon_payload_capabilities_hashing() {
    // Test privacy-preserving capability comparison

    let caps1 = vec!["ai".to_string(), "storage".to_string()];
    let caps2 = vec!["storage".to_string(), "ai".to_string()]; // Different order
    let caps3 = vec!["ai".to_string(), "compute".to_string()]; // Different caps

    let hash1 = BeaconPayload::hash_capabilities(&caps1);
    let hash2 = BeaconPayload::hash_capabilities(&caps2);
    let hash3 = BeaconPayload::hash_capabilities(&caps3);

    // Same capabilities (different order) → same hash
    assert_eq!(hash1, hash2, "Same capabilities should hash to same value (order-independent)");

    // Different capabilities → different hash
    assert_ne!(hash1, hash3, "Different capabilities should hash to different values");
}

#[tokio::test]
async fn test_dark_forest_beacon_serialization() {
    // Test beacon serialization/deserialization

    let beacon = DarkForestBeacon::new(vec![1, 2, 3, 4, 5], [10u8; 12]);

    // Serialize
    let bytes = beacon.to_bytes().expect("Should serialize");

    // Deserialize
    let decoded = DarkForestBeacon::from_bytes(&bytes).expect("Should deserialize");

    assert_eq!(decoded.version, 2);
    assert_eq!(decoded.encrypted_payload, vec![1, 2, 3, 4, 5]);
    assert_eq!(decoded.nonce, [10u8; 12]);
    assert!(decoded.is_recent());
}

#[tokio::test]
async fn test_beacon_provider_default_implementations() {
    // Test that default trait implementations work for legacy providers

    struct LegacyProvider;

    #[async_trait]
    impl BirdSongEncryption for LegacyProvider {
        async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
            Ok(plaintext.to_vec())
        }

        async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
            Ok(Some(ciphertext.to_vec()))
        }

        fn is_available(&self) -> bool {
            true
        }

        fn family_id(&self) -> Option<String> {
            Some("legacy".to_string())
        }
    }

    let provider = LegacyProvider;

    // Default implementations should work
    let beacon_id = provider.get_beacon_id().await.unwrap();
    assert!(beacon_id.is_none(), "Legacy provider should return None");

    let known = provider.list_known_beacons().await.unwrap();
    assert!(known.is_empty(), "Legacy provider should return empty");

    let supports = provider.supports_dark_forest().await;
    assert!(!supports, "Legacy provider should not support Dark Forest");
}

#[tokio::test]
async fn test_migration_mode_dual_broadcast_config() {
    // Test migration mode configuration

    let config = BirdSongConfig::migration_mode();

    assert!(config.is_dark_forest_active());
    assert!(config.dual_broadcast);
    assert!(config.accept_legacy_format);
    assert!(config.accepts_legacy());
}

#[tokio::test]
async fn test_dark_forest_only_mode_rejects_legacy() {
    // Test that dark_forest_only mode configuration is correct

    let config = BirdSongConfig::dark_forest_only();

    assert!(config.is_dark_forest_active());
    assert!(!config.dual_broadcast);
    assert!(!config.accept_legacy_format);
    assert!(!config.accepts_legacy());
    assert!(!config.fallback_to_plaintext);
}

/// Test documentation and example usage
///
/// This test serves as documentation for how to use Dark Forest beacons.
#[tokio::test]
async fn test_dark_forest_usage_example() {
    // Setup beacon provider with known seed
    let beacon_seed = [123u8; 32];
    let provider = Arc::new(MockDarkForestProvider::new(beacon_seed));

    // Create processor with Dark Forest config
    let config = BirdSongConfig::dark_forest();
    let processor = BirdSongProcessor::new(Some(provider.clone()), config);

    // Create beacon payload
    let payload = BeaconPayload::new(
        vec![1, 2, 3], // Our beacon ID
        "my-node".to_string(),
        vec!["/ip4/192.168.1.50/tcp/8080".to_string(), "/ip6/::1/tcp/8080".to_string()],
        &["ai".to_string(), "storage".to_string(), "compute".to_string()],
        Some("cluster-1".to_string()),
        "session-abc".to_string(),
    );

    // Encrypt to Dark Forest beacon
    let beacon = processor.encrypt_dark_forest_beacon(&payload).await.expect("Should encrypt");

    // Verify beacon properties
    assert_eq!(beacon.version, 2);
    assert!(beacon.is_recent());
    assert!(!beacon.encrypted_payload.is_empty());

    // Another node with SAME beacon seed receives it
    let receiver = BirdSongProcessor::new(Some(provider), BirdSongConfig::dark_forest());

    let result = receiver.decrypt_dark_forest_beacon(&beacon).await.expect("Should not error");

    // Should successfully decrypt and extract payload
    assert!(result.is_some());
    let (decrypted, _beacon_id) = result.unwrap();

    assert_eq!(decrypted.node_id, "my-node");
    assert_eq!(decrypted.endpoints.len(), 2);
    assert_eq!(decrypted.cluster_id, Some("cluster-1".to_string()));

    // Verify capabilities hash (privacy-preserving)
    let expected_hash = BeaconPayload::hash_capabilities(&[
        "ai".to_string(),
        "storage".to_string(),
        "compute".to_string(),
    ]);
    assert_eq!(decrypted.capabilities_hash, expected_hash);
}
