//! Test Fixtures Module
//!
//! Centralized test utilities, fixtures, and constants for consistent testing.

pub mod endpoints;
pub mod ports;

// Re-export for convenience
pub use endpoints::{test_endpoint, test_port, test_bind_address, test_socket_addr};
pub use ports::{endpoints as port_endpoints, hosts};
