//! # Songbird Universal Types and Patterns
//!
//! This crate provides universal types and patterns for Songbird ecosystem integration.
//! It enables seamless integration with any primal type while maintaining complete
//! agnosticism for future expansion.

pub mod capabilities;
pub mod communication;
pub mod discovery;
pub mod errors;
pub mod load_balancing;
pub mod registry;
pub mod traits;
pub mod types;

// Re-export commonly used types
pub use capabilities::*;
pub use communication::*;
pub use discovery::*;
pub use errors::*;
pub use load_balancing::*;
pub use registry::*;
pub use traits::*;
pub use types::*;
