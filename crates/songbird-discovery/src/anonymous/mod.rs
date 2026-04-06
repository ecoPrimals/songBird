// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
//! - `protocol` - Broadcast framing, serialization, BirdSong / Dark Forest helpers
//! - `scheduling` - Broadcast interval and rotating session IDs
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

mod protocol;
mod scheduling;

#[cfg(test)]
mod broadcaster_tests;

// Re-export public types for backward compatibility
pub use broadcaster::AnonymousDiscoveryBroadcaster;
pub use listener::AnonymousDiscoveryListener;
pub use messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};
pub use peer::DiscoveredPeer;
