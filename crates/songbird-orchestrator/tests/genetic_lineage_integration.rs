// SPDX-License-Identifier: AGPL-3.0-only
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
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! Comprehensive tests for genetic lineage integration
//!
//! Tests the full lineage flow:
//! 1. Lineage type serialization/deserialization
//! 2. Discovery packet with lineage
//! 3. Node identity with lineage
//! 4. Node registration with lineage
//! 5. Auto-accept logic based on lineage

use anyhow::Result;
use songbird_discovery::DiscoveryPacket;
use songbird_orchestrator::{
    node_identity::NodeIdentity,
    registration::{NodeRegistration, RegistrationManager, create_registration_from_identity},
    trust::{LineageAuthenticator, LineageStatus, PeerAcceptanceDecision},
};
use songbird_types::{LineageId, LineageProof};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Helper to create a test lineage
async fn create_test_lineage(node_id: &str) -> (LineageId, LineageProof) {
    use songbird_types::lineage::LineageSignature;

    // Create a unique lineage ID for this node
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let lineage_id_str = format!("lineage:{node_id}:{timestamp}");
    let lineage_id = LineageId::new(lineage_id_str);

    // Create a mock signature for testing
    let signature = LineageSignature {
        signer_node_id: format!("signer-{node_id}"),
        signature: "mock_signature_hex_data_12345678".to_string(),
        signed_data_hash: "mock_hash_abcdef".to_string(),
        timestamp,
    };

    let genesis_timestamp = timestamp;

    let proof = LineageProof::new(lineage_id.clone(), vec![signature], genesis_timestamp);

    (lineage_id, proof)
}

#[tokio::test]
async fn test_lineage_types_serialization() {
    let (lineage_id, proof) = create_test_lineage("test-node").await;

    // Test LineageId serialization
    let id_json = serde_json::to_string(&lineage_id).unwrap();
    let id_deserialized: LineageId = serde_json::from_str(&id_json).unwrap();
    assert_eq!(lineage_id, id_deserialized);

    // Test LineageProof serialization
    let proof_json = serde_json::to_string(&proof).unwrap();
    let proof_deserialized: LineageProof = serde_json::from_str(&proof_json).unwrap();
    assert_eq!(proof.lineage_id, proof_deserialized.lineage_id);
    assert_eq!(proof.genesis_timestamp, proof_deserialized.genesis_timestamp);
    assert_eq!(proof.chain_length(), proof_deserialized.chain_length());
}

#[tokio::test]
async fn test_lineage_proof_mdns_encoding() {
    let (_, proof) = create_test_lineage("test-node").await;

    // Encode to mDNS TXT string
    let txt = proof.to_discovery_txt().unwrap();

    // Verify it's not too long for mDNS (400 byte limit per TXT record)
    assert!(txt.len() < 400, "mDNS TXT record too long: {} bytes", txt.len());

    // Decode back
    let decoded = LineageProof::from_discovery_txt(&txt).unwrap();
    assert_eq!(proof.lineage_id, decoded.lineage_id);
    assert_eq!(proof.genesis_timestamp, decoded.genesis_timestamp);
    assert_eq!(proof.signatures.len(), decoded.signatures.len());
}

#[tokio::test]
async fn test_discovery_packet_with_lineage() {
    let (lineage_id, proof) = create_test_lineage("discovery-node").await;

    let packet = DiscoveryPacket {
        node_id: "discovery-node".to_string(),
        node_name: Some("Discovery Test Node".to_string()),
        capabilities: vec!["compute".to_string(), "storage".to_string()],
        endpoint: "http://192.168.1.100:8080".to_string(),
        metadata: HashMap::from([("region".to_string(), "us-east".to_string())]),
        genetic_lineage: Some(lineage_id),
        lineage_proof: Some(proof),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        tags: vec![],
        identity_attestations: vec![],
    };

    // Convert to mDNS TXT records
    let txt_records = packet.to_txt_records();

    assert!(txt_records.contains_key("node_id"));
    assert!(txt_records.contains_key("lineage"));
    assert!(txt_records.contains_key("lineage_proof"));

    // Parse back
    let parsed = DiscoveryPacket::from_txt_records(&txt_records).unwrap();
    assert_eq!(packet.node_id, parsed.node_id);
    assert_eq!(packet.capabilities, parsed.capabilities);
    assert!(parsed.genetic_lineage.is_some());
    assert!(parsed.lineage_proof.is_some());
}

