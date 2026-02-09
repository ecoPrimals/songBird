//! # songbird-tor-protocol
//!
//! Pure Rust Tor protocol implementation for Songbird.
//!
//! This crate implements a minimal subset of the Tor protocol focused on
//! .onion service hosting and client connectivity. All cryptographic operations
//! are delegated to BearDog (TRUE PRIMAL architecture).
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
//! 100% BearDog crypto delegation - zero direct crypto in this crate.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![warn(missing_docs)]

pub mod directory;
pub mod circuit;
pub mod connection;
pub mod onion_service;
pub mod stream;
pub mod crypto;
pub mod protocol;
pub mod storage;
pub mod error;
pub mod http_fetch;

// Re-export main types
pub use error::{Error, Result};
pub use directory::Consensus;
pub use crypto::BeardogCryptoClient;
pub use connection::TorConnection;

/// Tor client for connecting to .onion addresses
pub struct TorClient {
    _beardog: BeardogCryptoClient,
}

impl TorClient {
    /// Create new Tor client with BearDog delegation
    pub fn new(beardog: BeardogCryptoClient) -> Self {
        Self { _beardog: beardog }
    }
}

/// Tor service for hosting .onion addresses
pub struct TorService {
    _beardog: BeardogCryptoClient,
    _port: u16,
}

impl TorService {
    /// Create new Tor service
    pub async fn new(beardog: BeardogCryptoClient, port: u16) -> Result<Self> {
        Ok(Self { _beardog: beardog, _port: port })
    }
    
    /// Get onion address (placeholder)
    pub fn onion_address(&self) -> &str {
        "placeholder.onion"
    }
}
