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
//! │              BearDog Lineage Provider                       │
//! │   (Genetic crypto, lineage verification, relay authority)   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_lineage_relay::{LineageRelayCoordinator, BirdSongBroadcaster};
//! use songbird_lineage_relay::beardog::MockLineageProvider;
//! use songbird_lineage_relay::types::NodeId;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create lineage provider (BearDog in production, mock for testing)
//! let lineage_provider = Arc::new(MockLineageProvider::new());
//! let crypto = Arc::new(MockBirdSongCrypto::new(lineage_provider.clone(), "node-1".to_string()));
//! let broadcaster = Arc::new(BirdSongBroadcaster::new(/* ... */));
//! let relay_authority = Arc::new(MockRelayAuthority::new(lineage_provider));
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
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod birdsong;
pub mod coordinator;
pub mod error;
pub mod multi_tier_coordinator;
pub mod relay;
pub mod session;
pub mod types;
pub mod udp_hole_punch;
pub mod universal_coordinator_adapter;

// Mock BearDog implementations for testing
pub mod beardog;

// Re-exports
pub use birdsong::{BirdSongBroadcaster, BirdSongMessage, LineageHint};
pub use coordinator::LineageRelayCoordinator;
pub use error::{LineageRelayError, Result};
pub use multi_tier_coordinator::{ConnectionResult, MultiTierCoordinator, TierQualityReport};
pub use relay::{RelayDiscovery, RelaySession};
pub use session::ConnectionSession;
pub use types::*;
pub use udp_hole_punch::{coordinated_hole_punch, create_hole_punch_socket, udp_hole_punch, HolePunchConfig};
pub use universal_coordinator_adapter::{LineageRelayAdapter, LineageRelayPrimalConnection};
