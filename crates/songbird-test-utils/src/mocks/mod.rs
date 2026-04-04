// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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
//! - ⚠️ `MockSecurityProvider` → Use `MockCapabilityServer::new(CapabilityType::Security)`
//! - ⚠️ `MockStorageProvider` → Use `MockCapabilityServer::new(CapabilityType::Storage)`
//! - ⚠️ `MockComputeProvider` (legacy name: `MockToadStool`) → Use `MockCapabilityServer::new(CapabilityType::Compute)`
//! - ⚠️ `MockAiProvider` / legacy `MockSquirrel` → Use `MockCapabilityServer::new(CapabilityType::Ai)`
//!
//! ## Migration Guide
//!
//! **Before (Hardcoded):**
//! ```rust,ignore
//! let mut security = MockSecurityProvider::new();
//! security.start().await?;
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
pub mod ai_provider;
pub mod common;
pub mod security_provider;

/// Deprecated alias for [`ai_provider`].
#[deprecated(note = "use module `ai_provider` (capability-based naming)")]
pub mod squirrel {
    pub use super::ai_provider::*;
}
pub mod compute_provider;
pub mod storage_provider;

/// Deprecated alias for [`compute_provider`].
#[deprecated(note = "use module `compute_provider` (capability-based naming)")]
pub mod toadstool {
    pub use super::compute_provider::*;
}

/// Deprecated alias for [`security_provider`].
#[deprecated(note = "use module `security_provider` (capability-based naming)")]
pub mod beardog {
    pub use super::security_provider::*;
}

/// Deprecated alias for [`storage_provider`].
#[deprecated(note = "use module `storage_provider` (capability-based naming)")]
pub mod nestgate {
    pub use super::storage_provider::*;
}

// 🍼 NEW: Recommended exports
pub use capability_mocks::{
    CapabilityMetrics, CapabilityType, MockCapabilityEnvironment, MockCapabilityEnvironmentBuilder,
    MockCapabilityServer,
};

// ✅ REMOVED: Deprecated legacy exports (Nov 9, 2025)
// Use MockCapabilityServer::new(CapabilityType::*) instead
pub use ai_provider::MockAiProvider;
pub use common::{HealthStatus, MockPrimalServer, MockResponse};

/// Deprecated alias for [`security_provider::MockSecurityProvider`].
#[deprecated(note = "use `MockSecurityProvider`")]
pub type MockBearDog = security_provider::MockSecurityProvider;

/// Deprecated alias for [`storage_provider::MockStorageProvider`].
#[deprecated(note = "use `MockStorageProvider`")]
pub type MockNestGate = storage_provider::MockStorageProvider;
