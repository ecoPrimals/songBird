// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Consolidated Test Port Fixtures
//!
//! **Philosophy**: Tests can use hardcoded values, but they should be
//! centralized for easy maintenance and conflict prevention.
//!
//! ## Port Allocation Strategy
//!
//! - **8000-8099**: Core services (orchestrator, discovery)
//! - **8100-8199**: Primal services (toadstool, beardog, etc.)
//! - **8200-8299**: Test utilities and mocks
//! - **8300-8399**: Integration test scenarios

/// Core Songbird orchestrator test port
pub const ORCHESTRATOR: u16 = 8000;

/// Discovery service test port
pub const DISCOVERY: u16 = 8001;

/// Federation coordinator test port
pub const FEDERATION: u16 = 8002;

/// Metrics collection test port
pub const METRICS: u16 = 8003;

/// Health monitoring test port
pub const HEALTH: u16 = 8004;

/// Primal service test ports (discovered via capabilities in production)
pub mod primals {
    /// `ToadStool` compute service test port
    pub const TOADSTOOL: u16 = 8100;

    /// `security provider` security service test port
    pub const BEARDOG: u16 = 8101;

    /// `storage provider` storage service test port
    pub const NESTGATE: u16 = 8102;

    /// `Squirrel` AI service test port
    pub const SQUIRREL: u16 = 8103;
}

/// Mock service ports for testing
pub mod mocks {
    /// Mock HTTP server port
    pub const MOCK_HTTP: u16 = 8200;

    /// Mock gRPC server port
    pub const MOCK_GRPC: u16 = 8201;

    /// Mock WebSocket server port
    pub const MOCK_WS: u16 = 8202;
}

/// Integration test scenario ports
pub mod integration {
    /// Multi-node federation test base port
    pub const FEDERATION_BASE: u16 = 8300;

    /// Chaos testing base port
    pub const CHAOS_BASE: u16 = 8350;
}

/// Test endpoint builders (DRY principle)
pub mod endpoints {
    use super::{DISCOVERY, FEDERATION, ORCHESTRATOR, primals};

    /// Build orchestrator test endpoint
    #[must_use]
    pub fn orchestrator() -> String {
        format!("http://localhost:{}", ORCHESTRATOR)
    }

    /// Build discovery test endpoint
    #[must_use]
    pub fn discovery() -> String {
        format!("http://localhost:{}", DISCOVERY)
    }

    /// Build federation test endpoint
    #[must_use]
    pub fn federation() -> String {
        format!("http://localhost:{}", FEDERATION)
    }

    /// Build `ToadStool` test endpoint
    #[must_use]
    pub fn toadstool() -> String {
        format!("http://localhost:{}", primals::TOADSTOOL)
    }

    /// Build `security provider` test endpoint
    #[must_use]
    pub fn beardog() -> String {
        format!("http://localhost:{}", primals::BEARDOG)
    }

    /// Build storage-provider test endpoint
    #[must_use]
    pub fn nestgate() -> String {
        format!("http://localhost:{}", primals::NESTGATE)
    }

    /// Build `Squirrel` test endpoint
    #[must_use]
    pub fn squirrel() -> String {
        format!("http://localhost:{}", primals::SQUIRREL)
    }

    /// Build generic test endpoint
    #[must_use]
    pub fn generic(port: u16) -> String {
        format!("http://localhost:{port}")
    }

    /// Build with custom host
    #[must_use]
    pub fn with_host(host: &str, port: u16) -> String {
        format!("http://{host}:{port}")
    }

    /// Build HTTPS endpoint (for TLS tests)
    #[must_use]
    pub fn https(host: &str, port: u16) -> String {
        format!("https://{host}:{port}")
    }
}

/// Test host fixtures
pub mod hosts {
    /// Localhost IPv4
    pub const LOCALHOST_V4: &str = "127.0.0.1";

    /// Localhost IPv6
    pub const LOCALHOST_V6: &str = "::1";

    /// Localhost hostname
    pub const LOCALHOST: &str = "localhost";

    /// Test network interface (for LAN tests)
    pub const TEST_INTERFACE: &str = "192.168.1.100";
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_allocation_no_conflicts() {
        // Verify no port conflicts in our allocation
        let all_ports = [
            ORCHESTRATOR,
            DISCOVERY,
            FEDERATION,
            METRICS,
            HEALTH,
            primals::TOADSTOOL,
            primals::BEARDOG,
            primals::NESTGATE,
            primals::SQUIRREL,
            mocks::MOCK_HTTP,
            mocks::MOCK_GRPC,
            mocks::MOCK_WS,
        ];

        // Check for duplicates
        for (i, &port1) in all_ports.iter().enumerate() {
            for &port2 in all_ports.iter().skip(i + 1) {
                assert_ne!(port1, port2, "Port conflict detected: {port1}");
            }
        }
    }

    #[test]
    fn test_endpoint_builders() {
        assert_eq!(endpoints::orchestrator(), "http://localhost:8000");
        assert_eq!(endpoints::discovery(), "http://localhost:8001");
        assert_eq!(endpoints::toadstool(), "http://localhost:8100");
        assert_eq!(endpoints::beardog(), "http://localhost:8101");
    }

    #[test]
    fn test_custom_endpoints() {
        assert_eq!(endpoints::with_host("example.com", 9000), "http://example.com:9000");
        assert_eq!(endpoints::https("secure.example.com", 443), "https://secure.example.com:443");
    }
}
