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
//! - **Discovery Bridge** - Connects orchestrator's listener to IPC
//! - **Rendezvous Handler** - Relay server registration and lookup
//! - **Peer Handler** - Direct peer connections via hole punching
//!
//! ## Production Implementations (Deep Debt Compliant)
//!
//! - **HTTP Rendezvous Client** - Production rendezvous via HTTP
//! - **UDP Peer Connector** - Production hole punching via UDP

pub mod discovery_bridge;
pub mod discovery_handler;
pub mod http_handler;
pub mod stun_handler;
pub mod rendezvous_handler;
pub mod peer_handler;
pub mod http_rendezvous_client;
pub mod udp_peer_connector;

pub use discovery_bridge::*;
pub use discovery_handler::*;
pub use http_handler::*;
pub use stun_handler::*;
pub use rendezvous_handler::*;
pub use peer_handler::*;
pub use http_rendezvous_client::*;
pub use udp_peer_connector::*;
