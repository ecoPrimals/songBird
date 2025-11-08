//! Mock Servers for Testing
//!
//! Provides HTTP server mocks for testing Songbird's orchestration capabilities
//! without requiring actual service instances.
//!
//! ## 🍼 Capability-Based Mocks (RECOMMENDED)
//!
//! Use `MockCapabilityServer` for zero-hardcoding tests:
//!
//! ```rust,ignore
//! use songbird_test_utils::mocks::{MockCapabilityServer, CapabilityType};
//!
//! #[tokio::test]
//! async fn test_with_capability_discovery() {
//!     let mut env = MockCapabilityEnvironment::builder()
//!         .with_security()
//!         .with_storage()
//!         .with_ai()
//!         .build()
//!         .await?;
//!     
//!     // Environment variables are automatically set for discovery
//!     // Your code discovers capabilities via capability_endpoints
//!     
//!     env.shutdown().await;
//! }
//! ```
//!
//! ## Legacy Primal-Specific Mocks (DEPRECATED)
//!
//! The following mocks use hardcoded primal names and are deprecated:
//!
//! - ⚠️ `MockBearDog` → Use `MockCapabilityServer::new(CapabilityType::Security)`
//! - ⚠️ `MockNestGate` → Use `MockCapabilityServer::new(CapabilityType::Storage)`
//! - ⚠️ `MockToadStool` → Use `MockCapabilityServer::new(CapabilityType::Compute)`
//! - ⚠️ `MockSquirrel` → Use `MockCapabilityServer::new(CapabilityType::Ai)`
//!
//! ## Migration Guide
//!
//! **Before (Hardcoded):**
//! ```rust,ignore
//! let mut beardog = MockBearDog::new();
//! beardog.start().await?;
//! ```
//!
//! **After (Capability-Based):**
//! ```rust,ignore
//! let mut security = MockCapabilityServer::new(CapabilityType::Security);
//! security.start().await?;
//! ```

// 🍼 NEW: Capability-based mocks (zero hardcoding)
pub mod capability_mocks;

// ⚠️ DEPRECATED: Primal-specific mocks (hardcoded names)
pub mod beardog;
pub mod common;
pub mod nestgate;
pub mod squirrel;
pub mod toadstool;

// 🍼 NEW: Recommended exports
pub use capability_mocks::{
    CapabilityMetrics, CapabilityType, MockCapabilityEnvironment, MockCapabilityEnvironmentBuilder,
    MockCapabilityServer,
};

// ⚠️ DEPRECATED: Legacy exports
#[deprecated(
    since = "0.5.0",
    note = "Use MockCapabilityServer::new(CapabilityType::Security) instead"
)]
pub use beardog::MockBearDog;
pub use common::{HealthStatus, MockPrimalServer, MockResponse};
#[deprecated(
    since = "0.5.0",
    note = "Use MockCapabilityServer::new(CapabilityType::Storage) instead"
)]
pub use nestgate::MockNestGate;
#[deprecated(
    since = "0.5.0",
    note = "Use MockCapabilityServer::new(CapabilityType::Ai) instead"
)]
pub use squirrel::MockSquirrel;
#[deprecated(
    since = "0.5.0",
    note = "Use MockCapabilityServer::new(CapabilityType::Compute) instead"
)]
pub use toadstool::MockToadStool;
