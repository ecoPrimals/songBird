// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 Server Implementation - Modular Architecture
//!
//! **Design Philosophy**:
//! - ✅ Reuses ALL client modules (transcript, parser, keys, etc.)
//! - ✅ Modern idiomatic Rust (async/await, iterators, traits)
//! - ✅ Zero hardcoding (agnostic & capability-based)
//! - ✅ Safe Rust (no unnecessary unsafe)
//! - ✅ Complete implementation (no production mocks)
//! - ✅ Smart refactoring by domain (not arbitrary line splits)
//!
//! **Critical**: Uses EXACT same transcript logic as client for validation!
//!
//! ## Module Organization
//!
//! - `core` - Main `TlsServer` struct and public API
//! - `handshake` - Handshake orchestration and state machine
//! - `messages` - TLS message construction (`ServerHello`, Certificate, etc.)
//! - `crypto_ops` - Encryption and decryption operations
//! - `parsing` - `ClientHello` parsing and validation
//! - `transport` - TLS record layer I/O

mod core;
mod crypto_ops;
mod handshake;
mod messages;
mod parsing;
mod transport;

pub use core::TlsServer;
