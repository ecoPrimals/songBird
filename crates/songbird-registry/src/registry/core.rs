//! Core registry implementation
//!
//! The main Registry struct and its implementation.
//!
//! # Native Async Traits (Rust 1.75+)
//! Uses native async fn in traits for zero-cost registry operations

#![allow(async_fn_in_trait)]

use songbird_types::errors::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::registry::query::Query;
use crate::registry::traits::PluginRegistry;
use crate::types::{EventType, Plugin, PluginId, RegistryEvent};

/// The main plugin registry
pub struct Registry {
    /// Registered plugins
    plugins: Arc<RwLock<HashMap<PluginId, Plugin>>>,

    /// Event broadcaster
    events: tokio::sync::broadcast::Sender<RegistryEvent>,
}

impl Registry {
    /// Create a new registry
    #[must_use]
    pub fn new() -> Self {
        let (tx, _rx) = tokio::sync::broadcast::channel(100);

        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            events: tx,
        }
    }

    /// Emit an event
    fn emit_event(&self, event: RegistryEvent) {
        // Ignore send errors (no active receivers is fine)
        let _ = self.events.send(event);
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry for Registry {
    async fn register(&mut self, plugin: Plugin) -> SongbirdResult<PluginId> {
        let plugin_id = plugin.id.clone();

        // Check if plugin already exists
        let mut plugins = self.plugins.write().await;
        if plugins.contains_key(&plugin_id) {
            return Err(SongbirdError::service(
                "registry",
                format!("Plugin {plugin_id} already registered"),
            ));
        }

        // Validate plugin dependencies
        for dep in &plugin.dependencies {
            if !plugins.contains_key(dep) {
                return Err(SongbirdError::service(
                    "registry",
                    format!("Dependency {dep} not found for plugin {plugin_id}"),
                ));
            }
        }

        // Register the plugin
        plugins.insert(plugin_id.clone(), plugin);
        drop(plugins); // Release lock before emitting event

        // Emit registration event
        self.emit_event(RegistryEvent::new(EventType::PluginRegistered {
            plugin_id: plugin_id.clone(),
        }));

        tracing::info!("Registered plugin: {}", plugin_id);

        Ok(plugin_id)
    }

    async fn unregister(&mut self, id: &PluginId) -> SongbirdResult<()> {
        let mut plugins = self.plugins.write().await;

        // Check if plugin exists
        if !plugins.contains_key(id) {
            return Err(SongbirdError::service("registry", format!("Plugin {id} not found")));
        }

        // Check for dependent plugins
        let has_dependents = plugins.values().any(|p| p.dependencies.contains(id));
        if has_dependents {
            return Err(SongbirdError::service(
                "registry",
                format!("Plugin {id} has dependent plugins"),
            ));
        }

        // Unregister the plugin
        plugins.remove(id);
        drop(plugins); // Release lock before emitting event

        // Emit unregistration event
        self.emit_event(RegistryEvent::new(EventType::PluginUnregistered {
            plugin_id: id.clone(),
        }));

        tracing::info!("Unregistered plugin: {}", id);

        Ok(())
    }

    async fn get(&self, id: &PluginId) -> SongbirdResult<Plugin> {
        let plugins = self.plugins.read().await;

        plugins
            .get(id)
            .cloned()
            .ok_or_else(|| SongbirdError::service("registry", format!("Plugin {id} not found")))
    }

    async fn list(&self) -> Vec<Plugin> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }

    async fn search(&self, query: &Query) -> Vec<Plugin> {
        let plugins = self.plugins.read().await;

        let mut results: Vec<Plugin> = plugins
            .values()
            .filter(|plugin| {
                // Filter by ID (exact match)
                if let Some(ref id) = query.id {
                    if plugin.id.as_str() != id {
                        return false;
                    }
                }

                // Filter by name (substring match)
                if let Some(ref name) = query.name {
                    if !plugin.name.to_lowercase().contains(&name.to_lowercase()) {
                        return false;
                    }
                }

                // Filter by author
                if let Some(ref author) = query.author {
                    if plugin.metadata.author != *author {
                        return false;
                    }
                }

                // Filter by tags (must have ALL specified tags)
                if !query.tags.is_empty()
                    && !query.tags.iter().all(|tag| plugin.metadata.tags.contains(tag))
                {
                    return false;
                }

                // Filter by capabilities (must have ALL specified capability types)
                if !query.capabilities.is_empty() {
                    let plugin_cap_types: Vec<_> = plugin
                        .capabilities
                        .iter()
                        .map(|c| std::mem::discriminant(&c.capability_type))
                        .collect();

                    if !query
                        .capabilities
                        .iter()
                        .all(|qc| plugin_cap_types.contains(&std::mem::discriminant(qc)))
                    {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Apply limit if specified
        if query.limit > 0 && results.len() > query.limit {
            results.truncate(query.limit);
        }

        results
    }

    async fn exists(&self, id: &PluginId) -> bool {
        let plugins = self.plugins.read().await;
        plugins.contains_key(id)
    }

    fn watch_events(&self) -> tokio::sync::broadcast::Receiver<RegistryEvent> {
        self.events.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Plugin;

    #[tokio::test]
    async fn test_registry_register() -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = Registry::new();
        let plugin = Plugin::new("test", "Test Plugin", "1.0.0");

        let result = registry.register(plugin).await;
        assert!(result.is_ok());

        let id = result?;
        assert_eq!(id.as_str(), "test");
        Ok(())
    }

    #[tokio::test]
    async fn test_registry_duplicate() -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = Registry::new();
        let plugin1 = Plugin::new("test", "Test Plugin", "1.0.0");
        let plugin2 = Plugin::new("test", "Test Plugin", "1.0.0");

        registry.register(plugin1).await?;
        let result = registry.register(plugin2).await;

        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_registry_get() -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = Registry::new();
        let plugin = Plugin::new("test", "Test Plugin", "1.0.0");

        registry.register(plugin).await?;

        let id = PluginId::from("test");
        let result = registry.get(&id).await;

        assert!(result.is_ok());
        assert_eq!(
            result
                .map_err(|e| SongbirdError::configuration(format!(
                    "Registry operation failed: {e}"
                )))?
                .name,
            "Test Plugin"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_registry_list() -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = Registry::new();

        registry.register(Plugin::new("test1", "Plugin 1", "1.0.0")).await?;
        registry.register(Plugin::new("test2", "Plugin 2", "1.0.0")).await?;

        let plugins = registry.list().await;
        assert_eq!(plugins.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_registry_unregister() -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = Registry::new();
        let plugin = Plugin::new("test", "Test Plugin", "1.0.0");

        let id = registry.register(plugin).await?;
        let result = registry.unregister(&id).await;

        assert!(result.is_ok());
        assert!(!registry.exists(&id).await);
        Ok(())
    }

    #[tokio::test]
    async fn test_registry_search_by_name() -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = Registry::new();

        registry.register(Plugin::new("test1", "Test Plugin", "1.0.0")).await?;
        registry.register(Plugin::new("test2", "Other Plugin", "1.0.0")).await?;

        let query = Query::new().with_name("Test");
        let results = registry.search(&query).await;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Test Plugin");
        Ok(())
    }
}