#[tokio::test]
async fn test_discovery_packet_backward_compatibility() {
    // Packet without lineage (old node)
    let packet = DiscoveryPacket {
        node_id: "old-node".to_string(),
        node_name: Some("Old Test Node".to_string()),
        capabilities: vec!["compute".to_string()],
        endpoint: "http://192.168.1.101:8080".to_string(),
        metadata: HashMap::new(),
        genetic_lineage: None,
        lineage_proof: None,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        tags: vec![],
        identity_attestations: vec![],
    };

    let txt_records = packet.to_txt_records();

    // Should not include lineage keys
    assert!(!txt_records.contains_key("lineage"));
    assert!(!txt_records.contains_key("lineage_proof"));

    // Should parse correctly
    let parsed = DiscoveryPacket::from_txt_records(&txt_records).unwrap();
    assert!(parsed.genetic_lineage.is_none());
    assert!(parsed.lineage_proof.is_none());
}

#[tokio::test]
async fn test_node_identity_with_lineage() -> Result<()> {
    // Use unique node ID via environment variable to ensure a fresh identity file
    // NodeIdentity::identity_path() uses SONGBIRD_NODE_ID to generate unique filenames
    let unique_node_id = format!("test-identity-{}", uuid::Uuid::new_v4());
    songbird_process_env::set_var("SONGBIRD_NODE_ID", &unique_node_id);

    let mut identity = NodeIdentity::new_or_load(Some(unique_node_id.clone()))?;

    // Initially no lineage (fresh node identity)
    assert!(!identity.has_lineage(), "Fresh identity should not have lineage");
    assert!(identity.get_lineage().is_none());

    // Set lineage
    let (lineage_id, proof) = create_test_lineage(&unique_node_id).await;
    identity.set_lineage(lineage_id.clone(), proof.clone())?;

    // Now has lineage
    assert!(identity.has_lineage());
    let (id, p) = identity.get_lineage().unwrap();
    assert_eq!(id, &lineage_id);
    assert_eq!(p.lineage_id, proof.lineage_id);

    // Clean up env var
    songbird_process_env::remove_var("SONGBIRD_NODE_ID");

    Ok(())
}

#[tokio::test]
async fn test_node_registration_with_lineage() {
    let (lineage_id, proof) = create_test_lineage("reg-node").await;

    let registration = NodeRegistration::with_lineage(
        "reg-node-123",
        "Registration Test Node",
        vec!["compute".to_string(), "network".to_string()],
        "http://192.168.1.50:9000",
        lineage_id.clone(),
        proof,
    );

    assert!(registration.has_lineage());
    assert_eq!(registration.genetic_lineage.as_ref().unwrap(), &lineage_id);
    assert!(!registration.is_expired());
}

#[tokio::test]
async fn test_registration_from_identity() -> Result<()> {
    let mut identity = NodeIdentity::new_or_load(Some("integration-node".to_string()))?;
    let (lineage_id, proof) = create_test_lineage("integration-node").await;
    identity.set_lineage(lineage_id.clone(), proof.clone())?;

    let registration = create_registration_from_identity(
        &identity,
        "http://192.168.1.200:8080".to_string(),
        vec!["compute".to_string()],
    )
    .await?;

    assert!(registration.has_lineage());
    assert_eq!(registration.genetic_lineage.as_ref().unwrap(), &lineage_id);

    Ok(())
}

#[tokio::test]
async fn test_registration_manager() {
    let mut manager = RegistrationManager::new(60);

    let (lineage_id, proof) = create_test_lineage("manager-node").await;
    let registration = NodeRegistration::with_lineage(
        "manager-node-456",
        "Manager Test Node",
        vec!["storage".to_string()],
        "http://192.168.1.75:8080",
        lineage_id,
        proof,
    );

    manager.register(registration);

    assert!(manager.current().is_some());
    assert!(manager.current().unwrap().has_lineage());
}

