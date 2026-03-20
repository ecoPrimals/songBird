// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Lightweight federation peer bookkeeping for registry coordination layers.
//!
//! Tracks which remote peers are known and whether this node considers itself
//! joined to a federation group. Intended for orchestration and tests; not a
//! full distributed consensus implementation.

use std::collections::HashSet;

/// Mutable view of federation membership for a single registry instance.
#[derive(Debug, Clone, Default)]
pub struct FederationState {
    joined: bool,
    peers: HashSet<String>,
}

impl FederationState {
    /// Create an empty state (not joined, no peers).
    #[must_use]
    pub fn new() -> Self {
        Self {
            joined: false,
            peers: HashSet::new(),
        }
    }

    /// Whether this node has completed a logical join handshake.
    #[must_use]
    pub const fn is_joined(&self) -> bool {
        self.joined
    }

    /// Number of distinct peer identities tracked.
    #[must_use]
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Mark the node as joined to the federation.
    pub fn join(&mut self) {
        self.joined = true;
    }

    /// Leave the federation and clear peer knowledge.
    pub fn leave(&mut self) {
        self.joined = false;
        self.peers.clear();
    }

    /// Register a peer id. Returns `false` if the peer was already known.
    pub fn register_peer(&mut self, peer_id: impl Into<String>) -> bool {
        self.peers.insert(peer_id.into())
    }

    /// Remove a peer. Returns `true` if it existed.
    pub fn deregister_peer(&mut self, peer_id: &str) -> bool {
        self.peers.remove(peer_id)
    }

    /// Returns whether `peer_id` is tracked.
    #[must_use]
    pub fn has_peer(&self, peer_id: &str) -> bool {
        self.peers.contains(peer_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_leave_clears_state() {
        let mut s = FederationState::new();
        assert!(!s.is_joined());
        s.join();
        assert!(s.is_joined());
        assert!(s.register_peer("peer-a"));
        assert_eq!(s.peer_count(), 1);
        s.leave();
        assert!(!s.is_joined());
        assert_eq!(s.peer_count(), 0);
    }

    #[test]
    fn register_peer_idempotent_for_duplicates() {
        let mut s = FederationState::new();
        assert!(s.register_peer("p1"));
        assert!(!s.register_peer("p1"));
        assert_eq!(s.peer_count(), 1);
    }

    #[test]
    fn deregister_peer_missing_returns_false() {
        let mut s = FederationState::new();
        assert!(!s.deregister_peer("nope"));
    }

    #[test]
    fn deregister_peer_removes() {
        let mut s = FederationState::new();
        assert!(s.register_peer("x"));
        assert!(s.deregister_peer("x"));
        assert!(!s.has_peer("x"));
    }

    #[test]
    fn join_then_register_peers_tracks_count() {
        let mut s = FederationState::new();
        s.join();
        assert!(s.is_joined());
        assert!(s.register_peer("a"));
        assert!(s.register_peer("b"));
        assert_eq!(s.peer_count(), 2);
        assert!(s.has_peer("a"));
    }

    #[test]
    fn leave_clears_joined_and_peers() {
        let mut s = FederationState::new();
        s.join();
        s.register_peer("p");
        s.leave();
        assert!(!s.is_joined());
        assert_eq!(s.peer_count(), 0);
        assert!(!s.has_peer("p"));
    }
}
