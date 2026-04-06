// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🔧 Network Management
//!
//! **MODERN NETWORK MANAGEMENT** ✅

use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;

/// Network management service
#[derive(Debug)]
pub struct NetworkManagement;

impl Default for NetworkManagement {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkManagement {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub async fn get_stats(&self) -> SongbirdResult<NetworkStats> {
        Ok(NetworkStats {
            connections: 0,
            bandwidth_mbps: 0.0,
            latency_ms: 0.0,
        })
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub connections: u64,
    pub bandwidth_mbps: f64,
    pub latency_ms: f64,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn network_management_new_default() {
        let a = NetworkManagement::new();
        let b = NetworkManagement;
        let _ = (a, b);
    }

    #[tokio::test]
    async fn get_stats_returns_zeroed_defaults() {
        let m = NetworkManagement::new();
        let s = m.get_stats().await.unwrap();
        assert_eq!(s.connections, 0);
        assert_eq!(s.bandwidth_mbps, 0.0);
        assert_eq!(s.latency_ms, 0.0);
    }

    #[test]
    fn network_stats_serde_roundtrip() {
        let s = NetworkStats {
            connections: 3,
            bandwidth_mbps: 100.5,
            latency_ms: 12.0,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: NetworkStats = serde_json::from_str(&json).unwrap();
        assert_eq!(s.connections, back.connections);
        assert!((s.bandwidth_mbps - back.bandwidth_mbps).abs() < f64::EPSILON);
    }

    #[test]
    fn network_stats_clone_debug() {
        let s = NetworkStats {
            connections: 0,
            bandwidth_mbps: 0.0,
            latency_ms: 0.0,
        };
        let _ = format!("{s:?}");
        let t = s.clone();
        assert_eq!(t.latency_ms, s.latency_ms);
    }
}
