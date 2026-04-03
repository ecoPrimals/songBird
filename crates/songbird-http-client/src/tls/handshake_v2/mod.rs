// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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
//! use songbird_http_client::crypto::{CryptoCapability, SecurityCryptoProvider};
//! use std::sync::Arc;
//!
//! let crypto = Arc::new(SecurityCryptoProvider::new("/tmp/beardog.sock"));
//! let handshake = TlsHandshake::new(crypto);
//! ```

pub mod client_hello;
pub mod finished;
pub mod keys;
pub mod parser;
pub mod protocol;
pub mod server_hello;
pub mod transcript;

// Re-export main types
pub use client_hello::{ClientHelloBuilder, generate_random};
pub use transcript::Transcript;

// v2 layout: client_hello / transcript / parser / server_hello / keys / finished are the integrated
// surface; record-layer encrypt/decrypt helpers live alongside the legacy handshake until unified.
