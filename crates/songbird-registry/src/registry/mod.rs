//! Registry module
//!
//! Core registry functionality and traits.

pub mod core;
pub mod query;
pub mod traits;

// Re-export public items
pub use core::Registry;
pub use query::Query;
pub use traits::{Composable, PluginRegistry};
