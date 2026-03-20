// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-Based Routing for HTTP Gateway
//!
//! This module implements universal, agnostic routing based on capabilities
//! rather than hardcoded vendor-specific logic.
//!
//! # Philosophy
//! - **Zero Hardcoding**: No vendor names, API endpoints, or provider-specific logic
//! - **Capability-Based**: Routes based on abstract capabilities (e.g., "ai:text-generation")
//! - **Runtime Discovery**: Providers register themselves with capabilities
//! - **Agnostic Design**: Works with any provider that implements the capability interface
//!
//! # Architecture
//! ```text
//! Request (Unix Socket) → Capability Router → Universal Proxy → External API
//!                              ↓
//!                   Capability Registry
//!                   (Runtime-discovered)
//! ```
//!
//! # Example
//! ```text
//! // Squirrel requests "ai:text-generation" capability
//! // Router discovers available providers at runtime
//! // No hardcoded "OpenAI" or "HuggingFace" logic!
//! ```

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, trace, warn};

/// Capability identifier (e.g., "ai:text-generation", "ai:image-generation")
pub type CapabilityId = String;

/// Provider identifier (runtime-discovered, not hardcoded)
pub type ProviderId = String;

/// Capability definition - describes what a provider can do
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability identifier (e.g., "ai:text-generation")
    pub id: CapabilityId,

    /// Human-readable description
    pub description: String,

    /// Capability category (e.g., "ai", "storage", "compute")
    pub category: String,

    /// Capability type (e.g., "text-generation", "image-generation")
    pub capability_type: String,

    /// Optional sub-type for more specific capabilities
    pub sub_type: Option<String>,

    /// Metadata about the capability
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Capability {
    /// Parse capability ID into components
    ///
    /// Format: `category:type` or `category:type:subtype`
    ///
    /// # Examples
    /// - `ai:text-generation` → category="ai", type="text-generation"
    /// - `ai:text-generation:chat` → category="ai", type="text-generation", subtype="chat"
    #[must_use]
    pub fn parse(id: &str) -> Option<(String, String, Option<String>)> {
        let parts: Vec<&str> = id.split(':').collect();
        match parts.len() {
            2 => Some((parts[0].to_string(), parts[1].to_string(), None)),
            3 => Some((parts[0].to_string(), parts[1].to_string(), Some(parts[2].to_string()))),
            _ => None,
        }
    }

    /// Check if this capability matches a requested capability
    #[must_use]
    pub fn matches(&self, requested: &str) -> bool {
        // Exact match
        if self.id == requested {
            return true;
        }

        // Wildcard match (e.g., "ai:text-generation:*" matches "ai:text-generation")
        if let Some(prefix) = requested.strip_suffix(":*") {
            return self.id.starts_with(prefix);
        }

        false
    }
}

/// Provider configuration - runtime-discovered, not hardcoded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider identifier (runtime-discovered)
    pub id: ProviderId,

    /// Human-readable provider name
    pub name: String,

    /// Capabilities this provider offers
    pub capabilities: Vec<Capability>,

    /// Unix socket path for this provider (if local)
    pub socket_path: Option<String>,

    /// Backend configuration for external providers
    pub backend: Option<BackendConfig>,

    /// Provider metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Backend configuration for external API providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Base URL for the external API
    pub base_url: String,

    /// Environment variable name for API key
    pub api_key_env: Option<String>,

    /// Request transformation rules
    pub request_transform: Option<TransformConfig>,

    /// Response transformation rules
    pub response_transform: Option<TransformConfig>,

    /// Custom headers
    pub headers: HashMap<String, String>,
}

/// Transformation configuration for request/response mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    /// `JSONPath` mappings for field transformations
    pub field_mappings: HashMap<String, String>,

    /// Template for transforming the entire payload
    pub template: Option<String>,
}

/// Route information for a capability request
#[derive(Debug, Clone)]
pub struct Route {
    /// The provider that will handle this request
    pub provider: ProviderConfig,

    /// The specific capability being invoked
    pub capability: Capability,

    /// Routing metadata
    pub metadata: HashMap<String, String>,
}

/// Capability Router - Universal, agnostic routing based on capabilities
#[derive(Debug)]
pub struct CapabilityRouter {
    /// Registry of providers and their capabilities (runtime-discovered)
    providers: Arc<RwLock<HashMap<ProviderId, ProviderConfig>>>,

    /// Capability index for fast lookup
    capability_index: Arc<RwLock<HashMap<CapabilityId, Vec<ProviderId>>>>,
}

