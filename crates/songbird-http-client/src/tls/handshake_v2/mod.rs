//! TLS 1.3 handshake implementation (modularized)
//!
//! This module provides a production-grade TLS 1.3 handshake implementation
//! following RFC 8446, refactored into focused sub-modules for maintainability.
//!
//! ## Architecture
//!
//! The handshake is split into logical components:
//! - `protocol`: Protocol constants and helpers
//! - `transcript`: Transcript hash management
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_http_client::tls::handshake::TlsHandshake;
//! use songbird_http_client::crypto::CryptoCapability;
//! use std::sync::Arc;
//!
//! let crypto = Arc::new(BearDogProvider::new("/tmp/beardog.sock"));
//! let handshake = TlsHandshake::new(crypto);
//! ```

//! TLS 1.3 handshake implementation (modularized)
//!
//! This module provides a production-grade TLS 1.3 handshake implementation
//! following RFC 8446, refactored into focused sub-modules for maintainability.

pub mod client_hello;
pub mod finished;
pub mod keys;
pub mod parser;
pub mod protocol;
pub mod server_hello;
pub mod transcript;

// Re-export main types
pub use client_hello::{generate_random, ClientHelloBuilder};
pub use transcript::Transcript;

// TODO: Integrate remaining modules into main handshake flow
// - encryption.rs - Application data encryption
// - decryption.rs - Application data decryption
// - certificates.rs - Certificate processing (may be in parser.rs)
