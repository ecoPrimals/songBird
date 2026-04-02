// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `BirdSong` Encrypted Discovery Handler
//!
//! Provides JSON-RPC methods for Dark Forest federation via genetic lineage encryption.
//!
//! # Methods
//!
//! - `birdsong.generate_encrypted_beacon` - Generate family-encrypted beacon
//! - `birdsong.decrypt_beacon` - Decrypt beacon (family gate)
//! - `birdsong.verify_lineage` - Verify peer lineage via challenge-response
//! - `birdsong.get_lineage` - Get own lineage info
//! - `birdsong.schema` - Introspect beacon request schema (fields, types, required/optional)
//!
//! # Architecture
//!
//! ```text
//! Client → songbird.birdsong.* → BirdSongHandler
//!                                      ↓
//!                         BearDogBirdSongProvider (via songbird-discovery)
//!                                      ↓
//!                            beardog Unix socket IPC
//!                                      ↓
//!                         Crypto operations (ChaCha20-Poly1305)
//! ```
//!
//! # Deep Debt Compliance (Feb 2, 2026)
//!
//! - ✅ **Pure Rust**: Uses existing `BearDogBirdSongProvider` (zero C deps)
//! - ✅ **Zero Unsafe**: All operations safe
//! - ✅ **Runtime Discovery**: Finds beardog via `XDG_RUNTIME_DIR`, well-known paths
//! - ✅ **Self-Knowledge**: Only exposes own beacon generation
//! - ✅ **Mock Isolation**: Production code only (mocks in tests)
//! - ✅ **Agnostic Design**: Works with any family seed, discovers beardog at runtime

mod beacon_decryption;
mod beacon_generation;
mod lineage;
mod provider;
mod schema;
mod types;

pub use types::BirdSongHandler;

#[cfg(test)]
mod tests;
