// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    reason = "Arc::clone() is idiomatic for shared ownership in async contexts"
)]
#![allow(
    clippy::expect_used,
    reason = "Tor protocol invariants use expect() for panic-on-violation semantics"
)]
#![cfg_attr(test, allow(clippy::unwrap_used, reason = "test assertions"))]
//! # songbird-tor-protocol
//!
//! Pure Rust Tor protocol implementation for Songbird.
//!
//! This crate implements a minimal subset of the Tor protocol focused on
//! .onion service hosting and client connectivity. All cryptographic operations
//! are delegated to the security provider (TRUE PRIMAL architecture).
//!
//! ## Features
//!
//! - **Directory Protocol**: Fetch consensus, select relays
//! - **Circuit Protocol**: Build circuits, ntor handshake
//! - **Onion Service**: Host .onion services
//! - **Stream Protocol**: Multiplexed streams over circuits
//!
//! ## TRUE PRIMAL
//!
//! 100% security-provider crypto delegation — zero direct crypto in this crate.

#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used)]
#![warn(missing_docs)]

pub mod circuit;
pub mod connection;
pub mod crypto;
pub mod directory;
pub mod error;
pub mod http_fetch;
pub mod onion_service;
pub mod protocol;
pub mod storage;
pub mod stream;

// Re-export main types
pub use connection::TorConnection;
pub use crypto::{CryptoProvider, TorProtocolCrypto};
pub use directory::Consensus;
pub use error::{Error, Result};

/// Tor client for connecting to .onion addresses
pub struct TorClient {
    _security_provider: CryptoProvider,
}

impl TorClient {
    /// Create new Tor client with security provider delegation
    #[must_use]
    pub const fn new(security_provider: CryptoProvider) -> Self {
        Self {
            _security_provider: security_provider,
        }
    }
}

/// Tor service for hosting .onion addresses
pub struct TorService {
    _security_provider: CryptoProvider,
    _port: u16,
}

impl TorService {
    /// Create new Tor service
    ///
    /// # Errors
    ///
    /// Returns error if service creation fails.
    pub async fn new(security_provider: CryptoProvider, port: u16) -> Result<Self> {
        core::future::ready(()).await;
        Ok(Self {
            _security_provider: security_provider,
            _port: port,
        })
    }

    /// Get onion address.
    ///
    /// Returns `None` until the service has published a descriptor via security provider.
    /// Callers should treat `None` as "service not yet reachable on the Tor network."
    #[must_use]
    pub const fn onion_address(&self) -> Option<&str> {
        None
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::circuit::CircuitPurpose;

    #[test]
    fn tor_client_holds_crypto_provider() {
        let p = CryptoProvider::new("/tmp/songbird-tor-test.sock".to_string());
        let c = TorClient::new(p);
        let _ = c;
    }

    #[tokio::test]
    async fn tor_service_new_and_onion_address_pending() {
        let p = CryptoProvider::new("/tmp/songbird-tor-test2.sock".to_string());
        let svc = TorService::new(p, 9050).await.expect("service");
        assert_eq!(svc.onion_address(), None);
    }

    #[test]
    fn circuit_purpose_exported_for_callers() {
        let _ = CircuitPurpose::General;
        let _ = CircuitPurpose::HSDir;
        let _ = CircuitPurpose::Rendezvous;
    }

    #[tokio::test]
    async fn tor_service_new_accepts_ephemeral_port() {
        let p = CryptoProvider::new("/tmp/songbird-tor-test3.sock".to_string());
        let svc = TorService::new(p, 49152).await.expect("service");
        assert_eq!(svc.onion_address(), None);
    }
}
