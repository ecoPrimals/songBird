// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Network Timeout Configuration
//!
//! Timeout configurations for network operations including connections,
//! requests, health checks, and service operations.

#![allow(missing_docs, reason = "timeout buckets align with `NetworkConfig` documentation")]

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Network timeout configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTimeouts {
    pub connection: Duration,
    pub request: Duration,
    pub health_check: Duration,
    pub default: Duration,
}

impl Default for NetworkTimeouts {
    fn default() -> Self {
        Self {
            connection: Duration::from_secs(10),
            request: Duration::from_secs(60),
            health_check: Duration::from_secs(5),
            default: Duration::from_secs(30),
        }
    }
}

/// Network timeout configuration (alternative structure)
///
/// **Merged from**: `config/network/mod.rs`\
/// **Purpose**: Centralized timeout configuration for all network operations
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::network::TimeoutConfig;
///
/// let timeouts = TimeoutConfig {
///     default_timeout_secs: 30,
///     connection_timeout_secs: 10,
///     health_check_timeout_secs: 5,
///     registration_timeout_secs: 15,
///     discovery_timeout_secs: 30,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TimeoutConfig {
    /// Default operation timeout in seconds
    pub default_timeout_secs: u64,

    /// Connection establishment timeout in seconds
    pub connection_timeout_secs: u64,

    /// Health check timeout in seconds
    pub health_check_timeout_secs: u64,

    /// Service registration timeout in seconds
    pub registration_timeout_secs: u64,

    /// Service discovery timeout in seconds
    pub discovery_timeout_secs: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout_secs: 30,
            connection_timeout_secs: 10,
            health_check_timeout_secs: 5,
            registration_timeout_secs: 15,
            discovery_timeout_secs: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions")]
    #![expect(clippy::expect_used, reason = "test assertions")]

    use super::{NetworkTimeouts, TimeoutConfig};
    use songbird_test_utils::canonical_test_framework::TestContext;

    #[test]
    fn timeout_config_default_roundtrip() {
        let ctx = TestContext::new("timeouts");
        let t = TimeoutConfig::default();
        let json = serde_json::to_string(&t).unwrap();
        let back: TimeoutConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(t.default_timeout_secs, back.default_timeout_secs);
        assert_eq!(t.discovery_timeout_secs, back.discovery_timeout_secs);
        assert!(!ctx.is_timeout());
    }

    #[test]
    fn network_timeouts_default_nonzero() {
        let n = NetworkTimeouts::default();
        assert!(n.connection.as_secs() > 0);
        assert!(n.request >= n.health_check);
    }

    #[test]
    fn timeout_config_custom_values() {
        let t = TimeoutConfig {
            default_timeout_secs: 1,
            connection_timeout_secs: 2,
            health_check_timeout_secs: 3,
            registration_timeout_secs: 4,
            discovery_timeout_secs: 5,
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: TimeoutConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.registration_timeout_secs, 4);
    }
}
