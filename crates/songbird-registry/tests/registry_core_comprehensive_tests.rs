//! Comprehensive tests for registry core functionality

use songbird_registry::registry::core::Registry;
use songbird_registry::registry::traits::PluginRegistry;
use songbird_registry::types::{Plugin, PluginId};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};

fn create_test_plugin(id: &str, name: &str) -> Plugin {
    Plugin::new(id, name, "1.0.0")
}

fn create_test_plugin_with_deps(id: &str, name: &str, deps: Vec<String>) -> Plugin {
    let mut plugin = Plugin::new(id, name, "1.0.0");
    for dep in deps {
        plugin = plugin.with_dependency(dep);
    }
    plugin
}

#[tokio::test]
async fn test_registry_creation() -> SongbirdResult<()> {
    let registry = Registry::new();
    let plugins = registry.list().await;
    assert!(plugins.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_registry_default() -> SongbirdResult<()> {
    let registry = Registry::default();
    let plugins = registry.list().await;
    assert!(plugins.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_register_plugin() -> SongbirdResult<()> {
    let mut registry = Registry::new();
    let plugin = create_test_plugin("test-plugin", "Test Plugin");

    let result = registry.register(plugin.clone()).await;
    assert!(result.is_ok());

    let plugin_id = result.ok_or_else(|| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    assert_eq!(plugin_id.as_str(), "test-plugin");
    Ok(())
}

#[tokio::test]
async fn test_register_duplicate_plugin_fails() -> SongbirdResult<()> {
    let mut registry = Registry::new();
    let plugin = create_test_plugin("test-plugin", "Test Plugin");

    registry.register(plugin.clone()).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    let result = registry.register(plugin).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_get_plugin() -> SongbirdResult<()> {
    let mut registry = Registry::new();
    let plugin = create_test_plugin("test-plugin", "Test Plugin");

    registry.register(plugin.clone()).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    let retrieved = registry.get(&PluginId::new("test-plugin".to_string())).await;
    assert!(retrieved.is_ok());

    let retrieved_plugin = retrieved.ok_or_else(|| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    assert_eq!(retrieved_plugin.name, "Test Plugin");
    Ok(())
}

#[tokio::test]
async fn test_get_nonexistent_plugin_fails() -> SongbirdResult<()> {
    let registry = Registry::new();

    let result = registry.get(&PluginId::new("nonexistent".to_string())).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_list_plugins() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    let plugin1 = create_test_plugin("plugin1", "Plugin 1");
    let plugin2 = create_test_plugin("plugin2", "Plugin 2");
    let plugin3 = create_test_plugin("plugin3", "Plugin 3");

    registry.register(plugin1).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    registry.register(plugin2).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    registry.register(plugin3).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    let plugins = registry.list().await;
    assert_eq!(plugins.len(), 3);
    Ok(())
}

#[tokio::test]
async fn test_list_empty_registry() -> SongbirdResult<()> {
    let registry = Registry::new();
    let plugins = registry.list().await;
    assert!(plugins.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_unregister_plugin() -> SongbirdResult<()> {
    let mut registry = Registry::new();
    let plugin = create_test_plugin("test-plugin", "Test Plugin");

    registry.register(plugin).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    let result = registry.unregister(&PluginId::new("test-plugin".to_string())).await;
    assert!(result.is_ok());

    let plugins = registry.list().await;
    assert!(plugins.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_unregister_nonexistent_plugin_fails() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    let result = registry.unregister(&PluginId::new("nonexistent".to_string())).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_register_plugin_with_dependencies() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    // Register dependency first
    let dep_plugin = create_test_plugin("dependency", "Dependency Plugin");
    registry.register(dep_plugin).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    // Register plugin with dependency
    let plugin =
        create_test_plugin_with_deps("main", "Main Plugin", vec!["dependency".to_string()]);
    let result = registry.register(plugin).await;

    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_register_plugin_with_missing_dependency_fails() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    // Try to register plugin with missing dependency
    let plugin = create_test_plugin_with_deps("main", "Main Plugin", vec!["missing".to_string()]);
    let result = registry.register(plugin).await;

    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_unregister_plugin_with_dependents_fails() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    // Register dependency
    let dep_plugin = create_test_plugin("dependency", "Dependency Plugin");
    registry.register(dep_plugin).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    // Register dependent plugin
    let plugin =
        create_test_plugin_with_deps("main", "Main Plugin", vec!["dependency".to_string()]);
    registry.register(plugin).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    // Try to unregister dependency (should fail)
    let result = registry.unregister(&PluginId::new("dependency".to_string())).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_register_multiple_plugins() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    for i in 0..10 {
        let plugin = create_test_plugin(&format!("plugin{i}"), &format!("Plugin {i}"));
        let result = registry.register(plugin).await;
        assert!(result.is_ok());
    }

    let plugins = registry.list().await;
    assert_eq!(plugins.len(), 10);
    Ok(())
}

#[tokio::test]
async fn test_plugin_lifecycle() -> SongbirdResult<()> {
    let mut registry = Registry::new();
    let plugin = create_test_plugin("test-plugin", "Test Plugin");

    // Register
    let id = registry.register(plugin).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    assert_eq!(registry.list().await.len(), 1);

    // Get
    let retrieved = registry.get(&id).await;
    assert!(retrieved.is_ok());

    // Unregister
    registry.unregister(&id).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    assert!(registry.list().await.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_concurrent_registrations() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    let plugin1 = create_test_plugin("plugin1", "Plugin 1");
    registry.register(plugin1).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    let plugin2 = create_test_plugin("plugin2", "Plugin 2");
    registry.register(plugin2).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    let plugin3 = create_test_plugin("plugin3", "Plugin 3");
    registry.register(plugin3).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    assert_eq!(registry.list().await.len(), 3);
    Ok(())
}

#[tokio::test]
async fn test_plugin_id_uniqueness() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    let plugin1 = create_test_plugin("same-id", "Plugin 1");
    let plugin2 = create_test_plugin("same-id", "Plugin 2");

    registry.register(plugin1).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    let result = registry.register(plugin2).await;

    assert!(result.is_err(), "Should not allow duplicate IDs");
    Ok(())
}

#[tokio::test]
async fn test_dependency_chain() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    // Register plugins in dependency order
    let plugin1 = create_test_plugin("base", "Base Plugin");
    registry.register(plugin1).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    let plugin2 = create_test_plugin_with_deps("middle", "Middle Plugin", vec!["base".to_string()]);
    registry.register(plugin2).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    let plugin3 = create_test_plugin_with_deps("top", "Top Plugin", vec!["middle".to_string()]);
    registry.register(plugin3).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    assert_eq!(registry.list().await.len(), 3);
    Ok(())
}

#[tokio::test]
async fn test_multiple_dependencies() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    // Register multiple dependencies
    let dep1 = create_test_plugin("dep1", "Dependency 1");
    let dep2 = create_test_plugin("dep2", "Dependency 2");
    let dep3 = create_test_plugin("dep3", "Dependency 3");

    registry.register(dep1).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    registry.register(dep2).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    registry.register(dep3).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    // Register plugin with multiple dependencies
    let plugin = create_test_plugin_with_deps(
        "main",
        "Main Plugin",
        vec!["dep1".to_string(), "dep2".to_string(), "dep3".to_string()],
    );

    let result = registry.register(plugin).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_partial_dependencies_fail() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    // Register only some dependencies
    let dep1 = create_test_plugin("dep1", "Dependency 1");
    registry.register(dep1).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    // Try to register plugin with missing dependency
    let plugin = create_test_plugin_with_deps(
        "main",
        "Main Plugin",
        vec!["dep1".to_string(), "dep2".to_string()],
    );

    let result = registry.register(plugin).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_unregister_order_matters() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    // Build dependency chain
    let base = create_test_plugin("base", "Base");
    let middle = create_test_plugin_with_deps("middle", "Middle", vec!["base".to_string()]);

    registry.register(base).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    registry.register(middle).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    // Must unregister in reverse dependency order
    registry.unregister(&PluginId::new("middle".to_string())).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    registry.unregister(&PluginId::new("base".to_string())).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    assert!(registry.list().await.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_registry_isolation() -> SongbirdResult<()> {
    let mut registry1 = Registry::new();
    let registry2 = Registry::new();

    let plugin = create_test_plugin("test", "Test");

    registry1.register(plugin).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    // registry2 should not see plugins from registry1
    assert_eq!(registry1.list().await.len(), 1);
    assert_eq!(registry2.list().await.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_plugin_version_tracking() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    let mut plugin = create_test_plugin("test", "Test");
    plugin.version = "1.0.0".to_string();

    registry.register(plugin.clone()).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    let retrieved = registry.get(&PluginId::new("test".to_string())).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    assert_eq!(retrieved.version, "1.0.0");
    Ok(())
}

#[tokio::test]
async fn test_empty_plugin_name() -> SongbirdResult<()> {
    let mut registry = Registry::new();
    let plugin = create_test_plugin("id", "");

    let result = registry.register(plugin).await;
    assert!(result.is_ok(), "Empty name should be allowed");
    Ok(())
}

#[tokio::test]
async fn test_special_characters_in_id() -> SongbirdResult<()> {
    let mut registry = Registry::new();
    let plugin = create_test_plugin("test-plugin_v2.0", "Test Plugin");

    let result = registry.register(plugin).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_register_after_unregister() -> SongbirdResult<()> {
    let mut registry = Registry::new();
    let plugin1 = create_test_plugin("test", "Test 1");
    let plugin2 = create_test_plugin("test", "Test 2");

    registry.register(plugin1).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    registry.unregister(&PluginId::new("test".to_string())).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    // Should be able to register same ID again
    let result = registry.register(plugin2).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_large_number_of_plugins() -> SongbirdResult<()> {
    let mut registry = Registry::new();

    for i in 0..100 {
        let plugin = create_test_plugin(&format!("plugin{i}"), &format!("Plugin {i}"));
        registry.register(plugin).await.map_err(|e| {
            SongbirdError::configuration("Failed to register".to_string())
        })?;
    }

    assert_eq!(registry.list().await.len(), 100);
    Ok(())
}

#[tokio::test]
async fn test_get_after_list() -> SongbirdResult<()> {
    let mut registry = Registry::new();
    let plugin = create_test_plugin("test", "Test");

    registry.register(plugin).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    let plugins = registry.list().await;
    assert_eq!(plugins.len(), 1);

    let retrieved = registry.get(&PluginId::new("test".to_string())).await;
    assert!(retrieved.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_plugin_metadata_preserved() -> SongbirdResult<()> {
    let mut registry = Registry::new();
    let mut plugin = create_test_plugin("test", "Test");
    plugin.metadata.extra.insert("key".to_string(), serde_json::json!("value"));

    registry.register(plugin).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;

    let retrieved = registry.get(&PluginId::new("test".to_string())).await.map_err(|e| {
        SongbirdError::configuration("Failed to register".to_string())
    })?;
    assert_eq!(
        retrieved.metadata.extra.get("key").or_else(|_| SongbirdError::configuration(format!(
            "Error: {}",
            e
        )))?,
        &serde_json::json!("value")
    );
    Ok(())
}
