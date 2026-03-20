// SPDX-License-Identifier: AGPL-3.0-only
#![expect(
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
// SPDX-License-Identifier: AGPL-3.0-only
//! Coverage tests for songbird_discovery::discovery_stats
//!
//! Tests the atomic stats, snapshot, status manager, and network info.

use songbird_discovery::discovery_stats::{
    DiscoveryStats, DiscoveryStatsSnapshot, DiscoveryStatus, DiscoveryStatusManager, NetworkInfo,
};
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════════
// DiscoveryStats additional coverage
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_stats_default() {
    let stats = DiscoveryStats::default();
    let snapshot = stats.snapshot();
    assert_eq!(snapshot.broadcasts_sent, 0);
    assert_eq!(snapshot.packets_received, 0);
}

#[test]
fn test_stats_peer_discovered() {
    let stats = DiscoveryStats::new();
    stats.record_peer_discovered();
    stats.record_peer_discovered();
    stats.record_peer_discovered();

    let snapshot = stats.snapshot();
    assert_eq!(snapshot.peers_discovered, 3);
}

#[test]
fn test_stats_error_recording() {
    let stats = DiscoveryStats::new();
    stats.record_error();
    stats.record_error();

    let snapshot = stats.snapshot();
    assert_eq!(snapshot.errors, 2);
}

#[test]
fn test_stats_peers_active() {
    let stats = DiscoveryStats::new();
    stats.set_peers_active(5);
    let snapshot = stats.snapshot();
    assert_eq!(snapshot.peers_active, 5);

    stats.set_peers_active(3);
    let snapshot = stats.snapshot();
    assert_eq!(snapshot.peers_active, 3);
}

#[test]
fn test_stats_broadcasting_flag() {
    let stats = DiscoveryStats::new();
    assert!(!stats.snapshot().is_broadcasting);

    stats.set_broadcasting(true);
    assert!(stats.snapshot().is_broadcasting);

    stats.set_broadcasting(false);
    assert!(!stats.snapshot().is_broadcasting);
}

#[test]
fn test_stats_listening_flag() {
    let stats = DiscoveryStats::new();
    assert!(!stats.snapshot().is_listening);

    stats.set_listening(true);
    assert!(stats.snapshot().is_listening);

    stats.set_listening(false);
    assert!(!stats.snapshot().is_listening);
}

#[test]
fn test_stats_timestamp_updates() {
    let stats = DiscoveryStats::new();

    let before = stats.snapshot();
    assert_eq!(before.last_broadcast_time, 0);
    assert_eq!(before.last_received_time, 0);

    stats.record_broadcast();
    let after_broadcast = stats.snapshot();
    assert!(after_broadcast.last_broadcast_time > 0);

    stats.record_received();
    let after_received = stats.snapshot();
    assert!(after_received.last_received_time > 0);
}

