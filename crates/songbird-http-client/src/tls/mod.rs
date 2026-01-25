//! TLS 1.3 implementation with BearDog crypto delegation
//!
//! This module implements a Pure Rust TLS 1.3 client AND server by delegating all
//! cryptographic operations to BearDog via JSON-RPC.

pub mod adaptive;
pub mod alert;
pub mod config;
pub mod handshake_legacy;
pub mod handshake_v2; // New modularized handshake (Phase 4 refactoring)
pub mod negotiation;
pub mod profiler;
pub mod record;
pub mod server;
// TODO(Phase 4): Temporarily disabled until handshake_v2::keys module is complete
// pub mod server_complete; // Complete TLS 1.3 server implementation
pub mod session;

// Compatibility re-export for existing code
pub mod handshake {
    pub use super::handshake_legacy::*;
}

pub use adaptive::AdaptiveExtensions;
pub use alert::{AlertDescription, AlertLevel, TlsAlert};
pub use config::{
    CipherStrategy, CipherSuiteSet, ExtensionSet, ExtensionStrategy, ExtensionType,
    FallbackStrategy, TlsConfig,
};
pub use handshake_legacy::TlsHandshake; // Legacy implementation (to be migrated)
pub use profiler::{ServerProfile, ServerProfiler};
pub use record::TlsRecordLayer;
pub use server::TlsServer;
// TODO(Phase 4): Temporarily disabled until handshake_v2::keys module is complete
// pub use server_complete::TlsServer as TlsServerComplete; // Complete implementation
pub use session::TlsSession;

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
