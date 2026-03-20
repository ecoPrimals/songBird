// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `BearDog` RPC client for crypto operations
//!
//! Communicates with `BearDog` via JSON-RPC 2.0 over Unix sockets, supporting
//! both Direct mode (testing) and Neural API mode (production).
//!
//! ## Architecture (Smart Refactored - January 26, 2026)
//!
//! This module was refactored from a 2,020-line monolith into 7 logical modules:
//!
//! - `types.rs` - JSON-RPC types and `TlsSecrets` struct (~120 lines)
//! - `core.rs` - `BearDogClient` struct, `BearDogMode` enum, constructors (~180 lines)
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
//! use songbird_http_client::BearDogClient;
//!
//! // Production: Use Neural API mode (TRUE PRIMAL pattern)
//! let client = BearDogClient::new_neural_api("/tmp/neural-api.sock");
//!
//! // Testing: Use Direct mode
//! let client = BearDogClient::new_direct("/tmp/beardog.sock");
//!
//! // From environment (recommended)
//! let client = BearDogClient::from_env();
//!
//! // Generate keypair
//! let (public_key, private_key) = client.generate_keypair().await?;
//! ```

mod aead;
mod core;
mod hash;
mod key_exchange;
mod rpc;
mod tls_secrets;
mod types;

// Re-export public API
pub use core::{BearDogClient, BearDogMode};
pub use types::TlsSecrets;

// ═══════════════════════════════════════════════════════════════════════════
// NOTE: Semantic capability.call Integration
// ═══════════════════════════════════════════════════════════════════════════
//
// `BearDogClient` supports semantic routing via Neural API through its mode enum:
//
// - Direct mode: Talk directly to `BearDog` (testing)
//   Set: BEARDOG_MODE=direct, BEARDOG_SOCKET=/tmp/beardog.sock
//
// - Neural API mode: Route through Neural API (production)
//   Set: BEARDOG_MODE=neural, NEURAL_API_SOCKET=/tmp/neural-api.sock
//
// The client automatically uses semantic method names when in Neural API mode.
// See `BearDogClient::from_env()` for automatic mode detection.
