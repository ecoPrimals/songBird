//! IPC Handlers - JSON-RPC method handlers for Songbird capabilities
//!
//! This module provides handlers for exposing Songbird capabilities via IPC,
//! following the TRUE PRIMAL architecture (service-based, zero code embedding).
//!
//! ## Available Handlers
//!
//! - **HTTP Handler** - HTTP/HTTPS requests via Pure Rust TLS 1.3

pub mod http_handler;

pub use http_handler::*;
