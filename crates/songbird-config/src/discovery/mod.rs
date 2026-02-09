//! Capability-Based Service Discovery
//!
//! Zero-hardcoding discovery system where services discover each other
//! based on capabilities, not names.
//!
//! # Architecture
//!
//! This module provides multiple discovery backends that can be used
//! independently or in combination:
//!
//! - **Environment Variables**: Development/testing (always enabled)
//! - **mDNS**: Local network discovery (zero-configuration)
//! - **DNS-SD**: Standards-based DNS service discovery
//! - **Kubernetes**: Container orchestration discovery
//! - **Consul**: Service mesh discovery
//! - **etcd**: Distributed key-value store discovery
//!
//! # Principles
//!
//! 1. **No Hardcoded Names**: Services never know names of other services
//! 2. **Capability-Based**: Discovery by capability, not identity
//! 3. **Self-Knowledge**: Services know what they provide, discover what they need
//! 4. **Runtime Resolution**: All discovery happens at runtime
//!
//! # Example
//!
//! ```rust,ignore
//! use songbird_config::discovery::{CapabilityDiscoveryEngine, DiscoveryBackend};
//! use std::time::Duration;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create discovery engine with auto-detected backends
//!     let engine = CapabilityDiscoveryEngine::with_defaults();
//!
//!     // Discover services that provide "storage" capability
//!     let storage_services = engine.discover_by_capability("storage").await;
//!
//!     for service in storage_services {
//!         println!("Found storage at: {}", service.address);
//!     Ok(())
//! }
//! ```

pub mod mdns;
pub mod runtime_engine;

pub use runtime_engine::{CapabilityDiscoveryEngine, DiscoveredService, DiscoveryBackend};

pub use mdns::{MdnsDiscovery, MdnsError, MdnsServiceInfo};
