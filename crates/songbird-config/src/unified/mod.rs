//! Unified Configuration System
//!
//! This module provides the canonical configuration system that replaces all
//! fragmented configuration structs throughout the Songbird ecosystem.

pub use core::*;
pub use federation::*;
pub use network::*;
pub use security::*;
pub use discovery::*;
pub use observability::*;

// Core modules
pub mod core;
pub mod federation;
pub mod network;
pub mod security;
pub mod discovery;
pub mod observability;

// Re-export the main config type for convenience
pub use core::SongbirdConfig;
