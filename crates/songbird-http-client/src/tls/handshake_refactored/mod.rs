//! TLS 1.3 handshake implementation
//!
//! This module provides a Pure Rust TLS 1.3 handshake implementation
//! with crypto delegation to BearDog via JSON-RPC.
//!
//! ## Module Organization
//!
//! - `core` - `TlsHandshake` struct and constructors
//! - `transcript` - Transcript management (RFC 8446 Section 4.4.1)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_http_client::tls::handshake::TlsHandshake;
//! use std::sync::Arc;
//!
//! let crypto = Arc::new(BearDogProvider::new("/tmp/beardog.sock"));
//! let mut handshake = TlsHandshake::new(crypto);
//! let (keys, session_keys) = handshake.handshake(&mut stream, "api.github.com").await?;
//! ```

mod core;
mod transcript;

// Re-export main types
pub use core::TlsHandshake;

// Re-export from parent module
pub use crate::crypto::TlsHandshakeSecrets as TlsSecrets;
