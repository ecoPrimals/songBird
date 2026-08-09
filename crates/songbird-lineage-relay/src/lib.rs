// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Songbird Lineage Relay
//!
//! **Evolution beyond NAT/STUN/TURN** - Genetic lineage-based relay system
//!
//! ## Vision
//!
//! Replace legacy infrastructure-based relay (TURN servers) with cryptographic lineage:
//! - Nodes relay for their descendants
//! - Trust based on Genesis ceremony
//! - Privacy-preserving (masked by default)
//! - Distributed (any ancestor can help)
//! - Sovereign (no external dependencies)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                 Songbird Lineage Relay                      │
//! │                                                             │
//! │  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
//! │  │  BirdSong   │  │    Relay     │  │     Session      │  │
//! │  │  Broadcast  │  │  Discovery   │  │   Management     │  │
//! │  └─────────────┘  └──────────────┘  └──────────────────┘  │
//! └────────────────────┬────────────────────────────────────────┘
//!                      │
//!                      ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │         Security provider (capability-discovered)            │
//! │   (Genetic crypto, lineage verification, relay authority)   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_lineage_relay::{BirdSongBroadcaster, BirdSongCrypto, LineageRelayCoordinator, RelayAuthority};
//! use songbird_lineage_relay::security::{MockBirdSongCrypto, MockLineageProvider, MockRelayAuthority};
//! use songbird_lineage_relay::types::NodeId;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create lineage provider (security provider in production, mock for testing)
//! let lineage_provider = Arc::new(MockLineageProvider::new());
//! let crypto = Arc::new(BirdSongCrypto::from(MockBirdSongCrypto::new(
//!     lineage_provider.clone(),
//!     String::from("node-1"),
//! )));
//! let broadcaster = Arc::new(BirdSongBroadcaster::new(/* ... */));
//! let relay_authority = Arc::new(RelayAuthority::from(MockRelayAuthority::new(lineage_provider)));
//!
//! // Create relay coordinator
//! let config = LineageRelayConfig::default();
//! let coordinator = LineageRelayCoordinator::new(config, broadcaster, relay_authority).await?;
//!
//! // Attempt connection (tries direct first, falls back to lineage relay)
//! let peer_id = NodeId::from("peer-1");
//! let peer_address = "127.0.0.1:8080".parse()?;
//! let connection = coordinator
//!     .establish_connection(peer_id, peer_address)
//!     .await?;
//!
//! // Send data through connection (could be direct or relayed)
//! connection.send(b"Hello from genetic lineage!").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Terminology Evolution
//!
//! **Legacy Concepts** (reference only, considered limited):
//! - NAT traversal → **Lineage-based connectivity**
//! - STUN (discovery) → **Direct connectivity attempt**
//! - TURN (relay) → **Ancestor relay service**
//! - ICE (negotiation) → **Lineage-aware connection**
//!
//! **Modern Approach**:
//! - Genetic lineage establishes trust
//! - Ancestors relay for descendants
//! - Privacy through masking
//! - No external infrastructure

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::clone_on_ref_ptr,
    reason = "Arc::clone() is idiomatic for shared ownership in async contexts"
)]
#![allow(
    clippy::expect_used,
    reason = "relay protocol invariants use expect() for panic-on-violation semantics"
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions"))]
#![allow(
    clippy::module_name_repetitions,
    clippy::items_after_statements,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::unused_async,
    reason = "unused bindings/imports in this compilation unit; doc and style exceptions for relay paths"
)]
#![cfg_attr(
    test,
    allow(
        deprecated,
        dead_code,
        unused_imports,
        unused_variables,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::await_holding_lock,
        clippy::float_cmp,
        clippy::absurd_extreme_comparisons,
        clippy::nonminimal_bool,
        clippy::needless_collect,
        clippy::used_underscore_binding,
        clippy::overly_complex_bool_expr,
        clippy::assertions_on_constants,
        clippy::unreadable_literal,
        clippy::empty_line_after_doc_comments,
        clippy::field_reassign_with_default,
        clippy::unnecessary_wraps,
        clippy::no_effect_underscore_binding,
        clippy::return_self_not_must_use,
        clippy::duplicated_attributes,
        clippy::needless_pass_by_value,
        clippy::must_use_candidate,
        clippy::missing_panics_doc,
        clippy::missing_errors_doc,
        clippy::doc_markdown,
        clippy::wildcard_imports,
        clippy::enum_glob_use,
        clippy::unused_self,
        clippy::unnecessary_cast,
        clippy::items_after_test_module,
        clippy::clone_on_ref_ptr,
        clippy::default_trait_access,
        clippy::needless_range_loop,
        clippy::similar_names,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::unnecessary_unwrap,
        clippy::ignore_without_reason,
        clippy::case_sensitive_file_extension_comparisons,
        clippy::needless_update,
        clippy::await_holding_invalid_type,
        reason = "test harnesses: intentional leniency for assertion ergonomics and legacy test patterns"
    )
)]

pub mod birdsong;
pub mod cloudflared_tunnel;
pub mod coordinator;
pub mod error;
pub mod multi_tier_coordinator;
pub mod nat_field_test;
pub mod relay;
pub mod relay_handler;
pub mod relay_protocol;
pub mod relay_server;
pub mod session;
pub mod shadow_comparator;
pub mod transport_impl;
pub mod types;
pub mod udp_hole_punch;
pub mod universal_coordinator_adapter;

// Security-provider integration (BirdSong + relay authority; discovered by capability at runtime)
pub mod security;

#[cfg(test)]
mod security_tests;

// Re-exports
pub use birdsong::BirdSongBroadcaster;
pub use coordinator::LineageRelayCoordinator;
pub use error::{LineageRelayError, Result};
pub use multi_tier_coordinator::{
    CloudflaredTunnel, ConnectionResult, ConnectionTier, MultiTierCoordinator, TierQualityReport,
};
pub use relay::{RelayAuthority, RelayDiscovery, RelaySession};
pub use relay_handler::RelayHandler;
pub use relay_protocol::{AllocationRequest, AllocationResponse, RelayProtocol};
pub use relay_server::{RelayServer, RelayServerStats};
pub use security::BirdSongCrypto;
pub use session::ConnectionSession;
pub use types::*;
pub use udp_hole_punch::{
    HolePunchConfig, coordinated_hole_punch, create_hole_punch_socket, udp_hole_punch,
};
pub use transport_impl::LineageRelayTransport;
pub use universal_coordinator_adapter::{LineageRelayAdapter, LineageRelayPrimalConnection};
