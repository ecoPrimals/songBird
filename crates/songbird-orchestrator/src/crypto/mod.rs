// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Crypto Module — Capability-Based Crypto Provider Discovery & Delegation
//!
//! This module implements 100% Pure Rust TLS by delegating ALL crypto operations
//! to whichever primal provides the "crypto" capability, discovered at runtime.
//!
//! **Architecture**:
//! - Songbird: TLS protocol logic (Pure Rust — songbird-tls crate)
//! - Crypto Provider: ALL crypto operations (discovered at runtime)
//! - Result: 100% Pure Rust HTTPS! 🎉
//!
//! **Discovery**: Capability-based — Songbird discovers crypto providers
//! by searching for "crypto" capability sockets, not by hardcoded names.
//!
//! **Note**: The rustls integration was pivoted to Pure Songbird TLS.
//! See songbird-tls crate for the complete TLS 1.3 implementation.

pub mod beardog_crypto_client;
pub mod discovery;
pub mod provider;

// Re-export capability-based abstractions (preferred API — TRUE PRIMAL!)
pub use provider::{CryptoProvider, UnixSocketCryptoProvider, discover_crypto_provider};

// Re-export low-level functions for backward compatibility
pub use beardog_crypto_client::{
    blake3_hash, chacha20_poly1305_decrypt, chacha20_poly1305_encrypt, hmac_sha256, sign_ed25519,
    verify_ed25519, x25519_derive_secret, x25519_generate_ephemeral,
};

// Capability-based discovery (preferred API)
pub use discovery::{
    discover_crypto_socket, discover_crypto_socket_for_family, discover_crypto_socket_for_purpose,
    is_crypto_available,
};

// Backward-compatible aliases
pub use discovery::{
    get_beardog_crypto_socket, get_beardog_crypto_socket_for_family,
    get_beardog_crypto_socket_for_purpose, is_beardog_crypto_available,
};
