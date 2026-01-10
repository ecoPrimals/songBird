//! Discovery Statistics and Status Tracking
//!
//! Provides observability into the discovery system without relying on logs.
//! This enables "AI-first" monitoring and "user sovereignty" through programmatic APIs.
//!
//! ## Problem Solved
//!
//! When Songbird runs under Tower orchestration, stdout/stderr may be redirected to `/dev/null`,
//! making it impossible to verify discovery is working. This module provides an API-based
//! alternative for full observability.
//!
//! ## Date: January 5, 2026

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Discovery statistics for observability
///
/// Thread-safe atomic counters for tracking discovery activity
#[derive(Debug, Clone)]
pub struct DiscoveryStats {
    /// Number of broadcast packets sent
    pub broadcasts_sent: Arc<AtomicU64>,

    /// Number of packets received
    pub packets_received: Arc<AtomicU64>,

    /// Number of unique peers discovered
    pub peers_discovered: Arc<AtomicU64>,

    /// Number of currently active peers
    pub peers_active: Arc<AtomicU64>,

    /// Number of discovery errors encountered
    pub errors: Arc<AtomicU64>,

    /// Last broadcast timestamp (Unix epoch seconds)
    pub last_broadcast_time: Arc<AtomicU64>,

    /// Last received packet timestamp (Unix epoch seconds)
    pub last_received_time: Arc<AtomicU64>,

    /// Whether broadcasting is currently active
    pub is_broadcasting: Arc<AtomicBool>,

    /// Whether listening is currently active
    pub is_listening: Arc<AtomicBool>,
}

impl Default for DiscoveryStats {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoveryStats {
    /// Create a new statistics tracker
    pub fn new() -> Self {
        Self {
            broadcasts_sent: Arc::new(AtomicU64::new(0)),
            packets_received: Arc::new(AtomicU64::new(0)),
            peers_discovered: Arc::new(AtomicU64::new(0)),
            peers_active: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
            last_broadcast_time: Arc::new(AtomicU64::new(0)),
            last_received_time: Arc::new(AtomicU64::new(0)),
            is_broadcasting: Arc::new(AtomicBool::new(false)),
            is_listening: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Record a broadcast packet sent
    pub fn record_broadcast(&self) {
        self.broadcasts_sent.fetch_add(1, Ordering::Relaxed);
        let now =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        self.last_broadcast_time.store(now, Ordering::Relaxed);
    }

    /// Record a packet received
    pub fn record_received(&self) {
        self.packets_received.fetch_add(1, Ordering::Relaxed);
        let now =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        self.last_received_time.store(now, Ordering::Relaxed);
    }

    /// Record a peer discovered
    pub fn record_peer_discovered(&self) {
        self.peers_discovered.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an error
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Update active peer count
    pub fn set_peers_active(&self, count: u64) {
        self.peers_active.store(count, Ordering::Relaxed);
    }

    /// Mark broadcasting as started
    pub fn set_broadcasting(&self, active: bool) {
        self.is_broadcasting.store(active, Ordering::Relaxed);
    }

    /// Mark listening as started
    pub fn set_listening(&self, active: bool) {
        self.is_listening.store(active, Ordering::Relaxed);
    }

    /// Get a snapshot of current statistics
    pub fn snapshot(&self) -> DiscoveryStatsSnapshot {
        DiscoveryStatsSnapshot {
            broadcasts_sent: self.broadcasts_sent.load(Ordering::Relaxed),
            packets_received: self.packets_received.load(Ordering::Relaxed),
            peers_discovered: self.peers_discovered.load(Ordering::Relaxed),
            peers_active: self.peers_active.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            last_broadcast_time: self.last_broadcast_time.load(Ordering::Relaxed),
            last_received_time: self.last_received_time.load(Ordering::Relaxed),
            is_broadcasting: self.is_broadcasting.load(Ordering::Relaxed),
            is_listening: self.is_listening.load(Ordering::Relaxed),
        }
    }
}

/// Immutable snapshot of discovery statistics
///
/// Used for API responses and serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryStatsSnapshot {
    pub broadcasts_sent: u64,
    pub packets_received: u64,
    pub peers_discovered: u64,
    pub peers_active: u64,
    pub errors: u64,
    pub last_broadcast_time: u64,
    pub last_received_time: u64,
    pub is_broadcasting: bool,
    pub is_listening: bool,
}

/// Complete discovery status for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryStatus {
    pub enabled: bool,
    pub mode: String,
    pub running: bool,
    pub stats: DiscoveryStatsSnapshot,
    pub network: NetworkInfo,
}

/// Network configuration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub udp_port: u16,
    pub multicast_address: Option<String>,
    pub interfaces: Vec<String>,
}

impl NetworkInfo {
    /// Detect available network interfaces
    pub fn detect_interfaces() -> Vec<String> {
        // Use nix or pnet to detect interfaces
        // For now, return a simple list
        vec!["ens33".to_string(), "lo".to_string()]
    }
}

/// Discovery status manager
///
/// Aggregates statistics and configuration for status reporting
pub struct DiscoveryStatusManager {
    stats: Arc<DiscoveryStats>,
    config: Arc<RwLock<DiscoveryConfigSnapshot>>,
}

#[derive(Debug, Clone)]
struct DiscoveryConfigSnapshot {
    enabled: bool,
    mode: String,
    udp_port: u16,
    multicast_address: Option<String>,
}

impl DiscoveryStatusManager {
    /// Create a new status manager
    pub fn new(
        enabled: bool,
        mode: String,
        udp_port: u16,
        multicast_address: Option<String>,
    ) -> Self {
        Self {
            stats: Arc::new(DiscoveryStats::new()),
            config: Arc::new(RwLock::new(DiscoveryConfigSnapshot {
                enabled,
                mode,
                udp_port,
                multicast_address,
            })),
        }
    }

