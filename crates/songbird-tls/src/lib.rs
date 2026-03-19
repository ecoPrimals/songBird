//! # Songbird TLS - Pure Rust TLS 1.3 Implementation
//!
//! A 100% Pure Rust implementation of TLS 1.3 designed for the biomeOS ecosystem.
//! All cryptographic operations are delegated to `BearDog` via runtime-discovered
//! Unix sockets, ensuring TRUE Pure Rust sovereignty with zero C dependencies.
//!
//! ## Architecture
//!
//! ```text
//! Pure Songbird TLS = Protocol (Songbird) + Crypto (BearDog)
//!
//! Songbird TLS:                  BearDog:
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
//! - Handshake: < 10ms (includes `BearDog` round-trips)
//! - Throughput: > 1 GB/s (CPU-bound, parallel streams)
//! - Memory: < 16 KB per connection
//!
//! ## References
//!
//! - [RFC 8446 - TLS 1.3](https://datatracker.ietf.org/doc/html/rfc8446)
//! - [The Illustrated TLS 1.3 Connection](https://tls13.xargs.org/)
#![forbid(unsafe_code)]

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

// Error types
pub mod error;
pub use error::{Result, TlsError};

// Re-exports for convenience
pub use handshake::{HandshakeState, HandshakeStateMachine};
pub use messages::{Certificate, ClientHello, Finished, ServerHello};
pub use record_layer::RecordLayer;
pub use server::{TlsAcceptor, TlsServerConfig, TlsStream};

// Certificate generation (hybrid standalone + BearDog)
pub use cert::generator::{CertGenerationMode, CertificateGenerator};

/// TLS 1.3 protocol version (0x0304)
pub const TLS_VERSION_1_3: u16 = 0x0304;

/// Legacy TLS 1.2 version (used in TLS 1.3 for compatibility)
pub const TLS_VERSION_1_2: u16 = 0x0303;

/// Maximum TLS record size (2^14 = 16384 bytes)
pub const MAX_RECORD_SIZE: usize = 16384;

/// Maximum handshake message size (256 KB)
pub const MAX_HANDSHAKE_SIZE: usize = 262_144;

// Cipher suite constants
pub const TLS_CHACHA20_POLY1305_SHA256: u16 = 0x1303;

// Content types
pub const CONTENT_TYPE_INVALID: u8 = 0;
pub const CONTENT_TYPE_CHANGE_CIPHER_SPEC: u8 = 20;
pub const CONTENT_TYPE_ALERT: u8 = 21;
pub const CONTENT_TYPE_HANDSHAKE: u8 = 22;
pub const CONTENT_TYPE_APPLICATION_DATA: u8 = 23;

// Handshake message types
pub const HANDSHAKE_TYPE_CLIENT_HELLO: u8 = 1;
pub const HANDSHAKE_TYPE_SERVER_HELLO: u8 = 2;
pub const HANDSHAKE_TYPE_NEW_SESSION_TICKET: u8 = 4;
pub const HANDSHAKE_TYPE_END_OF_EARLY_DATA: u8 = 5;
pub const HANDSHAKE_TYPE_ENCRYPTED_EXTENSIONS: u8 = 8;
pub const HANDSHAKE_TYPE_CERTIFICATE: u8 = 11;
pub const HANDSHAKE_TYPE_CERTIFICATE_REQUEST: u8 = 13;
pub const HANDSHAKE_TYPE_CERTIFICATE_VERIFY: u8 = 15;
pub const HANDSHAKE_TYPE_FINISHED: u8 = 20;
pub const HANDSHAKE_TYPE_KEY_UPDATE: u8 = 24;
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
        assert_eq!(MAX_HANDSHAKE_SIZE, 262144);
    }
}
