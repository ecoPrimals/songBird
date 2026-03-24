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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn service_endpoint_serde_roundtrip() {
        let ep = ServiceEndpoint {
            id: "id-1".to_string(),
            url: "http://127.0.0.1:9000".to_string(),
            capabilities: vec!["compute".to_string()],
            health_score: 0.95,
            last_seen: std::time::SystemTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&ep).expect("ser");
        let back: ServiceEndpoint = serde_json::from_str(&json).expect("de");
        assert_eq!(back.id, ep.id);
        assert_eq!(back.url, ep.url);
        assert_eq!(back.capabilities, ep.capabilities);
    }

    #[test]
    fn discovery_method_registry_and_config_file_equality() {
        let a = DiscoveryMethod::Registry {
            endpoint: "http://r".to_string(),
        };
        let b = DiscoveryMethod::Registry {
            endpoint: "http://r".to_string(),
        };
        assert_eq!(a, b);
        let c = DiscoveryMethod::ConfigFile {
            path: "/etc/sb.yaml".to_string(),
        };
        assert_ne!(a, c);
    }

    #[test]
    fn discovery_method_environment_mdns_distinct() {
        assert_ne!(DiscoveryMethod::Environment, DiscoveryMethod::MDNS);
        assert_ne!(DiscoveryMethod::DnsSD, DiscoveryMethod::MDNS);
    }
}
