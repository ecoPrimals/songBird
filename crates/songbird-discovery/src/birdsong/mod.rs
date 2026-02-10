//! `BirdSong` Integration - Encrypted Discovery (REFACTORED v3.22.0)
//!
//! Integrates encrypted discovery into Songbird's anonymous discovery system.
//!
//! ## Philosophy
//!
//! "A broadcast that is obvious to family and noise otherwise"
//!
//! - Same family: Clear signal (can decrypt)
//! - Different family: Just noise (cannot decrypt)
//! - Works on LAN while learning the system
//! - Seamless upgrade path to internet-wide P2P
//!
//! ## Smart Refactoring (Feb 5, 2026)
//!
//! This module was refactored from a 1,089-line monolithic file into 5 focused modules:
//!
//! - `types` - `BirdSongPacket` struct and packet format (~60 lines)
//! - `trait` - `BirdSongEncryption` provider trait (~250 lines)
//! - `config` - `BirdSongConfig` and builder methods (~180 lines)
//! - `processor` - `BirdSongProcessor` implementation + tests (~600 lines)
//! - `mod` - Module documentation and re-exports (~80 lines)
//!
//! **Total**: ~1,170 lines across 5 modules (from 1,089-line monolith)
//! **Benefit**: Clear separation of concerns, better testability, easier navigation
//!
//! ## Modern Rust Patterns
//!
//! - Zero unsafe code
//! - Async/await throughout
//! - Comprehensive error handling with `anyhow`
//! - Graceful degradation (fallback to plaintext)
//! - Provider-agnostic (works with any security provider)
//!
//! ## Evolution Timeline
//!
//! - **v1.0 (Legacy)**: `BirdSongPacket` with plaintext `family_id` header
//! - **v2.0 (Feb 3, 2026)**: Dark Forest beacons (fully encrypted, zero metadata)
//! - **v3.22.0 (Feb 5, 2026)**: Smart refactoring into focused modules

// Submodules
mod config;
mod processor;
mod r#trait;
mod types;

// Re-export public API
pub use config::BirdSongConfig;
pub use processor::BirdSongProcessor;
pub use r#trait::BirdSongEncryption;
pub use types::BirdSongPacket;

// Re-export for backward compatibility
pub use crate::dark_forest_beacon::{BeaconPayload, DarkForestBeacon};
