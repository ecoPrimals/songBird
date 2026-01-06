//! Anonymous Discovery Protocol
//!
//! Implements secure anonymous discovery with UDP multicast.
//! Refactored into focused modules for maintainability (v3.12.1).
//!
//! ## Architecture
//!
//! - `messages` - Message types and serialization
//! - `peer` - Peer discovery and management (pending extraction)
//! - `broadcaster` - Broadcasting logic (pending extraction)
//! - `listener` - Listening and processing (pending extraction)

pub mod messages;

// Re-export public types for backward compatibility
pub use messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};

// TODO: Complete refactoring by extracting these from anonymous_discovery.rs
// pub mod peer;
// pub mod broadcaster;
// pub mod listener;
// pub use peer::DiscoveredPeer;
// pub use broadcaster::AnonymousDiscoveryBroadcaster;
// pub use listener::AnonymousDiscoveryListener;

