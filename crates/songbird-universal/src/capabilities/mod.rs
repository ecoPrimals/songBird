//! Universal Capability Adapter System
//!
//! This module provides name-agnostic capability adapters that work with any primal
//! without hardcoding specific primal names. The system discovers capabilities
//! dynamically and routes requests based on capability matching.

mod adapter;
mod connection;
mod error;
mod qos_selection; // ✨ NEW: QoS-aware provider selection
mod registry;
mod types;

// Re-export main types
pub use adapter::UniversalCapabilityAdapter;
pub use connection::{ConnectionHealth, PrimalConnection};
pub use error::CapabilityError;
pub use registry::CapabilityRegistry;
pub use types::{Capability, DiscoveryConfig, PrimalType, QoSMetrics, ResourceMetrics};

// Path constants to avoid Rust 2021 prefix parsing issues
const HEALTH_PATH: &str = "/health";

#[cfg(test)]
mod tests;
