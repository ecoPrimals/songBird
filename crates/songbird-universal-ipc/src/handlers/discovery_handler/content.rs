// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Content distribution — seeder/leecher announcement store.
//!
//! `ContentAnnouncementStore` is an in-memory registry with TTL-based expiration
//! for tracking which peers have announced availability for specific content
//! topics (e.g., `content:ludospring:assets`).

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Default TTL for content announcements (10 minutes).
pub(super) const CONTENT_ANNOUNCEMENT_TTL: Duration = Duration::from_secs(600);

/// A content availability announcement from a seeder node.
///
/// Stored in the in-memory `ContentAnnouncementStore` when a peer calls
/// `discovery.announce` with a `topic` param. Queried via `discovery.content_peers`.
#[derive(Debug, Clone)]
pub struct ContentAnnouncement {
    pub topic: String,
    /// BLAKE3 manifest hash (from `NestGate` `ContentManifest`).
    pub manifest_hash: Option<String>,
    pub family_id: String,
    pub node_id: String,
    pub seeder_count: u64,
    pub bond_types_accepted: Vec<String>,
    pub(super) announced_at: Instant,
}

/// In-memory store for content announcements with TTL-based expiration.
///
/// Keyed by `(topic, node_id)` so a node can update its announcement for
/// a given topic by re-announcing.
#[derive(Debug)]
pub(super) struct ContentAnnouncementStore {
    entries: HashMap<(String, String), ContentAnnouncement>,
    ttl: Duration,
}

impl ContentAnnouncementStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            ttl: CONTENT_ANNOUNCEMENT_TTL,
        }
    }

    #[cfg(test)]
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }

    pub fn insert(&mut self, announcement: ContentAnnouncement) {
        let key = (announcement.topic.clone(), announcement.node_id.clone());
        self.entries.insert(key, announcement);
    }

    pub fn query(&self, topic: &str) -> Vec<&ContentAnnouncement> {
        let now = Instant::now();
        self.entries
            .values()
            .filter(|a| a.topic == topic && now.duration_since(a.announced_at) < self.ttl)
            .collect()
    }

    pub fn gc(&mut self) {
        let now = Instant::now();
        let ttl = self.ttl;
        self.entries.retain(|_, a| now.duration_since(a.announced_at) < ttl);
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
