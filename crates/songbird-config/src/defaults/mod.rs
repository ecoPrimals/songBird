//! Default configuration values with environment variable support
//!
//! This module provides environment-aware default values for all configuration.
//! All defaults can be overridden via environment variables, enabling:
//! - Multi-instance deployments
//! - Environment-specific configuration
//! - Zero-hardcoding architecture
//!
//! # Environment Variables
//!
//! See individual modules for specific environment variables supported.

pub mod cache;
pub mod endpoints;
pub mod health;
pub mod hosts;
pub mod hosts_evolved;
pub mod performance;
pub mod ports;
pub mod ports_evolved;
pub mod resources;
pub mod scaling;
pub mod timeouts;

pub use cache::*;
pub use endpoints::*;
pub use health::*;
pub use hosts::*;
pub use performance::*;
pub use ports::*;
pub use resources::*;
pub use scaling::*;
pub use timeouts::*;
