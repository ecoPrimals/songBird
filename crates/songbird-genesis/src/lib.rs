// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🔐 Songbird Genesis Bootstrap
//!
//! **Physical Proximity Genesis** - "Never let a bird be alone in the dark forest"
//!
//! This crate provides physical genesis bootstrap for new Songbird nodes, ensuring
//! every node is born with witnessed genesis, cryptographic identity, and multi-primal
//! lineage from the first moment.
//!
//! ## Core Concepts
//!
//! - **Genesis Witness**: Existing trusted device that witnesses new node creation
//! - **Physical Channels**: Hardware keys, QR codes, Bluetooth for proximity proof
//! - **Multi-Primal Coordination**: Songbird, `BearDog`, etc. all sign genesis
//! - **Never Alone**: Every node has lineage and trust from birth
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_genesis::{GenesisCeremony, PhysicalChannel, GenesisWitness};
//!
//! // Conduct genesis ceremony via SoloKey
//! let ceremony = GenesisCeremony::new(
//!     PhysicalChannel::HardwareKey,
//!     witness_device,
//! );
//!
//! let new_node_identity = ceremony.conduct().await?;
//! // New node is born with full lineage and identity!
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

// Security capability client (provider-agnostic!)
pub mod ceremony;
pub mod coordination_bridge;
pub mod error;
pub mod identity;
pub mod physical_channels;
pub mod security_capability_client;
pub mod types;
pub mod witness;

// Re-exports
pub use ceremony::GenesisCeremony;
pub use error::{GenesisError, Result};
pub use identity::NewNodeIdentity;
pub use physical_channels::PhysicalChannel;
pub use types::*;
pub use witness::GenesisWitness;

/// Genesis module version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
