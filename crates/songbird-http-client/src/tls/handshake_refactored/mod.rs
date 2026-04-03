// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 handshake implementation (REFACTORED - Smart Modular Architecture)
//!
//! This module provides a Pure Rust TLS 1.3 handshake implementation
//! with crypto delegation to the `security provider` via JSON-RPC.
//!
//! ## Module Organization (Smart Refactoring Complete!)
//!
//! This implementation was refactored from a 3,086-line monolith into 6 logical modules:
//!
//! - `core` - `TlsHandshake` struct, constructors, and core state (84 lines)
//! - `transcript` - Transcript management (RFC 8446 Section 4.4.1) (459 lines)
//! - `extensions` - Strategy-based extension builders (438 lines)
//! - `record_io` - TLS record layer I/O and decryption (423 lines)
//! - `handshake_flow` - Main 13-step handshake orchestration (1,363 lines)
//! - `application_data` - Application data encryption/decryption (115 lines)
//!
//! **Total**: 2,882 lines across 6 modules (from 3,086-line monolith)
//!
//! ## Design Principles
//!
//! - **Logical separation**: Each module has a clear, single responsibility
//! - **RFC 8446 compliance**: Full TLS 1.3 specification adherence
//! - **Crypto delegation**: All cryptographic operations via `security provider` JSON-RPC
//! - **Zero behavioral changes**: Functionally identical to legacy implementation
//! - **Production-ready**: Incremental refactor, shipped after each session
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_http_client::tls::handshake::TlsHandshake;
//! use songbird_http_client::crypto::SecurityCryptoProvider;
//! use std::sync::Arc;
//!
//! let crypto = Arc::new(SecurityCryptoProvider::new("/tmp/beardog.sock"));
//! let mut handshake = TlsHandshake::new(crypto);
//! let session_keys = handshake.handshake(&mut stream, "api.github.com").await?;
//! ```

mod application_data;
mod client_finished;
mod core;
mod extensions;
mod handshake_flow;
mod post_handshake;
mod record_io;
mod transcript;

/// Encode a length that must fit in a TLS `uint16` field (RFC 5246 / 8446).
pub(super) fn tls_wire_u16(len: usize) -> crate::error::Result<u16> {
    u16::try_from(len).map_err(|_| {
        crate::error::Error::TlsHandshake("TLS message length exceeds u16 maximum".into())
    })
}

// Re-export main types
pub use core::TlsHandshake;

// Re-export from parent module
pub use crate::crypto::TlsHandshakeSecrets as TlsSecrets;
