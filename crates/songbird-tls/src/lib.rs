// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    reason = "Arc::clone() is idiomatic for shared ownership in async TLS contexts"
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions"))]
//! # Songbird TLS - Pure Rust TLS 1.3 Implementation
//!
//! A 100% Pure Rust implementation of TLS 1.3 designed for the biomeOS ecosystem.
//! All cryptographic operations are delegated to the security (crypto) provider via runtime-discovered
//! Unix sockets, ensuring TRUE Pure Rust sovereignty with zero C dependencies.
//!
//! ## Architecture
//!
//! ```text
//! Pure Songbird TLS = Protocol (Songbird) + Crypto (security provider)
//!
//! Songbird TLS:                  Security provider:
//! ├── Handshake State Machine    ├── Ed25519 (signing)
//! ├── Record Layer (framing)     ├── X25519 (key exchange)
//! ├── Key Schedule (HKDF)        ├── ChaCha20-Poly1305 (AEAD)
//! ├── Certificate Validation     ├── Blake3 (hashing)
//! └── Alert Protocol             └── HMAC-SHA256 (KDF)
//! ```
//!
//! ## Principles
//!
//! 1. **100% Pure Rust** - Zero C dependencies (TRUE ecoBin)
//! 2. **Capability-Based** - Discovers crypto provider at runtime
//! 3. **Protocol-Agnostic** - Foundation for HTTP/1.1, HTTP/2, HTTP/3, WebSocket
//! 4. **Deep Debt Solution** - Own the entire stack, not a workaround
//! 5. **Modern Idiomatic Rust** - async/await, Result<T, E>, no unsafe
//! 6. **Tower Architecture** - Designed for biomeOS relay deployments
//!
//! ## Example (Coming in Phase 4)
//!
//! ```rust,ignore
//! use songbird_tls::TlsConfig;
//! let _config = TlsConfig::default();
//! ```
//!
//! ## TLS 1.3 Support
//!
//! - ✅ TLS 1.3 only (no TLS 1.2 fallback for security)
//! - ✅ `TLS_CHACHA20_POLY1305_SHA256` cipher suite
//! - ✅ X25519 key exchange (ECDHE)
//! - ✅ Ed25519 certificates
//! - ✅ Server-side handshake
//! - 🔜 Client-side handshake (Q2 2026)
//! - 🔜 Session resumption (Q2 2026)
//!
//! ## Performance Targets
//!
//! - Handshake: < 10ms (includes security-provider round-trips)
//! - Throughput: > 1 GB/s (CPU-bound, parallel streams)
//! - Memory: < 16 KB per connection
//!
//! ## References
//!
//! - [RFC 8446 - TLS 1.3](https://datatracker.ietf.org/doc/html/rfc8446)
//! - [The Illustrated TLS 1.3 Connection](https://tls13.xargs.org/)
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Core modules
pub mod cert;
pub mod codec;
pub mod crypto;
pub mod handshake;
pub mod key_schedule;
pub mod messages;
pub mod record_layer;
pub mod server; // NEW: High-level server API
pub mod socket_discovery; // NEW: XDG-compliant socket discovery

/// Re-export of [`crate::crypto::SecurityTlsCryptoClient`].
pub use crypto::SecurityTlsCryptoClient;
/// Deprecated alias for [`SecurityTlsCryptoClient`].
#[deprecated(note = "use SecurityTlsCryptoClient (capability-based naming)")]
pub type LegacySecurityTlsCryptoClient = SecurityTlsCryptoClient;

// Error types
pub mod error;
pub use error::{Result, TlsError};

// Re-exports for convenience
pub use handshake::{HandshakeState, HandshakeStateMachine};
pub use messages::{Certificate, ClientHello, Finished, ServerHello};
pub use record_layer::RecordLayer;
pub use server::{TlsAcceptor, TlsServerConfig, TlsStream};

// Certificate generation (hybrid standalone + security provider)
pub use cert::generator::{CertGenerationMode, CertificateGenerator};

/// TLS 1.3 protocol version (0x0304)
pub const TLS_VERSION_1_3: u16 = 0x0304;

/// Legacy TLS 1.2 version (used in TLS 1.3 for compatibility)
pub const TLS_VERSION_1_2: u16 = 0x0303;

/// Maximum TLS record size (2^14 = 16384 bytes)
pub const MAX_RECORD_SIZE: usize = 16384;

/// Maximum handshake message size (256 KB)
pub const MAX_HANDSHAKE_SIZE: usize = 262_144;

/// ChaCha20-Poly1305 with SHA-256 (`TLS_CHACHA20_POLY1305_SHA256`, RFC 8446).
pub const TLS_CHACHA20_POLY1305_SHA256: u16 = 0x1303;

/// Reserved TLS record content type (invalid on the wire).
pub const CONTENT_TYPE_INVALID: u8 = 0;
/// `ChangeCipherSpec` content type (compatibility; TLS 1.3 uses other types for handshakes).
pub const CONTENT_TYPE_CHANGE_CIPHER_SPEC: u8 = 20;
/// Alert protocol records.
pub const CONTENT_TYPE_ALERT: u8 = 21;
/// Handshake message records.
pub const CONTENT_TYPE_HANDSHAKE: u8 = 22;
/// Application data records (post-handshake payload).
pub const CONTENT_TYPE_APPLICATION_DATA: u8 = 23;

/// `client_hello` handshake payload type.
pub const HANDSHAKE_TYPE_CLIENT_HELLO: u8 = 1;
/// `server_hello` handshake payload type.
pub const HANDSHAKE_TYPE_SERVER_HELLO: u8 = 2;
/// `new_session_ticket` handshake payload type.
pub const HANDSHAKE_TYPE_NEW_SESSION_TICKET: u8 = 4;
/// `end_of_early_data` handshake payload type.
pub const HANDSHAKE_TYPE_END_OF_EARLY_DATA: u8 = 5;
/// `encrypted_extensions` handshake payload type.
pub const HANDSHAKE_TYPE_ENCRYPTED_EXTENSIONS: u8 = 8;
/// `certificate` handshake payload type.
pub const HANDSHAKE_TYPE_CERTIFICATE: u8 = 11;
/// `certificate_request` handshake payload type.
pub const HANDSHAKE_TYPE_CERTIFICATE_REQUEST: u8 = 13;
/// `certificate_verify` handshake payload type.
pub const HANDSHAKE_TYPE_CERTIFICATE_VERIFY: u8 = 15;
/// `finished` handshake payload type.
pub const HANDSHAKE_TYPE_FINISHED: u8 = 20;
/// `key_update` handshake payload type.
pub const HANDSHAKE_TYPE_KEY_UPDATE: u8 = 24;
/// Synthetic `message_hash` handshake payload type (used in some transcript constructions).
pub const HANDSHAKE_TYPE_MESSAGE_HASH: u8 = 254;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        // Verify TLS version constants
        assert_eq!(TLS_VERSION_1_3, 0x0304);
        assert_eq!(TLS_VERSION_1_2, 0x0303);

        // Verify cipher suite constant
        assert_eq!(TLS_CHACHA20_POLY1305_SHA256, 0x1303);

        // Verify content types
        assert_eq!(CONTENT_TYPE_HANDSHAKE, 22);
        assert_eq!(CONTENT_TYPE_APPLICATION_DATA, 23);
        assert_eq!(CONTENT_TYPE_ALERT, 21);
    }

    #[test]
    fn test_max_sizes() {
        // Verify size constants
        assert_eq!(MAX_RECORD_SIZE, 16384);
        assert_eq!(MAX_HANDSHAKE_SIZE, 262_144);
    }
}
