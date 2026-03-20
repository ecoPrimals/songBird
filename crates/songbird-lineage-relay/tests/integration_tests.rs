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
    clippy::must_use_candidate
)]

//! Integration tests for lineage relay system

use songbird_lineage_relay::beardog::{
    MockBirdSongCrypto, MockLineageProvider, MockRelayAuthority,
};
use songbird_lineage_relay::birdsong::{BirdSongBroadcaster, BirdSongCrypto, LineageHint};
use songbird_lineage_relay::coordinator::{LineageRelayConfig, LineageRelayCoordinator};
use songbird_lineage_relay::relay::RelayAuthority;
use songbird_lineage_relay::types::NodeId;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::test]
async fn test_lineage_based_relay_system() {
    // Setup: Create lineage graph
    // grandparent → parent → child
    let lineage_provider = Arc::new(MockLineageProvider::new());
    lineage_provider.add_lineage("child", "parent").await;
    lineage_provider.add_lineage("parent", "grandparent").await;

    // Create crypto for child node
    let crypto = Arc::new(MockBirdSongCrypto::new(lineage_provider.clone(), "child".to_string()));

    // Create BirdSong broadcaster
    let broadcaster = Arc::new(
        BirdSongBroadcaster::new(
            crypto,
            NodeId::from("child"),
            "127.0.0.1:42500".parse::<SocketAddr>().unwrap(),
            "255.255.255.255:42500".parse().unwrap(),
        )
        .await
        .unwrap(),
    );

    // Create relay authority
    let relay_authority = Arc::new(MockRelayAuthority::new(lineage_provider));

    // Create coordinator
    let config = LineageRelayConfig {
        my_id: NodeId::from("child"),
        ..Default::default()
    };

    let _coordinator =
        LineageRelayCoordinator::new(config, broadcaster, relay_authority).await.unwrap();

    // Successfully created coordinator with lineage-based relay
}

#[tokio::test]
async fn test_ancestor_can_decrypt_descendant_birdsong() {
    let lineage_provider = Arc::new(MockLineageProvider::new());
    lineage_provider.add_lineage("child", "parent").await;

    // Child crypto (sender)
    let child_crypto = MockBirdSongCrypto::new(lineage_provider.clone(), "child".to_string());

    // Parent crypto (receiver)
    let parent_crypto = MockBirdSongCrypto::new(lineage_provider, "parent".to_string());

    // Child encrypts message for ancestors
    let message = b"help me relay!";
    let encrypted =
        child_crypto.encrypt_for_lineage(message, LineageHint::DirectAncestors).await.unwrap();

    // Parent should be able to decrypt (is ancestor of child)
    let decrypted =
        parent_crypto.decrypt_birdsong(&encrypted, &NodeId::from("child")).await.unwrap();

    assert_eq!(decrypted, Some(message.to_vec()));
}

#[tokio::test]
async fn test_unrelated_node_cannot_decrypt() {
    let lineage_provider = Arc::new(MockLineageProvider::new());
    lineage_provider.add_lineage("child", "parent").await;

    // Child crypto (sender)
    let child_crypto = MockBirdSongCrypto::new(lineage_provider.clone(), "child".to_string());

    // Unrelated crypto (receiver)
    let unrelated_crypto = MockBirdSongCrypto::new(lineage_provider, "unrelated".to_string());

    // Child encrypts message for ancestors
    let message = b"secret message";
    let encrypted =
        child_crypto.encrypt_for_lineage(message, LineageHint::DirectAncestors).await.unwrap();

    // Unrelated should NOT be able to decrypt
    let decrypted =
        unrelated_crypto.decrypt_birdsong(&encrypted, &NodeId::from("child")).await.unwrap();

    assert_eq!(decrypted, None); // Cannot decrypt!
}

#[tokio::test]
async fn test_relay_authorization_based_on_lineage() {
    let lineage_provider = Arc::new(MockLineageProvider::new());
    lineage_provider.add_lineage("child", "parent").await;

    let relay_authority = MockRelayAuthority::new(lineage_provider);

    // Parent should be authorized to relay for child (is ancestor)
    let auth = relay_authority
        .authorize_relay(&NodeId::from("parent"), &NodeId::from("child"))
        .await
        .unwrap();

    assert!(auth.authorized);

    // Child should NOT be authorized to relay for parent (not ancestor)
    let auth = relay_authority
        .authorize_relay(&NodeId::from("child"), &NodeId::from("parent"))
        .await
        .unwrap();

    assert!(!auth.authorized);
}
