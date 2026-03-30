// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Test Fixtures Module
//!
//! Centralized test utilities, fixtures, and constants for consistent testing.

pub mod beardog;
pub mod endpoints;
pub mod ports;

// Re-export for convenience
pub use beardog::BearDogFixture;
pub use endpoints::{test_bind_address, test_endpoint, test_port, test_socket_addr};
pub use ports::{endpoints as port_endpoints, hosts};
