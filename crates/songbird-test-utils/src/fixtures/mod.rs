// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Test Fixtures Module
//!
//! Centralized test utilities, fixtures, and constants for consistent testing.

pub mod endpoints;
pub mod ports;
pub mod security_provider;

// Re-export for convenience
pub use endpoints::{test_bind_address, test_endpoint, test_port, test_socket_addr};
pub use ports::{endpoints as port_endpoints, hosts};
pub use security_provider::SecurityProviderFixture;
