// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    reason = "Arc::clone() is idiomatic for shared ownership in async contexts"
)]
#![allow(
    clippy::expect_used,
    reason = "relay mesh invariants use expect() for panic-on-violation semantics"
)]
#![cfg_attr(test, allow(clippy::unwrap_used, reason = "test assertions"))]
//! Sovereign Rendezvous for Symmetric NAT
//!
//! **Pure Rust | Self-Hosted | `security provider` Crypto Delegation**
//!
//! ## ARCHITECTURE EVOLUTION (Feb 6, 2026)
//!
//! Previously: Arti (full Tor client) - 5MB, 10-30s startup
//! Now: Sovereign Onion Service - 200KB, instant startup, `security provider` crypto
//!
//! ## How It Works
//!
//! ```text
//! 1. BOOTSTRAP (Sovereign Onion - signaling only)
//!    Tower <──.onion──> Pixel
//!    Exchange STUN addresses via encrypted `BirdSong`
//!    Crypto delegated to security provider (TRUE PRIMAL pattern)
//!
//! 2. HOLE PUNCH (Direct UDP)
//!    Tower ──UDP──> Pixel's public address
//!    Pixel ──UDP──> Tower's public address
//!    Simultaneous open through NAT
//!
//! 3. MESH RELAY (Fallback)
//!    If hole punch fails → route via family relays
//!    Every connected node can be a relay
//!    Priority: Local > Direct > FamilyRelay > Onion
//! ```
//!
//! ## Features
//!
//! - `onion`: Sovereign Onion Service (Pure Rust, `security provider` crypto)
//!
//! See: `biomeOS/docs/handoffs/SOVEREIGN_BEACON_MESH_HANDOFF_FEB06_2026.md`
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// STUN, signaling, and relay-assisted UDP hole punch coordination.
pub mod coordinator;

/// [`OnionRelayError`] and result alias for this crate.
pub mod error;

/// Beacon mesh relay selection and family relay bookkeeping.
pub mod mesh;

/// Rendezvous messages exchanged while negotiating NAT traversal.
pub mod signaling;

// Sovereign Onion Service (optional - enable with `--features onion`)
#[cfg(feature = "onion")]
/// Optional `.onion` transport built on `songbird-sovereign-onion`.
pub mod onion_transport;
pub mod transport_impl;

pub use coordinator::{HolePunchConfig, HolePunchCoordinator, PunchResult};
pub use error::{OnionRelayError, Result};
pub use mesh::{BeaconMesh, EndpointType, RelayEndpoint};
pub use signaling::{NatType, PeerInfo, SignalingMessage};
pub use transport_impl::OnionRelayTransport;

// ✅ Sovereign Onion Transport (Phase 1 complete - Feb 6, 2026)
#[cfg(feature = "onion")]
pub use onion_transport::OnionTransport;
