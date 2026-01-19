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
// Current jsonrpsee-based implementation (98% Pure Rust)
// ============================================================================
// NOTE: jsonrpsee has 2% C dependencies (rustls → ring/aws-lc-rs)
// We keep this for now as IPC handlers use it extensively (88 references)
// Migration path: See JSONRPC_MIGRATION_STRATEGY_JAN_19_2026.md
pub mod jsonrpc;
pub use self::jsonrpc::{JsonRpcConfig, JsonRpcServer};

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
