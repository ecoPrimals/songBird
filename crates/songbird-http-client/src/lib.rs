// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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
#![forbid(unsafe_code)]

pub mod beardog_client; // ✅ ACTIVE: Smart refactored module (7 sub-modules)
pub mod client;
mod connection; // ✅ NEW: Connection management (HTTP/HTTPS) (extracted from client.rs)
pub mod connection_pool; // ✅ NEW: Connection pooling for resource optimization (Feb 3, 2026)
pub mod crypto;
pub mod error;
pub mod http_config; // ✅ NEW: Adaptive HTTP configuration (User-Agent, routing, etc.)
pub mod ipc_client;
mod redirect; // ✅ NEW: HTTP redirect handling (extracted from client.rs)
mod request; // ✅ NEW: HTTP request building (extracted from client.rs)
mod response; // ✅ NEW: HTTP response parsing (extracted from client.rs)
pub mod tls;
pub mod types;

// Legacy implementation moved to archive/legacy_implementations/beardog_client_jan_26_2026/
// Refactored into beardog_client/ module (7 sub-modules) on January 26, 2026
// Use beardog_client module for all new code

pub use client::SongbirdHttpClient;
pub use error::{Error, Result};
pub use ipc_client::{Form, IpcHttpClient, Part, RequestBuilder, Response};
pub use types::{HttpRequest, HttpResponse};

// Re-export multipart module for convenience
pub use ipc_client::multipart;

// Re-export crypto capability types for agnostic usage
pub use crypto::{
    BearDogProvider, CryptoCapability, IpcEndpoint, TlsApplicationSecrets, TlsHandshakeSecrets,
    discover_beardog_socket, discover_crypto_capability, discover_ipc_endpoint,
    discover_neural_api_socket,
};

// Re-export BearDogClient and types
pub use beardog_client::{BearDogClient, BearDogMode, TlsSecrets};

// Re-export HTTP configuration types for adaptive behavior
pub use http_config::{
    DomainPattern, HeaderRule, HttpClientConfig, RedirectMode, SONGBIRD_VERSION, default_user_agent,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if this is a Pure Rust build (always true)
#[must_use]
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
