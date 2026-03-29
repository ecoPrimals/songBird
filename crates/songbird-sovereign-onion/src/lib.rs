// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    reason = "Arc::clone() is idiomatic for shared ownership in async contexts"
)]
#![allow(
    clippy::expect_used,
    reason = "onion protocol invariants use expect() for panic-on-violation semantics"
)]
#![cfg_attr(test, allow(clippy::unwrap_used, reason = "test assertions"))]
//! # Songbird Sovereign Onion
//!
//! Pure Rust minimal onion service protocol for sovereign device-to-device communication.
//!
//! ## Overview
//!
//! Provides cryptographically-derived `.onion` addresses for reachability across NAT
//! without port forwarding. Inspired by Tor v3 onion services but simplified for
//! family mesh use cases.
//!
//! ## Features
//!
//! - **100% Pure Rust** - Zero C dependencies
//! - **Ed25519 Identity** - Cryptographic device IDs
//! - **X25519 Key Exchange** - Forward secrecy
//! - **ChaCha20-Poly1305** - Fast AEAD encryption
//! - **Sled Persistence** - Identity and peer storage
//! - **Minimal Protocol** - ~10% of Tor complexity
//!
//! ## Example
//!
//! ```rust,ignore
//! use songbird_sovereign_onion::{OnionService, OnionConnector};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create onion service
//! let service = OnionService::new(9735).await?;
//! println!("Address: {}", service.onion_address());
//!
//! // Connect to onion address
//! let connector = OnionConnector::new();
//! let conn = connector.connect("vww6ybal...npyyd.onion", 9735).await?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod address;
pub mod beardog_crypto;
pub mod connector;
pub mod crypto;
pub mod error;
pub mod keys;
pub mod protocol;
pub mod service;
pub mod storage;

// Re-exports - TRUE PRIMAL (BearDog-delegated)
// ✅ Production exports: ZERO direct crypto, 100% BearDog delegation
pub use address::{derive_onion_address_via_beardog, validate_onion_address_via_beardog};
pub use beardog_crypto::{BeardogCryptoClient, Ed25519Keypair, X25519Keypair};
pub use connector::OnionConnector;
pub use crypto::{decrypt_data_via_beardog, encrypt_data_via_beardog};
pub use error::{OnionError, Result};
pub use keys::OnionIdentity;
pub use storage::OnionStorage;

// ✅ Phase 3 Complete: OnionService & OnionConnector with BearDog
pub use connector::OnionConnection;
pub use service::OnionService;

// Re-exports - Standalone (for testing/offline)
// ⚠️ These methods use direct crypto - testing only!
#[cfg(feature = "standalone")]
pub use address::{derive_onion_address, parse_onion_address, validate_onion_address};
#[cfg(feature = "standalone")]
pub use crypto::{decrypt_data, encrypt_data};
#[cfg(feature = "standalone")]
pub use storage::OnionStorage as OnionStorageStandalone; // Standalone storage methods
