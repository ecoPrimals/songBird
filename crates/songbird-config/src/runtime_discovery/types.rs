// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovered service record and discovery channel taxonomy.

/// Discovered service information
#[derive(Debug, Clone)]
pub struct DiscoveredService {
    /// Capability provided by this service
    pub capability: String,

    /// Service endpoint (e.g., "<http://10.0.1.50:8001>")
    pub endpoint: String,

    /// How this service was discovered
    pub discovered_via: DiscoveryMethod,

    /// Health score (0.0 = unhealthy, 1.0 = healthy)
    pub health_score: f64,

    /// When this service was last seen
    pub last_seen: std::time::SystemTime,
}

/// Discovery method used to find a service
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Discovered via environment variable
    Environment,

    /// Discovered via mDNS (multicast DNS)
    MDNS,

    /// Discovered via central registry
    Registry,

    /// Discovered via peer announcement
    Announcement,
}

impl std::fmt::Display for DiscoveryMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Environment => write!(f, "Environment Variable"),
            Self::MDNS => write!(f, "mDNS"),
            Self::Registry => write!(f, "Central Registry"),
            Self::Announcement => write!(f, "Peer Announcement"),
        }
    }
}
