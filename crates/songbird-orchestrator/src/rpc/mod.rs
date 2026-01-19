//! Multi-protocol RPC module for Songbird
//!
//! Provides multiple RPC protocols for different use cases:
//! - JSON-RPC 2.0: Universal, language-agnostic access
//! - tarpc: High-performance binary RPC for primal-to-primal
//! - Protocol negotiation: Automatic protocol escalation (Phase 2)

// ============================================================================
// Pure Rust JSON-RPC 2.0 Implementation (100% Pure Rust, zero C dependencies!)
// ============================================================================
// Ready for full migration when IPC handlers are updated
pub mod pure_jsonrpc_types;
pub mod pure_jsonrpc_handler;

// Re-export for convenience
pub use pure_jsonrpc_types::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
pub use pure_jsonrpc_handler::handle_jsonrpc_request;

// ============================================================================
// Former jsonrpsee-based implementation (REMOVED - was dead code!)
// ============================================================================
// NOTE: JsonRpcServer was never actually used in production
// Production uses UnixSocketIpcServer (Pure Rust, v3.22.0)
// See: ipc/server_pure_rust.rs for the actual implementation
// See: PHASE4_JSONRPSEE_ANALYSIS_JAN_19_2026.md for analysis
// pub mod jsonrpc; // REMOVED: Dead code (not instantiated anywhere)
// pub use self::jsonrpc::{JsonRpcConfig, JsonRpcServer}; // REMOVED

// ============================================================================
// tarpc RPC (100% Pure Rust, production-ready)
// ============================================================================
pub mod tarpc_server;

pub use self::tarpc_server::{
    start_tarpc_server,
    start_tarpc_server_simple, // v3.12.0 - simplified version without Arc<Orchestrator>
    TarpcConfig,
    TarpcServer,
    TarpcServerSimple, // v3.12.0 - modern Rust, zero unsafe
};

// Re-export SongbirdRpc from songbird-universal (v3.12.0)
pub use songbird_universal::tarpc_types::SongbirdRpc;
