//! Unified Configuration System
//!
//! This module provides the canonical configuration system that replaces all
//! fragmented configuration structs throughout the Songbird ecosystem.

pub use core::*;
pub use federation::*;
pub use network::*;
// pub use security::*; // ARCHIVED: Moved to _archived_q2_2026/, use canonical::security instead
pub use discovery::*;
pub use observability::*;

// Core modules
pub mod core;
pub mod federation;
pub mod network;
// pub mod security; // ARCHIVED: November 8, 2025 - Use canonical::security instead
pub mod discovery;
pub mod observability;

// Re-export the main config type for convenience
pub use core::SongbirdConfig;
