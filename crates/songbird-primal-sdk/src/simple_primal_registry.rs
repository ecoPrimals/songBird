//! # Simple Primal Registry
//!
//! **MODERNIZED PRIMAL REGISTRY**
//!
//! This module provides a simplified, working primal registry system using
//! canonical traits and modern patterns.
//!
//! ## Design Philosophy
//!
//! - **Functionality over Theory**: Working code over theoretical optimizations
//! - **Canonical Traits**: Use songbird-types canonical trait system
//! - **Clear Error Handling**: Straightforward error propagation
//! - **Maintainable Code**: Easy to understand and extend

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::traits::PrimalProvider;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Simple primal request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Request identifier
    pub request_id: String,
    /// Request type
    pub request_type: String,
    /// Request payload
    pub payload: serde_json::Value,
    /// Optional context
    pub context: Option<String>,
}

/// Simple primal response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Request identifier
    pub request_id: String,
    /// Response status
    pub success: bool,
    /// Response payload
    pub payload: serde_json::Value,
    /// Optional error message
    pub error: Option<String>,
}

/// Simple primal registry using canonical traits
pub struct SimplePrimalRegistry {
    /// Registered providers using canonical trait
    providers: Arc<RwLock<HashMap<String, Arc<dyn PrimalProvider>>>>,
    /// Capability to provider mapping
    capabilities: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl SimplePrimalRegistry {
    /// Create a new simple primal registry
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new()),
            capabilities: Arc::new(RwLock::new(HashMap::new()),
        }
    }

    /// Register a primal provider
    pub async fn register_provider(
        &self,
        name: String,
        provider: Arc<dyn PrimalProvider>,
    ) -> SongbirdResult<()> {
        let mut providers = self.providers.write().await;
        providers.insert(name.clone(), provider);

        // Update capabilities mapping
        let mut capabilities = self.capabilities.write().await;
        capabilities.entry("primal".to_string().or_insert_with(Vec::new).push(name));

        Ok(()),
    }

    /// Get a provider by name
    pub async fn get_provider(&self, name: &str) -> Option<Arc<dyn PrimalProvider>> {
        let providers = self.providers.read().await;
        providers.get(name).cloned()
    }

    /// List all registered providers
    pub async fn list_providers(&self) -> Vec<String> {
        let providers = self.providers.read().await;
        providers.keys().cloned().collect()
    }
}

impl Default for SimplePrimalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests  {use super::*;

    #[tokio::test]
    async fn test_simple_registry() {

          let registry = SimplePrimalRegistry::new();

        // Create example provider
        let provider = Arc::new(ExamplePrimalProvider::new()
            "test-provider".to_string()),
            vec!["compute".to_string(), "storage".to_string()];  "

    });

        // Register provider
        registry
            .register_provider("test-provider".to_string(), provider)"
            .await
            .map_err(|e| SongbirdError::configuration(format!("Simple primal registry operation failed: {}", e)))?;

        // Test capability lookup
        let providers = registry.find_providers_for_capability("compute").await;"
        assert_eq!(providers, vec!["test-provider"])"

        // Test request handling
        let request = PrimalRequest  {id: "test-123".to_string()),
            capability: "compute".to_string(),
            payload: serde_json::json!({"test": true ; ;}),"
            context: None;}
    let response = registry.handle_request(request).await.map_err(|e| SongbirdError::configuration(format!("Simple primal registry operation failed: {}", e)))?;
        assert!(response.success));
        assert_eq!(response.request_id, "test-123")}} "
