// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 implementation with security-provider crypto delegation
//!
//! This module implements a Pure Rust TLS 1.3 client AND server by delegating all
//! cryptographic operations to the `security provider` via JSON-RPC.
//!
//! ## Architecture (Smart Refactored - January 26, 2026)
//!
//! The TLS handshake implementation has been refactored from a 3,128-line monolith
//! into a clean modular architecture:
//!
//! - `handshake_refactored/core.rs` - `TlsHandshake` struct and constructors (84 lines)
//! - `handshake_refactored/transcript.rs` - Transcript management (459 lines)
//! - `handshake_refactored/extensions.rs` - Extension builders (438 lines)
//! - `handshake_refactored/record_io.rs` - TLS record I/O (423 lines)
//! - `handshake_refactored/handshake_flow.rs` - Main handshake logic (1,364 lines)
//! - `handshake_refactored/application_data.rs` - App data crypto (115 lines)
//!
//! The legacy `handshake_legacy.rs` is preserved as a fossil record.

pub mod adaptive;
pub mod alert;
pub mod config;
pub mod handshake_refactored; // ✅ ACTIVE: Smart refactored handshake (production)
pub mod handshake_v2; // Alternative modularized handshake
pub mod negotiation;
pub mod profiler;
pub mod record;
pub(crate) mod record_crypto;
pub mod server; // ✅ REFACTORED: Modular TLS 1.3 server (6 focused modules)
pub mod session;
pub mod version; // ✅ NEW: TLS version config (1.3 + secure 1.2 fallback)

// Legacy implementation moved to archive/legacy_implementations/tls_handshake_jan_26_2026/
// Refactored into handshake_refactored/ module (6 sub-modules) on January 26, 2026
// Use handshake_refactored module for all new code

// Compatibility re-export for existing code
pub mod handshake {
    pub use super::handshake_refactored::*;
}

pub use adaptive::AdaptiveExtensions;
pub use alert::{AlertDescription, AlertLevel, TlsAlert};
pub use config::{
    CipherStrategy, CipherSuiteSet, ExtensionSet, ExtensionStrategy, ExtensionType,
    FallbackStrategy, TlsConfig,
};
pub use handshake_refactored::TlsHandshake; // ✅ ACTIVE: Refactored implementation
pub use handshake_refactored::TlsSecrets;
pub use profiler::{ServerProfile, ServerProfiler};
pub use record::TlsRecordLayer;
pub use server::TlsServer; // ✅ Refactored modular implementation
pub use session::TlsSession;
pub use version::{
    NegotiatedVersion, SecurityPolicy, TLS_1_2_EXTENDED_CIPHERS, TLS_1_2_SECURE_CIPHERS,
    TlsVersion, TlsVersionConfig, detect_server_version, tls_1_2_cipher_name,
};

/// TLS 1.3 version
pub const TLS_1_3: u16 = 0x0304;

/// TLS 1.2 version (for compatibility mode)
pub const TLS_1_2: u16 = 0x0303;

/// Supported cipher suites
pub const CIPHER_SUITES: &[u16] = &[
    0x1301, // TLS_AES_128_GCM_SHA256
    0x1302, // TLS_AES_256_GCM_SHA384
    0x1303, // TLS_CHACHA20_POLY1305_SHA256
];

/// Content types
pub mod content_type {
    pub const CHANGE_CIPHER_SPEC: u8 = 20;
    pub const ALERT: u8 = 21;
    pub const HANDSHAKE: u8 = 22;
    pub const APPLICATION_DATA: u8 = 23;
}

/// Handshake types
pub mod handshake_type {
    pub const CLIENT_HELLO: u8 = 1;
    pub const SERVER_HELLO: u8 = 2;
    pub const ENCRYPTED_EXTENSIONS: u8 = 8;
    pub const CERTIFICATE: u8 = 11;
    pub const CERTIFICATE_VERIFY: u8 = 15;
    pub const FINISHED: u8 = 20;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_versions() {
        assert_eq!(TLS_1_3, 0x0304);
        assert_eq!(TLS_1_2, 0x0303);
    }

    #[test]
    fn test_cipher_suites() {
        assert_eq!(CIPHER_SUITES.len(), 3, "Should have 3 TLS 1.3 cipher suites");
        assert!(CIPHER_SUITES.contains(&0x1303)); // ChaCha20-Poly1305
    }
}