    /// Get the statistics tracker
    pub fn stats(&self) -> Arc<DiscoveryStats> {
        Arc::clone(&self.stats)
    }

    /// Get complete discovery status
    pub async fn get_status(&self) -> DiscoveryStatus {
        let config = self.config.read().await;
        let stats_snapshot = self.stats.snapshot();

        DiscoveryStatus {
            enabled: config.enabled,
            mode: config.mode.clone(),
            running: stats_snapshot.is_broadcasting || stats_snapshot.is_listening,
            stats: stats_snapshot,
            network: NetworkInfo {
                udp_port: config.udp_port,
                multicast_address: config.multicast_address.clone(),
                interfaces: NetworkInfo::detect_interfaces(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_stats_new() {
        let stats = DiscoveryStats::new();
        let snapshot = stats.snapshot();

        assert_eq!(snapshot.broadcasts_sent, 0);
        assert_eq!(snapshot.packets_received, 0);
        assert_eq!(snapshot.peers_discovered, 0);
        assert_eq!(snapshot.peers_active, 0);
        assert_eq!(snapshot.errors, 0);
        assert!(!snapshot.is_broadcasting);
        assert!(!snapshot.is_listening);
    }

    #[test]
    fn test_record_broadcast() {
        let stats = DiscoveryStats::new();

        stats.record_broadcast();
        stats.record_broadcast();
        stats.record_broadcast();

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.broadcasts_sent, 3);
        assert!(snapshot.last_broadcast_time > 0);
    }

    #[test]
    fn test_record_received() {
        let stats = DiscoveryStats::new();

        stats.record_received();
        stats.record_received();

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.packets_received, 2);
        assert!(snapshot.last_received_time > 0);
    }

    #[test]
    fn test_concurrent_updates() {
        use std::thread;

        let stats = Arc::new(DiscoveryStats::new());
        let mut handles = vec![];

        // Spawn 10 threads each recording 100 broadcasts
        for _ in 0..10 {
            let stats_clone = Arc::clone(&stats);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    stats_clone.record_broadcast();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Should have 1000 total broadcasts
        let snapshot = stats.snapshot();
        assert_eq!(snapshot.broadcasts_sent, 1000);
    }

    #[tokio::test]
    async fn test_status_manager() {
        let manager = DiscoveryStatusManager::new(
            true,
            "Anonymous".to_string(),
            2300,
            Some("239.255.42.99:4242".to_string()),
        );

        // Record some activity
        manager.stats().record_broadcast();
        manager.stats().record_received();
        manager.stats().set_broadcasting(true);

        // Get status
        let status = manager.get_status().await;

        assert!(status.enabled);
        assert_eq!(status.mode, "Anonymous");
        assert!(status.running);
        assert_eq!(status.stats.broadcasts_sent, 1);
        assert_eq!(status.stats.packets_received, 1);
        assert_eq!(status.network.udp_port, 2300);
    }
}
