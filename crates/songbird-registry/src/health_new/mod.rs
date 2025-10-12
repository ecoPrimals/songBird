//! Health monitoring module
//!
//! Provides health checking and monitoring for plugins.

pub mod checks;
pub mod monitor;

// Re-export public items
pub use checks::{HttpCheck, MetricsCheck, ProcessCheck};
pub use monitor::HealthMonitor;
