// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use crate::birdsong::ProcessorXorMock;

fn xor_mock(family_id: &str, available: bool) -> Arc<BirdSongEncryption> {
    Arc::new(BirdSongEncryption::ProcessorXor(Arc::new(ProcessorXorMock {
        family_id: family_id.to_string(),
        available,
    })))
}

#[tokio::test]
async fn test_birdsong_encryption() {
    let enc = xor_mock("test-family", true);
    let config = BirdSongConfig {
        enabled: true,
        fallback_to_plaintext: false,
        mixed_mode: false,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc), config);

    let message = b"Hello, family!";
    let encrypted = processor.encrypt_packet(message).await.unwrap();
    assert_ne!(&encrypted[..], message, "Should be encrypted");

    let decrypted = processor.decrypt_packet(&encrypted).await.unwrap().unwrap();
    assert_eq!(&decrypted[..], message, "Should decrypt correctly");
}

#[tokio::test]
async fn test_different_family_noise() {
    let enc = xor_mock("test-family", true);
    let config = BirdSongConfig {
        enabled: true,
        fallback_to_plaintext: false,
        mixed_mode: false,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc), config);

    let different_family_packet = vec![0xFF, 0x01, 0x02, 0x03];
    let result = processor.decrypt_packet(&different_family_packet).await.unwrap();

    assert!(result.is_none(), "Should return None for different family");
}

#[tokio::test]
async fn test_plaintext_fallback() {
    let config = BirdSongConfig {
        enabled: false,
        fallback_to_plaintext: true,
        mixed_mode: true,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(None, config);

    let message = b"Plaintext message";
    let result = processor.encrypt_packet(message).await.unwrap();
    assert_eq!(&result[..], message, "Should stay plaintext");

    let decrypted = processor.decrypt_packet(&result).await.unwrap().unwrap();
    assert_eq!(&decrypted[..], message, "Should pass through");
}

#[tokio::test]
async fn test_encryption_disabled() {
    let enc = xor_mock("test", true);
    let config = BirdSongConfig {
        enabled: false,
        fallback_to_plaintext: true,
        mixed_mode: true,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc), config);

    assert!(!processor.is_encrypted());

    let message = b"Message";
    let result = processor.encrypt_packet(message).await.unwrap();
    assert_eq!(&result[..], message, "Should not encrypt when disabled");
}

#[tokio::test]
async fn test_provider_unavailable_with_fallback() {
    let enc = xor_mock("test", false);
    let config = BirdSongConfig {
        enabled: true,
        fallback_to_plaintext: true,
        mixed_mode: true,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc), config);

    let message = b"Message";
    let result = processor.encrypt_packet(message).await.unwrap();
    assert_eq!(&result[..], message, "Should fallback to plaintext");
}

#[tokio::test]
async fn test_mixed_mode() {
    let enc = xor_mock("test", true);
    let config = BirdSongConfig {
        enabled: true,
        fallback_to_plaintext: true,
        mixed_mode: true,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc), config);

    let plaintext_msg = b"plaintext";
    let encrypted_msg = processor.encrypt_packet(b"encrypted").await.unwrap();

    let result1 = processor.decrypt_packet(plaintext_msg).await.unwrap();
    let result2 = processor.decrypt_packet(&encrypted_msg).await.unwrap();

    assert!(result1.is_some(), "Plaintext should work");
    assert!(result2.is_some(), "Encrypted should work");
}

#[tokio::test]
async fn test_status_reporting() {
    let enc = xor_mock("test", true);
    let config = BirdSongConfig {
        enabled: true,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc), config);

    let status = processor.status();
    assert!(status.contains("Encrypted"), "Should report encrypted");
    assert!(status.contains("MockEncryption"), "Should include provider name");
}

#[tokio::test]
async fn test_encrypt_decrypt_roundtrip_preserves_payload() {
    let enc = xor_mock("roundtrip-family", true);
    let config = BirdSongConfig {
        enabled: true,
        fallback_to_plaintext: false,
        mixed_mode: false,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc), config);

    let message = b"discovery payload with unicode: \x00\xff\xfe";
    let encrypted = processor.encrypt_packet(message).await.unwrap();
    let decrypted = processor.decrypt_packet(&encrypted).await.unwrap().unwrap();
    assert_eq!(decrypted, message);
}

#[test]
fn test_is_encrypted_and_encryption_provider_getters() {
    let enc = xor_mock("getter-family", true);
    let config = BirdSongConfig {
        enabled: true,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc.clone()), config);

    assert!(processor.is_encrypted());
    assert!(processor.encryption_provider().is_some());
    assert_eq!(
        processor.encryption_provider().unwrap().family_id(),
        Some("getter-family".to_string())
    );
    assert!(processor.config().enabled);
}

#[test]
fn test_status_plaintext_when_disabled() {
    let processor = BirdSongProcessor::new(None, BirdSongConfig::default());
    assert_eq!(processor.status(), "Plaintext (disabled)");
    assert!(!processor.is_encrypted());
    assert!(processor.encryption_provider().is_none());
}

