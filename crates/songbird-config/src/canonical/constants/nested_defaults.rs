// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Nested constant namespaces (`network`, `health`, `resources`, `services`) and
//! [`CanonicalNetworkDefaults`].

use std::net::IpAddr;

use songbird_types::error_helpers::SafeEnv;

use super::{get_canonical_bind_address, is_production_environment};

/// Network-related constants
///
/// SOVEREIGNTY EVOLUTION: Hardcoded values removed. Use functions instead.
pub mod network {
    use std::time::Duration;

    use songbird_types::error_helpers::SafeEnv;

    use crate::canonical::constants::{get_bind_address, get_port_range_start};

    /// Default host identifier (`"localhost"`).
    pub const DEFAULT_HOST: &str = "localhost";

    /// Get default host for current environment (may differ from const in production).
    #[must_use]
    pub fn default_host() -> String {
        get_bind_address()
    }

    /// Get default orchestrator port
    #[must_use]
    pub fn default_orchestrator_port() -> u16 {
        SafeEnv::get_port("SONGBIRD_ORCHESTRATOR_PORT", get_port_range_start())
    }

    /// Get default dashboard port
    #[must_use]
    pub fn default_dashboard_port() -> u16 {
        crate::canonical::constants::get_dashboard_port()
    }

    /// Default retry delay (kept as const - no sovereignty issue)
    pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_millis(1000);
}

/// Health check related constants
pub mod health {
    use std::time::Duration;

    /// Default health check interval
    pub const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(30);

    /// Default health check timeout
    pub const DEFAULT_CHECK_TIMEOUT: Duration = Duration::from_secs(5);
}

/// Resource management related constants
pub mod resources {
    use std::time::Duration;

    /// Default resource cleanup interval
    pub const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(300);

    /// Default resource timeout
    pub const DEFAULT_RESOURCE_TIMEOUT: Duration = Duration::from_secs(60);

    /// Default max memory usage percentage
    pub const DEFAULT_MAX_MEMORY_USAGE: f64 = 0.8;

    /// Default max CPU usage percentage
    pub const DEFAULT_MAX_CPU_USAGE: f64 = 0.7;

    /// Default leak detection interval
    pub const DEFAULT_LEAK_DETECTION_INTERVAL: Duration = Duration::from_secs(600);

    /// Default max resource age
    pub const DEFAULT_MAX_RESOURCE_AGE: Duration = Duration::from_secs(3600);

    /// Default monitoring interval
    pub const DEFAULT_MONITORING_INTERVAL: Duration = Duration::from_secs(60);

    /// Default tracking interval
    pub const DEFAULT_TRACKING_INTERVAL: Duration = Duration::from_secs(10);
}

/// Service related constants
pub mod services {
    use std::time::Duration;

    /// Default shutdown timeout
    pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default startup timeout
    pub const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(60);

    /// Default service check interval
    pub const DEFAULT_SERVICE_CHECK_INTERVAL: Duration = Duration::from_secs(15);
}

/// Environment-aware network configuration
pub struct CanonicalNetworkDefaults;

impl CanonicalNetworkDefaults {
    /// Get bind address as `IpAddr`
    #[must_use]
    pub fn bind_address() -> IpAddr {
        get_canonical_bind_address().parse().unwrap_or_else(|_| {
            if is_production_environment() {
                IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
            } else {
                IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
            }
        })
    }

    /// Get allowed networks for security
    #[must_use]
    pub fn allowed_networks() -> Vec<String> {
        if is_production_environment() {
            SafeEnv::get_required("SONGBIRD_ALLOWED_NETWORKS").map_or_else(
                |_| {
                    vec![
                        String::from("10.0.0.0/8"),     // Private networks
                        String::from("172.16.0.0/12"),  // Private networks
                        String::from("192.168.0.0/16"), // Private networks
                    ]
                },
                |nets| nets.split(',').map(String::from).collect(),
            )
        } else {
            vec![
                String::from("127.0.0.0/8"), // Localhost only for development
                String::from("10.0.0.0/8"),  // Local development networks
            ]
        }
    }
}
