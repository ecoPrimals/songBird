/// Multi-protocol RPC module for Songbird
///
/// Provides multiple RPC protocols for different use cases:
/// - JSON-RPC 2.0: Universal, language-agnostic access
/// - tarpc: High-performance binary RPC for primal-to-primal
/// - Protocol negotiation: Automatic protocol escalation (Phase 2)
pub mod jsonrpc;
pub mod tarpc_server;
// pub mod negotiation; // Future: Phase 2 (v3.13.0)

pub use self::jsonrpc::{JsonRpcConfig, JsonRpcServer};
pub use self::tarpc_server::{
    start_tarpc_server,
    start_tarpc_server_simple, // v3.12.0 - simplified version without Arc<Orchestrator>
    TarpcConfig,
    TarpcServer,
    TarpcServerSimple, // v3.12.0 - modern Rust, zero unsafe
};
// Re-export SongbirdRpc from songbird-universal (v3.12.0)
pub use songbird_universal::tarpc_types::SongbirdRpc;
