// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Pure Rust STUN (RFC 5389) Client
//!
//! **Zero C Dependencies | ecoBin Compliant | Sovereignty-First**
//!
//! STUN (Session Traversal Utilities for NAT) enables discovery of public
//! IP addresses and ports for NAT traversal without external relay servers.
//!
//! ## Features
//!
//! - ✅ Pure Rust implementation of RFC 5389
//! - ✅ Zero C dependencies (ecoBin compliant)
//! - ✅ Async/await with tokio
//! - ✅ UDP-based STUN requests
//! - ✅ Public IP/port discovery
//! - ✅ NAT type detection
//! - ✅ Modern idiomatic Rust
//!
//! ## Privacy Note
//!
//! STUN servers can observe your public IP/port and connection timing.
//! Prefer genetic lineage relay (Tier 1) when sovereignty > convenience.
//!
//! ## Usage
//!
//! ```no_run
//! use songbird_stun::StunClient;
//! use songbird_types::constants::DEFAULT_STUN_SERVER_1;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = StunClient::new();
//!
//!     // Discover public address via STUN
//!     let public_addr = client.discover_public_address(DEFAULT_STUN_SERVER_1).await?;
//!
//!     println!("My public address: {}", public_addr);
//!     Ok(())
//! }
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod client;
pub mod error;
pub mod message;
mod protocol;
pub mod server;
mod transaction;
pub mod types;

// Re-exports
pub use client::StunClient;
pub use error::{StunError, StunResult};
pub use server::{StunServer, StunServerStats};
pub use types::{NatType, PortPattern, PublicEndpoint, StunCredentials};
