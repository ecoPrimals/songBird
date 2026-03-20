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
