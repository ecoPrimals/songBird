/// Multi-protocol RPC module for Songbird
///
/// Provides multiple RPC protocols for different use cases:
/// - JSON-RPC 2.0: Universal, language-agnostic access
/// - tarpc: High-performance binary RPC for primal-to-primal
/// - Protocol negotiation: Automatic protocol escalation
pub mod jsonrpc;
pub mod tarpc_server;
// pub mod negotiation; // Future: Phase 1, Task 1.3

pub use self::jsonrpc::{JsonRpcConfig, JsonRpcServer};
pub use self::tarpc_server::{start_tarpc_server, SongbirdRpc, TarpcConfig, TarpcServer};
