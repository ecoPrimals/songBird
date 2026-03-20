// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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

pub mod birdsong_handler; // BirdSong encrypted discovery (Feb 2, 2026)
pub mod discovery_bridge;
pub mod discovery_handler;
pub mod http_handler;
pub mod http_rendezvous_client;
pub mod igd_handler; // IGD router configuration (Feb 8, 2026)
pub mod mesh_handler; // Beacon mesh networking (Feb 4, 2026)
pub mod onion_handler; // Sovereign onion service (Feb 4, 2026)
pub mod peer_handler;
pub mod punch_handler; // Hole punch coordination (Feb 4, 2026)
pub mod rendezvous_handler;
pub mod stun_handler;
pub mod tor_handler; // Pure Rust Tor protocol (Feb 7, 2026)
pub mod udp_peer_connector;

pub use birdsong_handler::*; // BirdSong handler (Feb 2, 2026)
pub use discovery_bridge::*;
pub use discovery_handler::*;
pub use http_handler::*;
pub use http_rendezvous_client::*;
pub use igd_handler::*; // IGD handler (Feb 8, 2026)
pub use mesh_handler::*; // Mesh handler (Feb 4, 2026)
pub use onion_handler::*; // Onion handler (Feb 4, 2026)
pub use peer_handler::*;
pub use punch_handler::*; // Punch handler (Feb 4, 2026)
pub use rendezvous_handler::*;
pub use stun_handler::*;
pub use tor_handler::*; // Tor handler (Feb 7, 2026)
pub use udp_peer_connector::*;
