//! Multi-protocol RPC module for Songbird
//!
//! Provides multiple RPC protocols for different use cases:
//! - JSON-RPC 2.0: Universal, language-agnostic access (see server/jsonrpc_api.rs)
//! - tarpc: High-performance binary RPC for primal-to-primal
//! - Protocol negotiation: Automatic protocol escalation (Phase 2)

// ============================================================================
// JSON-RPC 2.0 Production Implementations
// ============================================================================
// 1. HTTP Gateway: crates/songbird-orchestrator/src/server/jsonrpc_api.rs
//    - Universal language-agnostic access over HTTP
//    - Full JSON-RPC 2.0 spec compliance
//    - Production-ready, actively used
//
// 2. Unix Socket Server: crates/songbird-orchestrator/src/ipc/pure_rust_server/
//    - JSON-RPC 2.0 over Unix sockets for IPC
//    - 100% Pure Rust, zero C dependencies
//    - Active in production for inter-primal communication
//
// Former pure_jsonrpc_handler.rs (ARCHIVED JAN 21, 2026):
//   - Was exported but never actually called
//   - All handlers were TODO stubs
//   - Superseded by the two implementations above
//   - See: ARCHIVE_CLEANUP_PLAN_JAN_21_2026.md

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
