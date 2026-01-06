//! Anonymous Discovery Protocol
//!
//! Implements secure anonymous discovery with UDP multicast.
//! Refactored into focused modules for maintainability (v3.12.1).
//!
//! ## Architecture
//!
//! - `messages` - Message types and serialization (✅ Complete)
//! - `peer` - Peer discovery and management (✅ Complete)
//! - `broadcaster` - Broadcasting logic (pending extraction)
//! - `listener` - Listening and processing (pending extraction)

pub mod messages;
pub mod peer;

// Re-export public types for backward compatibility
pub use messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};
pub use peer::DiscoveredPeer;

// TODO: Complete refactoring by extracting these from anonymous_discovery.rs
// pub mod broadcaster;
// pub mod listener;
// pub use broadcaster::AnonymousDiscoveryBroadcaster;
// pub use listener::AnonymousDiscoveryListener;

