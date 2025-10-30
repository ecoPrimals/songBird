//! Auto-scaling module
//!
//! Provides automatic scaling based on metrics and policies.

pub mod engine;
pub mod policy;

// Re-export public items
pub use engine::ScalingEngine;
pub use policy::{ScalingAction, ScalingPolicy};