#[test]
fn test_status_plaintext_when_provider_unavailable() {
    let enc = xor_mock("test", false);
    let config = BirdSongConfig {
        enabled: true,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc), config);

    assert!(!processor.is_encrypted());
    assert_eq!(processor.status(), "Plaintext (provider unavailable)");
}

#[tokio::test]
async fn test_provider_unavailable_without_fallback_errors() {
    let enc = xor_mock("test", false);
    let config = BirdSongConfig {
        enabled: true,
        fallback_to_plaintext: false,
        mixed_mode: false,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc), config);

    let err = processor.encrypt_packet(b"secret").await.unwrap_err();
    assert!(err.to_string().contains("fallback disabled"));
}

#[tokio::test]
async fn test_different_family_birdsong_packet_returns_none() {
    use crate::birdsong::types::BirdSongPacket;
    use base64::{Engine as _, engine::general_purpose};

    let enc = xor_mock("our-family", true);
    let config = BirdSongConfig {
        enabled: true,
        fallback_to_plaintext: false,
        mixed_mode: false,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc), config);

    let packet = BirdSongPacket::new(
        "1.0".to_string(),
        "other-family".to_string(),
        general_purpose::STANDARD.encode(b"xored"),
    );
    let bytes = serde_json::to_vec(&packet).unwrap();

    let result = processor.decrypt_packet(&bytes).await.unwrap();
    assert!(result.is_none(), "Different family_id should be ignored as noise");
}

#[tokio::test]
async fn test_malformed_base64_in_birdsong_packet_errors_without_fallback() {
    use crate::birdsong::types::BirdSongPacket;

    let enc = xor_mock("test-family", true);
    let config = BirdSongConfig {
        enabled: true,
        fallback_to_plaintext: false,
        mixed_mode: false,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc), config);

    let packet = BirdSongPacket::new(
        "1.0".to_string(),
        "test-family".to_string(),
        "not-valid-base64!!!".to_string(),
    );
    let bytes = serde_json::to_vec(&packet).unwrap();

    let err = processor.decrypt_packet(&bytes).await.unwrap_err();
    assert!(err.to_string().contains("base64"));
}

#[tokio::test]
async fn test_encrypted_only_mode_rejects_plaintext_packets() {
    let enc = xor_mock("test", true);
    let config = BirdSongConfig {
        enabled: true,
        fallback_to_plaintext: false,
        mixed_mode: false,
        ..Default::default()
    };
    let processor = BirdSongProcessor::new(Some(enc), config);

    let result = processor.decrypt_packet(b"plain discovery message").await.unwrap();
    assert!(result.is_none(), "Non-BirdSongPacket should be ignored in encrypted-only mode");
}

#[tokio::test]
async fn test_encrypt_dark_forest_beacon_roundtrip() {
    use crate::birdsong::BirdSongEncryption;
    use crate::birdsong::DarkForestTestProvider;
    use crate::dark_forest_beacon::BeaconPayload;

    let seed = [7u8; 32];
    let enc =
        Arc::new(BirdSongEncryption::DarkForestTest(Arc::new(DarkForestTestProvider::new(seed))));
    let config = BirdSongConfig::dark_forest();
    let processor = BirdSongProcessor::new(Some(enc), config);

    let payload = BeaconPayload::new(
        vec![1, 2, 3],
        "beacon-node".to_string(),
        vec!["/ip4/127.0.0.1/tcp/8080".to_string()],
        &["compute".to_string()],
        None,
        "session-1".to_string(),
    );

    let beacon = processor.encrypt_dark_forest_beacon(&payload).await.unwrap();
    let decrypted = processor.decrypt_dark_forest_beacon(&beacon).await.unwrap();
    assert!(decrypted.is_some());
    assert_eq!(decrypted.unwrap().0.node_id, "beacon-node");
}

#[tokio::test]
async fn test_decrypt_dark_forest_beacon_without_provider_returns_none() {
    use crate::dark_forest_beacon::DarkForestBeacon;

    let processor = BirdSongProcessor::new(None, BirdSongConfig::default());
    let beacon = DarkForestBeacon::new(vec![1, 2, 3], [0u8; 12]);

    let result = processor.decrypt_dark_forest_beacon(&beacon).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_encrypt_dark_forest_beacon_without_provider_errors() {
    use crate::dark_forest_beacon::BeaconPayload;

    let processor = BirdSongProcessor::new(None, BirdSongConfig::default());
    let payload =
        BeaconPayload::new(vec![], "node".to_string(), vec![], &[], None, "s".to_string());

    let err = processor.encrypt_dark_forest_beacon(&payload).await.unwrap_err();
    assert!(err.to_string().contains("No encryption provider"));
}

#[tokio::test]
async fn test_encrypt_dark_forest_beacon_unavailable_provider_errors() {
    use crate::dark_forest_beacon::BeaconPayload;

    let enc = xor_mock("test", false);
    let processor = BirdSongProcessor::new(Some(enc), BirdSongConfig::default());
    let payload =
        BeaconPayload::new(vec![], "node".to_string(), vec![], &[], None, "s".to_string());

    let err = processor.encrypt_dark_forest_beacon(&payload).await.unwrap_err();
    assert!(err.to_string().contains("not available"));
}
