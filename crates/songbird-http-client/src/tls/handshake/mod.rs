//! TLS 1.3 Handshake Module
//!
//! This module implements TLS 1.3 handshake logic by delegating cryptographic
//! operations to BearDog via JSON-RPC.
//!
//! ## Module Structure
//!
//! The handshake implementation is organized into cohesive, reusable modules:
//!
//! - `transcript`: Transcript tracking for key derivation
//! - `parser`: Handshake message parsing (RFC 8446 framing)
//! - `keys`: Key derivation types and cipher suite information
//! - (More modules to be extracted in Phase 2)
//!
//! ## Reusability
//!
//! These modules are designed to be reusable by BOTH TLS client and server:
//! - `transcript`: Used by client and server for key derivation
//! - `parser`: Used by both to parse handshake messages
//! - `keys`: Used by both for key management
//!
//! ## RFC 8446 Compliance
//!
//! All modules follow RFC 8446 (TLS 1.3) specifications precisely.

pub mod transcript;
pub mod parser;
pub mod keys;

// Re-export key types for convenience
pub use transcript::Transcript;
pub use parser::{HandshakeMessage, parse_handshake_messages, parse_single_handshake_message};
pub use keys::{CipherSuite, TrafficKeys};

// Re-export the main TlsHandshake from legacy file (to be refactored)
// This maintains backward compatibility while we refactor
#[path = "../handshake_legacy.rs"]
mod handshake_legacy;
pub use handshake_legacy::TlsHandshake;


