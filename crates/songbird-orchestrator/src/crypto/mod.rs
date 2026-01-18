//! Crypto Module - Pure Rust TLS via BearDog Delegation
//!
//! This module implements 100% Pure Rust TLS by delegating ALL crypto operations
//! to BearDog via JSON-RPC over Unix sockets.
//!
//! **Architecture**:
//! - Songbird: TLS protocol logic (Pure Rust state machine)
//! - BearDog: ALL crypto operations (Pure Rust RustCrypto!)
//! - Result: 100% Pure Rust HTTPS! 🎉
//!
//! **Status**: Phase 1 - Foundation (Week 1)
//! - [x] BearDog crypto client (JSON-RPC)
//! - [x] Capability-based discovery
//! - [ ] rustls CryptoProvider implementation
//! - [ ] Integration & testing

pub mod beardog_crypto_client;
pub mod discovery;
pub mod provider;

// Re-export capability-based abstractions (preferred API - TRUE PRIMAL!)
pub use provider::{
    discover_crypto_provider,
    CryptoProvider,
    UnixSocketCryptoProvider,
};

// Re-export low-level functions for backward compatibility
pub use beardog_crypto_client::{
    blake3_hash,
    chacha20_poly1305_decrypt,
    chacha20_poly1305_encrypt,
    hmac_sha256,
    sign_ed25519,
    verify_ed25519,
    x25519_derive_secret,
    x25519_generate_ephemeral,
};

pub use discovery::{
    get_beardog_crypto_socket,
    get_beardog_crypto_socket_for_family,
    get_beardog_crypto_socket_for_purpose,
    is_beardog_crypto_available,
};

