// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![expect(
    clippy::clone_on_ref_ptr,
    reason = "Arc::clone() is idiomatic for shared ownership in async service contexts"
)]
#![expect(
    clippy::expect_used,
    reason = "TLS protocol invariants use expect() for panic-on-violation semantics"
)]
#![cfg_attr(test, allow(clippy::unwrap_used, reason = "test assertions"))]
#![warn(missing_docs)]

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
//! │  - Security / crypto provider (default)                 │
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
//! let client = SongbirdHttpClient::new("/tmp/security-provider.sock");
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

/// Shared async I/O helpers for JSON-RPC socket communication.
pub(crate) mod io_util;

/// Security-provider RPC client submodules (refactored HTTP/TLS helpers).
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod security_rpc_client;

/// High-level `SongbirdHttpClient` and HTTP/HTTPS request execution.
pub mod client;
mod connection; // ✅ NEW: Connection management (HTTP/HTTPS) (extracted from client.rs)
/// Bounded connection pool with acquire/return and health-aware cleanup.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod connection_pool;
/// Crypto capability traits, security-provider discovery, and TLS secret bags.
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod crypto;
/// Error types and `Result` alias for this crate.
pub mod error;
/// Adaptive HTTP client config: headers, redirects, timeouts, domain rules.
pub mod http_config;
/// IPC-backed HTTP client, multipart forms, and request builders.
pub mod ipc_client;
mod redirect; // ✅ NEW: HTTP redirect handling (extracted from client.rs)
mod request; // ✅ NEW: HTTP request building (extracted from client.rs)
mod response; // ✅ NEW: HTTP response parsing (extracted from client.rs)
/// Pure Rust TLS 1.3 stack (record layer, handshake, adaptive extensions).
#[allow(missing_docs, reason = "internal module; public items documented incrementally")]
pub mod tls;
/// HTTP request/response value types shared by the client stack.
pub mod types;

/// High-level HTTP/HTTPS client built on the internal stack and TLS integration.
pub use client::SongbirdHttpClient;
/// Error type and `Result` alias for this crate.
pub use error::{Error, Result};
/// IPC-backed HTTP client, request builder, and multipart types for security-provider channels.
pub use ipc_client::{Form, IpcHttpClient, Part, RequestBuilder, Response};
/// Request and response value types used by [`SongbirdHttpClient`].
pub use types::{HttpRequest, HttpResponse};

// Re-export multipart module for convenience
/// Multipart form helpers used with [`IpcHttpClient`] and IPC transports.
pub use ipc_client::multipart;

/// Crypto capability traits, TLS secret bags, and runtime discovery helpers for security-provider sockets.
pub use crypto::{
    CryptoCapability, IpcEndpoint, SecurityCryptoProvider, TlsApplicationSecrets,
    TlsHandshakeSecrets, discover_crypto_capability, discover_ipc_endpoint,
    discover_neural_api_socket, discover_security_socket,
};

/// Lower-level security-provider RPC client and TLS secret handles for advanced use.
pub use security_rpc_client::{
    BtspCipher, BtspNegotiation, BtspSessionCreated, BtspSessionVerified, SecurityRpcClient,
    SecurityRpcMode, TlsSecrets,
};

// Re-export HTTP configuration types for adaptive behavior
/// Adaptive HTTP client configuration (headers, redirects, and version string constant).
pub use http_config::{
    DomainPattern, HeaderRule, HttpClientConfig, RedirectMode, SONGBIRD_VERSION, default_user_agent,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Base64-encode bytes using standard alphabet (re-exports `base64` for ecosystem use).
#[must_use]
pub fn base64_encode(input: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input)
}

/// Base64-decode a string using standard alphabet.
///
/// # Errors
///
/// Returns error if the input is not valid base64.
pub fn base64_decode(input: &str) -> std::result::Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(input)
}

/// Check if this is a Pure Rust build (always true)
#[must_use]
pub const fn is_pure_rust() -> bool {
    true // Always true - we have zero C dependencies
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_pure_rust() {
        assert!(is_pure_rust());
    }

    #[test]
    fn test_version() {
        // VERSION is set from CARGO_PKG_VERSION at compile time and is always non-empty
        assert!(VERSION.contains('.'), "Version should be in semver format: {VERSION}");
    }

    #[test]
    fn base64_encode_decode_roundtrip() {
        let bytes: &[u8] = b"hello songbird \x00\xff";
        let encoded = base64_encode(bytes);
        let decoded = base64_decode(&encoded).expect("valid base64 from encode");
        assert_eq!(decoded, bytes);
    }
}
