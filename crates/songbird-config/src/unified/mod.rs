//! Unified Configuration System
//!
//! This module provides the canonical configuration system that replaces all
//! fragmented configuration structs throughout the Songbird ecosystem.

pub use core::*;
pub use federation::*;
// pub use network::*; // ✅ REMOVED: Fully consolidated into canonical::network (Nov 9, 2025)
// pub use security::*; // ARCHIVED: Moved to _archived_q2_2026/, use canonical::security instead
// pub use discovery::*; // ✅ REMOVED: Fully consolidated into canonical::discovery (Nov 9, 2025)
pub use observability::*;

// Core modules
pub mod core;
pub mod federation;
pub mod robustness; // Robustness configs (circuit breakers, load balancers, etc.)
                    // pub mod network; // ✅ REMOVED: Fully consolidated into canonical::network (Nov 9, 2025)
                    // pub mod security; // ARCHIVED: November 8, 2025 - Use canonical::security instead
                    // pub mod discovery; // ✅ REMOVED: Fully consolidated into canonical::discovery (Nov 9, 2025)
pub mod observability;

// Re-export the main config type for convenience
pub use core::SongbirdConfig;
