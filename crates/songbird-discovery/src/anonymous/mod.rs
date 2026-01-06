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
//! - `listener` - Listening and processing (✅ Complete)
//!
//! ## Refactoring Complete (v3.12.2)
//!
//! All modules extracted from `anonymous_discovery.rs` (1396 lines)
//! into focused, testable modules with comprehensive unit tests.

pub mod broadcaster;
pub mod listener;
pub mod messages;
pub mod peer;

// Re-export public types for backward compatibility
pub use broadcaster::AnonymousDiscoveryBroadcaster;
pub use listener::AnonymousDiscoveryListener;
pub use messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};
pub use peer::DiscoveredPeer;

