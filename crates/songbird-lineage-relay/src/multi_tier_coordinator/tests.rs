// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::tiers::extract_trycloudflare_url;
use super::*;
use crate::types::NodeId;
use songbird_types::config::stun_relay::{StunRelayConfig, StunServerConfig, StunStrategy};

#[tokio::test]
async fn test_coordinator_creation() {
    let config = StunRelayConfig::default();
    let coordinator = MultiTierCoordinator::new(config.clone());
    assert_eq!(coordinator.config.strategy, config.strategy);
}

#[tokio::test]
async fn test_sovereignty_first_with_empty_config() {
    let config = StunRelayConfig::default();
    let coordinator = MultiTierCoordinator::new(config);

    let result = coordinator.discover_public_address().await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_stun_discovery_with_public_servers() {
    let mut config = StunRelayConfig::default();
    config.public_stun.enabled = true;
    config.public_stun.servers.push(StunServerConfig {
        address: "stun.nextcloud.com:3478".to_string(),
        protocol: Default::default(),
        priority: 10,
        enabled: true,
        verified: false,
        vetted: true,
        comment: "Community STUN server".to_string(),
    });

    let coordinator = MultiTierCoordinator::new(config);
    let result = coordinator.discover_public_address().await;

    if let Ok(addr) = result {
        println!("Discovered public address: {addr}");
        assert!(addr.port() > 0);
    }
}

#[tokio::test]
async fn test_tier_quality_report_empty() {
    let config = StunRelayConfig::default();
    let coordinator = MultiTierCoordinator::new(config);

    let report = coordinator.check_tier_quality().await;
    assert!(report.user_provided_latency.is_none());
    assert!(report.public_stun_latency.is_none());
}

#[tokio::test]
async fn discover_public_address_lineage_only_errors_without_network() {
    let mut config = StunRelayConfig::default();
    config.strategy = StunStrategy::LineageOnly;
    let coordinator = MultiTierCoordinator::new(config);
    let err = coordinator.discover_public_address().await.expect_err("lineage-only skips STUN");
    assert!(err.to_string().contains("LineageOnly") || err.to_string().contains("STUN"), "{}", err);
}

#[tokio::test]
async fn establish_connection_exhausts_fallback_chain() {
    let config = StunRelayConfig::default();
    let coordinator = MultiTierCoordinator::new(config);
    let err = coordinator
        .establish_connection(NodeId::from("p"), None)
        .await
        .expect_err("all tiers should fail in test without network");
    let msg = err.to_string();
    assert!(msg.contains("fallback chain exhausted"), "unexpected: {msg}");
    assert!(msg.contains("direct"), "should mention direct tier: {msg}");
    assert!(msg.contains("stun-punch"), "should mention stun tier: {msg}");
}

#[test]
fn connection_tier_display_all_variants() {
    assert_eq!(ConnectionTier::Direct.to_string(), "direct");
    assert_eq!(ConnectionTier::StunPunch.to_string(), "stun-punch");
    assert_eq!(ConnectionTier::LineageRelay.to_string(), "lineage-relay");
    assert_eq!(ConnectionTier::TurnRelay.to_string(), "turn-relay");
    assert_eq!(ConnectionTier::EmergencyTunnel.to_string(), "emergency-tunnel");
}

#[test]
fn tier_quality_report_default_is_empty() {
    let r = TierQualityReport::default();
    assert!(r.user_provided_latency.is_none());
    assert!(r.lineage_relay_latency.is_none());
}

#[test]
fn extract_trycloudflare_url_parses_typical_log_line() {
    let line = "2026-05-20T12:00:00Z INF +---------------------------------------------------+\n";
    assert_eq!(extract_trycloudflare_url(line), None);

    let line = "2026-05-20T12:00:01Z INF | https://foo-bar-baz.trycloudflare.com |";
    assert_eq!(
        extract_trycloudflare_url(line),
        Some("https://foo-bar-baz.trycloudflare.com".to_string())
    );
}

#[test]
fn extract_trycloudflare_url_ignores_non_tunnel_urls() {
    let line = "connecting to https://region1.argotunnel.com:7844";
    assert_eq!(extract_trycloudflare_url(line), None);
}
