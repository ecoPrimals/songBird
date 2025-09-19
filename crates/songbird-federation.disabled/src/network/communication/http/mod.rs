// Module imports
//! HTTP Communication Module Module
//!
//! HTTP-based communication with service registry and circuit breaker support

pub mod client;
pub mod registry;
// HTTP client components;
pub use registry::*; 
