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

pub mod endpoints;
pub mod hosts;
pub mod ports;
pub mod timeouts;

pub use endpoints::*;
pub use hosts::*;
pub use ports::*;
pub use timeouts::*;
