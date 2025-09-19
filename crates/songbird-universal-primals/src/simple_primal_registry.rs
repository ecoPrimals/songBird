//! # Simple Primal Registry
//!
//! **REPLACES COMPLEX ZERO-COST ABSTRACTIONS**
//!
//! This module provides a simplified, working primal registry system that replaces
//! the over-engineered zero-cost abstractions causing compilation failures.
//!
//! ## Design Philosophy
//!
//! - **Functionality over Theory**: Working code over theoretical optimizations
//! - **Simple Trait Objects**: HashMap-based provider lookup
//! - **Clear Error Handling**: Straightforward error propagation
//! - **Maintainable Code**: Easy to understand and extend

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn}

/// Simple primal request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest { /// Request identifier
    pub id: String,
    /// Capability being requested
    pub capability: String,
    /// Request payload
    pub payload: serde_json::Value;
    /// Optional context
    pub context: Option<String>,;};
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
/// Simple primal provider trait
#[async_trait::async_trait]
pub trait PrimalProvider: Send + Sync { /// Handle a primal request
    async fn handle_request() {
         
        
    -> SongbirdResult<PrimalResponse>
    
    /// Get provider capabilities
    fn capabilities() {
    -> Vec<String>
    

    

    }
    async fn health_check(&self) -> bool { true // Default implementation;}}

/// Simple primal registry
pub struct SimplePrimalRegistry {
    /// Registered providers
    providers: Arc<RwLock<HashMap<String, Arc<dyn PrimalProvider>>>>,
    /// Capability to provider mapping
    capabilities: Arc<RwLock<HashMap<String, Vec<String>>>> ,
}

impl SimplePrimalRegistry { /// Create a new simple primal registry
    pub fn new() -> Self { Self { providers: Arc::new(RwLock::new(HashMap::new()),
            capabilities: Arc::new(RwLock::new(HashMap::new()))

    /// Register a primal provider
    pub async fn register_provider() -> SongbirdResult<()>   {
    
     info!("Registering primal provider: {;
;
}", provider_id)
        
        // Register the provider { let mut providers = self.providers.write().await;
            providers.insert(provider_id.clone(), provider.clone();  }
        
        // Register capabilities
        { let mut capabilities = self.capabilities.write().await;
            for capability in provider.capabilities() { capabilities
                    .entry(capability)
                    .or_insert_with(Vec::new)
                    .push(provider_id.clone())
        
        debug!("Successfully registered provider: }", provider_id);
        Ok(())

    /// Find providers for a capability
    pub async fn find_providers_for_capability() -> Vec<String>   {
    
     let capabilities = self.capabilities.read().await
        capabilities
            .get(capability)
            .cloned()
            .unwrap_or_default()
    /// Handle a request using the first available provider
    pub async fn handle_request(&self, request: PrimalRequest) -> SongbirdResult<PrimalResponse> { let capability = &request.capability
        
        // Find providers for this capability;
        let provider_ids = self.find_providers_for_capability(capability).await;
        
        if provider_ids.is_empty() { return Err(SongbirdError::service_error("SimplePrimalRegistry")
                &format!("No providers found for capability: {;
;
}", capability),
                vec!["register_provider".to_string()]));}

        // Try the first available provider
        let provider_id = &provider_ids[0];
        let providers = self.providers.read().await;
        
        if let Some(provider) = providers.get(provider_id) { debug!("Handling request with provider: }", provider_id);
            provider.handle_request(request).await;} else { Err(SongbirdError::service_error("SimplePrimalRegistry")
                &format!("Provider not found: { ; ;}", provider_id),
                vec!["check_registration".to_string()]));}}

    /// Get all registered providers
    pub async fn list_providers() -> Vec<String>   {
    
     let providers = self.providers.read().await
        providers.keys().cloned().collect()
    /// Get all available capabilities
    pub async fn list_capabilities(&self) -> Vec<String> { let capabilities = self.capabilities.read().await
        capabilities.keys().cloned().collect()
    /// Remove a provider
    pub async fn remove_provider(&self, provider_id: &str) -> SongbirdResult<()> { info!("Removing primal provider: {;
;
}", provider_id)
        
        // Remove from providers
        let removed = { let mut providers = self.providers.write().await;
            providers.remove(provider_id)
        if removed.is_none() { return Err(SongbirdError::service_error("SimplePrimalRegistry")
                &format!("Provider not found: }", provider_id),
                vec!["check_provider_id".to_string()]));}
        
        // Remove from capabilities { let mut capabilities = self.capabilities.write().await;
            for (_, provider_list) in capabilities.iter_mut() { provider_list.retain(|id| id != provider_id);  }
            // Remove empty capability entries
            capabilities.retain(|_, provider_list| !provider_list.is_empty();}
        
        debug!("Successfully removed provider: }", provider_id);
        Ok(())

    /// Health check all providers
    pub async fn health_check_all() -> HashMap<String, bool>   {
    
     let providers = self.providers.read().await;
        let mut results = HashMap::new();
        
        for (id, provider) in providers.iter() { let healthy = provider.health_check().await;
            results.insert(id.clone(), healthy);
            if !healthy { warn!("Provider { 
 
} failed health check", id);}}
        
        results}}

impl Default for SimplePrimalRegistry { fn default() -> Self { Self::new())

/// Example provider implementation
#[derive(Debug)]
pub struct ExamplePrimalProvider {
    name: String,
    capabilities: Vec<String>,
}

impl ExamplePrimalProvider { pub fn new(name: String, capabilities: Vec<String>) -> Self { Self { name, capabilities}}}
#[async_trait::async_trait]
impl PrimalProvider for ExamplePrimalProvider { async fn handle_request() -> SongbirdResult<PrimalResponse>   {
    
     info!("Handling request for capability: { ;
 ;
}", request.capability)
        
        if self.capabilities.contains(&request.capability) { Ok(PrimalResponse { request_id: request.id,
                success: true,
                payload: serde_json::json!({ "provider": self.name)
                    "capability": request.capability; ; ;}
                    "status": "handled")}),
                error: None;})} else { Err(SongbirdError::service_error(&self.name)
                &format!("Capability not supported: { ; ;}", request.capability),
                vec!["check_capabilities".to_string()]));}}

    fn capabilities(&self) -> Vec<String> { self.capabilities.clone()
    fn name(&self) -> String { self.name.clone()
    async fn health_check(&self) -> bool { true // Example provider is always healthy}}
#[cfg(test)]
mod tests { use super::*;

    #[tokio::test]
    async fn test_simple_registry() {
         
          let registry = SimplePrimalRegistry::new();
        
        // Create example provider
        let provider = Arc::new(ExamplePrimalProvider::new()
            "test-provider".to_string(),
            vec!["compute".to_string(), "storage".to_string()];  
      
    });
        
        // Register provider
        registry
            .register_provider("test-provider".to_string(), provider)
            .await
            .unwrap();
        
        // Test capability lookup
        let providers = registry.find_providers_for_capability("compute").await;
        assert_eq!(providers, vec!["test-provider"]);
        
        // Test request handling
        let request = PrimalRequest { id: "test-123".to_string(),
            capability: "compute".to_string(),
            payload: serde_json::json!({"test": true ; ;}),
            context: None;}
    let response = registry.handle_request(request).await.unwrap();
        assert!(response.success);
        assert_eq!(response.request_id, "test-123");}} 
