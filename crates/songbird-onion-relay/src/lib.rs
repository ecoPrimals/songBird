//! Sovereign Rendezvous for Symmetric NAT
//!
//! **Pure Rust | Self-Hosted | Minimal Tor Usage**
//!
//! ## Architecture
//!
//! ```text
//! 1. BOOTSTRAP (Tor onion - signaling only)
//!    Tower <──.onion──> Pixel
//!    Exchange STUN addresses via encrypted BirdSong
//!
//! 2. HOLE PUNCH (Direct UDP)
//!    Tower ──UDP──> Pixel's public address
//!    Pixel ──UDP──> Tower's public address
//!    Simultaneous open through NAT
//!
//! 3. RESULT
//!    Success → Direct P2P (Tor disconnected)
//!    Fail → Relay via onion (fallback)
//! ```
//!
//! ## Why This Works
//!
//! - Tor onion service is reachable without port forwarding
//! - Both devices can connect OUTBOUND to the onion (works through any NAT)
//! - STUN gives us public addresses for hole punch attempt
//! - If hole punch works, we're direct with low latency
//! - If not, we fall back to the onion as relay

pub mod error;
pub mod signaling;
pub mod coordinator;
pub mod mesh;

#[cfg(feature = "tor")]
pub mod tor_transport;

pub use error::{OnionRelayError, Result};
pub use signaling::{SignalingMessage, PeerInfo, NatType};
pub use coordinator::HolePunchCoordinator;
pub use mesh::{BeaconMesh, RelayEndpoint, EndpointType};
