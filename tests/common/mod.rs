// SPDX-License-Identifier: AGPL-3.0-only
//! Common test utilities and helpers for E2E, chaos, and fault testing
//!
//! This module provides shared infrastructure for integration testing across
//! all test suites.

pub mod test_environment;
pub mod service_helpers;
pub mod assertions;

pub use test_environment::TestEnvironment;
pub use service_helpers::{ServiceHelper, MockServiceConfig};
pub use assertions::TestAssertions;

/// Common test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Whether to use real services or mocks
    pub use_real_services: bool,
    /// Base port for test services
    pub base_port: u16,
    /// Timeout for operations
    pub timeout_secs: u64,
    /// Whether to cleanup after tests
    pub cleanup: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            use_real_services: false,
            base_port: 19000, // High port to avoid conflicts
            timeout_secs: 10,
            cleanup: true,
        }
    }
}

/// Chaos testing configuration
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Network latency to inject (ms)
    pub network_latency_ms: Option<u64>,
    /// Packet loss percentage (0.0-1.0)
    pub packet_loss_rate: Option<f64>,
    /// Whether to inject random failures
    pub inject_failures: bool,
    /// Failure rate (0.0-1.0)
    pub failure_rate: f64,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            network_latency_ms: None,
            packet_loss_rate: None,
            inject_failures: false,
            failure_rate: 0.1,
        }
    }
}

