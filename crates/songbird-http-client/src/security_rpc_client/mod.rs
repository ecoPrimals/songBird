// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security-provider RPC client for crypto operations
//!
//! Communicates with the `security provider` via JSON-RPC 2.0 over Unix sockets, supporting
//! both Direct mode (testing) and Neural API mode (production).
//!
//! ## Architecture (Smart Refactored - January 26, 2026)
//!
//! This module was refactored from a 2,020-line monolith into 7 logical modules:
//!
//! - `types.rs` - JSON-RPC types and `TlsSecrets` struct (~120 lines)
//! - `core.rs` - `SecurityRpcClient` struct, `SecurityRpcMode` enum, constructors (~180 lines)
//! - `rpc.rs` - Base RPC call method with semantic routing (~270 lines)
//! - `key_exchange.rs` - X25519 keypair generation, ECDH (~80 lines)
//! - `tls_secrets.rs` - TLS 1.3 key derivation (RFC 8446) (~300 lines)
//! - `aead.rs` - AEAD encrypt/decrypt (ChaCha20-Poly1305, AES-GCM) (~300 lines)
//! - `hash.rs` - SHA-256, SHA-384, HKDF operations (~100 lines)
//!
//! **Total**: ~1,350 lines across 7 modules (from 2,020-line monolith)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_http_client::SecurityRpcClient;
//!
//! // From environment (recommended — honors --security-socket / BEARDOG_SOCKET)
//! let client = SecurityRpcClient::from_env();
//!
//! // Explicit Neural API mode
//! let client = SecurityRpcClient::new_neural_api("/run/user/1000/biomeos/neural-api.sock");
//!
//! // Testing: Use Direct mode
//! let client = SecurityRpcClient::new_direct("/run/user/1000/biomeos/security.sock");
//!
//! // Generate keypair
//! let (public_key, private_key) = client.generate_keypair().await?;
//! ```

mod aead;
mod auth;
pub mod btsp;
mod core;
mod hash;
mod key_exchange;
mod rpc;
mod tls_secrets;
mod types;

// Re-export public API
pub use btsp::{BtspCipher, BtspNegotiation, BtspSessionCreated, BtspSessionVerified};
pub use core::{SecurityRpcClient, SecurityRpcMode};
pub use types::TlsSecrets;

// ═══════════════════════════════════════════════════════════════════════════
// NOTE: Semantic capability.call Integration
// ═══════════════════════════════════════════════════════════════════════════
//
// `SecurityRpcClient` supports semantic routing via Neural API through its mode enum:
//
// - Direct mode: Talk directly to the crypto/security provider (testing)
//   Set: SECURITY_PROVIDER_MODE=direct (or BEARDOG_MODE=direct), SECURITY_PROVIDER_SOCKET or BEARDOG_SOCKET
//
// - Neural API mode: Route through Neural API (production)
//   Set: SECURITY_PROVIDER_MODE=neural (or BEARDOG_MODE=neural), NEURAL_API_SOCKET or BEARDOG_SOCKET
//
// The client automatically uses semantic method names when in Neural API mode.
// See `SecurityRpcClient::from_env()` for automatic mode detection.