impl CapabilityRouter {
    /// Create a new capability router
    #[must_use]
    pub fn new() -> Self {
        info!("Initializing Capability Router (agnostic design)");
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            capability_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a provider with its capabilities
    ///
    /// This is called at runtime when providers announce themselves.
    /// No hardcoding required!
    pub async fn register_provider(&self, provider: ProviderConfig) -> Result<()> {
        let provider_id = provider.id.clone();
        info!(
            "Registering provider '{}' with {} capabilities",
            provider.name,
            provider.capabilities.len()
        );

        // Update capability index
        {
            let mut index = self.capability_index.write().await;
            for capability in &provider.capabilities {
                index
                    .entry(capability.id.clone())
                    .or_insert_with(Vec::new)
                    .push(provider_id.clone());

                debug!("  • Capability '{}' registered", capability.id);
            }
        }

        // Store provider
        {
            let mut providers = self.providers.write().await;
            providers.insert(provider_id, provider);
        }

        Ok(())
    }

    /// Unregister a provider
    pub async fn unregister_provider(&self, provider_id: &str) -> Result<()> {
        info!("Unregistering provider '{}'", provider_id);

        // Remove from capability index
        {
            let mut index = self.capability_index.write().await;
            index.retain(|_, provider_ids| {
                provider_ids.retain(|id| id != provider_id);
                !provider_ids.is_empty()
            });
        }

        // Remove provider
        {
            let mut providers = self.providers.write().await;
            providers.remove(provider_id);
        }

        Ok(())
    }

    /// Route a capability request to an appropriate provider
    ///
    /// This method discovers providers at runtime based on the requested capability.
    /// No vendor-specific logic - pure capability-based routing!
    pub async fn route(&self, capability_id: &str) -> Result<Route> {
        trace!("Routing request for capability '{}'", capability_id);

        // Find providers that offer this capability
        let provider_ids = {
            let index = self.capability_index.read().await;
            index.get(capability_id).cloned()
        };

        let provider_ids = provider_ids.ok_or_else(|| {
            warn!("No providers found for capability '{}'", capability_id);
            anyhow!("No providers available for capability: {capability_id}")
        })?;

        // First registered provider wins; no load-balancing or scoring yet.
        let provider_id = provider_ids
            .first()
            .ok_or_else(|| anyhow!("Provider list empty for capability: {capability_id}"))?;

        // Retrieve provider configuration
        let provider = {
            let providers = self.providers.read().await;
            providers.get(provider_id).cloned()
        };

        let provider =
            provider.ok_or_else(|| anyhow!("Provider '{provider_id}' not found in registry"))?;

        // Find the matching capability
        let capability = provider
            .capabilities
            .iter()
            .find(|c| c.matches(capability_id))
            .cloned()
            .ok_or_else(|| {
            anyhow!("Capability '{}' not found in provider '{}'", capability_id, provider.name)
        })?;

        debug!("Routed '{}' → provider '{}' ({})", capability_id, provider.name, provider.id);

        Ok(Route {
            provider,
            capability,
            metadata: HashMap::new(),
        })
    }

    /// List all available capabilities
    #[must_use]
    pub async fn list_capabilities(&self) -> Vec<Capability> {
        let providers = self.providers.read().await;
        providers.values().flat_map(|p| p.capabilities.clone()).collect()
    }

    /// List all registered providers
    #[must_use]
    pub async fn list_providers(&self) -> Vec<ProviderConfig> {
        let providers = self.providers.read().await;
        providers.values().cloned().collect()
    }

    /// Load provider configurations from environment or discovery
    ///
    /// This replaces hardcoded provider lists with runtime discovery.
    pub async fn discover_providers(&self) -> Result<()> {
        info!("🔍 Discovering providers from environment...");

        // Check for provider registry file
        if let Ok(registry_path) = std::env::var("SONGBIRD_PROVIDER_REGISTRY") {
            info!("Loading providers from registry: {}", registry_path);
            self.load_registry_file(&registry_path).await?;
        }

        // Check for individual provider configs
        if let Ok(config_dir) = std::env::var("SONGBIRD_PROVIDER_CONFIG_DIR") {
            info!("Loading providers from config directory: {}", config_dir);
            self.load_config_directory(&config_dir).await?;
        }

        let provider_count = self.providers.read().await.len();
        let capability_count = self.capability_index.read().await.len();

        info!(
            "✅ Discovery complete: {} providers, {} capabilities",
            provider_count, capability_count
        );

        Ok(())
    }

    /// Load providers from a registry file
    async fn load_registry_file(&self, path: &str) -> Result<()> {
        let content = tokio::fs::read_to_string(path).await?;
        let registry: ProviderRegistry = serde_json::from_str(&content)?;

        for provider in registry.providers {
            self.register_provider(provider).await?;
        }

        Ok(())
    }

    /// Load providers from a configuration directory
    async fn load_config_directory(&self, dir: &str) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = tokio::fs::read_to_string(&path).await?;
                let provider: ProviderConfig = serde_json::from_str(&content)?;
                self.register_provider(provider).await?;
            }
        }

        Ok(())
    }
}

