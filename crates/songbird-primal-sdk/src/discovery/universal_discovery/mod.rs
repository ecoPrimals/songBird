//! # Universal Primal Discovery System
//!
//! This module implements a comprehensive service discovery system that can
//! dynamically detect, register, and manage primal services across different
//! environments, protocols, and network configurations.

pub mod engine;
pub mod channels;
pub mod types;
pub mod stats;

// Re-export main types and engine
pub use engine::UniversalDiscoveryEngine;
pub use types::{
    DiscoveryConfig, DiscoveredService, DiscoveryMethod, DiscoveryEvent,
    UniversalHealthStatus, ServiceCapability, ServiceMetadata
};
pub use channels::{
    DiscoveryChannel, NetworkScanChannel, DnsDiscoveryChannel,
    KubernetesDiscoveryChannel, ConsulDiscoveryChannel
};
pub use stats::DiscoveryStats; 