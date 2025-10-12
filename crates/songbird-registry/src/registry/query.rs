//! Query types for searching plugins
//!
//! Provides a flexible query system for finding plugins.

use crate::types::CapabilityType;
use serde::{Deserialize, Serialize};

/// A query for searching plugins
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Query {
    /// Filter by plugin name (substring match)
    pub name: Option<String>,

    /// Filter by exact plugin ID
    pub id: Option<String>,

    /// Filter by capabilities (plugin must have ALL specified capabilities)
    pub capabilities: Vec<CapabilityType>,

    /// Filter by tags (plugin must have ALL specified tags)
    pub tags: Vec<String>,

    /// Filter by author
    pub author: Option<String>,

    /// Maximum number of results (0 = unlimited)
    pub limit: usize,
}

impl Query {
    /// Create a new empty query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by name (substring match)
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Filter by exact ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Add a capability requirement
    pub fn with_capability(mut self, capability: CapabilityType) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add a tag requirement
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Filter by author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Limit the number of results
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let query = Query::new().with_name("test").with_author("developer").with_limit(10);

        assert_eq!(query.name, Some("test".to_string()));
        assert_eq!(query.author, Some("developer".to_string()));
        assert_eq!(query.limit, 10);
    }
}