impl Default for CapabilityRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider registry format (JSON file)
#[derive(Debug, Serialize, Deserialize)]
struct ProviderRegistry {
    version: String,
    providers: Vec<ProviderConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_capability(id: &str) -> Capability {
        let (category, capability_type, sub_type) = Capability::parse(id).unwrap();
        Capability {
            id: id.to_string(),
            description: format!("Test capability: {}", id),
            category,
            capability_type,
            sub_type,
            metadata: HashMap::new(),
        }
    }

    fn create_test_provider(id: &str, capabilities: Vec<&str>) -> ProviderConfig {
        ProviderConfig {
            id: id.to_string(),
            name: format!("Test Provider {}", id),
            capabilities: capabilities.iter().map(|c| create_test_capability(c)).collect(),
            socket_path: Some(format!("/tmp/provider-{}.sock", id)),
            backend: None,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_capability_router_creation() {
        let router = CapabilityRouter::new();
        assert_eq!(router.list_providers().await.len(), 0);
        assert_eq!(router.list_capabilities().await.len(), 0);
    }

    #[tokio::test]
    async fn test_provider_registration() {
        let router = CapabilityRouter::new();
        let provider = create_test_provider("test1", vec!["ai:text-generation"]);

        router.register_provider(provider).await.unwrap();

        assert_eq!(router.list_providers().await.len(), 1);
        assert_eq!(router.list_capabilities().await.len(), 1);
    }

    #[tokio::test]
    async fn test_capability_routing() {
        let router = CapabilityRouter::new();
        let provider =
            create_test_provider("test1", vec!["ai:text-generation", "ai:image-generation"]);

        router.register_provider(provider).await.unwrap();

        let route = router.route("ai:text-generation").await.unwrap();
        assert_eq!(route.provider.id, "test1");
        assert_eq!(route.capability.id, "ai:text-generation");
    }

    #[tokio::test]
    async fn test_capability_not_found() {
        let router = CapabilityRouter::new();
        let provider = create_test_provider("test1", vec!["ai:text-generation"]);

        router.register_provider(provider).await.unwrap();

        let result = router.route("ai:image-generation").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_provider_unregistration() {
        let router = CapabilityRouter::new();
        let provider = create_test_provider("test1", vec!["ai:text-generation"]);

        router.register_provider(provider).await.unwrap();
        assert_eq!(router.list_providers().await.len(), 1);

        router.unregister_provider("test1").await.unwrap();
        assert_eq!(router.list_providers().await.len(), 0);
    }

    #[tokio::test]
    async fn test_multiple_providers_same_capability() {
        let router = CapabilityRouter::new();
        let provider1 = create_test_provider("test1", vec!["ai:text-generation"]);
        let provider2 = create_test_provider("test2", vec!["ai:text-generation"]);

        router.register_provider(provider1).await.unwrap();
        router.register_provider(provider2).await.unwrap();

        let route = router.route("ai:text-generation").await.unwrap();
        assert!(route.provider.id == "test1" || route.provider.id == "test2");
    }

    #[tokio::test]
    async fn test_capability_parse() {
        let parsed = Capability::parse("ai:text-generation").unwrap();
        assert_eq!(parsed.0, "ai");
        assert_eq!(parsed.1, "text-generation");
        assert_eq!(parsed.2, None);

        let parsed = Capability::parse("ai:text-generation:chat").unwrap();
        assert_eq!(parsed.0, "ai");
        assert_eq!(parsed.1, "text-generation");
        assert_eq!(parsed.2, Some("chat".to_string()));
    }

    #[tokio::test]
    async fn test_capability_matching() {
        let cap = create_test_capability("ai:text-generation:chat");

        assert!(cap.matches("ai:text-generation:chat"));
        assert!(cap.matches("ai:text-generation:*"));
        assert!(!cap.matches("ai:image-generation"));
    }
}
