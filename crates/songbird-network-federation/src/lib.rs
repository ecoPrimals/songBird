//! # 🌐 Songbird Network Federation
//!
//! **CONSOLIDATED NETWORK & FEDERATION CRATE** ✅
//!
//! This crate provides unified networking and federation capabilities for Songbird,
//! consolidating the previously fragmented network and federation functionality.
//!
//! ## 🎯 **Domain Consolidation Benefits**
//!
//! - ✅ **Unified Network Stack**: Single source for all networking functionality
//! - ✅ **Federation Integration**: Seamless network-federation coordination
//! - ✅ **Modern Rust**: Latest networking patterns with async/await
//! - ✅ **Zero Technical Debt**: Clean implementation with no legacy baggage
//! - ✅ **Gaming Protocols**: Specialized gaming network protocol support

#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::pub_use)] // Re-exports are acceptable for consolidated crates
#![allow(
    clippy::missing_errors_doc,
    clippy::upper_case_acronyms,
    clippy::trivially_copy_pass_by_ref,
    clippy::cast_possible_truncation,
    clippy::unused_async,
    clippy::if_same_then_else,
    clippy::struct_field_names
)]

// Core modules
pub mod btsp; // ✨ NEW: BearDog Secure Tunnel Protocol interface
pub mod federation;
pub mod integration;
pub mod multi_federation; // ✨ NEW: Multi-federation support with context-aware boundaries
pub mod network;
pub mod protocol_capability; // ✨ NEW: Protocol capability advertisement
pub mod service_registry;
pub mod state;
pub mod tls;
pub mod zero_copy_registry; // ✨ NEW: Zero-copy evolved registry

// Re-export core types for convenience
pub use btsp::{BtspConfig, BtspProvider, LocalBtspProvider};
pub use federation::{FederationConfig, FederationCoordinator, NodeInfo};
pub use integration::NetworkFederationBridge;
pub use network::{NetworkConfig, NetworkManager, NetworkProvider};
pub use protocol_capability::{
    Protocol, ProtocolCapability, ProtocolCapabilityManager, TowerCapabilities,
};

// Legacy compatibility removed - use canonical APIs directly
