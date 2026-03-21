// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use serde::{Deserialize, Serialize};

/// Service endpoint discovered through capability-based discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Service identifier (not necessarily a primal name)
    pub id: String,

    /// Endpoint URL
    pub url: String,

    /// Capabilities this service offers
    pub capabilities: Vec<String>,

    /// Health score (0.0-1.0)
    pub health_score: f64,

    /// Last seen timestamp
    pub last_seen: std::time::SystemTime,
}

/// Discovery method for finding services
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Environment variables (`COMPUTE_ENDPOINT`, `STORAGE_ENDPOINT`, etc.)
    Environment,

    /// DNS Service Discovery (_compute._tcp, etc.)
    DnsSD,

    /// Multicast DNS (zero-conf)
    MDNS,

    /// Central registry (Songbird's capability registry)
    Registry {
        endpoint: String,
    },

    /// Direct configuration file
    ConfigFile {
        path: String,
    },
}
