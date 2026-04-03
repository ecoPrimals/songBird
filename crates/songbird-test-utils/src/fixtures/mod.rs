// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Test Fixtures Module
//!
//! Centralized test utilities, fixtures, and constants for consistent testing.

pub mod endpoints;
pub mod ports;
pub mod security_provider;

/// Deprecated alias for [`security_provider`].
#[deprecated(note = "use module `security_provider` (capability-based naming)")]
pub mod beardog {
    pub use super::security_provider::*;
}

// Re-export for convenience
pub use security_provider::SecurityProviderFixture;

/// Deprecated alias for [`SecurityProviderFixture`].
#[deprecated(note = "use `SecurityProviderFixture`")]
pub type BearDogFixture = SecurityProviderFixture;
pub use endpoints::{test_bind_address, test_endpoint, test_port, test_socket_addr};
pub use ports::{endpoints as port_endpoints, hosts};
