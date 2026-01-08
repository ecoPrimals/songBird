//! Inter-Primal Communication (IPC) via Unix Socket JSON-RPC
//!
//! v3.19.1: Modern async Rust patterns for primal-to-primal communication
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │ Unix Socket Server                                   │
//! │ /tmp/songbird-{node_id}.sock                         │
//! ├──────────────────────────────────────────────────────┤
//! │ JSON-RPC 2.0 APIs:                                   │
//! │  • discover_by_family (filter by genetic tags)       │
//! │  • create_genetic_tunnel (BTSP with genetic proof)   │
//! │  • announce_capabilities (update broadcaster)        │
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! ## Design Principles
//!
//! 1. **Zero Hardcoding**: Socket path derived from node_id
//! 2. **Modern Async**: jsonrpsee with tokio
//! 3. **Protocol Agnostic**: Works with any JSON-RPC 2.0 client
//! 4. **Observable**: Structured logging at every step
//! 5. **Secure**: Unix socket permissions, credential passing (future)

pub mod server;
pub mod handlers;
pub mod types;

pub use server::UnixSocketServer;
pub use types::{
    DiscoverByFamilyRequest,
    DiscoverByFamilyResponse,
    CreateGeneticTunnelRequest,
    CreateGeneticTunnelResponse,
    AnnounceCapabilitiesRequest,
    AnnounceCapabilitiesResponse,
};
