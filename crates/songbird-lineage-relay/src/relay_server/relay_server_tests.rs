// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::packet_handler;
use super::*;

use crate::relay::RelayAuthority;
use std::sync::Arc;
use std::time::SystemTime;
use uuid::Uuid;

#[tokio::test]
async fn test_relay_server_creation() {
    let authority = Arc::new(RelayAuthority::StubAllow);
    let server = RelayServer::new("127.0.0.1:0".parse().unwrap(), authority).await.unwrap();

    assert!(server.bind_addr().port() > 0);
}

#[tokio::test]
async fn test_relay_server_stats() {
    let authority = Arc::new(RelayAuthority::StubAllow);
    let server = RelayServer::new("127.0.0.1:0".parse().unwrap(), authority).await.unwrap();

    let stats = server.stats().await;

    assert_eq!(stats.sessions_active, 0);
    assert_eq!(stats.sessions_total, 0);
    assert!(stats.start_time.is_some());
}

#[tokio::test]
async fn test_masking_none() {
    let data = b"Hello, World!";
    let masked = packet_handler::apply_masking(data, MaskingLevel::None).unwrap();

    assert_eq!(masked, data);
}

#[tokio::test]
async fn test_masking_size_obfuscation() {
    let data = b"Hello"; // 5 bytes
    let masked = packet_handler::apply_masking(data, MaskingLevel::SizeObfuscation).unwrap();

    // Should be padded to 1KB
    assert_eq!(masked.len(), 1024);

    // First 5 bytes should be original data
    assert_eq!(&masked[..5], data);

    // Rest should be padding
    assert!(masked[5..].iter().all(|&b| b == 0));
}

#[tokio::test]
async fn test_masking_full() {
    let data = b"Secret message";
    let masked = packet_handler::apply_masking(data, MaskingLevel::Full).unwrap();

    // Currently same as SizeObfuscation (encryption is future)
    assert!(masked.len() >= data.len());
}

#[test]
fn relay_server_stats_uptime_zero_without_start() {
    let s = RelayServerStats {
        start_time: None,
        ..Default::default()
    };
    assert_eq!(s.uptime_seconds(), 0);
}

#[tokio::test]
async fn masking_legacy_masked_passes_through() {
    let data = b"x";
    let m = packet_handler::apply_masking(data, MaskingLevel::Masked).unwrap();
    assert_eq!(m, data);
}

#[tokio::test]
async fn masking_full_visibility_passes_through() {
    let data = b"y";
    let m = packet_handler::apply_masking(data, MaskingLevel::FullVisibility).unwrap();
    assert_eq!(m, data);
}

#[test]
fn relay_session_state_fields() {
    let st = RelaySessionState {
        session_id: Uuid::new_v4(),
        requester_addr: "127.0.0.1:1".parse().unwrap(),
        target_addr: "127.0.0.1:2".parse().unwrap(),
        requester_id: "a".into(),
        target_id: "b".into(),
        masking_level: MaskingLevel::TimingOnly,
        created_at: SystemTime::UNIX_EPOCH,
        last_activity: SystemTime::UNIX_EPOCH,
        bytes_forwarded: 0,
        packets_forwarded: 0,
    };
    assert_eq!(st.bytes_forwarded, 0);
}
