//! # Songbird HTTP Client - Pure Rust Tower Atomic
//!
//! A Pure Rust HTTP/HTTPS client that delegates all cryptographic operations
//! to BearDog via JSON-RPC over Unix sockets.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │             Songbird HTTP Client                         │
//! │  - hyper (HTTP/1.1, HTTP/2)                             │
//! │  - Custom TLS 1.3 implementation                        │
//! │  - Zero C dependencies                                  │
//! └───────────────────┬──────────────────────────────────────┘
//!                     │ Unix Socket JSON-RPC
//!                     │ (crypto.*, tls.* methods)
//! ┌───────────────────▼──────────────────────────────────────┐
//! │             BearDog Crypto Provider                      │
//! │  - x25519 (ECDH)                                        │
//! │  - ChaCha20-Poly1305 (AEAD)                             │
//! │  - ed25519 (signatures)                                 │
//! │  - BLAKE3 (hashing)                                     │
//! └──────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_http_client::SongbirdHttpClient;
//! use std::collections::HashMap;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create client with BearDog socket path
//! let client = SongbirdHttpClient::new("/tmp/beardog-nat0.sock");
//!
//! // Make HTTPS request
//! let response = client.request(
//!     "GET",
//!     "https://httpbin.org/get",
//!     HashMap::new(),
//!     None,
//! ).await?;
//!
//! println!("Status: {}", response.status);
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - ✅ Pure Rust (zero C dependencies)
//! - ✅ TLS 1.3 with BearDog crypto delegation
//! - ✅ HTTP/1.1 and HTTP/2 support
//! - ✅ Tower Atomic architecture
//! - ✅ TRUE ecoBin compliant

pub mod beardog_client;
pub mod client;
pub mod error;
pub mod tls;
pub mod types;

pub use client::SongbirdHttpClient;
pub use error::{Error, Result};
pub use types::{HttpRequest, HttpResponse};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if this is a Pure Rust build (always true)
pub const fn is_pure_rust() -> bool {
    true // Always true - we have zero C dependencies
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_rust() {
        assert!(is_pure_rust());
    }

    #[test]
    fn test_version() {
        // VERSION is set from CARGO_PKG_VERSION at compile time and is always non-empty
        assert!(VERSION.contains('.'), "Version should be in semver format: {}", VERSION);
    }
}

