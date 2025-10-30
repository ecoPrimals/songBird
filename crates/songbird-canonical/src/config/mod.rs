//! Canonical Configuration System for Songbird Ecosystem Ecosystem
//!
//! Modular configuration system that consolidates fragmented configuration patterns
//! into focused, maintainable modules. Each module handles a specific domain of
//! configuration while maintaining a unified interface.

// Re-export all configuration structures
pub use adapters::*;
pub use ai_first::*;
pub use environment::*;
pub use orchestration::*;
pub use performance::*;

// Module declarations
pub mod adapters;
pub mod ai_first;
pub mod environment;
pub mod orchestration;
pub mod performance;
