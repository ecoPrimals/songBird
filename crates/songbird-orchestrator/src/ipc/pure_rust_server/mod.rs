//! Pure Rust Unix Socket JSON-RPC Server (v3.22.0+)
//!
//! ## Evolution
//!
//! **v3.22.0**: Evolved from jsonrpsee to pure Rust (BearDog pattern)
//! **v4.9.0**: Refactored into domain-driven modules
//!
//! ## Module Structure
//!
//! - `protocol`: JSON-RPC 2.0 types and error codes
//! - `server`: Core server infrastructure and lifecycle
//! - `squirrel_handlers`: Squirrel integration endpoints
//!
//! ## Public API
//!
//! Re-exports the main types needed for using the Unix socket server:
//!
//! - `UnixSocketServer`: Main server type
//! - `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError`: Protocol types

pub mod protocol;
pub mod server;
pub mod squirrel_handlers;

// Re-export the main types for convenience
pub use protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
pub use server::UnixSocketServer;