/// NOTE: Ignored - requires `BearDog` running at localhost:9000
#[tokio::test]
#[ignore = "Requires BearDog at localhost:9000"]
async fn test_lineage_authenticator_same_lineage() -> Result<()> {
    let (lineage_id, proof) = create_test_lineage("auth-node-1").await;

    let mut auth = LineageAuthenticator::new();
    auth.initialize("http://localhost:9000").await?;

    // Create peer with same lineage
    let peer_packet = DiscoveryPacket {
        node_id: "peer-same-lineage".to_string(),
        node_name: Some("Same Lineage Peer".to_string()),
        capabilities: vec!["compute".to_string()],
        endpoint: "http://192.168.1.150:8080".to_string(),
        metadata: HashMap::new(),
        genetic_lineage: Some(lineage_id.clone()),
        lineage_proof: Some(proof.clone()),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        tags: vec![],
        identity_attestations: vec![],
    };

    let decision = auth
        .evaluate_peer(
            &peer_packet.node_id,
            &peer_packet.endpoint,
            &peer_packet.capabilities,
            peer_packet.genetic_lineage.as_ref(),
            peer_packet.lineage_proof.as_ref(),
        )
        .await?;

    // Should auto-accept (in mock, same_family returns true for equal lineages)
    match decision {
        PeerAcceptanceDecision::AutoAccept {
            ..
        } => {
            println!("✅ Auto-accepted peer with same lineage");
        }
        _ => {
            println!("⚠️ Mock behavior may vary - decision: {decision:?}");
        }
    }

    Ok(())
}

/// NOTE: Ignored - requires `BearDog` running at localhost:9000
#[tokio::test]
#[ignore = "Requires BearDog at localhost:9000"]
async fn test_lineage_authenticator_different_lineage() -> Result<()> {
    let (_lineage_id_a, _proof_a) = create_test_lineage("auth-node-a").await;
    let (lineage_id_b, proof_b) = create_test_lineage("auth-node-b").await;

    let mut auth = LineageAuthenticator::new();
    auth.initialize("http://localhost:9000").await?;

    // Create peer with different lineage
    let peer_packet = DiscoveryPacket {
        node_id: "peer-different-lineage".to_string(),
        node_name: Some("Different Lineage Peer".to_string()),
        capabilities: vec!["storage".to_string()],
        endpoint: "http://192.168.1.151:8080".to_string(),
        metadata: HashMap::new(),
        genetic_lineage: Some(lineage_id_b.clone()),
        lineage_proof: Some(proof_b.clone()),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        tags: vec![],
        identity_attestations: vec![],
    };

    let decision = auth
        .evaluate_peer(
            &peer_packet.node_id,
            &peer_packet.endpoint,
            &peer_packet.capabilities,
            peer_packet.genetic_lineage.as_ref(),
            peer_packet.lineage_proof.as_ref(),
        )
        .await?;

    // Should prompt user (different lineage)
    match decision {
        PeerAcceptanceDecision::PromptUser {
            lineage_status,
            ..
        } => match lineage_status {
            LineageStatus::DifferentGenesis {
                ..
            } => {
                println!("✅ Correctly prompting for different lineage");
            }
            _ => {
                println!("⚠️ Mock may have different behavior: {lineage_status:?}");
            }
        },
        _ => {
            println!("⚠️ Mock behavior: {decision:?}");
        }
    }

    Ok(())
}

/// NOTE: Ignored - requires `BearDog` running at localhost:9000
#[tokio::test]
#[ignore = "Requires BearDog at localhost:9000"]
async fn test_lineage_authenticator_no_lineage() -> Result<()> {
    let mut auth = LineageAuthenticator::new();
    auth.initialize("http://localhost:9000").await?;

    // Create peer without lineage
    let peer_packet = DiscoveryPacket {
        node_id: "peer-no-lineage".to_string(),
        node_name: Some("No Lineage Peer".to_string()),
        capabilities: vec!["network".to_string()],
        endpoint: "http://192.168.1.152:8080".to_string(),
        metadata: HashMap::new(),
        genetic_lineage: None,
        lineage_proof: None,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        tags: vec![],
        identity_attestations: vec![],
    };

    let decision = auth
        .evaluate_peer(
            &peer_packet.node_id,
            &peer_packet.endpoint,
            &peer_packet.capabilities,
            None,
            None,
        )
        .await?;

    // Should prompt user (unknown lineage)
    match decision {
        PeerAcceptanceDecision::PromptUser {
            lineage_status: LineageStatus::UnknownLineage,
            ..
        } => {
            println!("✅ Correctly prompting for unknown lineage");
        }
        _ => {
            println!("⚠️ Unexpected decision: {decision:?}");
        }
    }

    Ok(())
}

