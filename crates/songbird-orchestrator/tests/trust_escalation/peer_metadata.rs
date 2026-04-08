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

use songbird_orchestrator::app::connection_manager::PeerMetadata;
use std::time::SystemTime;

// ═══════════════════════════════════════════════════════════════════════════
// PeerMetadata serde tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_peer_metadata_serde_roundtrip() {
    let metadata = PeerMetadata {
        peer_id: "peer-42".to_string(),
        endpoint: "https://10.0.0.1:8443".to_string(),
        trust_level: songbird_types::TrustLevel::Elevated,
        discovery_method: "mdns".to_string(),
        capabilities: vec!["compute".to_string(), "storage".to_string()],
        established_at: SystemTime::now(),
    };

    let json = serde_json::to_string(&metadata).expect("serialize");
    let deserialized: PeerMetadata = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.peer_id, "peer-42");
    assert_eq!(deserialized.endpoint, "https://10.0.0.1:8443");
    assert_eq!(deserialized.capabilities.len(), 2);
}
