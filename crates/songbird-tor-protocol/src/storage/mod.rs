// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Storage backends for consensus and descriptors

use crate::directory::Consensus;
use crate::error::Result;

/// Storage trait for caching
pub trait Storage: Send + Sync {
    /// Store consensus
    ///
    /// # Errors
    ///
    /// Returns error if storage fails.
    fn store_consensus(&self, consensus: &Consensus) -> Result<()>;

    /// Load consensus
    ///
    /// # Errors
    ///
    /// Returns error if load fails.
    fn load_consensus(&self) -> Result<Option<Consensus>>;
}

/// In-memory storage (default)
#[derive(Default)]
pub struct MemoryStorage {
    // Intentionally stateless: MemoryStorage does not retain consensus between calls.
}

impl Storage for MemoryStorage {
    fn store_consensus(&self, _consensus: &Consensus) -> Result<()> {
        Ok(())
    }

    fn load_consensus(&self) -> Result<Option<Consensus>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::directory::Consensus;
    use std::time::{Duration, SystemTime};

    #[test]
    fn memory_storage_store_succeeds_and_load_is_none() {
        let store = MemoryStorage::default();
        let c = Consensus {
            valid_after: SystemTime::UNIX_EPOCH,
            fresh_until: SystemTime::UNIX_EPOCH + Duration::from_secs(1),
            valid_until: SystemTime::UNIX_EPOCH + Duration::from_secs(2),
            relays: vec![],
        };
        store.store_consensus(&c).expect("store");
        assert!(store.load_consensus().expect("load").is_none());
    }

    #[test]
    fn memory_storage_default_is_empty_loader() {
        let s = MemoryStorage::default();
        assert!(s.load_consensus().expect("load").is_none());
    }
}
