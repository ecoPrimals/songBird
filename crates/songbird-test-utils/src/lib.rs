// Songbird Test Utilities
//
// Canonical testing infrastructure following modernization patterns.
// Provides comprehensive testing capabilities for the Songbird ecosystem.

#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod async_helpers;
pub mod canonical_test_framework;
pub mod chaos_engineering;
pub mod cli_helpers;
pub mod config_helpers;
pub mod error_testing;
pub mod fixtures;
pub mod fixtures_legacy;
pub mod integration;
pub mod mocks;
pub mod network_mocks;
pub mod performance;
pub mod performance_testing;

// Re-export core testing types (canonical pattern)
pub use canonical_test_framework::{
    MockService, TestEnvironment, TestResult as FrameworkTestResult,
};
pub use chaos_engineering::ChaosEngineeringManager;
pub use error_testing::ErrorTestingFramework;
pub use fixtures::*;
pub use integration::IntegrationTestContext;
pub use network_mocks::NetworkMockManager;
pub use performance_testing::PerformanceTestFramework;
pub use songbird_types::SongbirdError;

// Re-export mocks for test convenience
pub use mocks::{
    HealthStatus, MockBearDog, MockNestGate, MockPrimalServer, MockResponse, MockSquirrel,
    MockToadStool,
};

// Re-export helper modules
pub use async_helpers::*;
pub use cli_helpers::*;
pub use config_helpers::*;