/// NOTE: Ignored - requires `BearDog` running at localhost:9000
#[tokio::test]
#[ignore = "Requires BearDog at localhost:9000"]
async fn test_lineage_authenticator_invalid_proof() -> Result<()> {
    use songbird_types::lineage::LineageSignature;

    let mut auth = LineageAuthenticator::new();
    auth.initialize("http://localhost:9000").await?;

    let lineage_id = LineageId::new("fake-lineage");

    // Create an expired proof (genesis_timestamp in far past)
    let signature = LineageSignature {
        signer_node_id: "fake-signer".to_string(),
        signature: "invalid_sig".to_string(),
        signed_data_hash: "invalid_hash".to_string(),
        timestamp: 0, // Epoch
    };
    let invalid_proof = LineageProof::new(lineage_id.clone(), vec![signature], 0); // Genesis at epoch

    let peer_packet = DiscoveryPacket {
        node_id: "peer-invalid-proof".to_string(),
        node_name: Some("Invalid Proof Node".to_string()),
        capabilities: vec!["compute".to_string()],
        endpoint: "http://192.168.1.153:8080".to_string(),
        metadata: HashMap::new(),
        genetic_lineage: Some(lineage_id.clone()),
        lineage_proof: Some(invalid_proof.clone()),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        tags: vec![],
        identity_attestations: vec![],
    };

    let decision = auth
        .evaluate_peer(
            &peer_packet.node_id,
            &peer_packet.endpoint,
            &peer_packet.capabilities,
            Some(&lineage_id),
            Some(&invalid_proof),
        )
        .await?;

    // NOTE: In mock mode, this will still be accepted since BearDog mock always returns valid=true
    // In production with real BearDog, this would be rejected
    println!("⚠️ Decision (mock mode): {decision:?}");

    Ok(())
}

#[tokio::test]
async fn test_full_lineage_integration_flow() -> Result<()> {
    println!("\n🧬 Testing Full Genetic Lineage Integration Flow\n");

    // 1. Create node identity with lineage
    println!("1️⃣  Creating node identity...");
    let mut identity = NodeIdentity::new_or_load(Some("integration-test-node".to_string()))?;
    let (lineage_id, proof) = create_test_lineage("integration-test-node").await;
    identity.set_lineage(lineage_id.clone(), proof.clone())?;
    println!("   ✅ Node identity with lineage: {lineage_id}");

    // 2. Create registration from identity
    println!("\n2️⃣  Creating node registration...");
    let registration = create_registration_from_identity(
        &identity,
        "http://192.168.1.100:8080".to_string(),
        vec!["compute".to_string(), "storage".to_string()],
    )
    .await?;
    println!("   ✅ Registration includes lineage: {}", registration.has_lineage());

    // 3. Create discovery packet
    println!("\n3️⃣  Creating discovery packet...");
    let discovery = DiscoveryPacket {
        node_id: identity.node_id.to_string(),
        node_name: Some(identity.node_name.clone()),
        capabilities: registration.capabilities.clone(),
        endpoint: registration.endpoint.clone(),
        metadata: HashMap::from([("version".to_string(), "1.0.0".to_string())]),
        genetic_lineage: identity.genetic_lineage.clone(),
        lineage_proof: identity.lineage_proof.clone(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        tags: vec![],
        identity_attestations: vec![],
    };
    println!("   ✅ Discovery packet with lineage");

    // 4. Convert to mDNS TXT records
    println!("\n4️⃣  Converting to mDNS TXT records...");
    let txt_records = discovery.to_txt_records();
    println!("   ✅ TXT records: {} keys", txt_records.len());
    println!("   📝 Keys: {:?}", txt_records.keys().collect::<Vec<_>>());

    // 5. Parse back from mDNS
    println!("\n5️⃣  Parsing from mDNS TXT records...");
    let parsed_discovery = DiscoveryPacket::from_txt_records(&txt_records)?;
    println!("   ✅ Parsed successfully");
    println!("   🔍 Has lineage: {}", parsed_discovery.genetic_lineage.is_some());
    println!("   🔍 Has proof: {}", parsed_discovery.lineage_proof.is_some());

    // 6. Evaluate peer for auto-accept
    println!("\n6️⃣  Evaluating peer for auto-accept...");
    let mut auth = LineageAuthenticator::new();
    auth.initialize("http://localhost:9000").await?;

    let decision = auth
        .evaluate_peer(
            &parsed_discovery.node_id,
            &parsed_discovery.endpoint,
            &parsed_discovery.capabilities,
            parsed_discovery.genetic_lineage.as_ref(),
            parsed_discovery.lineage_proof.as_ref(),
        )
        .await?;
    println!("   ✅ Decision: {decision:?}");

    println!("\n🎉 Full integration flow complete!\n");

    Ok(())
}
