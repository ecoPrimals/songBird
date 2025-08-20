//! Monitoring Module
//!
//! Canonical monitoring infrastructure following modernization patterns.

pub mod metrics_dashboard;
pub mod production_metrics;
pub mod types;

// Re-export canonical types
pub use types::*;
