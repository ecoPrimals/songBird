//! # Universal Storage Adapter
//!
//! NestGate delegation via universal adapter with modular architecture.
//!
//! This module provides capability-based storage operations that route
//! to appropriate storage providers without hardcoded dependencies.

pub mod adapter;
pub mod stats;
pub mod types;

// Re-export the main types for backward compatibility
pub use adapter::StorageAdapter;
pub use stats::StorageStats;
pub use types::*;
