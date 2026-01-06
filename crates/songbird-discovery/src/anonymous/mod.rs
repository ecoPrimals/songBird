//! Anonymous Discovery Protocol
//!
//! Implements secure anonymous discovery with UDP multicast.
//! Refactored into focused modules for maintainability (v3.12.1).
//!
//! ## Architecture
//!
//! - `messages` - Message types and serialization (✅ Complete)
//! - `peer` - Peer discovery and management (✅ Complete)
//! - `broadcaster` - Broadcasting logic (✅ Complete)
//! - `listener` - Listening and processing (pending extraction)

pub mod broadcaster;
pub mod messages;
pub mod peer;

// Re-export public types for backward compatibility
pub use broadcaster::AnonymousDiscoveryBroadcaster;
pub use messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};
pub use peer::DiscoveredPeer;

// TODO: Complete refactoring by extracting listener from anonymous_discovery.rs
// pub mod listener;
// pub use listener::AnonymousDiscoveryListener;

