//! Capability-based primal discovery
//!
//! This module integrates capability-based discovery with Universal IPC,
//! enabling TRUE PRIMAL self-knowledge while providing platform-agnostic
//! communication.
//!
//! ## Architecture
//!
//! ```text
//! Application:
//!   "I need crypto capability"
//!      ↓
//! Capability Registry:
//!   1. Discover providers (env, filesystem, mDNS, registry)
//!   2. Return virtual endpoint: "/primal/provider-id"
//!      ↓
//! Universal IPC:
//!   1. Resolve virtual → native endpoint
//!   2. Connect (platform-specific)
//!   3. Return unified stream
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_universal_ipc::capability::CapabilityRegistry;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = CapabilityRegistry::new().await?;
//!
//! // Discover crypto provider (no hardcoded primal names!)
//! let provider = registry.discover("crypto").await?;
//! println!("Found crypto at: {}", provider.virtual_endpoint);
//!
//! // Connect via Universal IPC (platform-agnostic!)
//! let stream = songbird_universal_ipc::ipc::connect(&provider.virtual_endpoint).await?;
//! // ✅ Capability-based + Platform-agnostic!
//! # Ok(())
//! # }
//! ```

pub mod discovery;
pub mod provider;
pub mod registry;
pub mod strategy;

// Re-exports
pub use provider::{Provider, ProviderMetadata};
pub use registry::CapabilityRegistry;
pub use strategy::{DiscoveryStrategy, EnvironmentStrategy, FilesystemStrategy};
