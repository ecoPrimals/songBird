//! Sovereign Rendezvous for Symmetric NAT
//!
//! **Pure Rust | Self-Hosted | BearDog Crypto Delegation**
//!
//! ## ARCHITECTURE EVOLUTION (Feb 6, 2026)
//!
//! Previously: Arti (full Tor client) - 5MB, 10-30s startup
//! Now: Sovereign Onion Service - 200KB, instant startup, BearDog crypto
//!
//! ## How It Works
//!
//! ```text
//! 1. BOOTSTRAP (Sovereign Onion - signaling only)
//!    Tower <──.onion──> Pixel
//!    Exchange STUN addresses via encrypted BirdSong
//!    Crypto delegated to BearDog (TRUE PRIMAL pattern)
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
//! - `onion`: Sovereign Onion Service (Pure Rust, BearDog crypto)
//!
//! See: biomeOS/docs/handoffs/SOVEREIGN_BEACON_MESH_HANDOFF_FEB06_2026.md
#![forbid(unsafe_code)]

pub mod coordinator;
pub mod error;
pub mod mesh;
pub mod signaling;

// Sovereign Onion Service (optional - enable with `--features onion`)
#[cfg(feature = "onion")]
pub mod onion_transport;

// DEPRECATED: Arti-based transport (removed Feb 6, 2026)
// Code removed, keeping this comment for historical reference
// See: BEARDOG_ONION_CRYPTO_HANDOFF_FEB_06_2026.md for Phase 2 guidance
// See: SOVEREIGN_ONION_TRUE_PRIMAL_ARCHITECTURE.md for architecture

pub use coordinator::{HolePunchConfig, HolePunchCoordinator, PunchResult};
pub use error::{OnionRelayError, Result};
pub use mesh::{BeaconMesh, EndpointType, RelayEndpoint};
pub use signaling::{NatType, PeerInfo, SignalingMessage};

// ✅ Sovereign Onion Transport (Phase 1 complete - Feb 6, 2026)
#[cfg(feature = "onion")]
pub use onion_transport::OnionTransport;
