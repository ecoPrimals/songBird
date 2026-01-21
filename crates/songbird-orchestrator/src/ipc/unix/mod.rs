//! Unix Socket IPC Module
//!
//! This module provides Unix socket-based inter-primal communication via JSON-RPC 2.0.
//! 
//! ## Module Structure
//!
//! - `server`: Core server infrastructure and lifecycle
//! - `jsonrpc`: JSON-RPC 2.0 protocol implementation
//! - `handlers`: Method handlers for all RPC endpoints
//!
//! ## Public API
//!
//! Re-exports the main types and traits needed for using the Unix socket IPC server:
//!
//! - `UnixSocketIpcServer`: Main server type
//! - `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError`: Protocol types

pub mod server;
pub mod jsonrpc;
pub mod handlers;

// Re-export the main types for convenience
pub use server::UnixSocketIpcServer;
pub use jsonrpc::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};

