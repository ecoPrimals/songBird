// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # songbird-tor-protocol
//!
//! Pure Rust Tor protocol implementation for Songbird.
//!
//! This crate implements a minimal subset of the Tor protocol focused on
//! .onion service hosting and client connectivity. All cryptographic operations
//! are delegated to `BearDog` (TRUE PRIMAL architecture).
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
//! 100% `BearDog` crypto delegation - zero direct crypto in this crate.

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
pub use crypto::BeardogCryptoClient;
pub use directory::Consensus;
pub use error::{Error, Result};

/// Tor client for connecting to .onion addresses
pub struct TorClient {
    _beardog: BeardogCryptoClient,
}

impl TorClient {
    /// Create new Tor client with `BearDog` delegation
    #[must_use]
    pub const fn new(beardog: BeardogCryptoClient) -> Self {
        Self {
            _beardog: beardog,
        }
    }
}

/// Tor service for hosting .onion addresses
pub struct TorService {
    _beardog: BeardogCryptoClient,
    _port: u16,
}

impl TorService {
    /// Create new Tor service
    ///
    /// # Errors
    ///
    /// Returns error if service creation fails.
    pub async fn new(beardog: BeardogCryptoClient, port: u16) -> Result<Self> {
        core::future::ready(()).await;
        Ok(Self {
            _beardog: beardog,
            _port: port,
        })
    }

    /// Get onion address (placeholder)
    #[must_use]
    pub const fn onion_address(&self) -> &'static str {
        "placeholder.onion"
    }
}
