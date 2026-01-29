//! IPC Handlers - JSON-RPC method handlers for Songbird capabilities
//!
//! This module provides handlers for exposing Songbird capabilities via IPC,
//! following the TRUE PRIMAL architecture (service-based, zero code embedding).
//!
//! ## Available Handlers
//!
//! - **HTTP Handler** - HTTP/HTTPS requests via Pure Rust TLS 1.3
//! - **STUN Handler** - NAT traversal and public address discovery
//! - **Discovery Handler** - Peer discovery from UDP beacons

pub mod discovery_handler;
pub mod http_handler;
pub mod stun_handler;

pub use discovery_handler::*;
pub use http_handler::*;
pub use stun_handler::*;
