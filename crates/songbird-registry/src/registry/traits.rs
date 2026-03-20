// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Registry traits
//!
//! Trait definitions for plugin registry functionality.
//!
//! # Native Async Traits (Rust 1.75+)
//! Uses native async fn in traits for zero-cost abstraction

#![expect(async_fn_in_trait, reason = "async fn in trait (edition / trait-object compatibility)")]

use crate::registry::query::Query;
use crate::types::{Capability, Plugin, PluginId, RegistryEvent};
use songbird_types::errors::SongbirdResult;

/// Trait for plugin registry operations
pub trait PluginRegistry: Send + Sync {
    /// Register a new plugin
    ///
    /// # Errors
    /// Returns an error if:
    /// - Plugin ID already exists
    /// - Plugin has invalid configuration
    /// - Storage operation fails
    async fn register(&mut self, plugin: Plugin) -> SongbirdResult<PluginId>;

    /// Unregister a plugin
    ///
    /// # Errors
    /// Returns an error if:
    /// - Plugin ID not found
    /// - Plugin has dependent plugins
    /// - Storage operation fails
    async fn unregister(&mut self, id: &PluginId) -> SongbirdResult<()>;

    /// Get a plugin by ID
    ///
    /// # Errors
    /// Returns an error if plugin not found
    async fn get(&self, id: &PluginId) -> SongbirdResult<Plugin>;

    /// List all registered plugins
    async fn list(&self) -> Vec<Plugin>;

    /// Search for plugins matching a query
    async fn search(&self, query: &Query) -> Vec<Plugin>;

    /// Check if a plugin exists
    async fn exists(&self, id: &PluginId) -> bool;

    /// Watch for registry events
    ///
    /// Returns a stream of registry events for monitoring changes
    fn watch_events(&self) -> tokio::sync::broadcast::Receiver<RegistryEvent>;
}

/// Trait for composable plugins
///
/// Plugins that implement this trait can be composed with other plugins.
pub trait Composable: Send + Sync {
    /// Get the capabilities this plugin provides
    fn capabilities(&self) -> &[Capability];

    /// Check if this plugin is compatible with another
    fn compatible_with(&self, other: &dyn Composable) -> bool {
        // Default implementation: check for capability overlap
        let self_caps = self.capabilities();
        let other_caps = other.capabilities();

        self_caps.iter().any(|cap1| other_caps.iter().any(|cap2| cap1.compatible_with(cap2)))
    }

    /// Get the name of this composable entity
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper struct
    struct TestComposable {
        name: String,
        capabilities: Vec<Capability>,
    }

    impl Composable for TestComposable {
        fn capabilities(&self) -> &[Capability] {
            &self.capabilities
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_composable_compatibility() {
        use crate::types::capability::{Capability, CapabilityType};

        let comp1 = TestComposable {
            name: "test1".to_string(),
            capabilities: vec![Capability::new(CapabilityType::Encryption {
                algorithms: vec!["aes256".to_string()],
                key_sizes: vec![256],
            })],
        };

        let comp2 = TestComposable {
            name: "test2".to_string(),
            capabilities: vec![Capability::new(CapabilityType::Encryption {
                algorithms: vec!["aes128".to_string()],
                key_sizes: vec![128],
            })],
        };

        assert!(comp1.compatible_with(&comp2));
    }
}
