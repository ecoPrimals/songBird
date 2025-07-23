//! Universal Service Registry Module
//!
//! Modular implementation of the Universal Service Registration standard.
//! This module is organized into logical submodules for better maintainability.

pub mod config;
pub mod memory_registry;
pub mod traits;
pub mod types;

// Re-export the main public API
pub use config::*;
pub use memory_registry::*;
pub use traits::*;
pub use types::*;
