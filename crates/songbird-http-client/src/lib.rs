//! # Songbird HTTP Client - Pure Rust Tower Atomic
//!
//! A Pure Rust HTTP/HTTPS client with capability-based crypto delegation.
//! Uses the `CryptoCapability` trait for agnostic provider support.
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
//!                     │ CryptoCapability trait
//!                     │ (agnostic crypto operations)
//! ┌───────────────────▼──────────────────────────────────────┐
//! │          Crypto Provider (discovered at runtime)         │
//! │  - BearDog (default)                                    │
//! │  - Future: Neural API semantic translation              │
//! │  - Future: Other providers                              │
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
//! // Create client with automatic crypto discovery
//! let client = SongbirdHttpClient::new("/tmp/beardog.sock");
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
//! - ✅ TLS 1.3 with capability-based crypto delegation
//! - ✅ HTTP/1.1 and HTTP/2 support
//! - ✅ Tower Atomic architecture
//! - ✅ TRUE ecoBin compliant
//! - ✅ Agnostic crypto provider support

pub mod beardog_client;
pub mod client;
pub mod crypto;
pub mod error;
pub mod ipc_client;
pub mod tls;
pub mod types;

pub use client::SongbirdHttpClient;
pub use error::{Error, Result};
pub use ipc_client::{IpcHttpClient, RequestBuilder, Response, Form, Part};
pub use types::{HttpRequest, HttpResponse};

// Re-export multipart module for convenience
pub use ipc_client::multipart;

// Re-export crypto capability types for agnostic usage
pub use crypto::{
    discover_crypto_capability, BearDogProvider, CryptoCapability, TlsApplicationSecrets,
    TlsHandshakeSecrets,
};

// Re-export BearDogClient for backward compatibility
pub use beardog_client::BearDogClient;

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
