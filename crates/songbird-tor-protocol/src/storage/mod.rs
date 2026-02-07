//! Storage backends for consensus and descriptors

use crate::directory::Consensus;
use crate::error::Result;

/// Storage trait for caching
pub trait Storage: Send + Sync {
    /// Store consensus
    fn store_consensus(&self, consensus: &Consensus) -> Result<()>;
    
    /// Load consensus
    fn load_consensus(&self) -> Result<Option<Consensus>>;
}

/// In-memory storage (default)
#[derive(Default)]
pub struct MemoryStorage {
    // TODO: Add RwLock<Option<Consensus>>
}

impl Storage for MemoryStorage {
    fn store_consensus(&self, _consensus: &Consensus) -> Result<()> {
        Ok(())
    }
    
    fn load_consensus(&self) -> Result<Option<Consensus>> {
        Ok(None)
    }
}
