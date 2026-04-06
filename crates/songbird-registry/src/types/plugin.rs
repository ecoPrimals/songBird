// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Plugin type definitions
//!
//! Defines the core Plugin type and related structures.

use crate::types::capability::Capability;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Unique identifier for a plugin
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(Arc<str>);

impl PluginId {
    /// Create a new `PluginId`
    pub fn new(id: impl Into<String>) -> Self {
        Self(Arc::from(id.into().as_str()))
    }

    /// Get the plugin ID as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Custom Serialize/Deserialize for PluginId
impl Serialize for PluginId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PluginId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PluginId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for PluginId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Author of the plugin
    pub author: String,

    /// Description of what the plugin does
    pub description: String,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Health check endpoint (if any)
    pub health_endpoint: Option<String>,

    /// Additional arbitrary metadata
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Default for PluginMetadata {
    fn default() -> Self {
        Self {
            author: String::from("unknown"),
            description: String::new(),
            tags: Vec::new(),
            health_endpoint: None,
            extra: serde_json::Map::new(),
        }
    }
}

/// A plugin in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    /// Unique identifier
    pub id: PluginId,

    /// Human-readable name
    pub name: String,

    /// Semantic version
    pub version: String,

    /// Plugin capabilities
    pub capabilities: Vec<Capability>,

    /// Plugin dependencies (other plugin IDs)
    pub dependencies: Vec<PluginId>,

    /// Additional metadata
    pub metadata: PluginMetadata,
}

impl Plugin {
    /// Create a new Plugin
    pub fn new(
        id: impl Into<PluginId>,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            capabilities: Vec::new(),
            dependencies: Vec::new(),
            metadata: PluginMetadata::default(),
        }
    }

    /// Add a capability to this plugin
    #[must_use]
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add a dependency to this plugin
    pub fn with_dependency(mut self, dep: impl Into<PluginId>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    /// Set metadata for this plugin
    #[must_use]
    pub fn with_metadata(mut self, metadata: PluginMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_id_creation() {
        let id = PluginId::new("test-plugin");
        assert_eq!(id.as_str(), "test-plugin");
    }

    #[test]
    fn test_plugin_builder() {
        let plugin = Plugin::new("test", "Test Plugin", "1.0.0");
        assert_eq!(plugin.id.as_str(), "test");
        assert_eq!(plugin.name, "Test Plugin");
        assert_eq!(plugin.version, "1.0.0");
    }
}