#[test]
fn test_stats_concurrent_mixed_operations() {
    use std::thread;

    let stats = Arc::new(DiscoveryStats::new());
    let mut handles = vec![];

    // Thread 1: broadcasts
    let s1 = Arc::clone(&stats);
    handles.push(thread::spawn(move || {
        for _ in 0..50 {
            s1.record_broadcast();
        }
    }));

    // Thread 2: receives
    let s2 = Arc::clone(&stats);
    handles.push(thread::spawn(move || {
        for _ in 0..50 {
            s2.record_received();
        }
    }));

    // Thread 3: peer discoveries
    let s3 = Arc::clone(&stats);
    handles.push(thread::spawn(move || {
        for _ in 0..50 {
            s3.record_peer_discovered();
        }
    }));

    // Thread 4: errors
    let s4 = Arc::clone(&stats);
    handles.push(thread::spawn(move || {
        for _ in 0..50 {
            s4.record_error();
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    let snapshot = stats.snapshot();
    assert_eq!(snapshot.broadcasts_sent, 50);
    assert_eq!(snapshot.packets_received, 50);
    assert_eq!(snapshot.peers_discovered, 50);
    assert_eq!(snapshot.errors, 50);
}

// ═══════════════════════════════════════════════════════════════════════
// DiscoveryStatsSnapshot tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_snapshot_serialization() {
    let snapshot = DiscoveryStatsSnapshot {
        broadcasts_sent: 100,
        packets_received: 200,
        peers_discovered: 10,
        peers_active: 5,
        errors: 2,
        last_broadcast_time: 1700000000,
        last_received_time: 1700000001,
        is_broadcasting: true,
        is_listening: true,
    };

    let json = serde_json::to_string(&snapshot).unwrap();
    let deserialized: DiscoveryStatsSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.broadcasts_sent, 100);
    assert_eq!(deserialized.packets_received, 200);
    assert!(deserialized.is_broadcasting);
    assert!(deserialized.is_listening);
}

#[test]
fn test_snapshot_clone() {
    let snapshot = DiscoveryStatsSnapshot {
        broadcasts_sent: 42,
        packets_received: 84,
        peers_discovered: 3,
        peers_active: 2,
        errors: 1,
        last_broadcast_time: 0,
        last_received_time: 0,
        is_broadcasting: false,
        is_listening: false,
    };

    let cloned = snapshot.clone();
    assert_eq!(snapshot.broadcasts_sent, cloned.broadcasts_sent);
    assert_eq!(snapshot.errors, cloned.errors);
}

// ═══════════════════════════════════════════════════════════════════════
// DiscoveryStatusManager tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_status_manager_creation() {
    let manager = DiscoveryStatusManager::new(
        true,
        "BirdSong".to_string(),
        4242,
        Some("239.255.42.99:4242".to_string()),
    );

    let status = manager.get_status().await;
    assert!(status.enabled);
    assert_eq!(status.mode, "BirdSong");
    assert!(!status.running); // Not broadcasting or listening yet
    assert_eq!(status.network.udp_port, 4242);
    assert_eq!(status.network.multicast_address, Some("239.255.42.99:4242".to_string()));
}

#[tokio::test]
async fn test_status_manager_disabled() {
    let manager = DiscoveryStatusManager::new(false, "Disabled".to_string(), 0, None);

    let status = manager.get_status().await;
    assert!(!status.enabled);
    assert_eq!(status.mode, "Disabled");
    assert!(status.network.multicast_address.is_none());
}

#[tokio::test]
async fn test_status_manager_running_when_broadcasting() {
    let manager = DiscoveryStatusManager::new(true, "Active".to_string(), 5000, None);

    manager.stats().set_broadcasting(true);
    let status = manager.get_status().await;
    assert!(status.running);
}

#[tokio::test]
async fn test_status_manager_running_when_listening() {
    let manager = DiscoveryStatusManager::new(true, "Active".to_string(), 5000, None);

    manager.stats().set_listening(true);
    let status = manager.get_status().await;
    assert!(status.running);
}

#[tokio::test]
async fn test_status_manager_stats_integration() {
    let manager = DiscoveryStatusManager::new(
        true,
        "Full".to_string(),
        6000,
        Some("239.0.0.1:6000".to_string()),
    );

    let stats = manager.stats();
    stats.record_broadcast();
    stats.record_broadcast();
    stats.record_received();
    stats.record_peer_discovered();
    stats.record_error();
    stats.set_peers_active(3);
    stats.set_broadcasting(true);
    stats.set_listening(true);

    let status = manager.get_status().await;
    assert!(status.running);
    assert_eq!(status.stats.broadcasts_sent, 2);
    assert_eq!(status.stats.packets_received, 1);
    assert_eq!(status.stats.peers_discovered, 1);
    assert_eq!(status.stats.peers_active, 3);
    assert_eq!(status.stats.errors, 1);
    assert!(status.stats.is_broadcasting);
    assert!(status.stats.is_listening);
}

// ═══════════════════════════════════════════════════════════════════════
// DiscoveryStatus tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_discovery_status_serialization() {
    let status = DiscoveryStatus {
        enabled: true,
        mode: "Anonymous".to_string(),
        running: true,
        stats: DiscoveryStatsSnapshot {
            broadcasts_sent: 10,
            packets_received: 5,
            peers_discovered: 2,
            peers_active: 1,
            errors: 0,
            last_broadcast_time: 1700000000,
            last_received_time: 1700000001,
            is_broadcasting: true,
            is_listening: true,
        },
        network: NetworkInfo {
            udp_port: 4242,
            multicast_address: Some("239.255.42.99:4242".to_string()),
            interfaces: vec!["eth0".to_string()],
        },
    };

    let json = serde_json::to_string(&status).unwrap();
    let deserialized: DiscoveryStatus = serde_json::from_str(&json).unwrap();
    assert!(deserialized.enabled);
    assert!(deserialized.running);
    assert_eq!(deserialized.stats.broadcasts_sent, 10);
    assert_eq!(deserialized.network.udp_port, 4242);
}

// ═══════════════════════════════════════════════════════════════════════
// NetworkInfo tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_network_info_detect_interfaces() {
    let interfaces = NetworkInfo::detect_interfaces();
    assert!(!interfaces.is_empty(), "Should detect at least one interface");
}

#[test]
fn test_network_info_serialization() {
    let info = NetworkInfo {
        udp_port: 8888,
        multicast_address: Some("224.0.0.1:8888".to_string()),
        interfaces: vec!["lo".to_string(), "eth0".to_string()],
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: NetworkInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.udp_port, 8888);
    assert_eq!(deserialized.interfaces.len(), 2);
}

#[test]
fn test_network_info_clone() {
    let info = NetworkInfo {
        udp_port: 1234,
        multicast_address: None,
        interfaces: vec!["wlan0".to_string()],
    };

    let cloned = info.clone();
    assert_eq!(info.udp_port, cloned.udp_port);
    assert_eq!(info.interfaces, cloned.interfaces);
}
